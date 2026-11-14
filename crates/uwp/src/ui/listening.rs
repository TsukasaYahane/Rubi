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


const PANEL: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="520" Padding="18"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /></Grid.RowDefinitions>
    <TextBlock Text="听视频" FontSize="20" FontWeight="Light" />
    <StackPanel Grid.Row="1" Orientation="Horizontal" Margin="0,12,0,0">
      <Border Width="140" Height="79" Background="#1c1c1c"><Image x:Name="Cover" Stretch="UniformToFill" /></Border>
      <StackPanel Margin="12,0,0,0" VerticalAlignment="Center" Width="320">
        <TextBlock x:Name="Song" FontSize="16" TextWrapping="Wrap" MaxLines="2" TextTrimming="CharacterEllipsis" />
        <TextBlock x:Name="Who" FontSize="12" Margin="0,6,0,0" TextTrimming="CharacterEllipsis"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      </StackPanel>
    </StackPanel>
    <StackPanel Grid.Row="2" Orientation="Horizontal" Margin="0,16,0,0">
      <Button x:Name="Prev" Content="上一首" Margin="0,0,8,0" Background="Transparent" BorderThickness="0"
              FontSize="14" Foreground="#0050EF" />
      <Button x:Name="NextSong" Content="下一首" Margin="0,0,8,0" Background="Transparent" BorderThickness="0"
              FontSize="14" Foreground="#0050EF" />
      <Button x:Name="StopSong" Content="停止" Background="Transparent" BorderThickness="0" FontSize="14"
              Foreground="#C04A4A" />
    </StackPanel>
    <StackPanel Grid.Row="3" Orientation="Horizontal" HorizontalAlignment="Right" Margin="0,16,0,0">
      <Button x:Name="ClosePanel" Content="关闭" Background="#0050EF" Foreground="White" BorderThickness="0" />
    </StackPanel>
  </Grid>
</Grid>"##;

thread_local! {
    /// The panel on show, so the worker can fill it in.
    static PANEL_ROOT: RefCell<Option<crate::xaml::IFrameworkElement>> = const { RefCell::new(None) };
}

/// Shows what is being listened to.
pub fn open_panel() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let root = match crate::ui::load(PANEL) {
        Ok(root) => root,
        Err(problem) => {
            toast(&format!("打不开：{problem}"));
            return;
        }
    };
    PANEL_ROOT.with_borrow_mut(|slot| *slot = Some(root.clone()));
    crate::ui::on_click(&root, "ClosePanel", || {
        let _ = crate::ui::popup::hide();
    })
    .ok();
    crate::ui::on_click(&root, "StopSong", || {
        stop();
        let _ = crate::ui::popup::hide();
        toast("已停止");
    })
    .ok();
    crate::ui::on_click(&root, "NextSong", || step(true)).ok();
    crate::ui::on_click(&root, "Prev", || step(false)).ok();
    if let Err(problem) = crate::ui::popup::show(&root) {
        toast(&format!("打不开：{problem}"));
        return;
    }
    show_current();
}

/// Fills the panel in with what the queue says is on now.
fn show_current() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let now = core.services.listening.current();
        let left = core.services.listening.items().map(|items| items.len()).unwrap_or(0);
        crate::ui::post(&dispatcher, move || {
            let Some(root) = PANEL_ROOT.with_borrow(Clone::clone) else { return };
            match now {
                Some(card) => {
                    let _ = crate::ui::set_text(&root, "Song", &card.title);
                    let _ = crate::ui::set_text(&root, "Who", &format!("{} · 队列 {left} 首", card.owner.name));
                    if let Ok(image) = crate::ui::named::<crate::xaml::IImage>(&root, "Cover") {
                        let covers = crate::ui::rows::Covers::default();
                        let _ = covers.show(&image, &card.cover, 240);
                    }
                }
                None => {
                    let _ = crate::ui::set_text(&root, "Song", "还没有在听的视频");
                    let _ = crate::ui::set_text(&root, "Who", "在播放页点「听」开始");
                }
            }
        });
    });
}

/// Moves to the next (or previous) video of the listening queue.
fn step(forward: bool) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let moved = if forward {
            core.services.listening.next(true).ok().flatten()
        } else {
            core.services.listening.previous().ok().flatten()
        };
        let Some(card) = moved else {
            crate::ui::post(&dispatcher, || toast("没有下一首了"));
            return;
        };
        let quality = core.services.preferences.settings().playback.preferred_quality;
        if let Ok(playback) = core.services.videos.playback(&card.bvid, quality).await
            && let Some(stream) = playback.url.audio.first()
        {
            let (url, headers, title, id) =
                (stream.url.clone(), playback.headers.clone(), card.title.clone(), card.bvid.clone());
            crate::ui::post(&dispatcher, move || {
                play(&id, &title, &url, &headers);
                show_current();
            });
        }
    });
}
