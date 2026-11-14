//! 听视频: the audio track alone, carrying on with the next video when one ends.

use std::cell::RefCell;

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::ui::media;
use crate::ui::shell::{shell, toast};
use crate::ui::{keep, post};
use crate::xaml::{IMediaPlayer, MediaPlayer};

thread_local! {
    /// The player making the sound, and the video it is playing.
    static HEARD: RefCell<Option<(MediaPlayer, String)>> = const { RefCell::new(None) };
}

/// Plays `bvid`'s audio, then whatever the listening queue holds next.
pub fn start(bvid: &str) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (bvid, dispatcher, core) = (bvid.to_owned(), shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let begun = core.services.listening.start_related(&bvid).await;
        let card = match begun {
            Ok(card) => card,
            Err(problem) => {
                post(&dispatcher, move || toast(&format!("取视频失败：{problem}")));
                return;
            }
        };
        let quality = core.services.preferences.settings().playback.preferred_quality;
        let found = core.services.videos.playback(&card.bvid, quality).await;
        let said = match found {
            Ok(playback) => match playback.url.audio.first() {
                Some(stream) => {
                    let (url, headers, title, id) =
                        (stream.url.clone(), playback.headers.clone(), card.title.clone(), card.bvid.clone());
                    let said = format!("正在听：{title}");
                    post(&dispatcher, move || play(&id, &title, &url, &headers));
                    said
                }
                None => "这个视频没有音轨".to_owned(),
            },
            Err(problem) => format!("取音轨失败：{problem}"),
        };
        post(&dispatcher, move || toast(&said));
    });
}

/// Stops the sound, if any.
pub fn stop() {
    HEARD.with_borrow_mut(|slot| {
        if let Some((player, _)) = slot.take() {
            let _ = player.Pause();
        }
    });
}

fn play(bvid: &str, title: &str, url: &str, headers: &[(String, String)]) {
    stop();
    let source = match media::stream_source(url, headers) {
        Ok(source) => source,
        Err(problem) => {
            toast(&format!("打不开音轨：{problem}"));
            return;
        }
    };
    let player = match media::player(&source) {
        Ok(player) => player,
        Err(problem) => {
            toast(&format!("打不开播放器：{problem}"));
            return;
        }
    };
    if let Err(problem) = player.Play() {
        toast(&format!("放不出来：{problem}"));
        return;
    }
    // The next video starts when this one ends.
    if let Ok(ended) = player.cast::<IMediaPlayer>() {
        if let Ok(revoker) =
            ended.MediaEnded(move |_: Ref<MediaPlayer>, _: Ref<IInspectable>| advance())
        {
            keep(revoker);
        }
    }
    HEARD.with_borrow_mut(|slot| *slot = Some((player, bvid.to_owned())));
    let _ = title;
}

/// Reads the next video of the listening queue and plays it.
fn advance() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let dispatcher = shell.dispatcher.clone();
    let core = backend.clone();
    backend.spawn(async move {
        let Some(next) = core.services.listening.next(false).ok().flatten() else {
            post(&dispatcher, || toast("听完了"));
            return;
        };
        let quality = core.services.preferences.settings().playback.preferred_quality;
        let found = core.services.videos.playback(&next.bvid, quality).await;
        if let Ok(playback) = found
            && let Some(stream) = playback.url.audio.first()
        {
            let (url, headers, title, id) =
                (stream.url.clone(), playback.headers.clone(), next.title.clone(), next.bvid.clone());
            post(&dispatcher, move || play(&id, &title, &url, &headers));
        }
    });
}

/// Whether something is being listened to.
pub fn playing() -> bool {
    HEARD.with_borrow(|slot| slot.is_some())
}

#[allow(dead_code)]
fn quiet(_: Result<()>) {}
