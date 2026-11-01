//! The play list, played as video or heard as audio (听视频).

use rubi_model::VideoCard;

use crate::{FfiPlaylistMode, FfiVideoCard, Res, RubiCore};

fn card(c: Option<VideoCard>) -> Option<FfiVideoCard> {
    c.map(|c| FfiVideoCard::of(&c, None))
}

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    pub fn playlist_items(&self) -> Res<Vec<FfiVideoCard>> {
        Ok(self.playlists.items()?.iter().map(|c| FfiVideoCard::of(c, None)).collect())
    }

    pub fn playlist_contains(&self, bvid: String) -> Res<bool> {
        Ok(self.playlists.contains(&bvid)?)
    }

    /// Adds a video (looked up if it was not shown); false when it was already listed.
    pub async fn playlist_add(&self, bvid: String) -> Res<bool> {
        Ok(self.playlists.add(&bvid).await?)
    }

    pub fn playlist_remove(&self, bvid: String) -> Res<()> {
        Ok(self.playlists.remove(&bvid)?)
    }

    pub fn playlist_move(&self, from: u32, to: u32) -> Res<()> {
        Ok(self.playlists.move_item(from as usize, to as usize)?)
    }

    pub fn playlist_clear(&self) -> Res<()> {
        Ok(self.playlists.clear()?)
    }

    pub fn playlist_mode(&self) -> Res<FfiPlaylistMode> {
        Ok(self.playlists.mode()?.into())
    }

    pub fn set_playlist_mode(&self, mode: FfiPlaylistMode) -> Res<()> {
        Ok(self.playlists.set_mode(mode.into())?)
    }

    /// Where 播放全部 / 听全部 starts.
    pub fn playlist_first(&self) -> Res<Option<FfiVideoCard>> {
        Ok(card(self.playlists.first()?))
    }

    /// What plays after `current` ended on the video page; `None` stops.
    pub fn playlist_next(&self, current: String) -> Res<Option<FfiVideoCard>> {
        Ok(card(self.playlists.next_after(&current)?))
    }

    // ----- 听视频 -----

    /// What is being heard; `None` when not listening.
    pub fn listening(&self) -> Option<FfiVideoCard> {
        card(self.playlists.listening())
    }

    /// Starts hearing the list from `bvid`.
    pub async fn listen_from(&self, bvid: String) -> Res<FfiVideoCard> {
        Ok(FfiVideoCard::of(&self.playlists.listen_from(&bvid).await?, None))
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

    pub fn listen_stop(&self) {
        self.playlists.listen_stop();
    }
}
