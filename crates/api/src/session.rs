use serde::{Deserialize, Serialize};

/// Cookies and tokens of the current user. Guests only carry the buvid pair.
/// Persisting this securely is the caller's job (Android Keystore on device).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub sessdata: Option<String>,
    pub bili_jct: Option<String>,
    pub dede_user_id: Option<String>,
    pub buvid3: Option<String>,
    pub buvid4: Option<String>,
    pub refresh_token: Option<String>,
}

impl Session {
    pub fn is_logged_in(&self) -> bool {
        self.sessdata.is_some()
    }

    pub fn csrf(&self) -> Option<&str> {
        self.bili_jct.as_deref()
    }

    pub fn cookie_header(&self) -> String {
        [
            ("SESSDATA", &self.sessdata),
            ("bili_jct", &self.bili_jct),
            ("DedeUserID", &self.dede_user_id),
            ("buvid3", &self.buvid3),
            ("buvid4", &self.buvid4),
        ]
        .iter()
        .filter_map(|(k, v)| v.as_ref().map(|v| format!("{k}={v}")))
        .collect::<Vec<_>>()
        .join("; ")
    }

    /// Absorbs `Set-Cookie` header values; unrelated cookies are ignored.
    pub fn absorb_set_cookies<'a>(&mut self, headers: impl IntoIterator<Item = &'a str>) {
        for h in headers {
            let Some((name, value)) = h.split(';').next().and_then(|kv| kv.split_once('=')) else { continue };
            let value = Some(value.trim().to_owned());
            match name.trim() {
                "SESSDATA" => self.sessdata = value,
                "bili_jct" => self.bili_jct = value,
                "DedeUserID" => self.dede_user_id = value,
                "buvid3" => self.buvid3 = value,
                "buvid4" => self.buvid4 = value,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absorbs_and_renders_cookies() {
        let mut s = Session::default();
        s.absorb_set_cookies(["SESSDATA=abc%2C1; Path=/; HttpOnly", "bili_jct=x; Path=/", "sid=zzz; Path=/"]);
        assert!(s.is_logged_in());
        assert_eq!(s.cookie_header(), "SESSDATA=abc%2C1; bili_jct=x");
    }
}
