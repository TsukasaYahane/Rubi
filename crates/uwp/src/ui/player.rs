//! 播放: one video through the system player. The manifest comes from `state`; the stream
//! requests carry the headers B站 asks for through the player's own HTTP client.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use rubi_core::{Playback, pick_streams};
use windows_core::{Error, HRESULT, Interface, Ref, Result};

use crate::state::manifest::dash_manifest;
use crate::ui::shell::shell;
use crate::ui::{load, named, on_click, post, set_text};
use crate::xaml::{
    AdaptiveMediaSource, AdaptiveMediaSourceCreationStatus, DataWriter, HttpClient, IDataWriter, IFrameworkElement,
    IInputStream, IMediaPlaybackSource, IMediaPlayer, IMediaPlayerElement, IMediaPlayerFailedEventArgs,
    IMediaPlayerSource2, IOutputStream, IRandomAccessStream, InMemoryRandomAccessStream, MediaPlayer,
    MediaPlayerFailedEventArgs, MediaSource, Uri,
};

/// Quality asked for; the core caps it to what the account may play.
const QUALITY: u32 = 80;
const DASH_TYPE: &str = "application/dash+xml";

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" />
    <RowDefinition Height="*" />
    <RowDefinition Height="Auto" />
  </Grid.RowDefinitions>
  <Button x:Name="Back" Background="Transparent" BorderThickness="0" Padding="12,8">
    <StackPanel Orientation="Horizontal">
      <FontIcon Glyph="&#xE72B;" FontSize="16" /><TextBlock Text="返回" Margin="10,0,0,0" />
    </StackPanel>
  </Button>
  <MediaPlayerElement x:Name="Video" Grid.Row="1" AreTransportControlsEnabled="True" Stretch="Uniform" />
  <StackPanel Grid.Row="2" Margin="16,10,16,14">
    <TextBlock x:Name="Title" FontSize="20" FontWeight="Light" TextWrapping="Wrap" />
    <TextBlock x:Name="Meta" FontSize="12" Margin="0,4,0,0"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </StackPanel>
</Grid>"##;

/// Bumped when a video opens or the page is left: an answer for an earlier one is dropped.
static RUN: AtomicU64 = AtomicU64::new(0);

thread_local! {
    /// The player of the page on show; on the UI thread.
    static PLAYING: RefCell<Option<MediaPlayer>> = const { RefCell::new(None) };
}

/// Stops the video and lets its player go.
pub fn leave() {
    RUN.fetch_add(1, Ordering::Relaxed);
    if let Some(player) = PLAYING.with(|p| p.borrow_mut().take()) {
        let _ = player.cast::<IMediaPlayer>().and_then(|p| p.Pause());
        let _ = player.cast::<IMediaPlayerSource2>().and_then(|p| p.SetSource(None::<&IMediaPlaybackSource>));
    }
}

/// Opens the video page.
pub fn show(bvid: &str, title: &str, uploader: &str) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    leave();
    let root = load(LAYOUT)?;
    set_text(&root, "Title", title)?;
    set_text(&root, "Meta", &format!("{uploader} · 正在打开…"))?;
    on_click(&root, "Back", go_back)?;
    shell.set_page(&root)?;

    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    let run = RUN.load(Ordering::Relaxed);
    let dispatcher = shell.dispatcher.clone();
    let core = backend.clone();
    let (bvid, uploader) = (bvid.to_owned(), uploader.to_owned());
    backend.spawn(async move {
        let source = match core.services.videos.playback(&bvid, QUALITY).await {
            Ok(playback) => open_source(&playback).map_err(|e| e.message()),
            Err(e) => Err(e.to_string()),
        };
        post(&dispatcher, move || {
            if RUN.load(Ordering::Relaxed) == run {
                let _ = shown(source, &uploader);
            }
        });
    });
    Ok(())
}

fn go_back() {
    if let Some(shell) = shell() {
        shell.back();
    }
}

fn shown(source: std::result::Result<MediaSource, String>, uploader: &str) -> Result<()> {
    let Some(root) = shell().and_then(|s| s.current_page().ok()) else { return Ok(()) };
    match source {
        Ok(source) => {
            play(&root, &source)?;
            set_text(&root, "Meta", uploader)
        }
        Err(problem) => set_text(&root, "Meta", &format!("{uploader} · 无法播放：{problem}")),
    }
}

/// A failure in the player shows on the page.
fn report_failures(player: &MediaPlayer) -> Result<()> {
    let Some(dispatcher) = shell().map(|s| s.dispatcher.clone()) else { return Ok(()) };
    let run = RUN.load(Ordering::Relaxed);
    let revoker = player.cast::<IMediaPlayer>()?.MediaFailed(
        move |_: Ref<MediaPlayer>, args: Ref<MediaPlayerFailedEventArgs>| {
            let reason = args
                .ok()
                .and_then(|a| {
                    let a = a.cast::<IMediaPlayerFailedEventArgs>()?;
                    Ok(format!("{:?} {} {:?}", a.Error()?, a.ErrorMessage()?, a.ExtendedErrorCode()?))
                })
                .unwrap_or_default();
            post(&dispatcher, move || {
                if RUN.load(Ordering::Relaxed) == run
                    && let Some(root) = shell().and_then(|s| s.current_page().ok())
                {
                    let _ = set_text(&root, "Meta", &format!("播放失败：{reason}"));
                }
            });
        },
    )?;
    std::mem::forget(revoker);
    Ok(())
}

fn play(root: &IFrameworkElement, source: &MediaSource) -> Result<()> {
    let player = MediaPlayer::new()?;
    player.cast::<IMediaPlayerSource2>()?.SetSource(&source.cast::<IMediaPlaybackSource>()?)?;
    named::<IMediaPlayerElement>(root, "Video")?.SetMediaPlayer(&player)?;
    report_failures(&player)?;
    player.cast::<IMediaPlayer>()?.Play()?;
    PLAYING.with(|p| *p.borrow_mut() = Some(player));
    Ok(())
}

/// The media source for `playback`. Blocks on WinRT calls: never on the UI thread.
fn open_source(playback: &Playback) -> Result<MediaSource> {
    let (video, audio) = pick_streams(&playback.url);
    let manifest = video
        .and_then(|v| dash_manifest(playback.url.duration_ms, v, audio))
        .ok_or_else(|| Error::new(HRESULT(0x8000_4005_u32 as i32), "这个视频没有可用的 DASH 信息"))?;

    let client = HttpClient::new()?;
    let headers = client.DefaultRequestHeaders()?;
    for (name, value) in &playback.headers {
        headers.TryAppendWithoutValidation(name, value)?;
    }

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
        &client,
    )?
    .join()?;
    let status = created.Status()?;
    if status != AdaptiveMediaSourceCreationStatus::Success {
        return Err(Error::new(HRESULT(0x8000_4005_u32 as i32), format!("系统播放器拒绝了清单：{status:?}")));
    }
    MediaSource::CreateFromAdaptiveMediaSource(&created.MediaSource()?)
}
