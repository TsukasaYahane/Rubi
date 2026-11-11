//! 直播: room lists, a room's streams, and its message channel with gift banners, popups and
//! effects held back by the room's filter switches.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rubi_api::{BEST_LIVE_QN, ChannelJoin, ChannelOpener, ChannelStream, LiveApi, event_of};
use rubi_model::{LiveArea, LiveCategory, LiveEvent, LivePlay, LiveRoom};

use crate::error::{CoreError, Result};
use crate::ports::RuleSource;

/// Rooms on air looked up per page of the followed list.
const LOOKED_UP: usize = 12;

/// What a room shows of each kind of extra; `true` holds it back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveFilter {
    pub gifts: bool,
    pub popups: bool,
    pub effects: bool,
}

impl Default for LiveFilter {
    fn default() -> Self {
        Self { gifts: true, popups: true, effects: true }
    }
}

impl LiveFilter {
    fn holds(&self, c: LiveCategory) -> bool {
        match c {
            LiveCategory::Gift => self.gifts,
            LiveCategory::Popup => self.popups,
            LiveCategory::Effect => self.effects,
        }
    }
}

/// What the channel said since the last read, and how much has been held back so far.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LiveBatch {
    pub events: Vec<LiveEvent>,
    pub blocked_gifts: u32,
    pub blocked_popups: u32,
    pub blocked_effects: u32,
}

pub struct Live {
    api: Arc<dyn LiveApi>,
    channels: Arc<dyn ChannelOpener>,
    rules: Arc<dyn RuleSource>,
    areas: Mutex<Option<Vec<LiveArea>>>,
}

impl Live {
    pub(crate) fn new(api: Arc<dyn LiveApi>, channels: Arc<dyn ChannelOpener>, rules: Arc<dyn RuleSource>) -> Self {
        Self { api, channels, rules, areas: Mutex::new(None) }
    }

    pub async fn recommended(&self) -> Result<Vec<LiveRoom>> {
        Ok(self.api.recommended().await?)
    }

    /// The areas, read once per process (they change rarely).
    pub async fn areas(&self) -> Result<Vec<LiveArea>> {
        if let Some(a) = self.areas.lock().unwrap_or_else(|e| e.into_inner()).clone() {
            return Ok(a);
        }
        let a = self.api.areas().await?;
        *self.areas.lock().unwrap_or_else(|e| e.into_inner()) = Some(a.clone());
        Ok(a)
    }

    /// One page of a parent area's rooms, busiest first; also whether more follow.
    pub async fn area_rooms(&self, parent: u32, page: u32) -> Result<(Vec<LiveRoom>, bool)> {
        Ok(self.api.area_rooms(parent, 0, page.max(1)).await?)
    }

    /// One page of followed UPs' rooms, also whether more follow. Login required. The list has no
    /// covers or viewers, so the rooms on air are looked up one by one; one that fails keeps what
    /// the list said.
    pub async fn followed(&self, page: u32) -> Result<(Vec<LiveRoom>, bool)> {
        let (mut rooms, more) = self.api.followed_rooms(page.max(1)).await?;
        for room in rooms.iter_mut().filter(|r| r.live).take(LOOKED_UP) {
            if let Ok(found) = self.api.room(room.room_id).await {
                room.online = found.online;
                room.area = found.area;
                if !found.cover.is_empty() {
                    room.cover = found.cover;
                }
            }
        }
        Ok((rooms, more))
    }

    pub async fn room(&self, room_id: u64) -> Result<LiveRoom> {
        Ok(self.api.room(room_id).await?)
    }

    /// The room's streams at `qn` (the best on offer when `None`), plus the headers to fetch
    /// them with.
    pub async fn play(&self, room_id: u64, qn: Option<u32>) -> Result<(LivePlay, Vec<(String, String)>)> {
        let play = self.api.play(room_id, qn.unwrap_or(BEST_LIVE_QN)).await?;
        Ok((play, self.api.stream_headers()))
    }

    pub async fn send(&self, room_id: u64, text: &str) -> Result<()> {
        let text = text.trim();
        if text.is_empty() {
            return Err(CoreError::Invalid("弹幕不能为空".into()));
        }
        Ok(self.api.send(room_id, text).await?)
    }

    /// Joins the room's message channel.
    pub async fn channel(&self, room_id: u64, filter: LiveFilter) -> Result<LiveChannel> {
        let server = self.api.danmu_server(room_id).await?;
        let (uid, buvid) = self.api.identity();
        let join = ChannelJoin { room_id, uid, buvid: &buvid, token: &server.token };
        let socket = self.channels.open(&server.urls, &join).await?;
        let keywords = self.rules.danmaku_options().keywords.clone();
        Ok(LiveChannel {
            socket: tokio::sync::Mutex::new(Some(socket)),
            sorter: Mutex::new(Sorter { filter, keywords, blocked: HashMap::new() }),
            closed: tokio::sync::watch::Sender::new(false),
        })
    }
}

