use std::process::Command;

use tauri::State;

use crate::db::{self, Db};
use crate::error::{AppError, AppResult};
use crate::models::EditorKind;

const VSCODE_SETTING_KEY: &str = "vscode_available";

/// Windows 下隐藏中间进程的控制台黑窗(最终弹出的终端窗口不受影响)
fn hidden(#[allow(unused_mut)] mut cmd: Command) -> Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd
}

/// 在系统终端打开目录,可选执行命令(跑完不关窗口)。
/// 优先用 Windows Terminal(wt.exe);未安装或启动失败时退回 cmd。
#[cfg(windows)]
pub fn spawn_terminal(path: &str, title: &str, command: Option<&str>) -> AppResult<()> {
    if let Some(wt) = find_wt() {
        // wt 是 GUI 子系统进程,自己创建窗口,不存在句柄透传问题,直接启动即可
        let spawned = Command::new(wt)
            .args(build_wt_args(path, title, command))
            .spawn();
        if spawned.is_ok() {
            return Ok(());
        }
        // 启动失败(如 AppX 执行别名被用户禁用)则继续走下面的 cmd 兜底
    }
    // 结构:cmd /C start "<title>" cmd /K "<inner>",外层 cmd 用 CREATE_NO_WINDOW 隐藏。
    //
    // 为什么不能直接 CREATE_NEW_CONSOLE 起 cmd /K:Rust 的 Command 会把父进程
    // (Tauri 应用)的标准句柄透传给子进程——即使子进程拿到了新控制台,它的
    // stdout/stderr 仍指向父进程的句柄(dev 终端或管道),命令照常执行但新窗口里
    // 看不到任何输出。而 start 拉起目标进程时不透传句柄,cmd /K 会拿到全新控制台
    // 的输入/输出句柄,输出正常显示。(已用 marker 实验证实两种方式的句柄流向)
    //
    // 引号解析(已实测):
    // - 外层 cmd /C 的命令串首字符是 's',不触发首尾引号剥离;
    // - `&&` 位于 cmd /K 的引号串内,不会在外层被顶层切分,整串交给 start;
    // - start 把第一个引号串当窗口标题,其余原样交给新进程;
    // - 内层 cmd /K 首字符是引号,剥掉首尾引号后得到 `cd /d "<path>" && <command>`,
    //   在同一窗口内依次执行,跑完窗口保留。
    let cmdline = build_start_cmdline(path, title, command);
    use std::os::windows::process::CommandExt;
    hidden(Command::new("cmd")).raw_arg(&cmdline).spawn()?;
    Ok(())
}

/// 构造外层 cmd 的命令串:`/C start "<title>" cmd /K "<inner>"`
#[cfg(windows)]
fn build_start_cmdline(path: &str, title: &str, command: Option<&str>) -> String {
    // title 是展示文本,剥掉 cmd 元字符,避免打乱引号配对;path 剥掉双引号
    let title = sanitize_cmd_text(title);
    let path = path.replace('"', "");
    let inner = match command {
        Some(c) => format!("cd /d \"{path}\" && {c}"),
        None => format!("cd /d \"{path}\""),
    };
    format!("/C start \"{title}\" cmd /K \"{inner}\"")
}

/// 剥掉会破坏 cmd 命令行解析的元字符(仅用于 title 这类展示文本;
/// 用户命令原样透传,允许包含 && | 等 shell 操作符)
#[cfg(windows)]
fn sanitize_cmd_text(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '"' | '&' | '|' | '<' | '>' | '^'))
        .collect()
}

/// 定位 wt.exe:先查 AppX 执行别名(Store / 官网安装都会注册),
/// 再用 where 搜 PATH(覆盖 scoop / choco / 绿色版等安装方式);找不到返回 None
#[cfg(windows)]
fn find_wt() -> Option<String> {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let alias = format!(r"{local}\Microsoft\WindowsApps\wt.exe");
        if std::path::Path::new(&alias).exists() {
            return Some(alias);
        }
    }
    let probe = hidden(Command::new("where")).arg("wt").output();
    match probe {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string),
        _ => None,
    }
}

