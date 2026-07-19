use std::process::{Command, Output};
use std::sync::OnceLock;

use serde::Serialize;
use tauri::{Emitter, Window};
use tokio::sync::Semaphore;

use crate::error::{AppError, AppResult};
use crate::models::{GitBranches, GitPullResult, GitStatus};

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
    // 禁止 git 在终端交互式询问凭据(GUI 应用无人应答会挂起);
    // 凭据管理器 helper 弹窗不受影响
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    // Windows: 避免 GUI 应用弹 git 时闪现控制台黑窗
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd
}

/// 执行 git 命令,非零退出时取 stderr(兜底 stdout)转为 External 错误
fn run_git(path: &str, args: &[&str]) -> AppResult<Output> {
    let output = git_command(path).args(args).output()?;
    if output.status.success() {
        return Ok(output);
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if stderr.is_empty() { stdout } else { stderr };
    Err(AppError::External(if detail.is_empty() {
        format!("git {} 退出码 {}", args.join(" "), output.status)
    } else {
        detail
    }))
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
            st.modified += 1; // 冲突文件仍计入未暂存,保持「干净」判断语义
            st.conflicted += 1;
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

/// 当前处于合并冲突状态的文件(相对仓库根的路径)
fn unmerged_files(path: &str) -> Vec<String> {
    let Ok(out) = git_command(path)
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

fn local_branch_names(path: &str) -> AppResult<Vec<String>> {
    let out = run_git(path, &["branch", "--format=%(refname:short)"])?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect())
}

#[tauri::command]
pub fn list_git_branches(path: String) -> AppResult<GitBranches> {
    // 远程分支:for-each-ref 附带 symref 列(tab 分隔),过滤掉 origin/HEAD 这类符号引用
    let remote_out = run_git(
        &path,
        &[
            "for-each-ref",
            "--format=%(refname:short)%09%(symref)",
            "refs/remotes",
        ],
    )?;
    let remote = String::from_utf8_lossy(&remote_out.stdout)
        .lines()
        .filter_map(|l| {
            let (name, symref) = l.split_once('\t').unwrap_or((l, ""));
            if name.is_empty() || !symref.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
        .collect();
    Ok(GitBranches {
        local: local_branch_names(&path)?,
        remote,
    })
}

/// 切换分支;create 为 true 时创建并切换(`git checkout -b`)。
/// remote 为 true 时 branch 形如 "origin/feature":本地已有同名分支则直接切换,
/// 否则创建跟踪分支(`git checkout -b feature --track origin/feature`)
#[tauri::command]
pub fn git_checkout(
    path: String,
    branch: String,
    create: bool,
    remote: bool,
) -> AppResult<GitStatus> {
    let branch = branch.trim();
    if branch.is_empty() {
        return Err(AppError::Invalid("分支名不能为空".into()));
    }
    if create {
        run_git(&path, &["checkout", "-b", branch])?;
    } else if remote {
        let short = branch.split_once('/').map(|(_, s)| s).unwrap_or(branch);
        if local_branch_names(&path)?.iter().any(|b| b == short) {
            run_git(&path, &["checkout", short])?;
        } else {
            run_git(&path, &["checkout", "-b", short, "--track", branch])?;
        }
    } else {
        run_git(&path, &["checkout", branch])?;
    }
    status(&path)
}

/// 提交更改,返回最新状态。
/// 参考 IDEA 提交模型:已暂存内容与未暂存修改(含已解决的冲突文件)始终提交;
/// 仅未跟踪文件需要显式勾选(include_untracked)才纳入
#[tauri::command]
pub fn git_commit(path: String, message: String, include_untracked: bool) -> AppResult<GitStatus> {
    let message = message.trim();
    if message.is_empty() {
        return Err(AppError::Invalid("提交信息不能为空".into()));
    }
    if include_untracked {
        run_git(&path, &["add", "-A"])?;
    } else {
        run_git(&path, &["add", "-u"])?;
    }
    run_git(&path, &["commit", "-m", message])?;
    status(&path)
}

/// 拉取远端。产生合并冲突时不算失败:返回冲突文件列表,由前端引导用户解决
#[tauri::command]
pub fn git_pull(path: String) -> AppResult<GitPullResult> {
    let result = git_command(&path).arg("pull").output()?;
    let conflicts = unmerged_files(&path);
    if !result.status.success() && conflicts.is_empty() {
        let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        return Err(AppError::External(if detail.is_empty() {
            "git pull 失败".into()
        } else {
            detail
        }));
    }
    Ok(GitPullResult {
        status: status(&path)?,
        conflicts,
    })
}

/// 推送当前分支;无 upstream 时(如新建分支首推)自动回退 `git push -u origin HEAD`
#[tauri::command]
pub fn git_push(path: String) -> AppResult<GitStatus> {
    match run_git(&path, &["push"]) {
        Ok(_) => {}
        Err(e) => {
            let no_upstream = e.to_string().contains("no upstream branch")
                || e.to_string().contains("has no upstream branch");
            if !no_upstream {
                return Err(e);
            }
            run_git(&path, &["push", "-u", "origin", "HEAD"])?;
        }
    }
    status(&path)
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
        let out = git_command(dir.to_str().unwrap())
            .args(args)
            .output()
            .unwrap();
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

    #[test]
    fn commit_stages_all_and_cleans_worktree() {
        let dir = temp_dir("commit");
        init_repo(&dir);
        fs::write(dir.join("a.txt"), "a").unwrap();

        let st = git_commit(dir.to_str().unwrap().to_string(), "init".into(), true).unwrap();
        assert!(st.is_repo);
        assert_eq!(st.branch.as_deref(), Some("main"));
        assert_eq!(st.staged, 0);
        assert_eq!(st.modified, 0);
        assert_eq!(st.untracked, 0);

        // 空提交信息被拒绝
        assert!(git_commit(dir.to_str().unwrap().to_string(), "  ".into(), true).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn commit_untracked_is_opt_in() {
        let dir = temp_dir("commit-untracked");
        init_repo(&dir);
        fs::write(dir.join("a.txt"), "a").unwrap();
        git(&dir, &["add", "a.txt"]);
        git(&dir, &["commit", "-m", "init"]);

        fs::write(dir.join("a.txt"), "changed").unwrap(); // 未暂存修改
        fs::write(dir.join("b.txt"), "b").unwrap(); // 未跟踪

        // 不勾选:未暂存修改照常提交,未跟踪文件保留
        let st = git_commit(dir.to_str().unwrap().to_string(), "tracked only".into(), false)
            .unwrap();
        assert_eq!(st.staged, 0);
        assert_eq!(st.modified, 0);
        assert_eq!(st.untracked, 1);

        // 勾选:未跟踪文件一并提交,工作区干净
        let st =
            git_commit(dir.to_str().unwrap().to_string(), "with untracked".into(), true).unwrap();
        assert_eq!(st.staged, 0);
        assert_eq!(st.modified, 0);
        assert_eq!(st.untracked, 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn branches_checkout_and_create() {
        let dir = temp_dir("branch");
        init_repo(&dir);
        fs::write(dir.join("a.txt"), "a").unwrap();
        git(&dir, &["add", "a.txt"]);
        git(&dir, &["commit", "-m", "init"]);

        let branches = list_git_branches(dir.to_str().unwrap().to_string()).unwrap();
        assert_eq!(branches.local, vec!["main".to_string()]);
        assert!(branches.remote.is_empty());

        // 新建并切换
        let st =
            git_checkout(dir.to_str().unwrap().to_string(), "feature".into(), true, false).unwrap();
        assert_eq!(st.branch.as_deref(), Some("feature"));

        let branches = list_git_branches(dir.to_str().unwrap().to_string()).unwrap();
        assert_eq!(branches.local, vec!["feature".to_string(), "main".to_string()]);

        // 切回 main
        let st =
            git_checkout(dir.to_str().unwrap().to_string(), "main".into(), false, false).unwrap();
        assert_eq!(st.branch.as_deref(), Some("main"));

        // 空分支名 / 不存在的分支
        assert!(git_checkout(dir.to_str().unwrap().to_string(), " ".into(), false, false).is_err());
        assert!(
            git_checkout(dir.to_str().unwrap().to_string(), "nope".into(), false, false).is_err()
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn checkout_remote_creates_tracking_branch() {
        let origin = temp_dir("track-origin");
        git(&origin, &["init", "--bare", "-b", "main"]);

        // clone_a:推 main 和 feature 两个分支到远端
        let clone_a = temp_dir("track-a");
        git(&clone_a, &["clone", origin.to_str().unwrap(), "."]);
        git(&clone_a, &["config", "user.email", "test@example.com"]);
        git(&clone_a, &["config", "user.name", "test"]);
        fs::write(clone_a.join("a.txt"), "a").unwrap();
        git(&clone_a, &["add", "a.txt"]);
        git(&clone_a, &["commit", "-m", "c1"]);
        git(&clone_a, &["push", "-u", "origin", "main"]);
        git(&clone_a, &["checkout", "-b", "feature"]);
        fs::write(clone_a.join("b.txt"), "b").unwrap();
        git(&clone_a, &["add", "b.txt"]);
        git(&clone_a, &["commit", "-m", "c2"]);
        git(&clone_a, &["push", "-u", "origin", "feature"]);

        let clone_b = temp_dir("track-b");
        git(&clone_b, &["clone", origin.to_str().unwrap(), "."]);

        // 远程分支列出 feature/main,不含 origin/HEAD 符号引用
        let branches = list_git_branches(clone_b.to_str().unwrap().to_string()).unwrap();
        assert_eq!(branches.local, vec!["main".to_string()]);
        assert_eq!(
            branches.remote,
            vec!["origin/feature".to_string(), "origin/main".to_string()]
        );

        // 检出远程分支:本地无同名分支 → 创建跟踪分支
        let st = git_checkout(
            clone_b.to_str().unwrap().to_string(),
            "origin/feature".into(),
            false,
            true,
        )
        .unwrap();
        assert_eq!(st.branch.as_deref(), Some("feature"));

        // 本地已有同名分支 → 直接切换(幂等,不报错)
        let st = git_checkout(
            clone_b.to_str().unwrap().to_string(),
            "origin/feature".into(),
            false,
            true,
        )
        .unwrap();
        assert_eq!(st.branch.as_deref(), Some("feature"));

        let _ = fs::remove_dir_all(&origin);
        let _ = fs::remove_dir_all(&clone_a);
        let _ = fs::remove_dir_all(&clone_b);
    }

    #[test]
    fn push_sets_upstream_when_missing() {
        let origin = temp_dir("push-origin");
        git(&origin, &["init", "--bare", "-b", "main"]);

        let clone = temp_dir("push-clone");
        git(&clone, &["clone", origin.to_str().unwrap(), "."]);
        git(&clone, &["config", "user.email", "test@example.com"]);
        git(&clone, &["config", "user.name", "test"]);
        fs::write(clone.join("a.txt"), "a").unwrap();
        git(&clone, &["add", "a.txt"]);
        git(&clone, &["commit", "-m", "c1"]);

        // 首次 push 无 upstream → 自动回退 `git push -u origin HEAD`
        let st = git_push(clone.to_str().unwrap().to_string()).unwrap();
        assert!(st.is_repo);

        // 已建立 upstream 后走普通 push 路径
        fs::write(clone.join("a.txt"), "a2").unwrap();
        git(&clone, &["commit", "-am", "c2"]);
        git_push(clone.to_str().unwrap().to_string()).unwrap();

        let out = git_command(origin.to_str().unwrap())
            .args(["rev-list", "--count", "main"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "2");

        let _ = fs::remove_dir_all(&origin);
        let _ = fs::remove_dir_all(&clone);
    }

    #[test]
    fn pull_reports_conflicts() {
        let origin = temp_dir("pull-origin");
        git(&origin, &["init", "--bare", "-b", "main"]);

        let clone_a = temp_dir("pull-a");
        git(&clone_a, &["clone", origin.to_str().unwrap(), "."]);
        git(&clone_a, &["config", "user.email", "test@example.com"]);
        git(&clone_a, &["config", "user.name", "test"]);
        fs::write(clone_a.join("a.txt"), "base\n").unwrap();
        git(&clone_a, &["add", "a.txt"]);
        git(&clone_a, &["commit", "-m", "c1"]);
        git(&clone_a, &["push", "-u", "origin", "main"]);

        let clone_b = temp_dir("pull-b");
        git(&clone_b, &["clone", origin.to_str().unwrap(), "."]);
        git(&clone_b, &["config", "user.email", "test@example.com"]);
        git(&clone_b, &["config", "user.name", "test"]);
        // 显式指定合并策略,避免新版 git 对分叉分支拒绝 pull
        git(&clone_b, &["config", "pull.rebase", "false"]);

        // 双方改同一行 → 合并冲突
        fs::write(clone_a.join("a.txt"), "remote\n").unwrap();
        git(&clone_a, &["commit", "-am", "remote"]);
        git(&clone_a, &["push"]);

        fs::write(clone_b.join("a.txt"), "local\n").unwrap();
        git(&clone_b, &["commit", "-am", "local"]);

        let res = git_pull(clone_b.to_str().unwrap().to_string()).unwrap();
        assert!(res.status.is_repo);
        assert_eq!(res.conflicts, vec!["a.txt".to_string()]);
        assert_eq!(res.status.conflicted, 1);

        let _ = fs::remove_dir_all(&origin);
        let _ = fs::remove_dir_all(&clone_a);
        let _ = fs::remove_dir_all(&clone_b);
    }
}
