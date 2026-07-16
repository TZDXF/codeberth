use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("记录不存在: {0}")]
    NotFound(String),
    #[error("冲突: {0}")]
    Conflict(String),
    #[error("无效输入: {0}")]
    Invalid(String),
    // Windows 上不会触发(仅非 Win/Mac 平台兜底使用)
    #[allow(dead_code)]
    #[error("外部命令失败: {0}")]
    External(String),
}

// Tauri 命令的 Err 必须实现 Serialize,序列化为错误文案传给前端
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
