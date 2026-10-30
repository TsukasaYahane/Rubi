use serde::{Deserialize, Serialize};

/// A pointer to a video found in a link, a notice or a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoRef {
    Bvid(String),
    Aid(u64),
}

/// Finds the first video reference in `text`: a `BV1…` id anywhere, or an av number in
/// the forms `/video/av123`, `bilibili://video/123`, `aid=123` or a bare `av123`.
pub fn find_video_ref(text: &str) -> Option<VideoRef> {
    find_bvid(text).map(VideoRef::Bvid).or_else(|| find_aid(text).map(VideoRef::Aid))
}

fn find_bvid(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(pos) = text[from..].find("BV1") {
        let start = from + pos;
        let id: String = text[start..].chars().take(12).collect();
        let boundary_before = start == 0 || !bytes[start - 1].is_ascii_alphanumeric();
        let boundary_after = bytes.get(start + 12).is_none_or(|b| !b.is_ascii_alphanumeric());
        if id.len() == 12 && id.chars().all(|c| c.is_ascii_alphanumeric()) && boundary_before && boundary_after {
            return Some(id);
        }
        from = start + 3;
    }
    None
}

fn find_aid(text: &str) -> Option<u64> {
    let lower = text.to_ascii_lowercase();
    for marker in ["bilibili://video/", "/video/av", "aid=", "av"] {
        let mut from = 0;
        while let Some(pos) = lower[from..].find(marker) {
            let start = from + pos;
            let digits: String = lower[start + marker.len()..].chars().take_while(char::is_ascii_digit).collect();
            // A bare "av" must start a word, so words like "java123" do not match.
            let word_start = marker != "av" || start == 0 || !lower.as_bytes()[start - 1].is_ascii_alphanumeric();
            if word_start && let Ok(aid) = digits.parse() {
                return Some(aid);
            }
            from = start + marker.len();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_bvids_and_aids() {
        assert_eq!(
            find_video_ref("https://www.bilibili.com/video/BV17x411w7KC?p=2"),
            Some(VideoRef::Bvid("BV17x411w7KC".into()))
        );
        assert_eq!(find_video_ref("快看 BV17x411w7KC 这个"), Some(VideoRef::Bvid("BV17x411w7KC".into())));
        assert_eq!(find_video_ref("bilibili://video/170001?comment_root_id=1"), Some(VideoRef::Aid(170001)));
        assert_eq!(find_video_ref("https://www.bilibili.com/video/av170001"), Some(VideoRef::Aid(170001)));
        assert_eq!(find_video_ref("av170001"), Some(VideoRef::Aid(170001)));
    }

    #[test]
    fn ignores_non_videos() {
        assert_eq!(find_video_ref("https://t.bilibili.com/123456"), None);
        assert_eq!(find_video_ref("java123 is not a video"), None);
        assert_eq!(find_video_ref("BV1short"), None);
        assert_eq!(find_video_ref("XBV17x411w7KC"), None);
    }
}
