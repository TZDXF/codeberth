//! 中国工作日判断:从 CDN 拉取 chinese-days 数据并缓存,提供 is_workday() 查询。
//!
//! 数据源: <https://cdn.jsdelivr.net/npm/chinese-days/dist/chinese-days.json>
//! 覆盖范围: 2004–2026 年官方节假日和调休安排。
//! 无数据年份回退为常规周一～周五判断。

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use chrono::{Datelike, NaiveDate, Utc};
use reqwest::blocking::Client;
use serde::Deserialize;
use tauri::Manager;

use crate::error::AppResult;
use crate::APP_DATA_DIR_NAME;

const CDN_URL: &str = "https://cdn.jsdelivr.net/npm/chinese-days/dist/chinese-days.json";
const CACHE_FILE: &str = "chinese-days.json";
/// 缓存有效期(天)
const CACHE_TTL_DAYS: i64 = 30;

#[derive(Debug, Deserialize)]
struct ChineseDaysData {
    holidays: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default)]
    workdays: Option<serde_json::Map<String, serde_json::Value>>,
}

/// 加载工作日数据:优先读缓存,过期或不存在时从 CDN 拉取并写入缓存。
/// 返回 holidays 与 workdays 两个日期集合(均为 "YYYY-MM-DD" 格式)。
fn load_data(data_dir: &PathBuf) -> AppResult<(HashSet<String>, HashSet<String>)> {
    let cache_path = data_dir.join(CACHE_FILE);

    // 缓存存在且未过期,直接读取
    if cache_path.exists() {
        if let Ok(meta) = cache_path.metadata() {
            if let Ok(modified) = meta.modified() {
                let age = Utc::now()
                    .timestamp()
                    .saturating_sub(
                        modified
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64,
                    );
                if age < CACHE_TTL_DAYS * 86400 {
                    if let Ok(data) = fs::read_to_string(&cache_path) {
                        if let Ok(parsed) = parse_data(&data) {
                            return Ok(parsed);
                        }
                    }
                }
            }
        }
    }

    // 从 CDN 拉取
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| crate::error::AppError::External(format!("创建 HTTP 客户端失败: {e}")))?;

    let resp = client
        .get(CDN_URL)
        .header("User-Agent", "tauri-appproject-dev/0.1")
        .send()
        .map_err(|e| {
            crate::error::AppError::External(format!("拉取中国工作日数据失败: {e}"))
        })?;

    let body = resp.text().map_err(|e| {
        crate::error::AppError::External(format!("读取中国工作日数据响应失败: {e}"))
    })?;

    // 写入缓存
    if let Err(e) = fs::write(&cache_path, &body) {
        eprintln!("[workday] 写入缓存失败: {e}");
    }

    parse_data(&body)
}

fn parse_data(json: &str) -> AppResult<(HashSet<String>, HashSet<String>)> {
    let data: ChineseDaysData =
        serde_json::from_str(json).map_err(|e| {
            crate::error::AppError::External(format!("解析中国工作日数据失败: {e}"))
        })?;

    let holidays: HashSet<String> = data
        .holidays
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    let workdays: HashSet<String> = data
        .workdays
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();

    Ok((holidays, workdays))
}

/// 判断给定日期是否为中国工作日(含调休补班,排除法定节假日)。
///
/// * 若日期在 workdays 集合中 → `true`(调休上班的周六/周日)
/// * 若日期不在 holidays 集合中且为周一～周五 → `true`(常规工作日)
/// * 其他情况 → `false`
///
/// `data_dir` 为 `~/.pm` 目录路径。
pub fn is_workday(date: NaiveDate, data_dir: &PathBuf) -> bool {
    // 数据拉取失败时回退为常规周一～周五判断
    let (holidays, workdays) = match load_data(data_dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[workday] 加载工作日数据失败,回退常规判断: {e}");
            return is_regular_weekday(date);
        }
    };

    let date_str = date.format("%Y-%m-%d").to_string();

    // 调休上班(周日/周六补班)优先级最高
    if workdays.contains(&date_str) {
        return true;
    }
    // 法定节假日
    if holidays.contains(&date_str) {
        return false;
    }
    // 常规判断
    is_regular_weekday(date)
}

fn is_regular_weekday(date: NaiveDate) -> bool {
    let w = date.weekday().num_days_from_monday();
    w < 5 // Monday(0) ~ Friday(4)
}

/// 获取数据目录路径(与 lib.rs 中 Db::open 一致)
pub fn data_dir(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .home_dir()
        .unwrap_or_default()
        .join(APP_DATA_DIR_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_weekday_check() {
        // 2026-07-21 是周二
        let tue = NaiveDate::from_ymd_opt(2026, 7, 21).unwrap();
        assert!(is_regular_weekday(tue));

        // 2026-07-25 是周六
        let sat = NaiveDate::from_ymd_opt(2026, 7, 25).unwrap();
        assert!(!is_regular_weekday(sat));

        // 2026-07-26 是周日
        let sun = NaiveDate::from_ymd_opt(2026, 7, 26).unwrap();
        assert!(!is_regular_weekday(sun));
    }
}
