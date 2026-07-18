use std::path::Path;

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

/// Docker Compose 候选文件名,按官方约定优先级排列
const COMPOSE_CANDIDATES: &[&str] = &[
    "compose.yaml",
    "compose.yml",
    "docker-compose.yml",
    "docker-compose.yaml",
];

/// README 读取上限 512KB,避免超大文件拖垮前端渲染
const README_MAX_BYTES: u64 = 512 * 1024;

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

/// 解析 compose 文件中的 services 列表;解析失败或没有 services 返回空列表
fn parse_services(content: &str) -> Vec<String> {
    let Ok(yaml) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(content) else {
        return vec![];
    };
    yaml.get("services")
        .and_then(|s| s.as_mapping())
        .map(|m| {
            m.keys()
                .filter_map(|k| k.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// 检测项目内 Docker Compose 文件;不存在时返回 None
#[tauri::command]
pub fn detect_compose_file(path: String) -> AppResult<Option<ComposeFile>> {
    ensure_dir(&path)?;
    let dir = Path::new(&path);
    let Some(file_name) = find_file(dir, COMPOSE_CANDIDATES) else {
        return Ok(None);
    };
    // 读失败(编码问题等)不阻塞检测,只是没有服务列表
    let services = std::fs::read_to_string(dir.join(&file_name))
        .map(|c| parse_services(&c))
        .unwrap_or_default();
    Ok(Some(ComposeFile {
        file_name,
        services,
    }))
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
    fn compose_detect_by_priority() {
        let dir = temp_project_dir("compose");
        let p = Path::new(&dir);

        assert!(detect_compose_file(dir.clone()).unwrap().is_none());

        fs::write(p.join("docker-compose.yml"), "services: {}").unwrap();
        assert_eq!(
            detect_compose_file(dir.clone()).unwrap().unwrap().file_name,
            "docker-compose.yml"
        );

        // compose.yaml 优先级更高
        fs::write(p.join("compose.yaml"), "services: {}").unwrap();
        assert_eq!(
            detect_compose_file(dir.clone()).unwrap().unwrap().file_name,
            "compose.yaml"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compose_parses_services() {
        let dir = temp_project_dir("services");
        let p = Path::new(&dir);

        fs::write(
            p.join("docker-compose.yml"),
            r#"
version: "3"
services:
  web:
    build: .
    ports: ["8080:80"]
  db:
    image: postgres:16
  cache:
    image: redis:7
"#,
        )
        .unwrap();
        let c = detect_compose_file(dir.clone()).unwrap().unwrap();
        assert_eq!(c.services, vec!["web", "db", "cache"]);

        // 无 services 字段 / 非法 YAML -> 空列表,不影响检测
        fs::write(p.join("docker-compose.yml"), "name: demo\n").unwrap();
        let c = detect_compose_file(dir.clone()).unwrap().unwrap();
        assert!(c.services.is_empty());

        fs::write(p.join("docker-compose.yml"), "services: [not a map").unwrap();
        let c = detect_compose_file(dir.clone()).unwrap().unwrap();
        assert!(c.services.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }
}
