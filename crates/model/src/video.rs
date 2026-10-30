use serde::{Deserialize, Serialize};

/// Where a candidate video came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    BiliRecommend,
    FollowedUps,
    Popular,
    Ranking,
    PreferredZone,
    Related,
    Subscription,
}

impl SourceKind {
    pub const ALL: [SourceKind; 7] = [
        SourceKind::BiliRecommend,
        SourceKind::FollowedUps,
        SourceKind::Popular,
        SourceKind::Ranking,
        SourceKind::PreferredZone,
        SourceKind::Related,
        SourceKind::Subscription,
    ];
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub view: u64,
    pub like: u64,
    pub coin: u64,
    pub favorite: u64,
    pub danmaku: u64,
    pub reply: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Owner {
    pub mid: u64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    pub tid: u32,
    pub name: String,
}

/// A card in any list or feed. Non-video cards (ads, live rooms…) keep an empty `bvid`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct VideoCard {
    pub bvid: String,
    pub aid: u64,
    pub cid: Option<u64>,
    pub title: String,
    pub desc: String,
    pub cover: String,
    pub owner: Owner,
    pub zone: Option<Zone>,
    pub tags: Vec<String>,
    pub duration_s: u32,
    /// Unix seconds.
    pub pubdate: i64,
    pub stats: Stats,
    pub sources: Vec<SourceKind>,
    /// The card exactly as the API returned it; the filter rule table matches against this.
    #[serde(default)]
    pub raw: serde_json::Value,
}

impl VideoCard {
    pub fn is_video(&self) -> bool {
        !self.bvid.is_empty()
    }

    /// Adds `kind` to the card's sources if missing.
    pub fn add_source(&mut self, kind: SourceKind) {
        if !self.sources.contains(&kind) {
            self.sources.push(kind);
        }
    }
}
