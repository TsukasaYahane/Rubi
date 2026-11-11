use serde::{Deserialize, Serialize};

use crate::VideoCard;

/// What kind of post a dynamic is (动态).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DynamicKind {
    Video,
    /// Text only.
    #[default]
    Word,
    /// Text with pictures.
    Draw,
    /// A 专栏 article.
    Article,
    /// A rich post (图文, the newer form of draws and articles).
    Opus,
    /// A repost of another dynamic.
    Forward,
    /// Anything else (decorations, live, music…): title / text / cover / link when present.
    Other,
}

/// One post of an UP's feed, flattened to what a card shows.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Dynamic {
    pub id: String,
    pub kind: DynamicKind,
    pub author_mid: u64,
    pub author_name: String,
    pub author_face: String,
    /// Unix seconds.
    pub published_at: i64,
    pub text: String,
    pub title: String,
    pub pictures: Vec<String>,
    pub video: Option<VideoCard>,
    /// Where the full article / post opens (https).
    pub link: String,
    /// The reposted dynamic (one level).
    pub orig: Option<Box<Dynamic>>,
    /// Its comment area (`type`, `oid`); kind 0 when it has none.
    #[serde(default)]
    pub comment_kind: u32,
    #[serde(default)]
    pub comment_oid: u64,
    #[serde(default)]
    pub comment_count: u64,
    /// Images of the emotes its text uses, by token (e.g. `[doge]`).
    #[serde(default)]
    pub emotes: std::collections::BTreeMap<String, String>,
}

/// Someone the user follows, from the full following list.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Following {
    pub mid: u64,
    pub name: String,
    pub face: String,
    /// When the user followed them (Unix seconds).
    pub followed_at: i64,
}

/// A followed UP in the 全部关注 list.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FollowingUp {
    pub mid: u64,
    pub name: String,
    pub face: String,
    /// Their latest post seen while looking back (Unix seconds); `None` when none was.
    pub last_post_at: Option<i64>,
    /// Posted since the feed was last seen.
    pub has_update: bool,
}

/// A followed UP in the 关注 bar.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FollowedUp {
    pub mid: u64,
    pub name: String,
    pub face: String,
    /// Has posted since the feed was last seen.
    pub has_update: bool,
}
