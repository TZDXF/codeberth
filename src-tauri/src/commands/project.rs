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
    archived_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

const PROJECT_COLS: &str = "id, path, name, description, archived_at, created_at, updated_at";

fn map_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectRow> {
    Ok(ProjectRow {
        id: r.get(0)?,
        path: r.get(1)?,
        name: r.get(2)?,
        description: r.get(3)?,
        archived_at: r.get(4)?,
        created_at: r.get(5)?,
        updated_at: r.get(6)?,
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

fn project_from_row(row: ProjectRow, tags: Vec<Tag>) -> Project {
    Project {
        id: row.id,
        path: row.path,
        name: row.name,
        description: row.description,
        tags,
        git: None,
        archived_at: row.archived_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn load_tags_by_project(
    conn: &Connection,
    project_ids: &[i64],
) -> AppResult<std::collections::HashMap<i64, Vec<Tag>>> {
    let mut tags_by_project = std::collections::HashMap::new();
    if project_ids.is_empty() {
        return Ok(tags_by_project);
    }

    let placeholders = vec!["?"; project_ids.len()].join(",");
    let sql = format!(
        "SELECT pt.project_id, t.id, t.name, t.color
         FROM project_tags pt
         JOIN tags t ON pt.tag_id = t.id
         WHERE pt.project_id IN ({placeholders})
         ORDER BY pt.project_id, t.name COLLATE NOCASE"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(project_ids.iter()), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            Tag {
                id: r.get(1)?,
                name: r.get(2)?,
                color: r.get(3)?,
            },
        ))
    })?;
    for row in rows {
        let (project_id, tag) = row?;
        tags_by_project
            .entry(project_id)
            .or_insert_with(Vec::new)
            .push(tag);
    }
    Ok(tags_by_project)
}

fn with_tags(conn: &Connection, row: ProjectRow) -> AppResult<Project> {
    let tags = load_tags(conn, row.id)?;
    Ok(project_from_row(row, tags))
}

fn projects_with_tags(conn: &Connection, rows: Vec<ProjectRow>) -> AppResult<Vec<Project>> {
    let project_ids: Vec<_> = rows.iter().map(|row| row.id).collect();
    let mut tags_by_project = load_tags_by_project(conn, &project_ids)?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let project_id = row.id;
            project_from_row(row, tags_by_project.remove(&project_id).unwrap_or_default())
        })
        .collect())
}

pub fn add(conn: &Connection, path: &str, name: &str, description: &str) -> AppResult<Project> {
    if !std::path::Path::new(path).is_dir() {
        return Err(AppError::invalid_path(path));
    }
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("名称不能为空".into()));
    }
    let ts = now();
    conn.execute(
        "INSERT INTO projects (path, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![path, name, description.trim(), ts, ts],
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
        None => Err(AppError::project_not_found(id)),
    }
}

pub fn list(
    conn: &Connection,
    query: Option<String>,
    tag_ids: Option<Vec<i64>>,
) -> AppResult<Vec<Project>> {
    let mut sql = format!("SELECT {PROJECT_COLS} FROM projects");
    // 归档项目不出现在列表中(数据保留,但不展示、不获取 git 状态)
    let mut conditions: Vec<String> = vec!["archived_at IS NULL".to_string()];
    let mut binds: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(q) = query.filter(|q| !q.trim().is_empty()) {
        conditions.push("(name LIKE ? OR description LIKE ?)".to_string());
        let pattern = format!("%{}%", q.trim());
        binds.push(Box::new(pattern.clone()));
        binds.push(Box::new(pattern));
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
    let project_rows = rows.collect::<Result<Vec<_>, _>>()?;
    projects_with_tags(conn, project_rows)
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
        return Err(AppError::project_not_found(id));
    }
    get(conn, id)
}

