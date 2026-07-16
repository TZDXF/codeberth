use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub ahead: i32,
    pub behind: i32,
    pub staged: i32,
    pub modified: i32,
    pub untracked: i32,
    pub remote_ahead: i32,
    pub last_fetch_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<Tag>,
    pub git: Option<GitStatus>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackageScript {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomCommand {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub command: String,
    pub description: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EditorKind {
    Vscode,
    Explorer,
    Terminal,
}
