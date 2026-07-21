//! 日报历史与定时任务管理。
//!
//! * 日报历史存 SQLite(report_history + report_commits 表)
//! * 定时任务配置存 ~/.pm/report-schedules.json
//! * 定时任务变更时通过 Notify 唤醒后台 scheduler

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::NaiveDate;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tokio::sync::Notify;

use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::GitCommitInfo;
use crate::workday;
use crate::APP_DATA_DIR_NAME;

const SCHEDULES_FILE: &str = "report-schedules.json";

// ── types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSchedule {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub project_ids: Vec<i64>,
    #[serde(default = "default_author_mode")]
    pub author_mode: String,
    pub time_of_day: String,
    #[serde(default)]
    pub weekdays_only: bool,
    #[serde(default)]
    pub chinese_workday_only: bool,
    #[serde(default)]
    pub last_run_at: Option<i64>,
}

fn default_enabled() -> bool { true }
fn default_author_mode() -> String { "me".into() }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportHistoryItem {
    pub id: i64,
    pub project_ids: Vec<i64>,
    pub date_from: String,
    pub date_to: String,
    pub range_label: String,
    pub author_mode: String,
    pub language: String,
    pub created_at: i64,
    pub project_names: Vec<String>,
    pub total_commits: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportHistoryDetail {
    #[serde(flatten)]
    pub item: ReportHistoryItem,
    pub result: String,
    pub commits: Vec<ReportCommitItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportCommitItem {
    pub project_id: Option<i64>,
    pub project_name: String,
    pub project_description: String,
    pub commits: Vec<GitCommitInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveReportCommit {
    pub project_id: Option<i64>,
    pub project_name: String,
    #[serde(default)]
    pub project_description: String,
    pub commits: Vec<GitCommitInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportGeneratedPayload {
    pub schedule_name: String,
    pub history_id: i64,
    pub date_from: String,
    pub date_to: String,
}

// ── schedule file helpers ─────────────────────────────────────────────

fn schedules_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let home = app
        .path()
        .home_dir()
        .map_err(|e| AppError::External(e.to_string()))?;
    Ok(home.join(APP_DATA_DIR_NAME))
}

fn read_schedules(app: &AppHandle) -> AppResult<Vec<ReportSchedule>> {
    let dir = schedules_dir(app)?;
    let path = dir.join(SCHEDULES_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = fs::read_to_string(&path)?;
    serde_json::from_str(&json).map_err(|e| AppError::External(format!("解析定时任务配置失败: {e}")))
}

fn write_schedules(app: &AppHandle, schedules: &[ReportSchedule]) -> AppResult<()> {
    let dir = schedules_dir(app)?;
    fs::create_dir_all(&dir)?;
    let path = dir.join(SCHEDULES_FILE);
    let json = serde_json::to_string_pretty(schedules)
        .map_err(|e| AppError::External(format!("序列化定时任务配置失败: {e}")))?;
    fs::write(&path, json)?;
    Ok(())
}

// ── commands: report history ───────────────────────────────────────────

/// 保存日报及其提交记录到历史,返回新记录 id。
/// 前端在生成日报后自动调用此命令(无需手动操作)。
#[tauri::command]
pub fn save_report_history(
    db: State<'_, Db>,
    project_ids: Vec<i64>,
    date_from: String,
    date_to: String,
    range_label: String,
    author_mode: String,
    language: String,
    result: String,
    commit_data: Vec<SaveReportCommit>,
) -> AppResult<i64> {
    let conn = db.0.lock().unwrap();
    let now = chrono::Utc::now().timestamp();
    let ids_json =
        serde_json::to_string(&project_ids).unwrap_or_default();

    conn.execute(
        "INSERT INTO report_history (project_ids, date_from, date_to, range_label, author_mode, language, result, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![ids_json, date_from, date_to, range_label, author_mode, language, result, now],
    )?;
    let report_id = conn.last_insert_rowid();

    for item in &commit_data {
        let commits_json = serde_json::to_string(&item.commits).unwrap_or_default();
        conn.execute(
            "INSERT INTO report_commits (report_id, project_id, project_name, project_description, commit_data)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                report_id,
                item.project_id,
                item.project_name,
                item.project_description,
                commits_json,
            ],
        )?;
    }

    Ok(report_id)
}

/// 分页查询日报历史列表,可按项目筛选。
#[tauri::command]
pub fn list_report_history(
    db: State<'_, Db>,
    limit: Option<usize>,
    offset: Option<usize>,
    project_id: Option<i64>,
) -> AppResult<Vec<ReportHistoryItem>> {
    let conn = db.0.lock().unwrap();
    let limit = limit.unwrap_or(50).min(200);
    let offset = offset.unwrap_or(0);

    let rows = if let Some(pid) = project_id {
        let mut stmt = conn.prepare(
            "SELECT h.id, h.project_ids, h.date_from, h.date_to, h.range_label,
                    h.author_mode, h.language, h.created_at
             FROM report_history h
             WHERE h.project_ids LIKE ?1
             ORDER BY h.created_at DESC
             LIMIT ?2 OFFSET ?3",
        )?;
        let pattern = format!("%{}%", pid); // JSON 数组中模糊匹配
        let result = stmt.query_map(params![pattern, limit as i64, offset as i64], map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        result
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, project_ids, date_from, date_to, range_label,
                    author_mode, language, created_at
             FROM report_history
             ORDER BY created_at DESC
             LIMIT ?1 OFFSET ?2",
        )?;
        let result = stmt.query_map(params![limit as i64, offset as i64], map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        result
    };

    // 为每条记录补全 project_names 与 total_commits
    let mut items = Vec::with_capacity(rows.len());
    for (mut item, ids) in rows {
        item.project_names = resolve_project_names(&conn, &ids)?;
        item.total_commits = count_commits(&conn, item.id)?;
        items.push(item);
    }

    Ok(items)
}

/// 查询单条日报详情(含 Markdown 正文与提交记录)。
#[tauri::command]
pub fn get_report_history(
    db: State<'_, Db>,
    id: i64,
) -> AppResult<ReportHistoryDetail> {
    let conn = db.0.lock().unwrap();

    let (mut item, ids, result) = conn.query_row(
        "SELECT id, project_ids, date_from, date_to, range_label,
                author_mode, language, created_at, result
         FROM report_history WHERE id = ?1",
        params![id],
        |r| {
            let ids_json: String = r.get(1)?;
            let ids: Vec<i64> =
                serde_json::from_str(&ids_json).unwrap_or_default();
            Ok((
                ReportHistoryItem {
                    id: r.get(0)?,
                    project_ids: ids.clone(),
                    date_from: r.get(2)?,
                    date_to: r.get(3)?,
                    range_label: r.get(4)?,
                    author_mode: r.get(5)?,
                    language: r.get(6)?,
                    created_at: r.get(7)?,
                    project_names: Vec::new(),
                    total_commits: 0,
                },
                ids,
                r.get::<_, String>(8)?,
            ))
        },
    )?;

    item.project_names = resolve_project_names(&conn, &ids)?;
    item.total_commits = count_commits(&conn, item.id)?;

    let mut stmt = conn.prepare(
        "SELECT project_id, project_name, project_description, commit_data
         FROM report_commits WHERE report_id = ?1",
    )?;
    let commits = stmt
        .query_map(params![id], |r| {
            let data_json: String = r.get(3)?;
            let commits: Vec<GitCommitInfo> =
                serde_json::from_str(&data_json).unwrap_or_default();
            Ok(ReportCommitItem {
                project_id: r.get(0)?,
                project_name: r.get(1)?,
                project_description: r.get(2)?,
                commits,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ReportHistoryDetail {
        item,
        result,
        commits,
    })
}

/// 删除日报历史(级联删除关联的提交记录)。
#[tauri::command]
pub fn delete_report_history(
    db: State<'_, Db>,
    id: i64,
) -> AppResult<()> {
    let conn = db.0.lock().unwrap();
    conn.execute("DELETE FROM report_history WHERE id = ?1", params![id])?;
    Ok(())
}

// ── calendar meta ──────────────────────────────────────────────────────

/// 日历标注数据：某月每天的报告数量 + 节假日/调休列表。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarMeta {
    pub dates: HashMap<String, i64>,
    pub holidays: Vec<String>,
    pub workdays: Vec<String>,
}

/// 返回某月的日历标注数据（每天报告数 + 节假日/调休），供前端日历渲染。
#[tauri::command]
pub fn get_calendar_meta(
    db: State<'_, Db>,
    app: AppHandle,
    year: i32,
    month: u32,
    project_ids: Vec<i64>,
    tag_ids: Vec<i64>,
) -> AppResult<CalendarMeta> {
    let conn = db.0.lock().unwrap();
    let mut dates: HashMap<String, i64> = HashMap::new();
    let has_projects = !project_ids.is_empty();
    let has_tags = !tag_ids.is_empty();

    // 生成当月所有日期
    let start = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| AppError::External("无效的年月".into()))?;
    let days_in_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| AppError::External("无效的年月".into()))?
    .signed_duration_since(start)
    .num_days() as u32;

    for d in 0..days_in_month {
        let date = start + chrono::Duration::days(d as i64);
        let ds = date.format("%Y-%m-%d").to_string();

        if !has_projects && !has_tags {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM report_history
                 WHERE date(created_at, 'unixepoch') = ?1",
                params![ds],
                |r| r.get(0),
            )?;
            if count > 0 { dates.insert(ds, count); }
            continue;
        }

        // 动态构建 WHERE 条件
        let mut conditions = vec!["date(h.created_at, 'unixepoch') = ?1".to_string()];
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(ds.clone())];
        let mut param_idx = 2;

        if has_projects {
            let placeholders: Vec<String> = (0..project_ids.len()).map(|i| format!("?{}", param_idx + i)).collect();
            conditions.push(format!(
                "EXISTS (SELECT 1 FROM json_each(h.project_ids) WHERE CAST(value AS INTEGER) IN ({}))",
                placeholders.join(",")
            ));
            for pid in &project_ids {
                params_vec.push(Box::new(*pid));
            }
            param_idx += project_ids.len();
        }

        if has_tags {
            let placeholders: Vec<String> = (0..tag_ids.len()).map(|i| format!("?{}", param_idx + i)).collect();
            let having_param = format!("?{}", param_idx + tag_ids.len());
            conditions.push(format!(
                "EXISTS (SELECT 1 FROM json_each(h.project_ids) j \
                 WHERE CAST(j.value AS INTEGER) IN ( \
                     SELECT pt.project_id FROM project_tags pt \
                     WHERE pt.tag_id IN ({}) \
                     GROUP BY pt.project_id \
                     HAVING COUNT(DISTINCT pt.tag_id) = {} \
                 ))",
                placeholders.join(","),
                having_param,
            ));
            for tid in &tag_ids {
                params_vec.push(Box::new(*tid));
            }
            params_vec.push(Box::new(tag_ids.len() as i64));
        }

        let sql = format!(
            "SELECT COUNT(*) FROM report_history h WHERE {}",
            conditions.join(" AND ")
        );

        let count: i64 = conn.query_row(&sql, rusqlite::params_from_iter(params_vec), |r| r.get(0))?;
        if count > 0 {
            dates.insert(ds, count);
        }
    }

    // 节假日/调休数据
    let data_dir = app
        .path()
        .home_dir()
        .map_err(|e| AppError::External(e.to_string()))?
        .join(APP_DATA_DIR_NAME);
    let (holidays, workdays) = workday::load_data(&data_dir).unwrap_or_default();
    let holidays: Vec<String> = holidays.into_iter().collect();
    let workdays: Vec<String> = workdays.into_iter().collect();

    Ok(CalendarMeta {
        dates,
        holidays,
        workdays,
    })
}

/// 查询指定生成日期的所有日报详情（含提交记录和 Markdown 正文）。
#[tauri::command]
pub fn get_reports_by_date(
    db: State<'_, Db>,
    date: String,
    project_ids: Vec<i64>,
    tag_ids: Vec<i64>,
) -> AppResult<Vec<ReportHistoryDetail>> {
    let conn = db.0.lock().unwrap();
    let has_projects = !project_ids.is_empty();
    let has_tags = !tag_ids.is_empty();

    let rows: Vec<(ReportHistoryItem, Vec<i64>, String)> = if !has_projects && !has_tags {
        let sql = "SELECT id, project_ids, date_from, date_to, range_label,
                          author_mode, language, created_at, result
                   FROM report_history
                   WHERE date(created_at, 'unixepoch') = ?1
                   ORDER BY created_at DESC";
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![date], map_detail_row)?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    } else {
        let mut conditions = vec!["date(h.created_at, 'unixepoch') = ?1".to_string()];
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(date.clone())];
        let mut param_idx = 2;

        if has_projects {
            let placeholders: Vec<String> = (0..project_ids.len()).map(|i| format!("?{}", param_idx + i)).collect();
            conditions.push(format!(
                "EXISTS (SELECT 1 FROM json_each(h.project_ids) WHERE CAST(value AS INTEGER) IN ({}))",
                placeholders.join(",")
            ));
            for pid in &project_ids {
                params_vec.push(Box::new(*pid));
            }
            param_idx += project_ids.len();
        }

        if has_tags {
            let placeholders: Vec<String> = (0..tag_ids.len()).map(|i| format!("?{}", param_idx + i)).collect();
            let having_param = format!("?{}", param_idx + tag_ids.len());
            conditions.push(format!(
                "EXISTS (SELECT 1 FROM json_each(h.project_ids) j \
                 WHERE CAST(j.value AS INTEGER) IN ( \
                     SELECT pt.project_id FROM project_tags pt \
                     WHERE pt.tag_id IN ({}) \
                     GROUP BY pt.project_id \
                     HAVING COUNT(DISTINCT pt.tag_id) = {} \
                 ))",
                placeholders.join(","),
                having_param,
            ));
            for tid in &tag_ids {
                params_vec.push(Box::new(*tid));
            }
            params_vec.push(Box::new(tag_ids.len() as i64));
        }

        let sql = format!(
            "SELECT id, project_ids, date_from, date_to, range_label,
                    author_mode, language, created_at, result
             FROM report_history h
             WHERE {}
             ORDER BY h.created_at DESC",
            conditions.join(" AND ")
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), map_detail_row)?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };

    // 为每条记录补全 project_names、total_commits 和 commits
    let mut results = Vec::with_capacity(rows.len());
    for (mut item, ids, result) in rows {
        item.project_names = resolve_project_names(&conn, &ids)?;
        item.total_commits = count_commits(&conn, item.id)?;

        let mut stmt = conn.prepare(
            "SELECT project_id, project_name, project_description, commit_data
             FROM report_commits WHERE report_id = ?1",
        )?;
        let commits = stmt
            .query_map(params![item.id], |r| {
                let data_json: String = r.get(3)?;
                let commits: Vec<GitCommitInfo> =
                    serde_json::from_str(&data_json).unwrap_or_default();
                Ok(ReportCommitItem {
                    project_id: r.get(0)?,
                    project_name: r.get(1)?,
                    project_description: r.get(2)?,
                    commits,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        results.push(ReportHistoryDetail {
            item,
            result,
            commits,
        });
    }

    Ok(results)
}

// ── commands: schedules ────────────────────────────────────────────────

/// 读取定时任务配置列表。
#[tauri::command]
pub fn list_report_schedules(app: AppHandle) -> AppResult<Vec<ReportSchedule>> {
    read_schedules(&app)
}

/// 保存定时任务配置(全量替换),同时唤醒后台 scheduler 重算下次触发时间。
#[tauri::command]
pub fn save_report_schedules(
    app: AppHandle,
    schedules: Vec<ReportSchedule>,
) -> AppResult<()> {
    write_schedules(&app, &schedules)?;
    // 唤醒 scheduler(若尚未启动则忽略)
    if let Some(notify) = app.try_state::<ScheduleNotify>() {
        notify.0.notify_one();
    }
    Ok(())
}

// ── Notify wrapper for Tauri state ─────────────────────────────────────

/// 用于 Tauri 托管状态的 Notify 包装(Arc<Notify> 自身不满足 State 要求)
pub struct ScheduleNotify(pub Arc<Notify>);

// ── helpers ────────────────────────────────────────────────────────────

fn map_row(
    r: &rusqlite::Row<'_>,
) -> rusqlite::Result<(ReportHistoryItem, Vec<i64>)> {
    let ids_json: String = r.get(1)?;
    let ids: Vec<i64> = serde_json::from_str(&ids_json).unwrap_or_default();
    Ok((
        ReportHistoryItem {
            id: r.get(0)?,
            project_ids: ids.clone(),
            date_from: r.get(2)?,
            date_to: r.get(3)?,
            range_label: r.get(4)?,
            author_mode: r.get(5)?,
            language: r.get(6)?,
            created_at: r.get(7)?,
            project_names: Vec::new(),
            total_commits: 0,
        },
        ids,
    ))
}

fn resolve_project_names(
    conn: &rusqlite::Connection,
    ids: &[i64],
) -> AppResult<Vec<String>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders: Vec<String> = ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
    let sql = format!(
        "SELECT name FROM projects WHERE id IN ({}) ORDER BY name COLLATE NOCASE",
        placeholders.join(",")
    );
    let mut stmt = conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::types::ToSql> =
        ids.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();
    let names = stmt
        .query_map(params.as_slice(), |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(names)
}

fn count_commits(
    conn: &rusqlite::Connection,
    report_id: i64,
) -> AppResult<i64> {
    let count: i64 = conn.query_row(
        "SELECT COALESCE(SUM(json_array_length(commit_data)), 0) FROM report_commits WHERE report_id = ?1",
        params![report_id],
        |r| r.get(0),
    )?;
    Ok(count)
}

/// 映射 get_reports_by_date 的查询行
fn map_detail_row(
    r: &rusqlite::Row<'_>,
) -> rusqlite::Result<(ReportHistoryItem, Vec<i64>, String)> {
    let ids_json: String = r.get(1)?;
    let ids: Vec<i64> = serde_json::from_str(&ids_json).unwrap_or_default();
    Ok((
        ReportHistoryItem {
            id: r.get(0)?,
            project_ids: ids.clone(),
            date_from: r.get(2)?,
            date_to: r.get(3)?,
            range_label: r.get(4)?,
            author_mode: r.get(5)?,
            language: r.get(6)?,
            created_at: r.get(7)?,
            project_names: Vec::new(),
            total_commits: 0,
        },
        ids,
        r.get::<_, String>(8)?,
    ))
}

// ── scheduler helper (used by scheduler.rs) ────────────────────────────

/// 供 scheduler 直接调用:读取定时任务(不经过 Tauri command 边界)
pub fn read_schedules_from_dir(data_dir: &PathBuf) -> AppResult<Vec<ReportSchedule>> {
    let path = data_dir.join(SCHEDULES_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = fs::read_to_string(&path)?;
    serde_json::from_str(&json)
        .map_err(|e| AppError::External(format!("解析定时任务配置失败: {e}")))
}

/// 供 scheduler 直接调用:更新 last_run_at
pub fn update_last_run_at(data_dir: &PathBuf, schedule_id: &str, timestamp: i64) -> AppResult<()> {
    let mut schedules = read_schedules_from_dir(data_dir)?;
    if let Some(s) = schedules.iter_mut().find(|s| s.id == schedule_id) {
        s.last_run_at = Some(timestamp);
    }
    let path = data_dir.join(SCHEDULES_FILE);
    let json = serde_json::to_string_pretty(&schedules)
        .map_err(|e| AppError::External(format!("序列化定时任务配置失败: {e}")))?;
    fs::write(&path, json)?;
    Ok(())
}
