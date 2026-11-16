//! 正在播放: the player a page is showing and the source it plays. Owned here and nowhere else, so
//! that the page is a view of it rather than its keeper — and so that a page that goes takes the
//! video with it instead of leaving a closed player in someone's hands.

use std::cell::RefCell;

use crate::xaml::{MediaPlayer, MediaSource};

use super::media;

thread_local! {
    static HELD: RefCell<Option<Held>> = const { RefCell::new(None) };
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

/// Lets both go, closing them: a page that goes takes its video with it.
pub(crate) fn drop_it() {
    if let Some(held) = take() {
        media::close(Some(held.player), Some(held.source));
    }
}
