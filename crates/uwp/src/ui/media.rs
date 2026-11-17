//! Building and letting go of a media source: what the video page and the live room share.

use windows_core::{Error, HRESULT, IInspectable, Interface, Ref, Result};

use crate::xaml::{
    AdaptiveMediaSource, AdaptiveMediaSourceCreationResult, AdaptiveMediaSourceCreationStatus, DataWriter, HttpClient,
    IClosable, IDataWriter, IInputStream, IMediaPlaybackSession, IMediaPlaybackSource, IMediaPlayer, IMediaPlayer3,
    IMediaPlayerSource2, IOutputStream, IRandomAccessStream, InMemoryRandomAccessStream, MediaPlayer, MediaSource, Uri,
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

/// The content type the system player expects for a DASH manifest.
const DASH_TYPE: &str = "application/dash+xml";

/// The source of a manifest handed in as a string, with `headers` on the requests it leads to.
/// Blocks on WinRT calls: never on the UI thread.
pub fn manifest_source(manifest: &str, headers: &[(String, String)]) -> Result<MediaSource> {
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream.cast::<IOutputStream>()?)?.cast::<IDataWriter>()?;
    writer.WriteBytes(manifest.as_bytes())?;
    writer.StoreAsync()?.join()?;
    writer.DetachStream()?;
    stream.cast::<IRandomAccessStream>()?.Seek(0)?;

    let base = Uri::CreateUri("https://www.bilibili.com/")?;
    let created = AdaptiveMediaSource::CreateFromStreamWithDownloaderAsync(
        &stream.cast::<IInputStream>()?,
        &base,
        DASH_TYPE,
        &client(headers)?,
    )?
    .join()?;
    source_of(&created)
}

/// A player on `source`, for a picture to show.
pub fn player(source: &MediaSource) -> Result<MediaPlayer> {
    let player = MediaPlayer::new()?;
    player.cast::<IMediaPlayerSource2>()?.SetSource(&source.cast::<IMediaPlaybackSource>()?)?;
    // Opening a source clears what the system media panel shows, so it is said again here.
    let revoker =
        player.cast::<IMediaPlayer>()?.MediaOpened(|_: Ref<MediaPlayer>, _: Ref<IInspectable>| {
            crate::ui::smtc::refresh();
        })?;
    crate::ui::keep(revoker);
    Ok(player)
}

/// As [`player`], with a title and artist for the system media panel. The panel reads them off
/// the playback item, not off the view's own transport controls, so the item has to carry them.
pub fn player_titled(source: &MediaSource, title: &str, artist: &str, cover: &str) -> Result<MediaPlayer> {
    let player = MediaPlayer::new()?;
    let item = crate::xaml::MediaPlaybackItem::Create(source)?;
    if let Ok(display) = item.cast::<crate::xaml::IMediaPlaybackItem2>()
        && let Ok(props) = display.GetDisplayProperties()
        && let Ok(meta) = props.cast::<crate::xaml::IMediaItemDisplayProperties>()
    {
        // Type and properties have to agree, or the panel drops what was set.
        let _ = meta.SetType(crate::xaml::MediaPlaybackType::Music);
        if let Ok(music) = meta.MusicProperties() {
            let _ = music.SetTitle(title);
            let _ = music.SetArtist(artist);
        }
        if !cover.is_empty()
            && let Ok(uri) = crate::xaml::Uri::CreateUri(cover)
            && let Ok(stream) = crate::xaml::RandomAccessStreamReference::CreateFromUri(&uri)
        {
            let _ = meta.SetThumbnail(&stream);
        }
        let _ = display.ApplyDisplayProperties(&props);
    }
    player.cast::<IMediaPlayerSource2>()?.SetSource(&item.cast::<IMediaPlaybackSource>()?)?;
    let revoker =
        player.cast::<IMediaPlayer>()?.MediaOpened(|_: Ref<MediaPlayer>, _: Ref<IInspectable>| {
            crate::ui::smtc::refresh();
        })?;
    crate::ui::keep(revoker);
    Ok(player)
}

/// A silent player on `source`, not on screen, to take over from another. It plays by itself when
/// `plays`.
pub fn standby(source: &MediaSource, plays: bool) -> Result<MediaPlayer> {
    let player = MediaPlayer::new()?;
    let control = player.cast::<IMediaPlayer>()?;
    control.SetIsMuted(true)?;
    control.SetAutoPlay(plays)?;
    player.cast::<IMediaPlayerSource2>()?.SetSource(&source.cast::<IMediaPlaybackSource>()?)?;
    Ok(player)
}

/// The session of `player`.
pub fn session_of(player: &MediaPlayer) -> Result<IMediaPlaybackSession> {
    player.cast::<IMediaPlayer3>()?.PlaybackSession()?.cast()
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
