use std::path::Path;

use crate::commands::walk;
use crate::error::{AppError, AppResult};
use crate::models::{ComposeFile, ReadmeContent};

/// README 候选文件名,按优先级排列(大小写常见变体)
const README_CANDIDATES: &[&str] = &[
    "README.md",
    "readme.md",
    "README.MD",
    "Readme.md",
    "README.markdown",
    "README.txt",
    "README",
];

/// README 读取上限 512KB,避免超大文件拖垮前端渲染
const README_MAX_BYTES: u64 = 512 * 1024;

/// compose 文件大小上限 256KB,超过的直接跳过(正常 compose 文件远小于此)
const COMPOSE_MAX_BYTES: u64 = 256 * 1024;

fn ensure_dir(path: &str) -> AppResult<()> {
    if !Path::new(path).is_dir() {
        return Err(AppError::Invalid(format!("目录不存在: {path}")));
    }
    Ok(())
}

/// 在目录中按候选名查找文件,返回第一个存在的文件名。
/// 用 read_dir 做大小写精确匹配,避免 Windows/macOS 大小写不敏感文件系统
/// 把 readme.md 误判成 README.md,保证候选优先级在所有平台行为一致。
fn find_file(dir: &Path, candidates: &[&str]) -> Option<String> {
    let existing: Vec<String> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    candidates
        .iter()
        .find(|name| existing.iter().any(|f| f == *name))
        .map(|name| name.to_string())
}

/// 读取项目 README;不存在时返回 None
#[tauri::command]
pub fn read_readme(path: String) -> AppResult<Option<ReadmeContent>> {
    ensure_dir(&path)?;
    let dir = Path::new(&path);
    let Some(file_name) = find_file(dir, README_CANDIDATES) else {
        return Ok(None);
    };
    let file = dir.join(&file_name);
    // 超过上限只取前 README_MAX_BYTES 字节(按 UTF-8 边界截断)
    let meta = std::fs::metadata(&file)?;
    let content = if meta.len() > README_MAX_BYTES {
        let bytes = std::fs::read(&file)?;
        // 按 UTF-8 边界截断:跳过 continuation byte(0b10xxxxxx)
        let mut end = README_MAX_BYTES as usize;
        while end > 0 && (bytes[end] & 0xC0) == 0x80 {
            end -= 1;
        }
        String::from_utf8_lossy(&bytes[..end]).into_owned()
    } else {
        std::fs::read_to_string(&file)?
    };
    Ok(Some(ReadmeContent { file_name, content }))
}

/// 判断 YAML 内容是否为 Docker Compose 格式:顶层含 mapping 类型的 services。
/// 是则返回服务名列表;非法 YAML / 无 services(CI 配置等)返回 None。
fn parse_compose(content: &str) -> Option<Vec<String>> {
    let yaml = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(content).ok()?;
    let services = yaml.get("services")?.as_mapping()?;
    Some(
        services
            .keys()
            .filter_map(|k| k.as_str().map(String::from))
            .collect(),
    )
}

/// 是否为可能包含 compose 定义的 YAML 文件(按扩展名粗筛)
fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("yml") || e.eq_ignore_ascii_case("yaml"))
}

