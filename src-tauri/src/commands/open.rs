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

/// 在系统终端打开目录,可选执行命令(跑完不关窗口)
#[cfg(windows)]
pub fn spawn_terminal(path: &str, title: &str, command: Option<&str>) -> AppResult<()> {
    let title = title.replace('"', "");
    let inner = match command {
        Some(c) => format!("cd /d \"{path}\" && {c}"),
        None => format!("cd /d \"{path}\""),
    };
    // 新 cmd 读注册表用户 PATH,覆盖 nvm 等场景
    hidden(Command::new("cmd"))
        .args(["/C", "start", &title, "cmd", "/K", &inner])
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn spawn_terminal(path: &str, _title: &str, command: Option<&str>) -> AppResult<()> {
    let inner = match command {
        Some(c) => format!("cd '{}' && {}", path.replace('\'', "'\\''"), c.replace('"', "\\\"")),
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
    hidden(Command::new("cmd")).args(["/C", "code", path]).spawn()?;
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
}
