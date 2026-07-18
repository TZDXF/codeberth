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
    // Windows `start` 命令的精确语法:`start ["<title>"] <command> [<args>...]`
    // - start 后只能接一个可选的引号包裹的 title,不能加空标题占位;
    //   否则 start 会把下一个 token 当成程序名去找,触发"找不到文件"。
    // - title 之后必须紧跟真正的程序名(我们这里是 `cmd`)。
    // - 必须用 raw_arg 直接控制命令行,因为 cmd.exe 不遵循 C runtime 转义规则,
    //   Rust 的 CommandLineToArgvW 自动加引号/转义会破坏 start 的解析。
    //   (见 Rust std::process::CommandExt::raw_arg 文档示例)
    // - cmd /C 的命令行外面还要再包一层引号,这是 cmd.exe /C /K 的特殊要求。
    //
    // cmd 实际收到的命令行形如:
    //   cmd /C "start "Terminal" cmd /K cd /d "D:\path" && npm run dev""
    //
    // CREATE_NEW_CONSOLE(0x00000010):Tauri 主进程是 GUI subsystem,没有自己的控制台。
    // 默认情况下 CreateProcess 给子进程不分配新控制台,导致 cmd 进程"无窗口运行"。
    // 加 CREATE_NEW_CONSOLE 强制给 cmd 分配一个全新控制台,start 弹出的窗口才会真正可见。
    let inner_cmdline = build_windows_cmdline(path, title, command);
    let cmdline = format!("\"{inner_cmdline}\"");
    use std::os::windows::process::CommandExt;
    Command::new("cmd")
        .creation_flags(CREATE_NEW_CONSOLE)
        .raw_arg(&cmdline)
        .spawn()?;
    Ok(())
}

/// 给 cmd.exe 用的 CREATE_NEW_CONSOLE 标志常量
/// (不放在 hidden() 里,因为 open_vscode / probe_vscode 不需要新控制台)
#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

/// 构造 `start "<title>" cmd /K <inner>` 这一段命令行
/// (注意:不含外层 cmd.exe 包装,因为外层包装在 spawn_terminal 中用 raw_arg 完成)
#[cfg(windows)]
fn build_windows_cmdline(path: &str, title: &str, command: Option<&str>) -> String {
    // 剥掉 title / path 中的双引号,避免出现奇数个引号打乱 cmd 解析
    let title = title.replace('"', "");
    let path = path.replace('"', "");
    let inner = match command {
        Some(c) => format!("cd /d \"{path}\" && {c}"),
        None => format!("cd /d \"{path}\""),
    };
    // start 的语法:`start "<title>" cmd /K <inner>`
    // - title 用引号包裹,start 把它当作新窗口的标题栏字符串;
    // - 紧跟的 `cmd` 才是 start 要执行的真正程序;
    // - 注意不要加空标题占位(`start ""`),那会让 start 把第二个 token
    //   `"<title>"` 当成程序名去找,触发"找不到文件 <title>"。
    format!("start \"{title}\" cmd /K {inner}")
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

    /// 回归测试:确保 Windows 下 `start` 命令行使用 `start "<title>" cmd /K <inner>`
    /// 这种精确形式——start 后面只能接一个可选的引号包裹的 title,紧跟真正的程序名 cmd。
    /// 否则 start 会把 `<title>` 当成可执行文件名去找,触发
    /// "Windows 找不到文件 'Terminal'" / Microsoft Store 提示。
    #[cfg(windows)]
    #[test]
    fn windows_cmdline_quote_title() {
        let s = build_windows_cmdline(r"D:\code\foo bar", "Terminal", None);
        // 关键结构:start "<title>" cmd /K <inner>(中间不能有空标题占位)
        assert_eq!(s, "start \"Terminal\" cmd /K cd /d \"D:\\code\\foo bar\"");
        // 严禁空标题占位:这会让 start 把 "Terminal" 当成程序名去找
        assert!(!s.contains("start \"\""));
    }

    #[cfg(windows)]
    #[test]
    fn windows_cmdline_with_command() {
        let s = build_windows_cmdline(
            r"D:\code\foo",
            "Project: my-app",
            Some("npm run dev"),
        );
        assert_eq!(
            s,
            "start \"Project: my-app\" cmd /K cd /d \"D:\\code\\foo\" && npm run dev"
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_cmdline_strip_inner_quotes() {
        // path 含双引号时,必须被剥掉,否则外层 cd /d 会出现奇数个引号
        let s = build_windows_cmdline(r#"D:\weird"path"#, "Terminal", None);
        assert_eq!(s, "start \"Terminal\" cmd /K cd /d \"D:\\weirdpath\"");
    }
}