/// 归档项目:软删除,保留历史数据(标签、自定义命令等关联数据不动)
pub fn archive(conn: &Connection, id: i64) -> AppResult<()> {
    let changed = conn.execute(
        "UPDATE projects SET archived_at = ?1 WHERE id = ?2",
        params![now(), id],
    )?;
    if changed == 0 {
        return Err(AppError::project_not_found(id));
    }
    Ok(())
}

/// 列出已归档项目(按归档时间倒序,设置页归档管理用)
pub fn list_archived(conn: &Connection) -> AppResult<Vec<Project>> {
    let sql = format!(
        "SELECT {PROJECT_COLS} FROM projects WHERE archived_at IS NOT NULL ORDER BY archived_at DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    let project_rows = rows.collect::<Result<Vec<_>, _>>()?;
    projects_with_tags(conn, project_rows)
}

/// 取消归档:恢复到项目列表
pub fn unarchive(conn: &Connection, id: i64) -> AppResult<()> {
    let changed = conn.execute(
        "UPDATE projects SET archived_at = NULL WHERE id = ?1 AND archived_at IS NOT NULL",
        params![id],
    )?;
    if changed == 0 {
        return Err(AppError::project_not_found(id));
    }
    Ok(())
}

/// 彻底删除项目(关联的标签指派、自定义命令随外键级联清理;不动磁盘文件)
pub fn remove(conn: &Connection, id: i64) -> AppResult<()> {
    let changed = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(AppError::project_not_found(id));
    }
    Ok(())
}

// ---- Tauri 命令包装 ----

