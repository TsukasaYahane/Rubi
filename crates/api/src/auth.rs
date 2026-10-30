use std::sync::Arc;

use async_trait::async_trait;

use crate::error::{ApiError, Result};
use crate::http::HttpClient;
use crate::params;
use crate::parse::{str_of, u64_of};
use crate::session::Session;

const QR_GENERATE: &str = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
const QR_POLL: &str = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";
const SPI: &str = "https://api.bilibili.com/x/frontend/finger/spi";
const NAV: &str = "https://api.bilibili.com/x/web-interface/nav";

#[derive(Debug, Clone, PartialEq)]
pub struct QrTicket {
    /// Encode this in the QR code.
    pub url: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QrState {
    Waiting,
    Scanned,
    Expired,
    /// Login finished; the session already holds the new cookies.
    Success(Session),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NavInfo {
    pub is_login: bool,
    pub mid: u64,
    pub uname: String,
}

#[async_trait]
pub trait AuthApi: Send + Sync {
    /// Gets buvid3/4 for a guest session if it has none.
    async fn ensure_device_ids(&self) -> Result<()>;
    async fn qr_generate(&self) -> Result<QrTicket>;
    async fn qr_poll(&self, key: &str) -> Result<QrState>;
    async fn nav(&self) -> Result<NavInfo>;
}

pub struct WebAuth {
    http: Arc<HttpClient>,
}

impl WebAuth {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl AuthApi for WebAuth {
    async fn ensure_device_ids(&self) -> Result<()> {
        if self.http.session().buvid3.is_some() {
            return Ok(());
        }
        let data = self.http.get_json(SPI, params![], false).await?;
        let (b3, b4) = (str_of(&data["b_3"]), str_of(&data["b_4"]));
        self.http.update_session(|s| {
            s.buvid3 = Some(b3).filter(|v| !v.is_empty());
            s.buvid4 = Some(b4).filter(|v| !v.is_empty());
        });
        Ok(())
    }

    async fn qr_generate(&self) -> Result<QrTicket> {
        let data = self.http.get_json(QR_GENERATE, params![], false).await?;
        Ok(QrTicket { url: str_of(&data["url"]), key: str_of(&data["qrcode_key"]) })
    }

    async fn qr_poll(&self, key: &str) -> Result<QrState> {
        let resp = self.http.get_full(QR_POLL, params!["qrcode_key" => key], false).await?;
        match resp.data["code"].as_i64().unwrap_or(-1) {
            0 => {
                let refresh = str_of(&resp.data["refresh_token"]);
                self.http.update_session(|s| {
                    s.absorb_set_cookies(resp.set_cookies.iter().map(String::as_str));
                    s.refresh_token = Some(refresh).filter(|t| !t.is_empty());
                });
                let session = self.http.session();
                if session.is_logged_in() {
                    Ok(QrState::Success(session))
                } else {
                    Err(ApiError::Decode("login succeeded without SESSDATA".into()))
                }
            }
            86101 => Ok(QrState::Waiting),
            86090 => Ok(QrState::Scanned),
            86038 => Ok(QrState::Expired),
            code => Err(ApiError::Api { code, message: str_of(&resp.data["message"]) }),
        }
    }

    async fn nav(&self) -> Result<NavInfo> {
        match self.http.get_json(NAV, params![], false).await {
            Ok(d) => Ok(NavInfo {
                is_login: d["isLogin"].as_bool().unwrap_or(false),
                mid: u64_of(&d["mid"]),
                uname: str_of(&d["uname"]),
            }),
            Err(ApiError::NotLoggedIn) => Ok(NavInfo { is_login: false, mid: 0, uname: String::new() }),
            Err(e) => Err(e),
        }
    }
}
