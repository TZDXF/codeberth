//! 日报定时调度引擎(订阅模式)。
//!
//! 启动一个 tokio 后台循环:
//! 1. 计算所有启用定时任务的下次触发时间
//! 2. `sleep_until` 精确等待到最早触发时刻
//! 3. 时间到 → 拉取 git_log → 调 AI → 保存日报历史 → emit 事件
//! 4. 定时任务变更时通过 Notify 唤醒重算

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use chrono::{Datelike, Local, NaiveTime, Timelike};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use tokio::select;
use tokio::time::{self, Duration, Instant};

use crate::commands::git::{run_git_current_user, run_git_log};
use crate::commands::report::{
    read_schedules_from_dir, update_last_run_at, ReportGeneratedPayload, ReportSchedule,
};
use crate::error::AppResult;
use crate::models::GitCommitInfo;
use crate::workday;

/// 调度循环间隔(用于空闲等待与错误重试)
const IDLE_INTERVAL: Duration = Duration::from_secs(60);

// ── AI config (from settings.json) ──────────────────────────────────────

#[derive(Deserialize, Default)]
struct AiConfig {
    #[serde(default = "default_base_url")]
    ai_base_url: String,
    #[serde(default)]
    ai_api_key: String,
    #[serde(default = "default_model")]
    ai_model: String,
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".into()
}
fn default_model() -> String {
    "gpt-4o-mini".into()
}

fn load_ai_config(data_dir: &PathBuf) -> AiConfig {
    let path = data_dir.join("settings.json");
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .map(|v| AiConfig {
            ai_base_url: v
                .get("aiBaseUrl")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(default_base_url),
            ai_api_key: v
                .get("aiApiKey")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            ai_model: v
                .get("aiModel")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(default_model),
        })
        .unwrap_or_default()
}

fn load_report_prompt(data_dir: &PathBuf) -> String {
    let path = data_dir.join("prompts").join("report.md");
    fs::read_to_string(&path).unwrap_or_default()
}

// ── OpenAI Chat Completions ────────────────────────────────────────────

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

async fn call_ai(
    client: &Client,
    config: &AiConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, String> {
    let url = format!("{}/chat/completions", config.ai_base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": config.ai_model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ]
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.ai_api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("AI 请求失败: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("AI 返回错误 ({status}): {text}"));
    }

    let data: ChatResponse = resp
        .json()
        .await
        .map_err(|e| format!("解析 AI 响应失败: {e}"))?;

    data.choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "AI 返回空响应".into())
}

// ── prompt builder ─────────────────────────────────────────────────────

fn build_report_prompt(
    commits_by_project: &[(String, String, Vec<GitCommitInfo>)], // (name, description, commits)
    range_label: &str,
    language: &str,
) -> String {
    let sections: Vec<String> = commits_by_project
        .iter()
        .map(|(name, desc, commits)| {
            let heading = if desc.is_empty() {
                format!("### {name}")
            } else {
                format!("### {name} — {desc}")
            };
            let lines: Vec<String> = commits
                .iter()
                .map(|c| format!("- [{}] {} ({}, {})", c.date, c.subject, c.hash, c.author))
                .collect();
            if lines.is_empty() {
                format!("{heading}\n(no commits)")
            } else {
                format!("{heading}\n{}", lines.join("\n"))
            }
        })
        .collect();

    let lang = if language == "zh-CN" { "中文" } else { "English" };
    format!(
        "Time range: {range_label}.\n\nCommit records:\n{}\n\nRespond in {lang}.",
        sections.join("\n\n")
    )
}

// ── next fire time calculation ─────────────────────────────────────────

/// 计算某个 schedule 的下次触发时刻。
/// 返回 `Option<Instant>`: None 表示该 schedule 无需等待(已禁用或无有效时间)。
fn next_fire(schedule: &ReportSchedule, now_local: &chrono::DateTime<Local>) -> Option<Instant> {
    if !schedule.enabled {
        return None;
    }

    let time_parts: Vec<&str> = schedule.time_of_day.split(':').collect();
    if time_parts.len() != 2 {
        return None;
    }
    let hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    let target_time = NaiveTime::from_hms_opt(hour, minute, 0)?;

    let today = now_local.date_naive();
    let today_target = today
        .and_time(target_time)
        .and_local_timezone(Local)
        .single()?;

    // 若今天的目标时刻已过,则推到明天
    if today_target <= *now_local {
        let tomorrow = today.succ_opt()?;
        return tomorrow
            .and_time(target_time)
            .and_local_timezone(Local)
            .single()
            .map(|dt| {
                let dur = dt.signed_duration_since(*now_local);
                Instant::now()
                    + Duration::from_secs(dur.to_std().unwrap_or_default().as_secs())
            });
    }

    let dur = today_target.signed_duration_since(*now_local);
    Some(Instant::now() + Duration::from_secs(dur.to_std().unwrap_or_default().as_secs()))
}

/// 返回所有启用 schedule 中最早的下次触发时刻
fn earliest_fire(
    schedules: &[ReportSchedule],
    now_local: &chrono::DateTime<Local>,
) -> Option<Instant> {
    schedules
        .iter()
        .filter_map(|s| next_fire(s, now_local))
        .min()
}

/// 筛选当前时刻应该触发的 schedule(±30 秒容差)
fn due_schedules(
    schedules: &[ReportSchedule],
    now_local: &chrono::DateTime<Local>,
    data_dir: &PathBuf,
) -> Vec<ReportSchedule> {
    let now_time = now_local.time();
    let today = now_local.date_naive();
    let _today_str = today.format("%Y-%m-%d").to_string();

    schedules
        .iter()
        .filter(|s| {
            if !s.enabled {
                return false;
            }

            // 时间匹配:±1 分钟容差
            let parts: Vec<&str> = s.time_of_day.split(':').collect();
            if parts.len() != 2 {
                return false;
            }
            let target_h: u32 = parts[0].parse().unwrap_or(99);
            let target_m: u32 = parts[1].parse().unwrap_or(99);
            let diff = (now_time.hour() as i32 * 60 + now_time.minute() as i32)
                - (target_h as i32 * 60 + target_m as i32);
            if diff.abs() > 1 {
                return false;
            }

            // 今天已运行过
            if let Some(last) = s.last_run_at {
                let last_date =
                    chrono::DateTime::from_timestamp(last, 0).map(|dt| dt.date_naive());
                if last_date == Some(today) {
                    return false;
                }
            }

            // 星期过滤
            if s.weekdays_only {
                let w = today.weekday().num_days_from_monday();
                if w >= 5 {
                    return false;
                }
            }

            // 中国工作日过滤
            if s.chinese_workday_only && !workday::is_workday(today, data_dir) {
                return false;
            }

            true
        })
        .cloned()
        .collect()
}

// ── project path lookup ────────────────────────────────────────────────

fn load_project_paths(data_dir: &PathBuf) -> AppResult<HashMap<i64, (String, String, String)>> {
    let db_path = data_dir.join("projects.db");
    let conn = rusqlite::Connection::open(&db_path)?;
    let mut stmt =
        conn.prepare("SELECT id, path, name, description FROM projects WHERE archived_at IS NULL")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (id, path, name, desc) = row?;
        map.insert(id, (path, name, desc));
    }
    Ok(map)
}

