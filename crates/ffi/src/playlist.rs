//! The play list, played as video or heard as audio (听视频).

use rubi_core::{DEFAULT_PLAYLIST, TEMP_PLAYLIST};
use rubi_model::VideoCard;

use crate::{FfiPlaylistMode, FfiVideoCard, Res, RubiCore};

fn card(c: Option<VideoCard>) -> Option<FfiVideoCard> {
    c.map(|c| FfiVideoCard::of(&c, None))
}

/// The list being played, for the 播放队列 on the video page.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPlayingList {
    pub id: i64,
    pub name: String,
    /// A 收藏夹 or 稍后再看 played through: gone when its video page closes.
    pub temporary: bool,
    pub items: Vec<FfiVideoCard>,
}

/// A play list as the list chips show it.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPlaylist {
    pub id: i64,
    pub name: String,
    pub count: u32,
    /// 默认: always there, cannot be renamed or deleted.
    pub fixed: bool,
}

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// Every play list, 默认 first.
    pub fn playlists(&self) -> Res<Vec<FfiPlaylist>> {
        Ok(self
            .playlists
            .lists()?
            .into_iter()
            .map(|l| FfiPlaylist { id: l.id, name: l.name, count: l.count as u32, fixed: l.id == DEFAULT_PLAYLIST })
            .collect())
    }

    /// A new empty list; returns its id. Errors name what is wrong with the name.
    pub fn playlist_create(&self, name: String) -> Res<i64> {
        Ok(self.playlists.create(&name)?)
    }

    pub fn playlist_rename(&self, list: i64, name: String) -> Res<()> {
        Ok(self.playlists.rename(list, &name)?)
    }

    pub fn playlist_delete(&self, list: i64) -> Res<()> {
        Ok(self.playlists.delete(list)?)
    }

    pub fn playlist_items(&self, list: i64) -> Res<Vec<FfiVideoCard>> {
        Ok(self.playlists.items(list)?.iter().map(|c| FfiVideoCard::of(c, None)).collect())
    }

    /// The ids of the lists holding `bvid`.
    pub fn playlists_with(&self, bvid: String) -> Res<Vec<i64>> {
        Ok(self.playlists.lists_with(&bvid)?)
    }

    /// Adds a video (looked up if it was not shown); false when it was already listed.
    pub async fn playlist_add(&self, list: i64, bvid: String) -> Res<bool> {
        Ok(self.playlists.add(list, &bvid).await?)
    }

    pub fn playlist_remove(&self, list: i64, bvid: String) -> Res<()> {
        Ok(self.playlists.remove(list, &bvid)?)
    }

    pub fn playlist_move(&self, list: i64, from: u32, to: u32) -> Res<()> {
        Ok(self.playlists.move_item(list, from as usize, to as usize)?)
    }

    pub fn playlist_clear(&self, list: i64) -> Res<()> {
        Ok(self.playlists.clear(list)?)
    }

    pub fn playlist_mode(&self) -> Res<FfiPlaylistMode> {
        Ok(self.playlists.mode()?.into())
    }

    pub fn set_playlist_mode(&self, mode: FfiPlaylistMode) -> Res<()> {
        Ok(self.playlists.set_mode(mode.into())?)
    }

    /// Makes `list` the one whose next item follows a video that ends (a video picked from it).
    pub fn playlist_play(&self, list: i64) {
        self.playlists.play(list);
    }

    /// Where 播放全部 / 听全部 starts.
    pub fn playlist_first(&self, list: i64) -> Res<Option<FfiVideoCard>> {
        Ok(card(self.playlists.first(list)?))
    }

    /// What plays after `current` ended on the video page; `None` stops.
    pub fn playlist_next(&self, current: String) -> Res<Option<FfiVideoCard>> {
        Ok(card(self.playlists.next_after(&current)?))
    }

    /// The list being played, for the 播放队列.
    pub fn playing_list(&self) -> Res<FfiPlayingList> {
        let (id, name, items) = self.playlists.playing()?;
        Ok(FfiPlayingList {
            id,
            name,
            temporary: id == TEMP_PLAYLIST,
            items: items.iter().map(|c| FfiVideoCard::of(c, None)).collect(),
        })
    }

    /// Plays videos already shown (a 收藏夹, 稍后再看) as a list that is not kept; returns
    /// the first to play.
    pub async fn play_temp(&self, name: String, bvids: Vec<String>) -> Res<Option<FfiVideoCard>> {
        Ok(card(self.playlists.play_temp(&name, &bvids).await?))
    }

    /// Adds later pages to the list being played through.
    pub async fn extend_temp(&self, bvids: Vec<String>) {
        self.playlists.extend_temp(&bvids).await;
    }

    /// The video page playing the temporary list closed.
    pub fn drop_temp(&self) {
        self.playlists.drop_temp();
    }

    // ----- 听视频 -----

    /// Starts hearing one video, then its 相关推荐 as they are added.
    pub async fn listen_related(&self, bvid: String) -> Res<FfiVideoCard> {
        Ok(FfiVideoCard::of(&self.playlists.listen_related(&bvid).await?, None))
    }

    /// When following 相关推荐: how many are left after the one heard; `None` otherwise.
    pub fn related_left(&self) -> Option<u32> {
        self.playlists.related_left().map(|n| n as u32)
    }

    /// Adds 相关推荐 to come after what is heard.
    pub async fn listen_append(&self, bvids: Vec<String>) {
        self.playlists.listen_append(&bvids).await;
    }

    /// What is being heard; `None` when not listening.
    pub fn listening(&self) -> Option<FfiVideoCard> {
        card(self.playlists.listening())
    }

    /// Starts hearing `list` from `bvid`.
    pub async fn listen_from(&self, list: i64, bvid: String) -> Res<FfiVideoCard> {
        Ok(FfiVideoCard::of(&self.playlists.listen_from(list, &bvid).await?, None))
    }

    /// Switches to `bvid` picked from the list.
    pub async fn listen_jump(&self, bvid: String) -> Res<FfiVideoCard> {
        Ok(FfiVideoCard::of(&self.playlists.listen_jump(&bvid).await?, None))
    }

    /// 下一首 (`by_user`) or after an item ended by itself; `None` means listening stopped.
    pub fn listen_next(&self, by_user: bool) -> Res<Option<FfiVideoCard>> {
        Ok(card(self.playlists.listen_next(by_user)?))
    }

    /// 上一首 (in shuffle, back through what was heard).
    pub fn listen_previous(&self) -> Res<Option<FfiVideoCard>> {
        Ok(card(self.playlists.listen_previous()?))
    }

    /// The list being heard, for the 听视频 page.
    pub fn listening_items(&self) -> Res<Vec<FfiVideoCard>> {
        Ok(self.playlists.listening_items()?.iter().map(|c| FfiVideoCard::of(c, None)).collect())
    }

    pub fn listen_stop(&self) {
        self.playlists.listen_stop();
    }
}
