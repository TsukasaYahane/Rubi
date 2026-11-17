//! 正在播放: the player a page is showing and the source it plays. Owned here and nowhere else, so
//! that the page is a view of it rather than its keeper — and so that a page that goes takes the
//! video with it instead of leaving a closed player in someone's hands.

use std::cell::RefCell;

use windows_core::Interface;

use crate::xaml::{
    IMediaItemDisplayProperties, IMediaPlaybackItem2, MediaPlaybackItem, MediaPlayer, MediaSource,
    RandomAccessStreamReference, Uri,
};

use super::media;

thread_local! {
    static HELD: RefCell<Option<Held>> = const { RefCell::new(None) };
    /// The item the player was given: the cover is put on it once the page knows it, which is
    /// after playback has already started.
    static ITEM: RefCell<Option<MediaPlaybackItem>> = const { RefCell::new(None) };
}

/// The player on show, and the source it plays.
pub(crate) struct Held {
    pub(crate) player: MediaPlayer,
    pub(crate) source: MediaSource,
}

/// Takes `player` and `source` in, and gives back what was there: the caller closes that one.
pub(crate) fn put(player: MediaPlayer, source: MediaSource) -> Option<Held> {
    HELD.with_borrow_mut(|slot| slot.replace(Held { player, source }))
}

/// The player on show, if a page gave one.
pub(crate) fn player() -> Option<MediaPlayer> {
    HELD.with_borrow(|slot| slot.as_ref().map(|held| held.player.clone()))
}

/// Whether a player is on show.
pub(crate) fn holding() -> bool {
    HELD.with_borrow(|slot| slot.is_some())
}

/// Both, for a caller that means to keep or close them itself.
pub(crate) fn take() -> Option<Held> {
    HELD.with_borrow_mut(Option::take)
}

/// Keeps the item the player was handed, so its display properties can be filled in later.
pub(crate) fn set_item(item: MediaPlaybackItem) {
    ITEM.with_borrow_mut(|slot| *slot = Some(item));
}

/// Puts the cover on the item, once the page has the card it belongs to.
pub(crate) fn set_cover(cover: &str) {
    if cover.is_empty() {
        return;
    }
    ITEM.with_borrow(|slot| {
        let Some(item) = slot.as_ref() else { return };
        let Ok(display) = item.cast::<IMediaPlaybackItem2>() else { return };
        let Ok(props) = display.GetDisplayProperties() else { return };
        let Ok(meta) = props.cast::<IMediaItemDisplayProperties>() else { return };
        let Ok(uri) = Uri::CreateUri(cover) else { return };
        let Ok(stream) = RandomAccessStreamReference::CreateFromUri(&uri) else { return };
        let _ = meta.SetThumbnail(&stream);
        let _ = display.ApplyDisplayProperties(&props);
    });
}

/// Lets both go, closing them: a page that goes takes its video with it.
pub(crate) fn drop_it() {
    ITEM.with_borrow_mut(|slot| *slot = None);
    if let Some(held) = take() {
        media::close(Some(held.player), Some(held.source));
    }
}
