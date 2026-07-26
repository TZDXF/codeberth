use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

/// 递归列出项目内未被 git 排除的文件,返回相对 root 的路径。
///
/// 规则:
/// - 尊重 .gitignore / .ignore / 全局 gitignore 及父目录 gitignore;
///   `require_git(false)` 表示即使目录不是 git 仓库,只要存在 .gitignore 就生效。
/// - 默认跳过隐藏条目(.git、.github 等点开头目录/文件)。
/// - 无条件跳过 node_modules:未被 gitignore 时(如非 git 项目)扫描它既慢又吵,
///   其内部的 package.json / yml 对本工具没有价值。
///
/// 结果按路径排序,保证输出确定性。
pub fn project_files(root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = WalkBuilder::new(root)
        .require_git(false)
        .filter_entry(|e| {
            // 目录(或文件)名为 node_modules 时不再深入
            e.file_name() != "node_modules"
        })
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()))
        .filter_map(|e| e.path().strip_prefix(root).ok().map(PathBuf::from))
        .collect();
    files.sort();
    files
}

/// 相对路径转 '/' 分隔字符串(Windows 下 '\' 归一化)
pub fn to_slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "codeberth-walk-{tag}-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn respects_gitignore_and_skips_node_modules() {
        let dir = temp_dir("gitignore");
        fs::write(dir.join("keep.yml"), "a: 1").unwrap();
        fs::create_dir_all(dir.join("logs")).unwrap();
        fs::write(dir.join("logs/app.yml"), "a: 1").unwrap();
        fs::write(dir.join(".gitignore"), "logs/\n").unwrap();

        let files = project_files(&dir);
        let names: Vec<String> = files.iter().map(|p| to_slash(p)).collect();
        assert!(names.contains(&"keep.yml".to_string()));
        assert!(!names.iter().any(|n| n.starts_with("logs/")));

        // node_modules 即使未被 gitignore 也跳过
        fs::remove_file(dir.join(".gitignore")).unwrap();
        fs::create_dir_all(dir.join("node_modules/dep")).unwrap();
        fs::write(dir.join("node_modules/dep/package.json"), "{}").unwrap();
        let files = project_files(&dir);
        assert!(!files
            .iter()
            .any(|p| to_slash(p).starts_with("node_modules/")));
        // .gitignore 文件本身是隐藏文件,不出现在结果里
        assert!(!files.iter().any(|p| to_slash(p) == ".gitignore"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn skips_hidden_dirs() {
        let dir = temp_dir("hidden");
        fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        fs::write(dir.join(".github/workflows/ci.yml"), "on: push").unwrap();
        fs::write(dir.join("app.yml"), "a: 1").unwrap();

        let files = project_files(&dir);
        let names: Vec<String> = files.iter().map(|p| to_slash(p)).collect();
        assert_eq!(names, vec!["app.yml"]);

        let _ = fs::remove_dir_all(&dir);
    }
}
