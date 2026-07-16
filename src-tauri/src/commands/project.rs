use rusqlite::{params, params_from_iter, Connection, OptionalExtension, ToSql};
use tauri::State;

use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{Project, Tag};

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

struct ProjectRow {
    id: i64,
    path: String,
    name: String,
    description: String,
    created_at: i64,
    updated_at: i64,
}

const PROJECT_COLS: &str = "id, path, name, description, created_at, updated_at";

fn map_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRow> {
    Ok(ProjectRow {
        id: r.get(0)?,
        path: r.get(1)?,
        name: r.get(2)?,
        description: r.get(3)?,
        created_at: r.get(4)?,
        updated_at: r.get(5)?,
    })
}

pub fn load_tags(conn: &Connection, project_id: i64) -> AppResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color
         FROM tags t
         JOIN project_tags pt ON pt.tag_id = t.id
         WHERE pt.project_id = ?1
         ORDER BY t.name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map(params![project_id], |r| {
        Ok(Tag {
            id: r.get(0)?,
            name: r.get(1)?,
            color: r.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn with_tags(conn: &Connection, row: ProjectRow) -> AppResult<Project> {
    let tags = load_tags(conn, row.id)?;
    Ok(Project {
        id: row.id,
        path: row.path,
        name: row.name,
        description: row.description,
        tags,
        git: None,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub fn add(conn: &Connection, path: &str, name: &str) -> AppResult<Project> {
    if !std::path::Path::new(path).is_dir() {
        return Err(AppError::Invalid(format!("目录不存在: {path}")));
    }
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("名称不能为空".into()));
    }
    let ts = now();
    conn.execute(
        "INSERT INTO projects (path, name, description, created_at, updated_at)
         VALUES (?1, ?2, '', ?3, ?4)",
        params![path, name, ts, ts],
    )
    .map_err(|e| match e {
        rusqlite::Error::SqliteFailure(err, _)
            if err.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            AppError::Conflict(format!("项目已存在: {path}"))
        }
        other => AppError::Db(other),
    })?;
    get(conn, conn.last_insert_rowid())
}

pub fn get(conn: &Connection, id: i64) -> AppResult<Project> {
    let sql = format!("SELECT {PROJECT_COLS} FROM projects WHERE id = ?1");
    let row = conn.query_row(&sql, params![id], map_row).optional()?;
    match row {
        Some(r) => with_tags(conn, r),
        None => Err(AppError::NotFound(format!("project {id}"))),
    }
}

pub fn list(
    conn: &Connection,
    query: Option<String>,
    tag_ids: Option<Vec<i64>>,
) -> AppResult<Vec<Project>> {
    let mut sql = format!("SELECT {PROJECT_COLS} FROM projects");
    let mut conditions: Vec<String> = Vec::new();
    let mut binds: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(q) = query.filter(|q| !q.trim().is_empty()) {
        conditions.push("name LIKE ?".to_string());
        binds.push(Box::new(format!("%{}%", q.trim())));
    }
    if let Some(ids) = tag_ids.filter(|v| !v.is_empty()) {
        let placeholders = vec!["?"; ids.len()].join(",");
        conditions.push(format!(
            "id IN (SELECT project_id FROM project_tags WHERE tag_id IN ({placeholders}))"
        ));
        for id in ids {
            binds.push(Box::new(id));
        }
    }
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    sql.push_str(" ORDER BY name COLLATE NOCASE");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(binds.iter()), map_row)?;
    let mut projects = Vec::new();
    for row in rows {
        projects.push(with_tags(conn, row?)?);
    }
    Ok(projects)
}

pub fn update(conn: &Connection, id: i64, name: &str, description: &str) -> AppResult<Project> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("名称不能为空".into()));
    }
    let changed = conn.execute(
        "UPDATE projects SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
        params![name, description, now(), id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("project {id}")));
    }
    get(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> AppResult<()> {
    let changed = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("project {id}")));
    }
    Ok(())
}

// ---- Tauri 命令包装 ----

