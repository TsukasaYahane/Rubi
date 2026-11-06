use std::sync::RwLock;
use std::time::Duration;

use reqwest::header::{COOKIE, ORIGIN, REFERER, SET_COOKIE, USER_AGENT};
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
/// A server that does not answer within this is given up on.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Whole API calls (small JSON bodies).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
/// Media bodies run for minutes, so only a stalled read fails them.
const MEDIA_READ_TIMEOUT: Duration = Duration::from_secs(30);
/// Idempotent GETs are tried this many times in all when the network fails or the server errs.
const GET_ATTEMPTS: u32 = 3;
const RETRY_BACKOFF: Duration = Duration::from_millis(400);

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
    /// Media streams: no overall deadline, only a read timeout.
    media_client: reqwest::Client,
    /// Same client without redirect following, to read where a short link points.
    no_redirect: reqwest::Client,
    session: RwLock<Session>,
    wbi: Mutex<Option<(WbiKeys, i64)>>,
    limiter: RateLimiter,
}

pub struct Response {
    pub data: Value,
    pub set_cookies: Vec<String>,
}

/// Who the requests are made as: the cookies and device ids, read and replaced as a whole.
pub trait SessionHolder: Send + Sync {
    fn session(&self) -> Session;
    fn update_session(&self, f: &mut dyn FnMut(&mut Session));
}

impl SessionHolder for HttpClient {
    fn session(&self) -> Session {
        HttpClient::session(self)
    }

    fn update_session(&self, f: &mut dyn FnMut(&mut Session)) {
        HttpClient::update_session(self, f)
    }
}

