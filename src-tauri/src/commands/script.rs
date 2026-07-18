use rusqlite::{params, Connection, OptionalExtension};
use tauri::State;

use crate::commands::open::spawn_terminal;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{CustomCommand, PackageScript};

/// 解析 <path>/package.json 的 scripts 字段;文件缺失或解析失败返回空列表
pub fn package_scripts(path: &str) -> AppResult<Vec<PackageScript>> {
    let file = std::path::Path::new(path).join("package.json");
    let Ok(content) = std::fs::read_to_string(&file) else {
        return Ok(vec![]);
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Ok(vec![]);
    };
    let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) else {
        return Ok(vec![]);
    };
    Ok(scripts
        .iter()
        .filter_map(|(name, cmd)| {
            cmd.as_str().map(|c| PackageScript {
                name: name.clone(),
                command: c.to_string(),
            })
        })
        .collect())
}

fn map_command_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<CustomCommand> {
    Ok(CustomCommand {
        id: r.get(0)?,
        project_id: r.get(1)?,
        name: r.get(2)?,
        command: r.get(3)?,
        description: r.get(4)?,
        sort_order: r.get(5)?,
    })
}

const COMMAND_COLS: &str = "id, project_id, name, command, description, sort_order";

pub fn list_commands(conn: &Connection, project_id: i64) -> AppResult<Vec<CustomCommand>> {
    let sql = format!(
        "SELECT {COMMAND_COLS} FROM custom_commands WHERE project_id = ?1 ORDER BY sort_order, id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![project_id], map_command_row)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn get_command(conn: &Connection, id: i64) -> AppResult<CustomCommand> {
    let sql = format!("SELECT {COMMAND_COLS} FROM custom_commands WHERE id = ?1");
    conn.query_row(&sql, params![id], map_command_row)
        .optional()?
        .ok_or_else(|| AppError::NotFound(format!("custom command {id}")))
}

fn validate_command(name: &str, command: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::Invalid("名称不能为空".into()));
    }
    if command.trim().is_empty() {
        return Err(AppError::Invalid("命令不能为空".into()));
    }
    Ok(())
}

fn to_conflict(e: rusqlite::Error, name: &str) -> AppError {
    match e {
        rusqlite::Error::SqliteFailure(err, _)
            if err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::Conflict(format!("命令名已存在: {name}"))
        }
        other => AppError::Db(other),
    }
}

pub fn create_command(
    conn: &Connection,
    project_id: i64,
    name: &str,
    command: &str,
    description: &str,
) -> AppResult<CustomCommand> {
    validate_command(name, command)?;
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM projects WHERE id = ?1",
        params![project_id],
        |r| r.get(0),
    )?;
    if !exists {
        return Err(AppError::NotFound(format!("project {project_id}")));
    }
    let next_order: i64 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM custom_commands WHERE project_id = ?1",
        params![project_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO custom_commands (project_id, name, command, description, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            project_id,
            name.trim(),
            command.trim(),
            description.trim(),
            next_order
        ],
    )
    .map_err(|e| to_conflict(e, name))?;
    get_command(conn, conn.last_insert_rowid())
}

pub fn update_command(
    conn: &Connection,
    id: i64,
    name: &str,
    command: &str,
    description: &str,
) -> AppResult<CustomCommand> {
    validate_command(name, command)?;
    let changed = conn
        .execute(
            "UPDATE custom_commands SET name = ?1, command = ?2, description = ?3 WHERE id = ?4",
            params![name.trim(), command.trim(), description.trim(), id],
        )
        .map_err(|e| to_conflict(e, name))?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("custom command {id}")));
    }
    get_command(conn, id)
}

pub fn delete_command(conn: &Connection, id: i64) -> AppResult<()> {
    let changed = conn.execute("DELETE FROM custom_commands WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("custom command {id}")));
    }
    Ok(())
}

// ---- Tauri 命令包装 ----

#[tauri::command]
pub fn list_package_scripts(path: String) -> AppResult<Vec<PackageScript>> {
    package_scripts(&path)
}

