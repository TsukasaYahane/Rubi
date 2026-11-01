//! The one error type the app sees.

use rubi_api::ApiError;
use rubi_core::CoreError;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum RubiError {
    #[error("网络错误：{msg}")]
    Network { msg: String },
    #[error("请求被风控拦截：{msg}")]
    RiskControl { msg: String },
    #[error("需要登录")]
    NotLoggedIn,
    #[error("{msg}")]
    Invalid { msg: String },
    #[error("{msg}")]
    Internal { msg: String },
}

impl From<ApiError> for RubiError {
    fn from(e: ApiError) -> Self {
        match e {
            ApiError::Http(e) => RubiError::Network { msg: e.to_string() },
            ApiError::RiskControl { .. } => RubiError::RiskControl { msg: e.to_string() },
            ApiError::NotLoggedIn => RubiError::NotLoggedIn,
            other => RubiError::Internal { msg: other.to_string() },
        }
    }
}

impl From<CoreError> for RubiError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Api(a) => a.into(),
            CoreError::Rules(_) | CoreError::Json(_) | CoreError::Invalid(_) => {
                RubiError::Invalid { msg: e.to_string() }
            }
            other => RubiError::Internal { msg: other.to_string() },
        }
    }
}

impl From<serde_json::Error> for RubiError {
    fn from(e: serde_json::Error) -> Self {
        RubiError::Invalid { msg: e.to_string() }
    }
}
