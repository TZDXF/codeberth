//! 系统托盘:图标/菜单、迷你项目列表弹窗的创建与定位。

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_store::StoreExt;

use crate::APP_DATA_DIR_NAME;

/// 迷你项目列表弹窗的窗口 label
pub(crate) const TRAY_POPUP_LABEL: &str = "tray-popup";
/// 主窗口 label(tauri.conf.json 默认)
pub(crate) const MAIN_WINDOW_LABEL: &str = "main";

const POPUP_WIDTH: f64 = 360.0;
const POPUP_HEIGHT: f64 = 480.0;
/// 弹窗与托盘图标/屏幕边缘的间距(px)
const POPUP_MARGIN: f64 = 12.0;
/// 单击响应延迟:双击事件总是先于第二击的 DoubleClick 到达两次 Click,
/// 延迟等待以区分单击/双击,避免双击时弹窗闪现
const CLICK_DELAY: Duration = Duration::from_millis(300);

/// 单击代际计数:每次单击/双击 +1,延迟任务仅在自己仍是最后一代时才执行
static CLICK_GENERATION: AtomicU64 = AtomicU64::new(0);
/// 最近一次双击的时间戳(ms,UNIX epoch):双击后系统还会补发一次 Click(Up),
/// 用它来丢弃这次尾随单击,避免双击时弹窗被重新打开
static LAST_DOUBLE_CLICK_MS: AtomicU64 = AtomicU64::new(0);
/// 双击后忽略尾随单击的时间窗口
const DOUBLE_CLICK_SUPPRESS: Duration = Duration::from_millis(500);

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 从 ~/.repomeow/settings.json 读取字符串设置项(与前端 tauri-plugin-store 同一文件)。
/// 读取失败返回 None,调用方自行回退默认值。
pub(crate) fn read_setting_string(app: &AppHandle, key: &str) -> Option<String> {
    let path = app
        .path()
        .home_dir()
        .ok()?
        .join(APP_DATA_DIR_NAME)
        .join("settings.json");
    let store = app.store(path).ok()?;
    store.get(key)?.as_str().map(str::to_owned)
}

/// 创建托盘图标:左键单击切换迷你弹窗,左键双击显示主窗口,右键打开菜单。
pub(crate) fn setup(app: &App) -> tauri::Result<()> {
    let lang = read_setting_string(&app.handle(), "language").unwrap_or_default();
    let (open_text, quit_text) = if lang == "en-US" {
        ("Show RepoMeow", "Quit")
    } else {
        ("显示主窗口", "退出")
    };
    let open_item = MenuItem::with_id(app, "tray-open", open_text, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "tray-quit", quit_text, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .tooltip("RepoMeow")
        // 左键留给单击/双击弹窗行为,菜单仅在右键弹出
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "tray-open" => show_main_window(app, None),
            "tray-quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let app = tray.app_handle();
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    position,
                    ..
                } => {
                    // 双击尾巴上的 Click(Up):直接忽略,不再排入延迟任务
                    let since_double = now_millis().saturating_sub(LAST_DOUBLE_CLICK_MS.load(Ordering::SeqCst));
                    if since_double < DOUBLE_CLICK_SUPPRESS.as_millis() as u64 {
                        return;
                    }
                    // 延迟触发:若随后到来双击则该代作废,弹窗不会闪现
                    let generation = CLICK_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
                    let app = app.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(CLICK_DELAY);
                        if CLICK_GENERATION.load(Ordering::SeqCst) == generation {
                            toggle_popup(&app, position);
                        }
                    });
                }
                TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => {
                    // 作废待处理的单击任务并记录双击时间(抑制随后的尾随 Click),直接打开主窗口
                    CLICK_GENERATION.fetch_add(1, Ordering::SeqCst);
                    LAST_DOUBLE_CLICK_MS.store(now_millis(), Ordering::SeqCst);
                    show_main_window(app, None);
                }
                _ => {}
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// 显示并聚焦主窗口;带 project_id 时通知前端跳转项目详情页。
pub(crate) fn show_main_window(app: &AppHandle, project_id: Option<i64>) {
    hide_popup(app);
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        if let Some(id) = project_id {
            let _ = window.emit("main://navigate", serde_json::json!({ "projectId": id }));
        }
    }
}

/// 隐藏迷你弹窗(窗口保留以便下次快速显示)。
pub(crate) fn hide_popup(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(TRAY_POPUP_LABEL) {
        let _ = window.hide();
    }
}

fn toggle_popup(app: &AppHandle, anchor: PhysicalPosition<f64>) {
    if let Some(window) = app.get_webview_window(TRAY_POPUP_LABEL) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            return;
        }
    }
    show_popup(app, anchor);
}

/// 在托盘图标附近显示迷你弹窗(首次调用时懒创建窗口)。
fn show_popup(app: &AppHandle, anchor: PhysicalPosition<f64>) {
    let window = match app.get_webview_window(TRAY_POPUP_LABEL) {
        Some(window) => window,
        None => {
            let result = WebviewWindowBuilder::new(
                app,
                TRAY_POPUP_LABEL,
                WebviewUrl::App("index.html#/tray".into()),
            )
            .title("RepoMeow")
            .inner_size(POPUP_WIDTH, POPUP_HEIGHT)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .decorations(false)
            .skip_taskbar(true)
            .always_on_top(true)
            .visible(false)
            .build();
            match result {
                Ok(window) => window,
                Err(e) => {
                    eprintln!("failed to create tray popup window: {e}");
                    return;
                }
            }
        }
    };

    // 默认:水平居中于托盘点击点,位于其上方
    let mut x = anchor.x - POPUP_WIDTH / 2.0;
    let mut y = anchor.y - POPUP_HEIGHT - POPUP_MARGIN;
    if let Ok(Some(monitor)) = app.monitor_from_point(anchor.x, anchor.y) {
        let work = monitor.work_area();
        let (wx, wy) = (work.position.x as f64, work.position.y as f64);
        let (ww, wh) = (work.size.width as f64, work.size.height as f64);
        // 托盘位于工作区下半部分(任务栏在底部)时弹窗放上方,否则放下方
        y = if anchor.y > wy + wh / 2.0 {
            anchor.y - POPUP_HEIGHT - POPUP_MARGIN
        } else {
            anchor.y + POPUP_MARGIN
        };
        x = x.clamp(wx + 8.0, wx + ww - POPUP_WIDTH - 8.0);
        y = y.clamp(wy + 8.0, wy + wh - POPUP_HEIGHT - 8.0);
    }
    let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
    let _ = window.show();
    let _ = window.set_focus();
    // 通知弹窗刷新项目列表(主窗口的数据变更不会同步到弹窗的独立 Pinia 实例)
    let _ = window.emit("tray-popup://refresh", ());
}