/// 递归扫描项目内的 Docker Compose 文件(尊重 git 排除规则,按内容识别)
#[tauri::command]
pub fn scan_compose_files(path: String) -> AppResult<Vec<ComposeFile>> {
    ensure_dir(&path)?;
    let dir = Path::new(&path);
    let mut files: Vec<ComposeFile> = walk::project_files(dir)
        .iter()
        .filter(|rel| is_yaml_file(rel))
        .filter(|rel| {
            std::fs::metadata(dir.join(rel))
                .map(|m| m.len() <= COMPOSE_MAX_BYTES)
                .unwrap_or(false)
        })
        .filter_map(|rel| {
            let content = std::fs::read_to_string(dir.join(rel)).ok()?;
            let services = parse_compose(&content)?;
            let file_name = rel
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())?;
            Some(ComposeFile {
                path: walk::to_slash(rel),
                file_name,
                services,
            })
        })
        .collect();
    // 根目录文件优先,同级按路径字典序
    files.sort_by(|a, b| {
        (a.path.contains('/'), &a.path).cmp(&(b.path.contains('/'), &b.path))
    });
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// 建一个带唯一名字的临时目录,返回路径字符串
    fn temp_project_dir(tag: &str) -> String {
        let dir = std::env::temp_dir().join(format!(
            "projectdev-files-{tag}-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir.to_string_lossy().to_string()
    }

    #[test]
    fn readme_missing_and_found() {
        let dir = temp_project_dir("readme");
        let p = Path::new(&dir);

        assert!(read_readme(dir.clone()).unwrap().is_none());

        fs::write(p.join("readme.md"), "# Hello").unwrap();
        let r = read_readme(dir.clone()).unwrap().unwrap();
        assert_eq!(r.file_name, "readme.md");
        assert_eq!(r.content, "# Hello");

        // 优先级:README.md 高于 readme.md。
        // 注意先删 readme.md:Windows/macOS 大小写不敏感文件系统上,
        // 直接写 README.md 会覆盖同名文件但保留原有目录项大小写。
        fs::remove_file(p.join("readme.md")).unwrap();
        fs::write(p.join("README.md"), "# Priority").unwrap();
        let r = read_readme(dir.clone()).unwrap().unwrap();
        assert_eq!(r.file_name, "README.md");
        assert_eq!(r.content, "# Priority");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn readme_rejects_missing_dir() {
        assert!(matches!(
            read_readme("D:/no/such/dir-xyz".into()),
            Err(AppError::Invalid(_))
        ));
    }

    #[test]
    fn compose_scan_by_content_not_name() {
        let dir = temp_project_dir("compose-content");
        let p = Path::new(&dir);

        // 非标准文件名,但内容是 compose 格式 -> 识别
        fs::write(p.join("app.yml"), "services:\n  web:\n    image: nginx\n").unwrap();
        // 标准文件名但无 services -> 不识别
        fs::write(p.join("docker-compose.yml"), "name: demo\n").unwrap();
        // CI 配置(yml 但非 compose)-> 不识别
        fs::write(p.join("ci.yaml"), "on: push\njobs: {}\n").unwrap();
        // 非法 YAML -> 不识别
        fs::write(p.join("broken.yml"), "services: [not a map").unwrap();
        // 非 yml 文件不参与
        fs::write(p.join("services.txt"), "services:\n  x: {}\n").unwrap();

        let files = scan_compose_files(dir.clone()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "app.yml");
        assert_eq!(files[0].file_name, "app.yml");
        assert_eq!(files[0].services, vec!["web"]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compose_scan_nested_and_gitignored() {
        let dir = temp_project_dir("compose-nested");
        let p = Path::new(&dir);

        // 嵌套子目录中的 compose
        fs::create_dir_all(p.join("deploy/prod")).unwrap();
        fs::write(
            p.join("deploy/prod/stack.yaml"),
            "services:\n  api:\n    build: .\n  db:\n    image: postgres:16\n",
        )
        .unwrap();
        // 被 .gitignore 排除的目录不扫描
        fs::create_dir_all(p.join("ignored")).unwrap();
        fs::write(p.join("ignored/svc.yml"), "services:\n  x: {}\n").unwrap();
        fs::write(p.join(".gitignore"), "ignored/\n").unwrap();

        let files = scan_compose_files(dir.clone()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "deploy/prod/stack.yaml");
        assert_eq!(files[0].services, vec!["api", "db"]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compose_scan_root_first_ordering() {
        let dir = temp_project_dir("compose-order");
        let p = Path::new(&dir);

        fs::create_dir_all(p.join("abc")).unwrap();
        fs::write(p.join("abc/x.yml"), "services:\n  a: {}\n").unwrap();
        // 根目录文件名字典序更大,但仍应排在前面
        fs::write(p.join("z.yml"), "services:\n  z: {}\n").unwrap();
        fs::write(p.join("a.yml"), "services:\n  a: {}\n").unwrap();

        let files = scan_compose_files(dir.clone()).unwrap();
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, vec!["a.yml", "z.yml", "abc/x.yml"]);

        let _ = fs::remove_dir_all(&dir);
    }
}