// ── schedule execution ─────────────────────────────────────────────────

async fn fire_schedule(
    app: &AppHandle,
    client: &Client,
    data_dir: &PathBuf,
    schedule: &ReportSchedule,
) {
    let schedule_name = if schedule.name.is_empty() {
        "日报定时任务".to_string()
    } else {
        schedule.name.clone()
    };

    eprintln!(
        "[scheduler] 触发定时任务: {schedule_name} @ {}",
        Local::now().format("%Y-%m-%d %H:%M")
    );

    // 1. 读取项目路径
    let projects = match load_project_paths(data_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[scheduler] 读取项目列表失败: {e}");
            return;
        }
    };

    // 2. 读取 AI 配置
    let ai_config = load_ai_config(data_dir);
    if ai_config.ai_api_key.is_empty() {
        eprintln!("[scheduler] AI API Key 未配置,跳过生成");
        return;
    }

    // 3. 读取提示词模板
    let custom_prompt = load_report_prompt(data_dir);
    let system_prompt = if custom_prompt.trim().is_empty() {
        // 使用内置默认 prompt(与前端 ai-prompts.ts 一致)
        "You are a technical project manager. Generate a concise daily report in Markdown based on git commit records.\n\nGuidelines:\n- Group commits by project\n- Highlight key changes and their impact\n- Use bullet points for clarity\n- Keep it professional and actionable"
            .to_string()
    } else {
        custom_prompt
    };
    let system_prompt = if schedule.language() == "zh-CN" {
        format!("{system_prompt}\n\nRespond in 中文.")
    } else {
        format!("{system_prompt}\n\nRespond in English.")
    };

    // 4. 获取日期范围(今天)
    let today = Local::now().date_naive();
    let date_str = today.format("%Y-%m-%d").to_string();
    let since = format!("{date_str} 00:00:00");
    let until = format!("{date_str} 23:59:59");
    let range_label = format!("{date_str}");

    // 5. 对每个项目拉取 git_log
    let mut commits_by_project: Vec<(i64, String, String, Vec<GitCommitInfo>)> = Vec::new();
    for &pid in &schedule.project_ids {
        if let Some((path, name, desc)) = projects.get(&pid) {
            // 解析作者过滤
            let author: Option<String> = if schedule.author_mode == "me" {
                run_git_current_user(path)
                    .ok()
                    .and_then(|u| {
                        let name = u.name;
                        if name.is_empty() { None } else { Some(name) }
                    })
            } else {
                None
            };
            let commits = run_git_log(path, Some(&since), Some(&until), Some(500), author.as_deref())
                .unwrap_or_default();
            commits_by_project.push((pid, name.clone(), desc.clone(), commits));
        }
    }

    // 无提交则跳过
    if !commits_by_project.iter().any(|(_, _, _, c)| !c.is_empty()) {
        eprintln!("[scheduler] {schedule_name}: 无提交记录,跳过");
        // 仍标记为已运行,避免当天重复检查
        let now_ts = Local::now().timestamp();
        if let Err(e) = update_last_run_at(data_dir, &schedule.id, now_ts) {
            eprintln!("[scheduler] 更新 last_run_at 失败: {e}");
        }
        return;
    }

    // 6. 组装 prompt
    let prompt_data: Vec<(String, String, Vec<GitCommitInfo>)> = commits_by_project
        .iter()
        .map(|(_, name, desc, commits)| (name.clone(), desc.clone(), commits.clone()))
        .collect();
    let user_prompt = build_report_prompt(&prompt_data, &range_label, &schedule.language());

    // 7. 调用 AI
    let result = match call_ai(client, &ai_config, &system_prompt, &user_prompt).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[scheduler] {schedule_name}: AI 调用失败: {e}");
            return;
        }
    };

    // 8. 保存到日报历史(SQLite)
    let db_path = data_dir.join("projects.db");
    let save_result: Result<i64, _> = (|| {
        let conn = rusqlite::Connection::open(&db_path)?;
        let now = chrono::Utc::now().timestamp();
        let project_ids: Vec<i64> = commits_by_project.iter().map(|(id, _, _, _)| *id).collect();
        let ids_json = serde_json::to_string(&project_ids).unwrap_or_default();

        conn.execute(
            "INSERT INTO report_history (project_ids, date_from, date_to, range_label, author_mode, language, result, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![ids_json, date_str, date_str, range_label, &schedule.author_mode, schedule.language(), result, now],
        )?;
        let report_id = conn.last_insert_rowid();

        for (pid, name, desc, commits) in &commits_by_project {
            let commits_json = serde_json::to_string(commits).unwrap_or_default();
            conn.execute(
                "INSERT INTO report_commits (report_id, project_id, project_name, project_description, commit_data)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![report_id, pid, name, desc, commits_json],
            )?;
        }

        Ok::<i64, crate::error::AppError>(report_id)
    })();

    match save_result {
        Ok(history_id) => {
            // 9. 更新 last_run_at
            let now_ts = Local::now().timestamp();
            if let Err(e) = update_last_run_at(data_dir, &schedule.id, now_ts) {
                eprintln!("[scheduler] 更新 last_run_at 失败: {e}");
            }

            // 10. 通知前端
            let payload = ReportGeneratedPayload {
                schedule_name: schedule_name.clone(),
                history_id,
                date_from: date_str.clone(),
                date_to: date_str,
            };
            if let Err(e) = app.emit("report://generated", payload) {
                eprintln!("[scheduler] 发送前端通知失败: {e}");
            }

            eprintln!("[scheduler] {schedule_name}: 日报已生成 (id={history_id})");
        }
        Err(e) => {
            eprintln!("[scheduler] {schedule_name}: 保存日报历史失败: {e}");
        }
    }
}

