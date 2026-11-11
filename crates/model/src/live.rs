//! 直播: rooms, their areas, streams, and what a room's message channel says.

/// A live room as lists and the room header show it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LiveRoom {
    /// The real (long) room id.
    pub room_id: u64,
    pub title: String,
    pub cover: String,
    pub uid: u64,
    pub uname: String,
    pub face: String,
    /// Viewers (人气) as the list reports them.
    pub online: u64,
    pub area: String,
    pub live: bool,
}

/// A 直播 area: a parent (网游, 手游…) with its sub-areas.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LiveArea {
    pub id: u32,
    pub name: String,
    pub children: Vec<LiveSubArea>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LiveSubArea {
    pub id: u32,
    pub parent_id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveProtocol {
    Hls,
    Flv,
}

/// One stream URL, already joined from host, path and query.
#[derive(Debug, Clone, PartialEq)]
pub struct LiveStream {
    pub url: String,
    pub protocol: LiveProtocol,
}

/// What a room can be watched at: the streams of the chosen quality, most suitable first.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LivePlay {
    pub live: bool,
    /// qn of the streams returned.
    pub qn: u32,
    /// Every quality on offer as (qn, name), best first.
    pub qualities: Vec<(u32, String)>,
    pub streams: Vec<LiveStream>,
}

/// One message from a room's channel, sorted into what the room page shows.
#[derive(Debug, Clone, PartialEq)]
pub enum LiveEvent {
    Danmaku {
        uname: String,
        text: String,
        color: u32,
        /// Images of the room emotes the text uses, by token (e.g. `[dog]`).
        emotes: std::collections::BTreeMap<String, String>,
        /// A big emote sent on its own: its image stands for the text.
        sticker: Option<String>,
    },
    /// 礼物横幅: gifts, combos, 舰长.
    Gift {
        text: String,
    },
    /// 抽奖 / 红包 / 购物 popups.
    Popup {
        text: String,
    },
    /// 高能弹幕 (SC, big emotes) and entry effects.
    Effect {
        text: String,
    },
    Online {
        count: u64,
    },
    /// The broadcast started or stopped.
    Status {
        live: bool,
    },
}

impl LiveEvent {
    /// Which of the room's filter switches covers this message, if any.
    pub fn category(&self) -> Option<LiveCategory> {
        match self {
            Self::Gift { .. } => Some(LiveCategory::Gift),
            Self::Popup { .. } => Some(LiveCategory::Popup),
            Self::Effect { .. } | Self::Danmaku { sticker: Some(_), .. } => Some(LiveCategory::Effect),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LiveCategory {
    Gift,
    Popup,
    Effect,
}