/// An open room channel. Read it in a loop with [`next`](Self::next); switches can change while
/// it runs, and [`close`](Self::close) ends a read that is waiting.
pub struct LiveChannel {
    socket: tokio::sync::Mutex<Option<Box<dyn ChannelStream>>>,
    sorter: Mutex<Sorter>,
    /// Set by `close`: a waiting `next` gives up the socket at once instead of holding it until
    /// the room says something.
    closed: tokio::sync::watch::Sender<bool>,
}

impl LiveChannel {
    pub fn set_filter(&self, filter: LiveFilter) {
        self.sorter.lock().unwrap_or_else(|e| e.into_inner()).filter = filter;
    }

    /// The next messages worth showing; `None` once the channel has closed.
    pub async fn next(&self) -> Result<Option<LiveBatch>> {
        let mut closed = self.closed.subscribe();
        let mut socket = tokio::select! {
            s = self.socket.lock() => s,
            _ = closed.wait_for(|c| *c) => return Ok(None),
        };
        loop {
            let Some(s) = socket.as_mut() else { return Ok(None) };
            let raw = tokio::select! {
                r = s.next() => r?,
                _ = closed.wait_for(|c| *c) => return Ok(None),
            };
            let Some(raw) = raw else {
                *socket = None;
                return Ok(None);
            };
            let batch = self.sorter.lock().unwrap_or_else(|e| e.into_inner()).sort(raw.iter().filter_map(event_of));
            if !batch.events.is_empty() {
                return Ok(Some(batch));
            }
        }
    }

    /// Leaves the channel; a read under way returns `None`.
    pub async fn close(&self) {
        self.closed.send_replace(true);
        if let Some(s) = self.socket.lock().await.take() {
            s.close().await;
        }
    }
}

struct Sorter {
    filter: LiveFilter,
    keywords: Vec<regex::Regex>,
    blocked: HashMap<LiveCategory, u32>,
}

impl Sorter {
    /// Keeps what the switches let through and the rule table's danmaku keywords allow.
    fn sort(&mut self, events: impl Iterator<Item = LiveEvent>) -> LiveBatch {
        let mut kept = Vec::new();
        for ev in events {
            if let Some(c) = ev.category()
                && self.filter.holds(c)
            {
                *self.blocked.entry(c).or_default() += 1;
                continue;
            }
            if let LiveEvent::Danmaku { text, .. } = &ev
                && self.keywords.iter().any(|k| k.is_match(text))
            {
                continue;
            }
            kept.push(ev);
        }
        let count = |c| self.blocked.get(&c).copied().unwrap_or(0);
        LiveBatch {
            events: kept,
            blocked_gifts: count(LiveCategory::Gift),
            blocked_popups: count(LiveCategory::Popup),
            blocked_effects: count(LiveCategory::Effect),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorter(filter: LiveFilter) -> Sorter {
        Sorter { filter, keywords: vec![regex::Regex::new("广告").unwrap()], blocked: HashMap::new() }
    }

    fn dm(text: &str) -> LiveEvent {
        LiveEvent::Danmaku { uname: "u".into(), text: text.into(), color: 0, emotes: Default::default(), sticker: None }
    }

    #[test]
    fn holds_back_what_the_switches_cover_and_counts_it() {
        let mut s = sorter(LiveFilter::default());
        let b = s.sort(
            vec![
                dm("666"),
                LiveEvent::Gift { text: "g".into() },
                LiveEvent::Gift { text: "g".into() },
                LiveEvent::Popup { text: "p".into() },
                dm("加群看广告"),
                LiveEvent::Online { count: 3 },
            ]
            .into_iter(),
        );
        assert_eq!(b.events, vec![dm("666"), LiveEvent::Online { count: 3 }]);
        assert_eq!((b.blocked_gifts, b.blocked_popups, b.blocked_effects), (2, 1, 0));
        // Counts add up over the session; a switch turned off lets its kind through.
        s.filter.gifts = false;
        let b = s.sort(vec![LiveEvent::Gift { text: "g".into() }, LiveEvent::Popup { text: "p".into() }].into_iter());
        assert_eq!(b.events, vec![LiveEvent::Gift { text: "g".into() }]);
        assert_eq!((b.blocked_gifts, b.blocked_popups), (2, 2));
    }
}