// ── main loop ──────────────────────────────────────────────────────────

/// 启动调度器后台循环。
/// 应在 `tauri::Builder::setup` 中通过 `tauri::async_runtime::spawn` 调用。
pub async fn run(app: AppHandle) {
    let data_dir = workday::data_dir(&app);
    let notify = app
        .state::<crate::commands::report::ScheduleNotify>()
        .0
        .clone();
    let client = Client::new();

    loop {
        // 重新加载定时任务(解析时间,不用持久化的 last_run_at 判断是否应跳过今天)
        let schedules = match read_schedules_from_dir(&data_dir) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[scheduler] 读取定时任务失败: {e}");
                time::sleep(IDLE_INTERVAL).await;
                continue;
            }
        };

        let now_local = Local::now();

        // 先检查是否有当前时刻应触发的任务(±1 分钟容差,防止 sleep_until 因重算延迟漏掉)
        let due = due_schedules(&schedules, &now_local, &data_dir);
        for s in &due {
            fire_schedule(&app, &client, &data_dir, s).await;
        }

        // 计算下次触发时刻
        if let Some(deadline) = earliest_fire(&schedules, &now_local) {
            select! {
                _ = time::sleep_until(deadline) => {
                    // 醒来后重新加载 schedules 并检查触发(下一轮循环)
                    continue;
                }
                _ = notify.notified() => {
                    // 定时任务变更,重新计算
                    eprintln!("[scheduler] 定时任务配置已变更,重算触发时间");
                    continue;
                }
            }
        } else {
            // 无启用任务,等待通知或超时
            select! {
                _ = notify.notified() => {
                    // 有新任务加入
                    eprintln!("[scheduler] 收到通知,检查定时任务");
                    continue;
                }
                _ = time::sleep(IDLE_INTERVAL) => {
                    // 定期检查(防止文件外部变更)
                    continue;
                }
            }
        }
    }
}

// ── ReportSchedule helpers ─────────────────────────────────────────────

impl ReportSchedule {
    fn language(&self) -> &str {
        // schedule 中没有 language 字段,默认 zh-CN
        // 可以从 settings.json 读取,但暂时用默认值
        "zh-CN"
    }
}
