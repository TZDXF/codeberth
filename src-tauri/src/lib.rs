mod commands;
mod db;
mod error;
mod models;

use db::Db;
use tauri::Manager;

/// 应用数据目录名(位于用户主目录下)
const APP_DATA_DIR_NAME: &str = ".pm";

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
            commands::open::open_with,
            commands::open::detect_vscode,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
