use std::sync::RwLock;

use reqwest::header::{COOKIE, REFERER, SET_COOKIE, USER_AGENT};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::error::{ApiError, Result};
use crate::limiter::RateLimiter;
use crate::session::Session;
use crate::wbi::{self, WbiKeys};

pub const UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";
pub const REFERER_URL: &str = "https://www.bilibili.com/";
const NAV: &str = "https://api.bilibili.com/x/web-interface/nav";
/// WBI keys rotate daily.
const WBI_TTL_SECS: i64 = 6 * 3600;

pub type Params = Vec<(String, String)>;

/// Builds owned query params from `(&str, impl ToString)` pairs.
#[macro_export]
#[doc(hidden)]
macro_rules! params {
    ($($k:expr => $v:expr),* $(,)?) => { vec![$(($k.to_string(), $v.to_string())),*] };
}

/// The transport shared by every API implementation: cookies, WBI, rate limit.
pub struct HttpClient {
    client: reqwest::Client,
    session: RwLock<Session>,
    wbi: Mutex<Option<(WbiKeys, i64)>>,
    limiter: RateLimiter,
}

pub struct Response {
    pub data: Value,
    pub set_cookies: Vec<String>,
}

impl HttpClient {
    pub fn new(session: Session) -> Result<Self> {
        let client = reqwest::Client::builder().user_agent(UA).gzip(true).brotli(true).deflate(true).build()?;
        Ok(Self { client, session: RwLock::new(session), wbi: Mutex::new(None), limiter: RateLimiter::new(6, 3.0) })
    }

    pub fn session(&self) -> Session {
        self.session.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn update_session(&self, f: impl FnOnce(&mut Session)) {
        f(&mut self.session.write().unwrap_or_else(|e| e.into_inner()));
    }

    /// Headers the player must send when fetching media streams.
    pub fn stream_headers(&self) -> Vec<(String, String)> {
        vec![("Referer".into(), REFERER_URL.into()), ("User-Agent".into(), UA.into())]
    }

    /// GET returning the envelope's `data`, failing on a non-zero `code`.
    pub async fn get_json(&self, url: &str, params: Params, sign: bool) -> Result<Value> {
        Ok(self.get_full(url, params, sign).await?.data)
    }

    pub async fn get_full(&self, url: &str, params: Params, sign: bool) -> Result<Response> {
        let resp = self.send_get(url, params, sign).await?;
        let set_cookies =
            resp.headers().get_all(SET_COOKIE).iter().filter_map(|v| v.to_str().ok().map(str::to_owned)).collect();
        let body: Value = resp.json().await?;
        Ok(Response { data: unwrap_envelope(body)?, set_cookies })
    }

    pub async fn get_bytes(&self, url: &str, params: Params, sign: bool) -> Result<Vec<u8>> {
        let resp = self.send_get(url, params, sign).await?.error_for_status()?;
        Ok(resp.bytes().await?.to_vec())
    }

    /// POST form with the `csrf` token added; requires a logged-in session.
    pub async fn post_form(&self, url: &str, mut form: Params) -> Result<Value> {
        let csrf = self.session().csrf().map(str::to_owned).ok_or(ApiError::NotLoggedIn)?;
        form.push(("csrf".into(), csrf));
        self.limiter.acquire().await;
        let resp = self.decorate(self.client.post(url)).form(&form).send().await?;
        unwrap_envelope(resp.json().await?)
    }

    async fn send_get(&self, url: &str, params: Params, sign: bool) -> Result<reqwest::Response> {
        let params = if sign {
            let mixin = self.wbi_keys().await?.mixin_key();
            wbi::sign(&params, &mixin, rubi_model::now_secs())
        } else {
            params
        };
        self.limiter.acquire().await;
        Ok(self.decorate(self.client.get(url)).query(&params).send().await?)
    }

    fn decorate(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let cookie = self.session().cookie_header();
        let rb = rb.header(REFERER, REFERER_URL).header(USER_AGENT, UA);
        if cookie.is_empty() { rb } else { rb.header(COOKIE, cookie) }
    }

    async fn wbi_keys(&self) -> Result<WbiKeys> {
        let mut cached = self.wbi.lock().await;
        let now = rubi_model::now_secs();
        if let Some((keys, at)) = cached.as_ref()
            && now - at < WBI_TTL_SECS
        {
            return Ok(keys.clone());
        }
        // `nav` answers -101 for guests but still carries `wbi_img`, so read the raw body.
        self.limiter.acquire().await;
        let body: Value = self.decorate(self.client.get(NAV)).send().await?.json().await?;
        let img = &body["data"]["wbi_img"];
        let keys = WbiKeys::from_urls(
            img["img_url"].as_str().unwrap_or_default(),
            img["sub_url"].as_str().unwrap_or_default(),
        )
        .ok_or_else(|| ApiError::Decode("nav response has no wbi_img".into()))?;
        *cached = Some((keys.clone(), now));
        Ok(keys)
    }
}

fn unwrap_envelope(body: Value) -> Result<Value> {
    let code = body["code"].as_i64().unwrap_or(0);
    if code != 0 {
        let message = body["message"].as_str().unwrap_or_default().to_owned();
        return Err(ApiError::from_code(code, message));
    }
    // Some endpoints use `result` instead of `data`.
    let mut body = body;
    Ok(match body.get_mut("data") {
        Some(d) => d.take(),
        None => body.get_mut("result").map(Value::take).unwrap_or(Value::Null),
    })
}
