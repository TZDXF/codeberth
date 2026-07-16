mod commands;
mod db;
mod error;
mod models;

use db::Db;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 数据库文件: %APPDATA%/com.projectdev.app/projects.db
            // (Mac: ~/Library/Application Support/com.projectdev.app/projects.db)
            let dir = app.path().app_data_dir()?;
            let db = Db::open(&dir.join("projects.db"))?;
            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::project::add_project,
            commands::project::list_projects,
            commands::project::get_project,
            commands::project::update_project,
            commands::project::delete_project,
            commands::git::get_git_status,
            commands::git::fetch_git_remote_async,
            commands::open::open_with,
            commands::open::detect_vscode,
            commands::tag::list_tags,
            commands::tag::create_tag,
            commands::tag::delete_tag,
            commands::tag::set_project_tags,
            commands::script::list_package_scripts,
            commands::script::list_custom_commands,
            commands::script::create_custom_command,
            commands::script::update_custom_command,
            commands::script::delete_custom_command,
            commands::script::run_in_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
