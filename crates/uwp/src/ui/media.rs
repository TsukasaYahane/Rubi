//! Building and letting go of a media source: what the video page and the live room share.

use windows_core::{Error, HRESULT, Interface, Result};

use crate::xaml::{
    AdaptiveMediaSource, AdaptiveMediaSourceCreationResult, AdaptiveMediaSourceCreationStatus, HttpClient, IClosable,
    IMediaPlaybackSource, IMediaPlayer, IMediaPlayerElement, IMediaPlayerSource2, MediaPlayer, MediaSource, Uri,
};

/// An HTTP client sending `headers` with each request.
pub fn client(headers: &[(String, String)]) -> Result<HttpClient> {
    let client = HttpClient::new()?;
    let sent = client.DefaultRequestHeaders()?;
    for (name, value) in headers {
        sent.TryAppendWithoutValidation(name, value)?;
    }
    Ok(client)
}

/// The source of an adaptive stream the system player agreed to.
pub fn source_of(created: &AdaptiveMediaSourceCreationResult) -> Result<MediaSource> {
    let status = created.Status()?;
    if status != AdaptiveMediaSourceCreationStatus::Success {
        return Err(Error::new(HRESULT(0x8000_4005_u32 as i32), format!("系统播放器拒绝了清单：{status:?}")));
    }
    MediaSource::CreateFromAdaptiveMediaSource(&created.MediaSource()?)
}

/// The source of the stream at `url`. Blocks on WinRT calls: never on the UI thread.
pub fn stream_source(url: &str, headers: &[(String, String)]) -> Result<MediaSource> {
    let created =
        AdaptiveMediaSource::CreateFromUriWithDownloaderAsync(&Uri::CreateUri(url)?, &client(headers)?)?.join()?;
    source_of(&created)
}

/// Plays `source` in the element `video`.
pub fn play(video: &IMediaPlayerElement, source: &MediaSource) -> Result<MediaPlayer> {
    let player = MediaPlayer::new()?;
    player.cast::<IMediaPlayerSource2>()?.SetSource(&source.cast::<IMediaPlaybackSource>()?)?;
    video.SetMediaPlayer(&player)?;
    Ok(player)
}

/// Plays `source` in place of what `player` plays.
pub fn replace(player: &MediaPlayer, source: &MediaSource) -> Result<()> {
    player.cast::<IMediaPlayerSource2>()?.SetSource(&source.cast::<IMediaPlaybackSource>()?)?;
    player.cast::<IMediaPlayer>()?.Play()
}

/// Lets go of a player and what it played. The element must have let go first: left holding a
/// closed player, it brings the app down.
pub fn close(player: Option<MediaPlayer>, source: Option<MediaSource>) {
    if let Some(player) = player {
        let _ = player.cast::<IMediaPlayer>().and_then(|p| p.Pause());
        let _ = player.cast::<IMediaPlayerSource2>().and_then(|p| p.SetSource(None::<&IMediaPlaybackSource>));
        let _ = player.cast::<IClosable>().and_then(|p| p.Close());
    }
    close_source(source);
}

/// Lets go of a source no player plays any more.
pub fn close_source(source: Option<MediaSource>) {
    if let Some(source) = source {
        let _ = source.cast::<IClosable>().and_then(|s| s.Close());
    }
}
