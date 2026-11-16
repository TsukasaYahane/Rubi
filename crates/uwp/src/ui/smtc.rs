//! 系统媒体控制: the panel Windows shows for the volume keys and the media flyout, and the reason
//! 后台播放 keeps going — the player hands its title and state here.

use std::cell::RefCell;

use crate::xaml::{MediaPlaybackStatus, MediaPlaybackType, SystemMediaTransportControls};

thread_local! {
    /// What the panel is meant to show: a player clears it whenever a source opens.
    static NOW: RefCell<Option<(String, String)>> = const { RefCell::new(None) };
}

/// Says what is playing, once the page knows.
pub fn now(title: &str, artist: &str) {
    NOW.with_borrow_mut(|slot| *slot = Some((title.to_owned(), artist.to_owned())));
    push(title, artist);
}

/// Says it again: `MediaPlayer` resets the panel each time it opens a source, which is after the
/// page has set the title.
pub fn refresh() {
    let Some((title, artist)) = NOW.with_borrow(Clone::clone) else { return };
    push(&title, &artist);
}

fn push(title: &str, artist: &str) {
    let Ok(smtc) = SystemMediaTransportControls::GetForCurrentView() else { return };
    let _ = smtc.SetIsEnabled(true);
    let Ok(updater) = smtc.DisplayUpdater() else { return };
    let _ = updater.SetType(MediaPlaybackType::Music);
    if let Ok(props) = updater.MusicProperties() {
        let _ = props.SetTitle(title);
        let _ = props.SetArtist(artist);
    }
    let _ = updater.Update();
}

/// Playing or not, for the same panel.
pub fn state(playing: bool) {
    let Ok(smtc) = SystemMediaTransportControls::GetForCurrentView() else { return };
    let status = if playing { MediaPlaybackStatus::Playing } else { MediaPlaybackStatus::Paused };
    let _ = smtc.SetPlaybackStatus(status);
}
