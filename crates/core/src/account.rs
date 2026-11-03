//! The session: who is signed in, the ways to sign in, signing out.

use std::sync::Arc;

use rubi_api::{AuthApi, LoginCaptcha, LoginKey, NavInfo, QrState, QrTicket, Session, SessionHolder, SolvedCaptcha};

use crate::error::Result;
use crate::ports::LoginState;

pub struct Account {
    http: Arc<dyn SessionHolder>,
    auth: Arc<dyn AuthApi>,
}

impl LoginState for Account {
    fn is_logged_in(&self) -> bool {
        Account::is_logged_in(self)
    }
}

impl Account {
    pub(crate) fn new(http: Arc<dyn SessionHolder>, auth: Arc<dyn AuthApi>) -> Self {
        Self { http, auth }
    }

    /// The session to store (encrypted, by the caller).
    pub fn session(&self) -> Session {
        self.http.session()
    }

    pub fn is_logged_in(&self) -> bool {
        self.http.session().is_logged_in()
    }

    /// Forgets the credentials; the device ids stay, so the device is not new to B站.
    pub fn logout(&self) {
        self.http.update_session(&mut |s| {
            *s = Session { buvid3: s.buvid3.take(), buvid4: s.buvid4.take(), ..Session::default() };
        });
    }

    pub async fn ensure_device_ids(&self) -> Result<()> {
        Ok(self.auth.ensure_device_ids().await?)
    }

    pub async fn whoami(&self) -> Result<NavInfo> {
        Ok(self.auth.nav().await?)
    }

    pub async fn qr_generate(&self) -> Result<QrTicket> {
        Ok(self.auth.qr_generate().await?)
    }

    pub async fn qr_poll(&self, key: &str) -> Result<QrState> {
        Ok(self.auth.qr_poll(key).await?)
    }

    pub async fn login_captcha(&self) -> Result<LoginCaptcha> {
        Ok(self.auth.login_captcha().await?)
    }

    /// RSA key and salt; the app encrypts `hash + password` itself, so the plain password
    /// never reaches Rust.
    pub async fn login_key(&self) -> Result<LoginKey> {
        Ok(self.auth.login_key().await?)
    }

    pub async fn password_login(&self, username: &str, encrypted: &str, captcha: &SolvedCaptcha) -> Result<Session> {
        Ok(self.auth.password_login(username, encrypted, captcha).await?)
    }

    /// Sends the SMS code; returns the key [`sms_login`](Self::sms_login) needs.
    pub async fn sms_send(&self, cid: u32, tel: &str, captcha: &SolvedCaptcha) -> Result<String> {
        Ok(self.auth.sms_send(cid, tel, captcha).await?)
    }

    pub async fn sms_login(&self, cid: u32, tel: &str, code: &str, captcha_key: &str) -> Result<Session> {
        Ok(self.auth.sms_login(cid, tel, code, captcha_key).await?)
    }
}