/// 构造 wt 参数:`wt --title "<title>" -d "<path>" [cmd /k "<command>"]`
/// 不带命令时由 wt 打开默认配置文件的 shell;带命令时套 cmd /k,跑完窗口保留
#[cfg(windows)]
fn build_wt_args(path: &str, title: &str, command: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "--title".into(),
        sanitize_cmd_text(title),
        "-d".into(),
        path.replace('"', ""),
    ];
    if let Some(c) = command {
        args.extend(["cmd".into(), "/k".into(), c.into()]);
    }
    args
}

#[cfg(target_os = "macos")]
pub fn spawn_terminal(path: &str, _title: &str, command: Option<&str>) -> AppResult<()> {
    let inner = match command {
        Some(c) => format!(
            "cd '{}' && {}",
            path.replace('\'', "'\\''"),
            c.replace('"', "\\\"")
        ),
        None => format!("cd '{}'", path.replace('\'', "'\\''")),
    };
    let script = format!("tell application \"Terminal\" to do script \"{inner}\"");
    Command::new("osascript").args(["-e", &script]).spawn()?;
    Ok(())
}

#[cfg(all(not(windows), not(target_os = "macos")))]
pub fn spawn_terminal(_path: &str, _title: &str, _command: Option<&str>) -> AppResult<()> {
    Err(AppError::External("当前平台暂不支持打开终端".into()))
}

fn open_vscode(path: &str) -> AppResult<()> {
    #[cfg(windows)]
    hidden(Command::new("cmd"))
        .args(["/C", "code", path])
        .spawn()?;
    #[cfg(not(windows))]
    Command::new("code").arg(path).spawn()?;
    Ok(())
}

fn open_explorer(path: &str) -> AppResult<()> {
    #[cfg(windows)]
    Command::new("explorer").arg(path).spawn()?;
    #[cfg(target_os = "macos")]
    Command::new("open").arg(path).spawn()?;
    #[cfg(all(not(windows), not(target_os = "macos")))]
    return Err(AppError::External("当前平台暂不支持打开文件管理器".into()));
    Ok(())
}

fn probe_vscode() -> bool {
    #[cfg(windows)]
    let probe = hidden(Command::new("cmd"))
        .args(["/C", "code", "--version"])
        .output();
    #[cfg(not(windows))]
    let probe = Command::new("code").arg("--version").output();
    matches!(probe, Ok(out) if out.status.success())
}

#[tauri::command]
pub fn open_with(path: String, kind: EditorKind) -> AppResult<()> {
    if !std::path::Path::new(&path).is_dir() {
        return Err(AppError::Invalid(format!("目录不存在: {path}")));
    }
    match kind {
        EditorKind::Vscode => open_vscode(&path),
        EditorKind::Explorer => open_explorer(&path),
        EditorKind::Terminal => spawn_terminal(&path, "Terminal", None),
    }
}

