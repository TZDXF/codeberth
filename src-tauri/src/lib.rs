mod commands;
mod db;
mod error;
mod models;
mod scheduler;
mod workday;

use std::sync::Arc;

use db::Db;
use tauri::Manager;
use tokio::sync::Notify;

/// 应用数据目录名(位于用户主目录下)
pub(crate) const APP_DATA_DIR_NAME: &str = ".pm";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            // 数据库文件: ~/.pm/projects.db
            // (Windows: C:\Users\<user>\.pm\projects.db)
            let dir = app.path().home_dir()?.join(APP_DATA_DIR_NAME);
            let db = Db::open(&dir.join("projects.db"))?;
            app.manage(db);

            // 调度通知:用于定时任务变更时唤醒后台 scheduler
            let notify = Arc::new(Notify::new());
            app.manage(commands::report::ScheduleNotify(notify));

            // 启动日报定时调度器(后台 tokio 任务,仅 App 运行时生效)
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                scheduler::run(handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::add_project,
            commands::project::list_projects,
            commands::project::get_project,
            commands::project::update_project,
            commands::project::archive_project,
            commands::project::list_archived_projects,
            commands::project::unarchive_project,
            commands::project::delete_project,
            commands::git::get_git_status,
            commands::git::list_git_remotes,
            commands::git::fetch_git_remote_async,
            commands::git::list_git_branches,
            commands::git::git_checkout,
            commands::git::git_commit,
            commands::git::git_pull,
            commands::git::git_push,
            commands::git::git_commit_context,
            commands::git::git_log,
            commands::git::git_current_user,
            commands::open::open_with,
            commands::open::detect_vscode,
            commands::prompt::get_ai_prompts,
            commands::prompt::set_ai_prompts,
            commands::prompt::open_prompts_dir,
            commands::tag::list_tags,
            commands::tag::create_tag,
            commands::tag::update_tag,
            commands::tag::delete_tag,
            commands::tag::set_project_tags,
            commands::script::list_package_scripts,
            commands::script::list_custom_commands,
            commands::script::create_custom_command,
            commands::script::update_custom_command,
            commands::script::delete_custom_command,
            commands::script::run_in_terminal,
            commands::files::read_readme,
            commands::files::scan_compose_files,
            commands::docker::compose_ps,
            commands::hidden::list_hidden_items,
            commands::hidden::set_hidden_item,
            commands::report::save_report_history,
            commands::report::list_report_history,
            commands::report::get_report_history,
            commands::report::delete_report_history,
            commands::report::get_calendar_meta,
            commands::report::get_reports_by_date,
            commands::report::get_work_week_ranges,
            commands::report::list_report_schedules,
            commands::report::save_report_schedules,
            commands::report::run_report_schedule_now,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
