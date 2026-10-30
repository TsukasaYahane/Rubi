//! WBI request signing (docs/misc/sign/wbi.md).

use md5::{Digest, Md5};

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29, 28, 14, 39, 12, 38,
    41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25, 54, 21, 56, 59, 6, 63, 57, 62, 11, 36,
    20, 34, 44, 52,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WbiKeys {
    pub img_key: String,
    pub sub_key: String,
}

impl WbiKeys {
    /// Extracts the keys from the `wbi_img` urls of `/x/web-interface/nav`.
    pub fn from_urls(img_url: &str, sub_url: &str) -> Option<Self> {
        let stem = |u: &str| u.rsplit('/').next()?.split('.').next().map(str::to_owned);
        Some(Self { img_key: stem(img_url)?, sub_key: stem(sub_url)? })
    }

    pub fn mixin_key(&self) -> String {
        let raw: Vec<char> = format!("{}{}", self.img_key, self.sub_key).chars().collect();
        MIXIN_KEY_ENC_TAB.iter().filter_map(|&i| raw.get(i)).take(32).collect()
    }
}

/// Percent-encodes like JavaScript's `encodeURIComponent`.
pub fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Returns `params` plus `wts` and `w_rid`, ready to be sent.
pub fn sign(params: &[(String, String)], mixin_key: &str, wts: i64) -> Vec<(String, String)> {
    let mut sorted: Vec<(String, String)> = params
        .iter()
        .map(|(k, v)| (k.clone(), v.chars().filter(|c| !"!'()*".contains(*c)).collect()))
        .chain(std::iter::once(("wts".to_string(), wts.to_string())))
        .collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let query = sorted.iter().map(|(k, v)| format!("{}={}", encode(k), encode(v))).collect::<Vec<_>>().join("&");
    let w_rid = format!("{:x}", Md5::digest(format!("{query}{mixin_key}").as_bytes()));
    sorted.push(("w_rid".to_string(), w_rid));
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> WbiKeys {
        WbiKeys::from_urls(
            "https://i0.hdslb.com/bfs/wbi/7cd084941338484aae1ad9425b84077c.png",
            "https://i0.hdslb.com/bfs/wbi/4932caff0ff746eab6f01bf08b70ac45.png",
        )
        .unwrap()
    }

    #[test]
    fn mixin_key_matches_docs() {
        assert_eq!(keys().mixin_key(), "ea1db124af3c7062474693fa704f4ff8");
    }

    #[test]
    fn signature_matches_docs() {
        let params = [("foo", "114"), ("bar", "514"), ("zab", "1919810")].map(|(k, v)| (k.to_string(), v.to_string()));
        let signed = sign(&params, &keys().mixin_key(), 1_702_204_169);
        let w_rid = signed.iter().find(|(k, _)| k == "w_rid").unwrap();
        assert_eq!(w_rid.1, "8f6f2b5b3d485fe1886cec6a0be8c5d4");
    }

    #[test]
    fn encodes_like_encode_uri_component() {
        assert_eq!(encode("a b/中"), "a%20b%2F%E4%B8%AD");
    }
}