/// 探测 code 命令,结果缓存进 settings(首次调用真实探测)
#[tauri::command]
pub fn detect_vscode(db: State<'_, Db>) -> AppResult<bool> {
    let conn = db.0.lock().unwrap();
    if let Some(cached) = db::get_setting(&conn, VSCODE_SETTING_KEY)? {
        return Ok(cached == "1");
    }
    let available = probe_vscode();
    db::set_setting(&conn, VSCODE_SETTING_KEY, if available { "1" } else { "0" })?;
    Ok(available)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn settings_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        db::init(&conn).unwrap();
        assert_eq!(db::get_setting(&conn, "k").unwrap(), None);
        db::set_setting(&conn, "k", "v1").unwrap();
        db::set_setting(&conn, "k", "v2").unwrap();
        assert_eq!(db::get_setting(&conn, "k").unwrap().as_deref(), Some("v2"));
    }

    #[test]
    fn editor_kind_deserialize() {
        let kind: EditorKind = serde_json::from_str("\"vscode\"").unwrap();
        assert!(matches!(kind, EditorKind::Vscode));
        let kind: EditorKind = serde_json::from_str("\"explorer\"").unwrap();
        assert!(matches!(kind, EditorKind::Explorer));
        let kind: EditorKind = serde_json::from_str("\"terminal\"").unwrap();
        assert!(matches!(kind, EditorKind::Terminal));
        assert!(serde_json::from_str::<EditorKind>("\"word\"").is_err());
    }

    #[test]
    fn probe_vscode_does_not_panic() {
        let _available = probe_vscode();
    }

    /// 回归测试:终端必须通过 隐藏外层 cmd + start 启动。
    /// 直接 CREATE_NEW_CONSOLE 起 cmd 时,Rust 会把父进程的标准句柄透传给子进程,
    /// 命令输出写到父进程的 dev 终端/管道,新窗口里什么都看不到;
    /// start 拉起的进程才会拿到全新控制台的输入/输出句柄。
    /// 同时 `&&` 必须位于 cmd /K 的引号串内,避免在外层被顶层切分。
    #[cfg(windows)]
    #[test]
    fn start_cmdline_with_command() {
        let s = build_start_cmdline(r"D:\code\foo", "Project: my-app", Some("npm run dev"));
        assert_eq!(
            s,
            r#"/C start "Project: my-app" cmd /K "cd /d "D:\code\foo" && npm run dev""#
        );
    }

    #[cfg(windows)]
    #[test]
    fn start_cmdline_without_command() {
        let s = build_start_cmdline(r"D:\code\foo bar", "Terminal", None);
        assert_eq!(s, r#"/C start "Terminal" cmd /K "cd /d "D:\code\foo bar"""#);
    }

    #[cfg(windows)]
    #[test]
    fn start_cmdline_sanitizes_display_text_but_keeps_command() {
        // path / title 中的引号与 cmd 元字符必须剥掉,避免打乱引号配对
        let s = build_start_cmdline(r#"D:\weird"path"#, r#"a&b"c"#, Some("echo hi"));
        assert_eq!(
            s,
            r#"/C start "abc" cmd /K "cd /d "D:\weirdpath" && echo hi""#
        );
        // 用户命令原样透传,允许 shell 操作符
        let s = build_start_cmdline(r"D:\p", "t", Some("cargo build && cargo run"));
        assert!(s.ends_with("&& cargo build && cargo run\""));
    }

    #[cfg(windows)]
    #[test]
    fn wt_args_without_command() {
        let args = build_wt_args(r"D:\code\foo bar", "Terminal", None);
        assert_eq!(args, ["--title", "Terminal", "-d", r"D:\code\foo bar"]);
    }

    #[cfg(windows)]
    #[test]
    fn wt_args_with_command_keeps_window() {
        // 带命令时必须套 cmd /k,保证跑完窗口保留
        let args = build_wt_args(r"D:\code\foo", "Project: my-app", Some("npm run dev"));
        assert_eq!(
            args,
            [
                "--title",
                "Project: my-app",
                "-d",
                r"D:\code\foo",
                "cmd",
                "/k",
                "npm run dev"
            ]
        );
    }

    #[cfg(windows)]
    #[test]
    fn wt_args_sanitizes_display_text_but_keeps_command() {
        let args = build_wt_args(r#"D:\weird"path"#, r#"a&b"c"#, Some("cargo build && cargo run"));
        assert_eq!(args[1], "abc");
        assert_eq!(args[3], r"D:\weirdpath");
        assert_eq!(args[6], "cargo build && cargo run");
    }

    #[cfg(windows)]
    #[test]
    fn find_wt_does_not_panic() {
        let _wt = find_wt();
    }
}
