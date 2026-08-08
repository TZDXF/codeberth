use std::path::Path;

use crate::commands::{files, script, walk};
use crate::error::AppResult;
use crate::models::ProjectAssets;

/// 进入详情页时的资产扫描:单次目录遍历同时产出 package scripts 与 compose 文件,
/// 替代原先 list_package_scripts / scan_compose_files 两条命令各扫一遍。
/// walk 结果带 30s TTL 缓存(见 walk::project_files_cached),
/// 反复进出详情页、多个卡片同时挂载不会重复遍历目录。
#[tauri::command]
pub fn scan_project_assets(path: String) -> AppResult<ProjectAssets> {
    files::ensure_dir(&path)?;
    let dir = Path::new(&path);
    let walked = walk::project_files_cached(dir);
    Ok(ProjectAssets {
        package_scripts: script::package_scripts_from_files(dir, &walked),
        compose_files: files::compose_files_from_files(dir, &walked),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_project_dir(tag: &str) -> String {
        let dir = std::env::temp_dir().join(format!(
            "repomeow-scan-{tag}-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir.to_string_lossy().to_string()
    }

    #[test]
    fn scans_scripts_and_compose_in_one_pass() {
        let dir = temp_project_dir("assets");
        let p = Path::new(&dir);

        fs::write(p.join("package.json"), r#"{"name":"demo","scripts":{"dev":"vite"}}"#).unwrap();
        fs::create_dir_all(p.join("deploy")).unwrap();
        fs::write(p.join("deploy/app.yml"), "services:\n  web:\n    image: nginx\n").unwrap();

        let assets = scan_project_assets(dir.clone()).unwrap();
        assert_eq!(assets.package_scripts.len(), 1);
        assert_eq!(assets.package_scripts[0].dir, ".");
        assert_eq!(assets.compose_files.len(), 1);
        assert_eq!(assets.compose_files[0].path, "deploy/app.yml");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_missing_dir() {
        assert!(matches!(
            scan_project_assets("D:/no/such/dir-xyz".into()),
            Err(ref e) if e.is_code(crate::error::ErrorCode::InvalidPath)
        ));
    }
}
