use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::models::ComposeServiceState;

fn docker_command() -> Command {
    let mut cmd = Command::new("docker");
    // Windows: 避免 GUI 应用拉起 docker 时闪现控制台黑窗
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd
}

/// `docker compose ps --format json` 输出的单条记录(只取关心的字段)
#[derive(Debug, Deserialize)]
struct PsEntry {
    #[serde(rename = "Service")]
    service: Option<String>,
    #[serde(rename = "State")]
    state: Option<String>,
    #[serde(rename = "Status")]
    status: Option<String>,
}

/// 解析 ps 输出:compose v2.21+ 输出 JSON 数组,更早版本输出 NDJSON(每行一个对象)
fn parse_ps(output: &str) -> Vec<ComposeServiceState> {
    let entries: Vec<PsEntry> = serde_json::from_str(output.trim())
        .or_else(|_| {
            output
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(serde_json::from_str)
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();
    entries
        .into_iter()
        .filter_map(|e| {
            Some(ComposeServiceState {
                name: e.service?,
                running: e.state.as_deref() == Some("running"),
                status: e.status.unwrap_or_default(),
            })
        })
        .collect()
}

fn ps_blocking(path: &str, file: &str) -> AppResult<Vec<ComposeServiceState>> {
    let dir = Path::new(path);
    if !dir.is_dir() {
        return Err(AppError::Invalid(format!("目录不存在: {path}")));
    }
    // 与前端 up/down 的执行方式保持一致:项目根目录 + 相对 -f 路径,
    // 这样 compose 项目名解析一致,ps 才能命中同一组容器。
    // docker 未安装 / 守护进程未运行 / 项目未启动:一律视为无运行中服务(不报错打扰)
    let output = docker_command()
        .args(["compose", "-f", file, "ps", "--format", "json"])
        .current_dir(dir)
        .output();
    match output {
        Ok(out) if out.status.success() => Ok(parse_ps(&String::from_utf8_lossy(&out.stdout))),
        _ => Ok(Vec::new()),
    }
}

/// 查询 compose 文件中各服务的运行状态(阻塞调用放入线程池,避免卡住 UI)
#[tauri::command]
pub async fn compose_ps(path: String, file: String) -> AppResult<Vec<ComposeServiceState>> {
    tokio::task::spawn_blocking(move || ps_blocking(&path, &file))
        .await
        .map_err(|e| AppError::External(format!("任务执行失败: {e}")))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ndjson_output() {
        // 早期 compose:每行一个 JSON 对象
        let out = concat!(
            r#"{"Name":"app-web-1","Service":"web","State":"running","Status":"Up 2 hours"}"#,
            "\n",
            r#"{"Name":"app-db-1","Service":"db","State":"exited","Status":"Exited (0) 5 minutes ago"}"#,
            "\n"
        );
        let states = parse_ps(out);
        assert_eq!(states.len(), 2);
        assert_eq!(states[0].name, "web");
        assert!(states[0].running);
        assert_eq!(states[0].status, "Up 2 hours");
        assert_eq!(states[1].name, "db");
        assert!(!states[1].running);
    }

    #[test]
    fn parses_json_array_output() {
        // compose v2.21+:整体一个 JSON 数组
        let out = r#"[
          {"Name":"app-web-1","Service":"web","State":"running","Status":"Up 3 seconds"},
          {"Name":"app-api-1","Service":"api","State":"restarting","Status":"Restarting"}
        ]"#;
        let states = parse_ps(out);
        assert_eq!(states.len(), 2);
        assert!(states[0].running);
        assert!(!states[1].running);
    }

    #[test]
    fn empty_or_garbage_output_yields_no_states() {
        assert!(parse_ps("").is_empty());
        assert!(parse_ps("\n\n").is_empty());
        assert!(parse_ps("not json at all").is_empty());
    }
}