#[tauri::command]
pub fn add_project(
    db: State<'_, Db>,
    path: String,
    name: String,
    description: Option<String>,
) -> AppResult<Project> {
    let conn = db.0.lock().unwrap();
    add(&conn, &path, &name, description.as_deref().unwrap_or(""))
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
pub fn archive_project(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().unwrap();
    archive(&conn, id)
}

#[tauri::command]
pub fn list_archived_projects(db: State<'_, Db>) -> AppResult<Vec<Project>> {
    let conn = db.0.lock().unwrap();
    list_archived(&conn)
}

#[tauri::command]
pub fn unarchive_project(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().unwrap();
    unarchive(&conn, id)
}

#[tauri::command]
pub fn delete_project(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let conn = db.0.lock().unwrap();
    remove(&conn, id)
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
    fn archive_hides_from_list_but_keeps_data() {
        let conn = test_conn();
        let dir = std::env::temp_dir().to_string_lossy().to_string();

        let p = add(&conn, &dir, "demo", "").unwrap();
        assert_eq!(p.name, "demo");
        assert!(p.tags.is_empty());
        assert!(p.git.is_none());
        assert!(p.archived_at.is_none());

        let fetched = get(&conn, p.id).unwrap();
        assert_eq!(fetched.path, p.path);

        let all = list(&conn, None, None).unwrap();
        assert_eq!(all.len(), 1);

        archive(&conn, p.id).unwrap();
        // 归档后不再出现在列表中,但数据保留(get 仍可取到)
        assert!(list(&conn, None, None).unwrap().is_empty());
        let archived = get(&conn, p.id).unwrap();
        assert!(archived.archived_at.is_some());

        assert!(
            matches!(archive(&conn, 9999), Err(ref e) if e.is_code(crate::error::ErrorCode::ProjectNotFound))
        );
    }

    #[test]
    fn unarchive_restores_to_list() {
        let conn = test_conn();
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        let p = add(&conn, &dir, "demo", "").unwrap();
        archive(&conn, p.id).unwrap();

        // 归档列表按归档时间倒序返回
        let archived = list_archived(&conn).unwrap();
        assert_eq!(archived.len(), 1);
        assert_eq!(archived[0].id, p.id);
        assert!(archived[0].archived_at.is_some());

        unarchive(&conn, p.id).unwrap();
        assert!(list_archived(&conn).unwrap().is_empty());
        assert_eq!(list(&conn, None, None).unwrap().len(), 1);
        assert!(get(&conn, p.id).unwrap().archived_at.is_none());

        // 未归档 / 不存在的项目
        assert!(
            matches!(unarchive(&conn, p.id), Err(ref e) if e.is_code(crate::error::ErrorCode::ProjectNotFound))
        );
        assert!(
            matches!(unarchive(&conn, 9999), Err(ref e) if e.is_code(crate::error::ErrorCode::ProjectNotFound))
        );
    }

    #[test]
    fn remove_deletes_permanently() {
        let conn = test_conn();
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        let p = add(&conn, &dir, "demo", "").unwrap();
        archive(&conn, p.id).unwrap();

        remove(&conn, p.id).unwrap();
        assert!(
            matches!(get(&conn, p.id), Err(ref e) if e.is_code(crate::error::ErrorCode::ProjectNotFound))
        );
        assert!(list_archived(&conn).unwrap().is_empty());
        assert!(
            matches!(remove(&conn, p.id), Err(ref e) if e.is_code(crate::error::ErrorCode::ProjectNotFound))
        );
    }

    #[test]
    fn duplicate_path_conflicts() {
        let conn = test_conn();
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        add(&conn, &dir, "a", "").unwrap();
        assert!(matches!(
            add(&conn, &dir, "b", ""),
            Err(AppError::Conflict(_))
        ));
    }

    #[test]
    fn rejects_bad_input() {
        let conn = test_conn();
        assert!(matches!(add(&conn, "C:/definitely/not/exist", "x", ""),
                Err(ref e) if e.is_code(crate::error::ErrorCode::InvalidPath)));
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        assert!(matches!(
            add(&conn, &dir, "   ", ""),
            Err(AppError::Invalid(_))
        ));
    }

    #[test]
    fn update_changes_fields() {
        let conn = test_conn();
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        let p = add(&conn, &dir, "old", "").unwrap();
        let p2 = update(&conn, p.id, "new", "desc").unwrap();
        assert_eq!(p2.name, "new");
        assert_eq!(p2.description, "desc");
        assert!(p2.updated_at >= p.updated_at);
        assert!(
            matches!(update(&conn, 9999, "x", ""), Err(ref e) if e.is_code(crate::error::ErrorCode::ProjectNotFound))
        );
    }

    #[test]
    fn list_loads_tags_in_project_order_and_keeps_empty_projects() {
        let conn = test_conn();
        let dir = std::env::temp_dir();
        let a_path = dir.join("codeberth-batch-a");
        let b_path = dir.join("codeberth-batch-b");
        std::fs::create_dir_all(&a_path).unwrap();
        std::fs::create_dir_all(&b_path).unwrap();
        let a = add(&conn, &a_path.to_string_lossy(), "Alpha", "").unwrap();
        let b = add(&conn, &b_path.to_string_lossy(), "Beta", "").unwrap();
        conn.execute("INSERT INTO tags (name, color) VALUES ('zeta', '#z')", [])
            .unwrap();
        let zeta = conn.last_insert_rowid();
        conn.execute("INSERT INTO tags (name, color) VALUES ('alpha', '#a')", [])
            .unwrap();
        let alpha = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO project_tags (project_id, tag_id) VALUES (?1, ?2), (?1, ?3)",
            params![a.id, zeta, alpha],
        )
        .unwrap();

        let projects = list(&conn, None, None).unwrap();
        assert_eq!(
            projects.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![a.id, b.id]
        );
        assert_eq!(
            projects[0]
                .tags
                .iter()
                .map(|tag| tag.name.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "zeta"]
        );
        assert!(projects[1].tags.is_empty());
    }
    #[test]
    fn list_filters_by_name_and_tags() {
        let conn = test_conn();
        let dir = std::env::temp_dir().to_string_lossy().to_string();
        let dir_b = std::env::temp_dir().join("codeberth-test-beta");
        std::fs::create_dir_all(&dir_b).unwrap();
        let dir_b = dir_b.to_string_lossy().to_string();
        let a = add(&conn, &dir, "Alpha", "").unwrap();
        let _b = add(&conn, &dir_b, "Beta", "").unwrap();

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
