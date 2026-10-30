#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("network error: {0}")]
    Http(#[from] reqwest::Error),
    /// -352 / -412: the request was rejected by risk control.
    #[error("rejected by risk control (code {code}): {message}")]
    RiskControl { code: i64, message: String },
    #[error("not logged in")]
    NotLoggedIn,
    #[error("API error {code}: {message}")]
    Api { code: i64, message: String },
    #[error("unexpected response: {0}")]
    Decode(String),
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError::Decode(e.to_string())
    }
}

impl From<prost::DecodeError> for ApiError {
    fn from(e: prost::DecodeError) -> Self {
        ApiError::Decode(e.to_string())
    }
}

impl ApiError {
    pub(crate) fn from_code(code: i64, message: String) -> Self {
        match code {
            -352 | -412 => ApiError::RiskControl { code, message },
            -101 => ApiError::NotLoggedIn,
            _ => ApiError::Api { code, message },
        }
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;
