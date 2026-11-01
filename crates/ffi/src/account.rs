//! Signing in and out.

use rubi_api::QrState;

use crate::{FfiAccount, FfiLoginCaptcha, FfiLoginKey, FfiQrState, FfiQrTicket, FfiSolvedCaptcha, Res, RubiCore};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// The current session, to be stored encrypted by the caller.
    pub fn session_json(&self) -> Res<String> {
        Ok(serde_json::to_string(&self.account.session())?)
    }

    pub fn logout(&self) {
        self.account.logout();
    }

    pub async fn ensure_device_ids(&self) -> Res<()> {
        Ok(self.account.ensure_device_ids().await?)
    }

    pub async fn account(&self) -> Res<FfiAccount> {
        let nav = self.account.whoami().await?;
        Ok(FfiAccount { logged_in: nav.is_login, mid: nav.mid, name: nav.uname })
    }

    pub async fn qr_generate(&self) -> Res<FfiQrTicket> {
        let t = self.account.qr_generate().await?;
        Ok(FfiQrTicket { url: t.url, key: t.key })
    }

    pub async fn qr_poll(&self, key: String) -> Res<FfiQrState> {
        Ok(match self.account.qr_poll(&key).await? {
            QrState::Waiting => FfiQrState::Waiting,
            QrState::Scanned => FfiQrState::Scanned,
            QrState::Expired => FfiQrState::Expired,
            QrState::Success(s) => FfiQrState::Success { session_json: serde_json::to_string(&s)? },
        })
    }

    pub async fn login_captcha(&self) -> Res<FfiLoginCaptcha> {
        let c = self.account.login_captcha().await?;
        Ok(FfiLoginCaptcha { token: c.token, gt: c.gt, challenge: c.challenge })
    }

    /// RSA key (PEM) and salt for the password; the app encrypts `hash + password` itself.
    pub async fn login_key(&self) -> Res<FfiLoginKey> {
        let k = self.account.login_key().await?;
        Ok(FfiLoginKey { hash: k.hash, public_key_pem: k.key })
    }

    /// `encrypted_password` is `base64(RSA-PKCS1(hash + password))`. Returns the session JSON
    /// to persist, like a finished QR login.
    pub async fn password_login(
        &self,
        username: String,
        encrypted_password: String,
        captcha: FfiSolvedCaptcha,
    ) -> Res<String> {
        let s = self.account.password_login(&username, &encrypted_password, &captcha.into()).await?;
        Ok(serde_json::to_string(&s)?)
    }

    /// Sends the SMS code; keep the returned key for `sms_login`.
    pub async fn sms_send(&self, cid: u32, tel: String, captcha: FfiSolvedCaptcha) -> Res<String> {
        Ok(self.account.sms_send(cid, &tel, &captcha.into()).await?)
    }

    pub async fn sms_login(&self, cid: u32, tel: String, code: String, captcha_key: String) -> Res<String> {
        let s = self.account.sms_login(cid, &tel, &code, &captcha_key).await?;
        Ok(serde_json::to_string(&s)?)
    }
}
