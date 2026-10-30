use serde::{Deserialize, Serialize};

/// Why the user marked something as not interesting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DislikeReason {
    Up,
    Zone,
    Tag(String),
    Title,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Signal {
    Impression,
    Click,
    QuickExit,
    Completion { ratio: f32 },
    Rewatch,
    Like,
    Coin,
    Favorite,
    Follow,
    Dislike { reason: DislikeReason },
    Search { term: String },
}

/// The features of the item a signal refers to, captured at the time of the signal.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ItemRef {
    pub bvid: String,
    pub up_mid: u64,
    pub zone_tid: Option<u32>,
    pub tags: Vec<String>,
    pub title: String,
}

impl ItemRef {
    pub fn of(card: &crate::VideoCard) -> Self {
        Self {
            bvid: card.bvid.clone(),
            up_mid: card.owner.mid,
            zone_tid: card.zone.as_ref().map(|z| z.tid),
            tags: card.tags.clone(),
            title: card.title.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedbackEvent {
    pub signal: Signal,
    /// `None` for item-less signals such as a search.
    pub item: Option<ItemRef>,
    /// Unix seconds.
    pub at: i64,
}
