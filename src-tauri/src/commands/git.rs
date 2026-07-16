use std::process::Command;
use std::sync::OnceLock;

use serde::Serialize;
use tauri::{Emitter, Window};
use tokio::sync::Semaphore;

use crate::error::AppResult;
use crate::models::GitStatus;

/// 后台 fetch 并发上限(超出排队)
static FETCH_PERMITS: OnceLock<Semaphore> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
pub struct GitUpdatedPayload {
    pub project_id: i64,
    pub remote_ahead: i32,
    pub last_fetch_at: i64,
}

fn git_command(path: &str) -> Command {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path);
    // Windows: 避免 GUI 应用弹 git 时闪现控制台黑窗
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd
}

pub fn status(path: &str) -> AppResult<GitStatus> {
    let output = git_command(path)
        .args(["status", "--porcelain=v2", "--branch"])
        .output()?;
    if !output.status.success() {
        // 不是 git 仓库(git 退出码 128)
        return Ok(GitStatus::default());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(parse_porcelain(&text))
}

/// fetch 远端(无 remote 时跳过),返回最新状态
pub fn fetch_and_status(path: &str) -> AppResult<GitStatus> {
    let remotes = git_command(path).arg("remote").output()?;
    if remotes.status.success() && !String::from_utf8_lossy(&remotes.stdout).trim().is_empty() {
        // 失败(如离线)不阻断,退回本地已知状态
        let _ = git_command(path).args(["fetch", "--quiet"]).output();
    }
    status(path)
}

/// 解析 `git status --porcelain=v2 --branch` 输出
fn parse_porcelain(text: &str) -> GitStatus {
    let mut st = GitStatus {
        is_repo: true,
        ..Default::default()
    };
    for line in text.lines() {
        if let Some(head) = line.strip_prefix("# branch.head ") {
            st.branch = Some(head.trim().to_string());
        } else if let Some(ab) = line.strip_prefix("# branch.ab ") {
            // 形如 "+2 -1"
            for part in ab.split_whitespace() {
                if let Some(a) = part.strip_prefix('+') {
                    st.ahead = a.parse().unwrap_or(0);
                } else if let Some(b) = part.strip_prefix('-') {
                    st.behind = b.parse().unwrap_or(0);
                }
            }
        } else if line.starts_with("1 ") || line.starts_with("2 ") {
            // 普通/重命名条目: 第 3-4 字节是 XY 状态码
            let bytes = line.as_bytes();
            if bytes.len() >= 4 {
                if bytes[2] != b'.' {
                    st.staged += 1;
                }
                if bytes[3] != b'.' {
                    st.modified += 1;
                }
            }
        } else if line.starts_with("u ") {
            st.modified += 1; // 冲突文件计入未暂存
        } else if line.starts_with("? ") {
            st.untracked += 1;
        }
    }
    st.remote_ahead = st.behind;
    st
}

#[tauri::command]
pub fn get_git_status(path: String) -> AppResult<GitStatus> {
    status(&path)
}

/// 后台 fetch:不返回数据,完成后 emit "git://updated"
#[tauri::command]
pub fn fetch_git_remote_async(window: Window, project_id: i64, path: String) {
    tauri::async_runtime::spawn(async move {
        let semaphore = FETCH_PERMITS.get_or_init(|| Semaphore::new(3));
        let _permit = semaphore.acquire().await;
        let result = tokio::task::spawn_blocking(move || fetch_and_status(&path)).await;
        if let Ok(Ok(st)) = result {
            if st.is_repo {
                let payload = GitUpdatedPayload {
                    project_id,
                    remote_ahead: st.behind,
                    last_fetch_at: chrono::Utc::now().timestamp(),
                };
                let _ = window.emit("git://updated", payload);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "projectdev-git-test-{tag}-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn git(dir: &PathBuf, args: &[&str]) {
        let out = git_command(dir.to_str().unwrap()).args(args).output().unwrap();
        assert!(
            out.status.success(),
            "git {:?} 失败: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn init_repo(dir: &PathBuf) {
        git(dir, &["init", "-b", "main"]);
        git(dir, &["config", "user.email", "test@example.com"]);
        git(dir, &["config", "user.name", "test"]);
    }

    #[test]
    fn non_repo_returns_is_repo_false() {
        let dir = temp_dir("plain");
        let st = status(dir.to_str().unwrap()).unwrap();
        assert!(!st.is_repo);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parses_working_tree_counts() {
        let dir = temp_dir("repo");
        init_repo(&dir);
        fs::write(dir.join("a.txt"), "a").unwrap();
        git(&dir, &["add", "a.txt"]);
        git(&dir, &["commit", "-m", "init"]);

        // staged: 新文件 b; modified: a; untracked: c
        fs::write(dir.join("b.txt"), "b").unwrap();
        git(&dir, &["add", "b.txt"]);
        fs::write(dir.join("a.txt"), "changed").unwrap();
        fs::write(dir.join("c.txt"), "c").unwrap();

        let st = status(dir.to_str().unwrap()).unwrap();
        assert!(st.is_repo);
        assert_eq!(st.branch.as_deref(), Some("main"));
        assert_eq!(st.staged, 1);
        assert_eq!(st.modified, 1);
        assert_eq!(st.untracked, 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn fetch_reports_remote_ahead() {
        // origin(bare) <- clone_a 推送; clone_b 作为被测项目
        let origin = temp_dir("origin");
        git(&origin, &["init", "--bare", "-b", "main"]);

        let clone_a = temp_dir("clone-a");
        git(&clone_a, &["clone", origin.to_str().unwrap(), "."]);
        git(&clone_a, &["config", "user.email", "test@example.com"]);
        git(&clone_a, &["config", "user.name", "test"]);
        fs::write(clone_a.join("a.txt"), "a").unwrap();
        git(&clone_a, &["add", "a.txt"]);
        git(&clone_a, &["commit", "-m", "c1"]);
        git(&clone_a, &["push", "-u", "origin", "main"]);

        let clone_b = temp_dir("clone-b");
        git(&clone_b, &["clone", origin.to_str().unwrap(), "."]);

        // clone_a 再推一个提交,clone_b fetch 后 remote 领先 1
        fs::write(clone_a.join("a.txt"), "a2").unwrap();
        git(&clone_a, &["commit", "-am", "c2"]);
        git(&clone_a, &["push"]);

        let st = fetch_and_status(clone_b.to_str().unwrap()).unwrap();
        assert!(st.is_repo);
        assert_eq!(st.remote_ahead, 1);
        assert!(st.last_fetch_at.is_none());

        let _ = fs::remove_dir_all(&origin);
        let _ = fs::remove_dir_all(&clone_a);
        let _ = fs::remove_dir_all(&clone_b);
    }
}