#[tauri::command]
pub fn list_custom_commands(db: State<'_, Db>, project_id: i64) -> AppResult<Vec<CustomCommand>> {
    let conn = db.0.lock().unwrap();
    list_commands(&conn, project_id)
}

#[tauri::command]
pub fn create_custom_command(
    db: State<'_, Db>,
    project_id: i64,
    name: String,
    command: String,
    description: String,
) -> AppResult<CustomCommand> {
    let conn = db.0.lock().unwrap();
    create_command(&conn, project_id, &name, &command, &description)
}

#[tauri::command]
pub fn update_custom_command(
    db: State<'_, Db>,
    id: i64,
    name: String,
    command: String,
    description: String,
) -> AppResult<CustomCommand> {
    let conn = db.0.lock().unwrap();
    update_command(&conn, id, &name, &command, &description)
}

#[tauri::command]
pub fn delete_custom_command(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().unwrap();
    delete_command(&conn, id)
}

/// 在系统终端执行命令(新窗口,跑完不关)
#[tauri::command]
pub fn run_in_terminal(path: String, project_name: String, command: String) -> AppResult<()> {
    if !std::path::Path::new(&path).is_dir() {
        return Err(AppError::Invalid(format!("目录不存在: {path}")));
    }
    spawn_terminal(&path, &format!("Project: {project_name}"), Some(&command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::project;
    use crate::db;
    use std::fs;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::init(&conn).unwrap();
        conn
    }

    fn add_project(conn: &Connection) -> i64 {
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        project::add(conn, &dir, "demo").unwrap().id
    }

    #[test]
    fn parses_package_scripts() {
        let dir = std::env::temp_dir().join(format!(
            "projectdev-pkg-test-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        assert!(package_scripts(dir.to_str().unwrap()).unwrap().is_empty());

        fs::write(dir.join("package.json"), "{ not json").unwrap();
        assert!(package_scripts(dir.to_str().unwrap()).unwrap().is_empty());

        fs::write(
            dir.join("package.json"),
            r#"{"scripts":{"dev":"vite","build":"vite build"}}"#,
        )
        .unwrap();
        let scripts = package_scripts(dir.to_str().unwrap()).unwrap();
        assert_eq!(scripts.len(), 2);
        assert!(scripts
            .iter()
            .any(|s| s.name == "dev" && s.command == "vite"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn custom_command_crud() {
        let conn = test_conn();
        let pid = add_project(&conn);

        let c = create_command(&conn, pid, "clean", "rm -rf dist", "清理产物").unwrap();
        assert_eq!(c.sort_order, 0);
        let c2 = create_command(&conn, pid, "logs", "tail -f app.log", "").unwrap();
        assert_eq!(c2.sort_order, 1);

        let list = list_commands(&conn, pid).unwrap();
        assert_eq!(list.len(), 2);

        let updated = update_command(&conn, c.id, "clean2", "rm -rf build", "改后").unwrap();
        assert_eq!(updated.name, "clean2");
        assert_eq!(updated.description, "改后");

        // 重名冲突
        assert!(matches!(
            update_command(&conn, c.id, "logs", "x", ""),
            Err(AppError::Conflict(_))
        ));

        delete_command(&conn, c2.id).unwrap();
        assert_eq!(list_commands(&conn, pid).unwrap().len(), 1);
        assert!(matches!(
            delete_command(&conn, c2.id),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn custom_command_validates_and_cascades() {
        let conn = test_conn();
        let pid = add_project(&conn);
        assert!(matches!(
            create_command(&conn, pid, "", "x", ""),
            Err(AppError::Invalid(_))
        ));
        assert!(matches!(
            create_command(&conn, 9999, "a", "b", ""),
            Err(AppError::NotFound(_))
        ));

        create_command(&conn, pid, "a", "b", "").unwrap();
        // 项目已无删除入口(仅归档),这里用原生 SQL 验证 schema 的外键级联仍然有效
        conn.execute("DELETE FROM projects WHERE id = ?1", [pid])
            .unwrap();
        assert!(list_commands(&conn, pid).unwrap().is_empty());
    }
}