impl HttpClient {
    pub fn new(session: Session) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(UA)
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        let media_client = reqwest::Client::builder()
            .user_agent(UA)
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(MEDIA_READ_TIMEOUT)
            .build()?;
        let no_redirect = reqwest::Client::builder()
            .user_agent(UA)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        Ok(Self {
            client,
            media_client,
            no_redirect,
            session: RwLock::new(session),
            wbi: Mutex::new(None),
            limiter: RateLimiter::new(6, 3.0),
        })
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
        let body = json_body(resp).await?;
        Ok(Response { data: unwrap_envelope(body)?, set_cookies })
    }

    pub async fn get_bytes(&self, url: &str, params: Params, sign: bool) -> Result<Vec<u8>> {
        let resp = self.send_get(url, params, sign).await?.error_for_status()?;
        Ok(resp.bytes().await?.to_vec())
    }

    /// GET of a media stream with the headers players send (no cookies), optionally with one
    /// more header (a `Range`). The caller reads the body.
    pub(crate) async fn media(
        &self,
        url: &str,
        extra: Option<(reqwest::header::HeaderName, &str)>,
    ) -> Result<reqwest::Response> {
        self.limiter.acquire().await;
        let mut req = self.media_client.get(url).header(REFERER, REFERER_URL);
        if let Some((name, value)) = extra {
            req = req.header(name, value);
        }
        Ok(req.send().await?)
    }

    /// Where `url` redirects to (its `Location`), without fetching the target.
    pub async fn redirect_target(&self, url: &str) -> Result<Option<String>> {
        self.limiter.acquire().await;
        let resp = self.no_redirect.get(url).header(REFERER, REFERER_URL).send().await?;
        if !resp.status().is_redirection() {
            return Ok(None);
        }
        Ok(resp.headers().get(reqwest::header::LOCATION).and_then(|v| v.to_str().ok()).map(str::to_owned))
    }

    /// POST form without a csrf token (login endpoints), keeping the `Set-Cookie` headers.
    pub async fn post_form_public(&self, url: &str, form: Params) -> Result<Response> {
        self.limiter.acquire().await;
        let resp = self.decorate(self.client.post(url)).form(&form).send().await?;
        let set_cookies =
            resp.headers().get_all(SET_COOKIE).iter().filter_map(|v| v.to_str().ok().map(str::to_owned)).collect();
        Ok(Response { data: unwrap_envelope(json_body(resp).await?)?, set_cookies })
    }

    /// POST form with the `csrf` token added; requires a logged-in session.
    pub async fn post_form(&self, url: &str, mut form: Params) -> Result<Value> {
        let csrf = self.session().csrf().map(str::to_owned).ok_or(ApiError::NotLoggedIn)?;
        form.push(("csrf".into(), csrf));
        self.limiter.acquire().await;
        let resp = self.decorate(self.client.post(url)).form(&form).send().await?;
        unwrap_envelope(json_body(resp).await?)
    }

    /// POST a multipart form from the page at `referer`, for endpoints that look at where a request
    /// comes from. Gives the whole answer: some say no with a success code.
    pub async fn post_multipart_from(&self, url: &str, referer: &str, form: Params) -> Result<Value> {
        let csrf = self.session().csrf().map(str::to_owned).ok_or(ApiError::NotLoggedIn)?;
        let parts = form.into_iter().chain([("csrf".to_owned(), csrf)]);
        let form = parts.fold(reqwest::multipart::Form::new(), |form, (name, value)| form.text(name, value));
        self.limiter.acquire().await;
        let origin = referer.splitn(4, '/').take(3).collect::<Vec<_>>().join("/");
        let request = self.decorate_from(self.client.post(url), referer).header(ORIGIN, origin);
        let body = json_body(request.multipart(form).send().await?).await?;
        let mut checked = body.clone();
        if checked["message"].as_str().is_none_or(str::is_empty)
            && let Some(said) = body["msg"].as_str()
        {
            checked["message"] = said.into();
        }
        unwrap_envelope(checked)?;
        Ok(body)
    }

    /// POST form like `post_form`, with WBI-signed `query` parameters on the URL (as the web
    /// private-message sender does).
    pub async fn post_form_signed(&self, url: &str, query: Params, mut form: Params) -> Result<Value> {
        let csrf = self.session().csrf().map(str::to_owned).ok_or(ApiError::NotLoggedIn)?;
        form.push(("csrf".into(), csrf));
        let mixin = self.wbi_keys().await?.mixin_key();
        let query = wbi::sign(&query, &mixin, rubi_model::now_secs());
        self.limiter.acquire().await;
        let resp = self.decorate(self.client.post(url)).query(&query).form(&form).send().await?;
        unwrap_envelope(json_body(resp).await?)
    }

    async fn send_get(&self, url: &str, params: Params, sign: bool) -> Result<reqwest::Response> {
        let params = if sign {
            let mixin = self.wbi_keys().await?.mixin_key();
            wbi::sign(&params, &mixin, rubi_model::now_secs())
        } else {
            params
        };
        let mut attempt = 1;
        loop {
            self.limiter.acquire().await;
            let outcome = self.decorate(self.client.get(url)).query(&params).send().await;
            let transient = match &outcome {
                Ok(resp) => resp.status().is_server_error(),
                Err(e) => e.is_connect() || e.is_timeout(),
            };
            if !transient || attempt >= GET_ATTEMPTS {
                return Ok(outcome?);
            }
            tokio::time::sleep(RETRY_BACKOFF * attempt).await;
            attempt += 1;
        }
    }

    fn decorate(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        self.decorate_from(rb, REFERER_URL)
    }

    fn decorate_from(&self, rb: reqwest::RequestBuilder, referer: &str) -> reqwest::RequestBuilder {
        let cookie = self.session().cookie_header();
        let rb = rb.header(REFERER, referer).header(USER_AGENT, UA);
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

/// Reads a JSON body. A 412 or a non-JSON page (the risk-control block page) is reported
/// as such instead of a bare decode error.
async fn json_body(resp: reqwest::Response) -> Result<Value> {
    let status = resp.status();
    let text = resp.text().await?;
    if status.as_u16() == 412 {
        return Err(ApiError::RiskControl { code: -412, message: "HTTP 412".into() });
    }
    serde_json::from_str(&text).map_err(|e| {
        let head: String = text.chars().take(160).collect();
        ApiError::Decode(format!("HTTP {status}, {e}: {head}"))
    })
}

fn unwrap_envelope(body: Value) -> Result<Value> {
    let code = body["code"].as_i64().unwrap_or(0);
    if code != 0 {
        let message = body["message"].as_str().unwrap_or_default().to_owned();
        return Err(ApiError::from_code(code, message));
    }
    // Some endpoints use `result` instead of `data`; a few (s.search hotword) put their
    // payload next to `code`, so the body itself is the data.
    let mut body = body;
    for key in ["data", "result"] {
        if let Some(d) = body.get_mut(key) {
            return Ok(d.take());
        }
    }
    Ok(body)
}
