//! 窗口控制命令:托盘迷你弹窗与主窗口的显示/隐藏。

use tauri::AppHandle;

use crate::error::AppResult;
use crate::tray;

/// 显示主窗口并聚焦;带 project_id 时前端跳转到该项目详情页。
#[tauri::command]
pub fn show_main_window(app: AppHandle, project_id: Option<i64>) -> AppResult<()> {
    tray::show_main_window(&app, project_id);
    Ok(())
}

/// 隐藏托盘迷你弹窗(如弹窗内按下 Esc)。
#[tauri::command]
pub fn hide_tray_popup(app: AppHandle) -> AppResult<()> {
    tray::hide_popup(&app);
    Ok(())
}
