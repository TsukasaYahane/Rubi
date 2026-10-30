use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeKind {
    Reply,
    At,
    Like,
}

/// One entry of 回复 / @我 / 赞.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Notice {
    pub id: u64,
    /// Who acted; for likes, e.g. "阿蓝 等 12 人".
    pub from: String,
    pub from_mid: u64,
    /// What they wrote (reply / @ text), or the liked content.
    pub text: String,
    /// The video or comment it happened on.
    pub context: String,
    pub uri: String,
    /// Unix seconds.
    pub at: i64,
    /// The video this notice is about, when it is about one.
    #[serde(default)]
    pub video: Option<crate::VideoRef>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnreadCounts {
    pub reply: u32,
    pub at: u32,
    pub like: u32,
    pub chat: u32,
}

impl UnreadCounts {
    pub fn total(&self) -> u32 {
        self.reply + self.at + self.like + self.chat
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatSession {
    pub talker_mid: u64,
    pub name: String,
    pub unread: u32,
    pub last_text: String,
    pub last_from_me: bool,
    /// Unix seconds.
    pub last_at: i64,
    pub ack_seqno: u64,
    /// The session exactly as the API returned it; the notice rules match against this.
    #[serde(default)]
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub seqno: u64,
    pub sender_mid: u64,
    /// Plain text; non-text messages are summarised, e.g. "[图片]".
    pub text: String,
    /// Unix seconds.
    pub at: i64,
    /// A shared video card, or a video id mentioned in the text.
    #[serde(default)]
    pub video: Option<SharedVideo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedVideo {
    pub video: crate::VideoRef,
    /// Empty when the video was only mentioned by id in text.
    pub title: String,
    pub cover: String,
    pub author: String,
}