#[tauri::command]
pub fn add_project(db: State<'_, Db>, path: String, name: String) -> AppResult<Project> {
    let conn = db.0.lock().unwrap();
    add(&conn, &path, &name)
}

#[tauri::command]
pub fn list_projects(
    db: State<'_, Db>,
    query: Option<String>,
    tag_ids: Option<Vec<i64>>,
) -> AppResult<Vec<Project>> {
    let conn = db.0.lock().unwrap();
    list(&conn, query, tag_ids)
}

#[tauri::command]
pub fn get_project(db: State<'_, Db>, id: i64) -> AppResult<Project> {
    let conn = db.0.lock().unwrap();
    let project = get(&conn, id)?;
    Ok(project)
}

#[tauri::command]
pub fn update_project(
    db: State<'_, Db>,
    id: i64,
    name: String,
    description: String,
) -> AppResult<Project> {
    let conn = db.0.lock().unwrap();
    update(&conn, id, &name, &description)
}

#[tauri::command]
pub fn delete_project(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().unwrap();
    delete(&conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::init(&conn).unwrap();
        conn
    }

    #[test]
    fn add_get_list_delete_roundtrip() {
        let conn = test_conn();
        let dir = std::env::temp_dir().to_string_lossy().to_string();

        let p = add(&conn, &dir, "demo").unwrap();
        assert_eq!(p.name, "demo");
        assert!(p.tags.is_empty());
        assert!(p.git.is_none());

        let fetched = get(&conn, p.id).unwrap();
        assert_eq!(fetched.path, p.path);

        let all = list(&conn, None, None).unwrap();
        assert_eq!(all.len(), 1);

        delete(&conn, p.id).unwrap();
        assert!(matches!(get(&conn, p.id), Err(AppError::NotFound(_))));
        assert!(matches!(delete(&conn, p.id), Err(AppError::NotFound(_))));
    }

    #[test]
    fn duplicate_path_conflicts() {
        let conn = test_conn();
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        add(&conn, &dir, "a").unwrap();
        assert!(matches!(
            add(&conn, &dir, "b"),
            Err(AppError::Conflict(_))
        ));
    }

    #[test]
    fn rejects_bad_input() {
        let conn = test_conn();
        assert!(matches!(
            add(&conn, "C:/definitely/not/exist", "x"),
            Err(AppError::Invalid(_))
        ));
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        assert!(matches!(
            add(&conn, &dir, "   "),
            Err(AppError::Invalid(_))
        ));
    }

    #[test]
    fn update_changes_fields() {
        let conn = test_conn();
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        let p = add(&conn, &dir, "old").unwrap();
        let p2 = update(&conn, p.id, "new", "desc").unwrap();
        assert_eq!(p2.name, "new");
        assert_eq!(p2.description, "desc");
        assert!(p2.updated_at >= p.updated_at);
        assert!(matches!(
            update(&conn, 9999, "x", ""),
            Err(AppError::NotFound(_))
        ));
    }

    #[test]
    fn list_filters_by_name_and_tags() {
        let conn = test_conn();
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        let dir_b = std::env::temp_dir().join("projectdev-test-beta");
        std::fs::create_dir_all(&dir_b).unwrap();
        let dir_b = dir_b.to_string_lossy().to_string();
        let a = add(&conn, &dir, "Alpha").unwrap();
        let _b = add(&conn, &dir_b, "Beta").unwrap();

        let hit = list(&conn, Some("alph".into()), None).unwrap();
        assert_eq!(hit.len(), 1);
        assert_eq!(hit[0].name, "Alpha");

        // 直接造标签数据验证 tag_ids 过滤
        conn.execute("INSERT INTO tags (name, color) VALUES ('work', '#fff')", [])
            .unwrap();
        let tag_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO project_tags (project_id, tag_id) VALUES (?1, ?2)",
            params![a.id, tag_id],
        )
        .unwrap();

        let filtered = list(&conn, None, Some(vec![tag_id])).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].tags.len(), 1);
        assert_eq!(filtered[0].tags[0].name, "work");

        let empty = list(&conn, None, Some(vec![tag_id + 100])).unwrap();
        assert!(empty.is_empty());
    }
}
