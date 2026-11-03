//! 直播 and 赛事.

use std::sync::Arc;

use rubi_core::{LiveBatch, LiveChannel, LiveFilter};
use rubi_model::LiveProtocol;

use crate::{
    FfiEsportsGame, FfiHeader, FfiLiveArea, FfiLiveBatch, FfiLiveFilter, FfiLivePlay, FfiLiveQuality, FfiLiveRoom,
    FfiLiveRoomPage, FfiLiveStream, FfiMatch, FfiMatchPage, Res, RubiCore,
};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// The 推荐 rooms; each call gives a fresh batch.
    pub async fn live_recommended(&self) -> Res<Vec<FfiLiveRoom>> {
        Ok(self.live.recommended().await?.into_iter().map(Into::into).collect())
    }

    pub async fn live_areas(&self) -> Res<Vec<FfiLiveArea>> {
        Ok(self.live.areas().await?.into_iter().map(|a| FfiLiveArea { id: a.id, name: a.name }).collect())
    }

    /// One page (from 1) of an area's rooms, busiest first.
    pub async fn live_area_rooms(&self, area: u32, page: u32) -> Res<FfiLiveRoomPage> {
        let (rooms, has_more) = self.live.area_rooms(area, page).await?;
        Ok(FfiLiveRoomPage { rooms: rooms.into_iter().map(Into::into).collect(), has_more })
    }

    pub async fn live_room(&self, room_id: u64) -> Res<FfiLiveRoom> {
        Ok(self.live.room(room_id).await?.into())
    }

    /// The room's streams at `qn` (the best offered when absent).
    pub async fn live_play(&self, room_id: u64, qn: Option<u32>) -> Res<FfiLivePlay> {
        let (p, headers) = self.live.play(room_id, qn).await?;
        Ok(FfiLivePlay {
            live: p.live,
            qn: p.qn,
            qualities: p.qualities.into_iter().map(|(qn, name)| FfiLiveQuality { qn, name }).collect(),
            streams: p
                .streams
                .into_iter()
                .map(|s| FfiLiveStream { url: s.url, hls: s.protocol == LiveProtocol::Hls })
                .collect(),
            headers: headers.into_iter().map(|(name, value)| FfiHeader { name, value }).collect(),
        })
    }

    pub async fn live_send(&self, room_id: u64, text: String) -> Res<()> {
        Ok(self.live.send(room_id, &text).await?)
    }

    /// Joins a room's message channel; read it with `next` until it returns null.
    pub async fn live_channel(&self, room_id: u64, filter: FfiLiveFilter) -> Res<Arc<FfiLiveChannel>> {
        Ok(Arc::new(FfiLiveChannel(self.live.channel(room_id, filter.into()).await?)))
    }

    /// One page (from 1) of the schedule around today; `game` 0 for every game.
    pub async fn esports_schedule(&self, game: u32, page: u32) -> Res<FfiMatchPage> {
        let (matches, has_more) = self.esports.schedule(game, page).await?;
        Ok(FfiMatchPage { matches: matches.into_iter().map(Into::into).collect(), has_more })
    }

    pub async fn esports_games(&self) -> Res<Vec<FfiEsportsGame>> {
        Ok(self.esports.games().await?.into_iter().map(|g| FfiEsportsGame { id: g.id, title: g.title }).collect())
    }

    /// Matches a search is about (a team or an event on the schedule), to show above results.
    pub async fn esports_related(&self, keyword: String) -> Res<Vec<FfiMatch>> {
        Ok(self.esports.related(&keyword).await?.into_iter().map(Into::into).collect())
    }

    /// 预约 a match or cancel it.
    pub async fn esports_subscribe(&self, match_id: u64, on: bool) -> Res<()> {
        Ok(self.esports.subscribe(match_id, on).await?)
    }
}

/// A room's open message channel.
#[derive(uniffi::Object)]
pub struct FfiLiveChannel(LiveChannel);

#[uniffi::export(async_runtime = "tokio")]
impl FfiLiveChannel {
    pub fn set_filter(&self, filter: FfiLiveFilter) {
        self.0.set_filter(filter.into());
    }

    /// The next messages to show; null once the channel has closed.
    pub async fn next(&self) -> Res<Option<FfiLiveBatch>> {
        Ok(self.0.next().await?.map(Into::into))
    }

    /// Leaves the channel (`close` is taken: it frees the object on the Kotlin side).
    pub async fn leave(&self) {
        self.0.close().await;
    }
}

impl From<FfiLiveFilter> for LiveFilter {
    fn from(f: FfiLiveFilter) -> Self {
        Self { gifts: f.gifts, popups: f.popups, effects: f.effects }
    }
}

impl From<LiveBatch> for FfiLiveBatch {
    fn from(b: LiveBatch) -> Self {
        Self {
            events: b.events.into_iter().map(Into::into).collect(),
            blocked_gifts: b.blocked_gifts,
            blocked_popups: b.blocked_popups,
            blocked_effects: b.blocked_effects,
        }
    }
}
