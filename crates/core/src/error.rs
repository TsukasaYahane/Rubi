#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error(transparent)]
    Api(#[from] rubi_api::ApiError),
    #[error(transparent)]
    Store(#[from] rubi_store::StoreError),
    #[error(transparent)]
    Rules(#[from] rubi_filter::RuleSetError),
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("export archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("invalid data: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
