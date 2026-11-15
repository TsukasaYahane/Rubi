//! 听视频: the audio track alone, carrying on with the next video when one ends.

use std::cell::RefCell;

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::manifest::dash_manifest;
use crate::ui::media;
use rubi_core::PlaylistMode;

use crate::ui::shell::{shell, toast};
use crate::ui::{keep, post};
use crate::xaml::{IMediaPlayer, MediaPlaybackState, MediaPlayer, MediaSource};

thread_local! {
    /// Set while the next video is being fetched, so one ending moves on once.
    static ADVANCING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };

    /// The one player the sound comes out of. Changing track changes its source rather than
    /// building another, so what the buttons hold is always what is heard.
    static SOUND: RefCell<Option<MediaPlayer>> = const { RefCell::new(None) };
    /// The video it is playing.
    static NOW: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Set as soon as listening is asked for, before its track is loaded: the video page may bring
    /// its player up a moment later, and that one must not start sounding under this one.
    static WANTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn sound() -> Option<MediaPlayer> {
    SOUND.with_borrow(Clone::clone)
}

/// Plays `bvid`'s audio, then whatever the listening queue holds next.
pub fn start(bvid: &str) {
    WANTED.set(true);
    crate::ui::player::stop_video();
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    // The list the video page is playing, when 播放全部 put one on: hearing follows it, in the
    // order it was played, rather than bringing 相关推荐 in. Read here, on the UI thread, because
    // the queue is only there.
    let listed = crate::ui::queue::view()
        .filter(|queue| queue.entries().iter().any(|entry| entry.bvid == bvid))
        .map(|queue| (queue.name().to_owned(), queue.entries().iter().map(|e| e.bvid.clone()).collect::<Vec<_>>()));
    let (bvid, dispatcher, core) = (bvid.to_owned(), shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let begun = if core.services.listening.holds(&bvid) {
            core.services.listening.start_here(&bvid).await
        } else if let Some((name, bvids)) = listed {
            core.services.listening.start_listing(&name, &bvids, &bvid).await
        } else {
            core.services.listening.start_related(&bvid).await
        };
        let card = match begun {
            Ok(card) => card,
            Err(problem) => {
                let said = format!("取视频失败：{problem}");
                post(&dispatcher, move || {
                    say(&said);
                    toast(&said);
                });
                return;
            }
        };
        let quality = core.services.preferences.settings().playback.preferred_quality;
        let found = core.services.videos.playback(&card.bvid, quality).await;
        say("正在取音轨…");
        let said = match found {
            Ok(playback) => match audio_source(&playback) {
                Ok(source) => {
                    let (title, id) = (card.title.clone(), card.bvid.clone());
                    let said = format!("正在听：{title}");
                    post(&dispatcher, move || play(&id, &title, source));
                    said
                }
                Err(problem) => format!("打不开音轨：{problem}"),
            },
            Err(problem) => {
                say(&format!("取音轨失败：{problem}"));
                format!("取音轨失败：{problem}")
            }
        };
        post(&dispatcher, move || toast(&said));
    });
}

/// Stops the sound, if any, and lets the queue and where it got to go with it.
pub fn stop() {
    WANTED.set(false);
    // The element lets go first: left holding a closed player, it brings the app down.
    detach();
    let player = SOUND.with_borrow_mut(|slot| slot.take());
    NOW.with_borrow_mut(|slot| *slot = None);
    if let Some(player) = player {
        media::close(Some(player), None);
    }
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let core = backend.clone();
    backend.spawn(async move {
        core.services.listening.stop();
    });
}

/// Folds the queue away to the left, or brings it back.
fn fold(away: bool) {
    let Some(root) = WINDOW_ROOT.with_borrow(Clone::clone) else { return };
    let width = crate::xaml::GridLength {
        value: if away { 0.0 } else { 340.0 },
        grid_unit_type: crate::xaml::GridUnitType::Pixel,
    };
    if let Ok(column) = crate::ui::named::<crate::xaml::IColumnDefinition>(&root, "QueueColumn") {
        let _ = column.SetWidth(width);
    }
    let show = |name: &str, on: bool| {
        if let Ok(element) = crate::ui::named::<crate::xaml::IUIElement>(&root, name) {
            let _ = element.SetVisibility(if on {
                crate::xaml::Visibility::Visible
            } else {
                crate::xaml::Visibility::Collapsed
            });
        }
    };
    show("QueuePane", !away);
    show("Unfold", away);
}

/// Says when a button could not be wired at all.
fn report(outcome: windows_core::Result<()>, name: &str) {
    if let Err(problem) = outcome {
        say(&format!("{name} 绑不上：{problem}"));
    }
}

/// The two styles: picture buttons carry their icon, the words style the word alone.
fn style_buttons(root: &crate::xaml::IFrameworkElement) {
    let words = shell().is_some_and(|s| s.prefs().words);
    for name in ["CloseShape", "PrevShape", "NextShape", "WatchShape"] {
        show_part(root, name, !words);
    }
    for name in ["CloseWords", "PrevWords", "NextWords", "WatchWords"] {
        show_part(root, name, words);
    }
    show_part(root, "ModeWords", words);
    // The mode has three shapes, so its own drawing decides which one shows: see `write_mode`.
    if words {
        for name in ["ModeSeq", "ModeLoop", "ModeShuffle"] {
            show_part(root, name, false);
        }
    }
}

fn show_part(root: &crate::xaml::IFrameworkElement, name: &str, on: bool) {
    if let Ok(part) = crate::ui::named::<crate::xaml::IUIElement>(root, name) {
        let _ = part.SetVisibility(if on { crate::xaml::Visibility::Visible } else { crate::xaml::Visibility::Collapsed });
    }
}

/// Says what is going on in the window itself: a toast would be under the page it opened.
fn say(text: &str) {
    let Some(root) = WINDOW_ROOT.with_borrow(Clone::clone) else { return };
    let _ = crate::ui::set_text(&root, "Who", text);
}

/// The DASH manifest of `playback`'s audio track alone, wrapped as a source the system player
/// can drive. A bare mp4a file only ever made sound: no session, no position, no pause.
fn audio_source(playback: &rubi_core::Playback) -> Result<MediaSource> {
    let audio =
        playback.url.audio.first().ok_or_else(|| {
            windows_core::Error::new(windows_core::HRESULT(0x8000_4005_u32 as i32), "这个视频没有音轨")
        })?;
    let manifest = dash_manifest(playback.url.duration_ms, None, Some(audio)).ok_or_else(|| {
        windows_core::Error::new(windows_core::HRESULT(0x8000_4005_u32 as i32), "这条音轨没有可用的 DASH 信息")
    })?;
    media::manifest_source(&manifest, &playback.headers)
}

fn play(bvid: &str, title: &str, source: MediaSource) {
    let outcome = SOUND.with_borrow_mut(|slot| {
        if let Some(player) = slot.as_ref() {
            let swap = player
                .cast::<crate::xaml::IMediaPlayerSource2>()
                .and_then(|p| p.SetSource(&source.cast::<crate::xaml::IMediaPlaybackSource>()?));
            swap?;
            player.Play()
        } else {
            let player = media::player(&source)?;
            watch_end(&player)?;
            attach(&player);
            player.Play()?;
            *slot = Some(player);
            Ok(())
        }
    });
    if let Err(problem) = outcome {
        say(&format!("放不出来：{problem}"));
        toast(&format!("放不出来：{problem}"));
        return;
    }
    NOW.with_borrow_mut(|slot| *slot = Some(bvid.to_owned()));
    let _ = title;
    fill_window();
}

/// The track ends: move on to whatever the queue holds next.
fn watch_end(player: &MediaPlayer) -> Result<()> {
    let Some(dispatcher) = shell().map(|s| s.dispatcher.clone()) else { return Ok(()) };
    let revoker =
        player.cast::<IMediaPlayer>()?.MediaEnded(move |_: Ref<MediaPlayer>, _: Ref<windows_core::IInspectable>| {
            crate::ui::post(&dispatcher, advance);
        })?;
    keep(revoker);
    Ok(())
}

/// Puts the player on the window's media element, which is what makes it controllable.
fn attach(player: &MediaPlayer) {
    let Some(root) = WINDOW_ROOT.with_borrow(Clone::clone) else { return };
    if let Ok(element) = crate::ui::named::<crate::xaml::MediaPlayerElement>(&root, "Sound")
        && let Ok(host) = element.cast::<crate::xaml::IMediaPlayerElement>()
    {
        let _ = host.SetMediaPlayer(player);
    }
}

/// The element lets go of the player, which has to happen before the player is closed.
fn detach() {
    let Some(root) = WINDOW_ROOT.with_borrow(Clone::clone) else { return };
    if let Ok(element) = crate::ui::named::<crate::xaml::MediaPlayerElement>(&root, "Sound")
        && let Ok(host) = element.cast::<crate::xaml::IMediaPlayerElement>()
    {
        let _ = host.SetMediaPlayer(None);
    }
}

/// Reads the next video of the listening queue and plays it.
fn advance() {
    let Some(shell) = shell() else {
        ADVANCING.set(false);
        return;
    };
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
            && let Ok(source) = audio_source(&playback)
        {
            let (title, id) = (next.title.clone(), next.bvid.clone());
            post(&dispatcher, move || play(&id, &title, source));
        }
    });
}

/// Whether something is being listened to.
pub fn playing() -> bool {
    SOUND.with_borrow(|slot| slot.is_some())
}

/// Whether listening was asked for: its track may still be on its way.
pub fn wanted() -> bool {
    WANTED.get()
}

/// Whether `bvid` is the one being heard: coming back to its page must not restart the queue.
pub fn hearing(bvid: &str) -> bool {
    NOW.with_borrow(|slot| slot.as_deref() == Some(bvid))
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
    crate::ui::on_click(&root, "Mode", cycle_mode).ok();
    if let Ok(bar) = crate::ui::named::<crate::xaml::IRangeBase>(&root, "Bar")
        && let Ok(revoker) =
            bar.ValueChanged(move |_: Ref<IInspectable>, _: Ref<crate::xaml::RangeBaseValueChangedEventArgs>| {
                drag_to();
            })
    {
        keep(revoker);
    }
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
                    show_cover(&root, "Cover", &card.cover, 480);
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
            crate::ui::post(&dispatcher, || {
                say("队列里没有下一首了");
                toast("没有下一首了");
            });
            return;
        };
        let quality = core.services.preferences.settings().playback.preferred_quality;
        if let Ok(playback) = core.services.videos.playback(&card.bvid, quality).await
            && let Ok(source) = audio_source(&playback)
        {
            let (title, id) = (card.title.clone(), card.bvid.clone());
            crate::ui::post(&dispatcher, move || {
                play(&id, &title, source);
                fill_window();
            });
        }
    });
}

// ---------------------------------------------------------------------------------------------
// The listening window: the whole window becomes a player, with no navigation.

const WINDOW: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
      Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
  <Grid.Resources>
    <!-- Metro buttons have square corners. -->
    <CornerRadius x:Key="ControlCornerRadius">0</CornerRadius>
    <!-- The pointer only fades a button: nothing is repainted under it, and pressing is the
         feedback. -->
    <SolidColorBrush x:Key="ButtonForegroundPointerOver" Color="{ThemeResource SystemBaseHighColor}" Opacity="0.5" />
    <SolidColorBrush x:Key="ButtonBackgroundPointerOver" Color="Transparent" />
    <SolidColorBrush x:Key="ButtonBorderBrushPointerOver" Color="Transparent" />
    <SolidColorBrush x:Key="ButtonForegroundPressed" Color="#0050EF" />
    <SolidColorBrush x:Key="ButtonBackgroundPressed" Color="Transparent" />
    <!-- The centre is the cover, frosted: acrylic takes what it is laid over. -->
    <AcrylicBrush x:Key="Frost" BackgroundSource="Backdrop" TintColor="{ThemeResource SystemAltHighColor}"
                  TintOpacity="0.62" FallbackColor="#D9FFFFFF" />
  </Grid.Resources>
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <MediaPlayerElement x:Name="Sound" Width="1" Height="1" Opacity="0" IsHitTestVisible="False"
                      AreTransportControlsEnabled="False" />
  <Grid Margin="24,16,24,8">
    <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
    <Button x:Name="Close" Background="Transparent" BorderThickness="0" Padding="0"
            Foreground="{ThemeResource SystemControlForegroundBaseHighBrush}">
      <StackPanel Orientation="Horizontal">
        <Path x:Name="CloseShape" Data="M20,12 L5,12 M11,6 L5,12 L11,18" Width="20" Height="20"
              Stretch="Uniform" StrokeThickness="2" StrokeStartLineCap="Square" StrokeEndLineCap="Square"
              StrokeLineJoin="Miter" VerticalAlignment="Center" Margin="0,0,8,0"
              Stroke="{ThemeResource SystemControlForegroundBaseHighBrush}" />
        <TextBlock x:Name="CloseWords" Text="退出" FontSize="15" VerticalAlignment="Center" />
      </StackPanel>
    </Button>
    <TextBlock Grid.Column="1" Text="听视频" FontSize="15" Margin="16,0,0,0" VerticalAlignment="Center"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </Grid>
  <Grid Grid.Row="1" Margin="24,0,24,24">
    <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition x:Name="QueueColumn" Width="340" /></Grid.ColumnDefinitions>
    <Grid>
      <Image x:Name="StageCover" Stretch="UniformToFill" Opacity="0.9" />
      <Border Background="{StaticResource Frost}" />
      <StackPanel VerticalAlignment="Center" HorizontalAlignment="Center" MaxWidth="560">
        <Border Width="340" Height="191" HorizontalAlignment="Center"
                Background="{ThemeResource SystemControlForegroundBaseLowBrush}"
                BorderBrush="{ThemeResource ApplicationPageBackgroundThemeBrush}" BorderThickness="2">
          <Image x:Name="Cover" Stretch="UniformToFill" />
        </Border>
        <TextBlock x:Name="Song" Margin="0,22,0,0" FontSize="20" FontWeight="SemiBold" TextAlignment="Center"
                   TextWrapping="Wrap" MaxLines="2" TextTrimming="CharacterEllipsis"
                   Foreground="{ThemeResource SystemControlForegroundBaseHighBrush}" />
        <TextBlock x:Name="Who" Margin="0,8,0,0" FontSize="13" TextAlignment="Center" TextTrimming="CharacterEllipsis"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        <Grid Margin="0,20,0,0">
          <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
          <Slider x:Name="Bar" Minimum="0" Maximum="1" StepFrequency="1" Margin="0,0,0,0"
                  VerticalAlignment="Center" Foreground="#0050EF"
                  Background="{ThemeResource SystemControlForegroundBaseLowBrush}" />
          <TextBlock x:Name="Clock" Grid.Column="1" Margin="12,0,0,0" FontSize="11" FontFamily="Consolas"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}"
                     VerticalAlignment="Center" />
        </Grid>
        <StackPanel Orientation="Horizontal" HorizontalAlignment="Center" Margin="0,22,0,0">
          <Button x:Name="Mode" Background="Transparent" BorderThickness="0" Padding="12,10" VerticalAlignment="Center"
                  Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}">
            <StackPanel Orientation="Horizontal">
              <Grid Width="20" Height="20" VerticalAlignment="Center">
                <Path x:Name="ModeSeq" Data="M3,12 L19,12 M14,7 L19,12 L14,17" Stretch="Uniform"
                      StrokeThickness="2" StrokeStartLineCap="Square" StrokeEndLineCap="Square" StrokeLineJoin="Miter"
                      Stroke="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
                <Path x:Name="ModeLoop" Data="M4,11 L4,7 L19,7 M16,4 L19,7 L16,10 M20,13 L20,17 L5,17 M8,14 L5,17 L8,20 M11,10.5 L12.5,10.5 L12.5,14"
                      Stretch="Uniform" Visibility="Collapsed" StrokeThickness="2" StrokeStartLineCap="Square"
                      StrokeEndLineCap="Square" StrokeLineJoin="Miter"
                      Stroke="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
                <Path x:Name="ModeShuffle" Data="M3,7 L7,7 L15,17 L20,17 M17,14 L20,17 L17,20 M3,17 L7,17 L9.5,14 M12.5,10 L15,7 L20,7 M17,4 L20,7 L17,10"
                      Stretch="Uniform" Visibility="Collapsed" StrokeThickness="2" StrokeStartLineCap="Square"
                      StrokeEndLineCap="Square" StrokeLineJoin="Miter"
                      Stroke="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
              </Grid>
              <TextBlock x:Name="ModeWords" Text="顺序" FontSize="13" VerticalAlignment="Center" Margin="8,0,0,0" />
            </StackPanel>
          </Button>
          <Button x:Name="Prev" Background="Transparent" BorderThickness="0" Padding="12,10" Margin="8,0,8,0"
                  Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" VerticalAlignment="Center">
            <StackPanel Orientation="Horizontal">
              <Path x:Name="PrevShape" Data="M3,4 L6,4 L6,20 L3,20 Z M21,4 L21,20 L7.5,12 Z" Width="20" Height="20"
                    Stretch="Uniform" VerticalAlignment="Center" Margin="0,0,8,0"
                    Fill="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
              <TextBlock x:Name="PrevWords" Text="上一首" FontSize="14" VerticalAlignment="Center" />
            </StackPanel>
          </Button>
          <Button x:Name="PlayPause" Width="64" Height="64" Background="#0050EF" BorderThickness="0"
                  Foreground="#FFFFFF">
            <Button.Resources>
              <SolidColorBrush x:Key="ButtonBackgroundPointerOver" Color="#D90050EF" />
              <SolidColorBrush x:Key="ButtonBackgroundPressed" Color="#B30050EF" />
              <SolidColorBrush x:Key="ButtonForegroundPointerOver" Color="#FFFFFF" />
              <SolidColorBrush x:Key="ButtonForegroundPressed" Color="#FFFFFF" />
            </Button.Resources>
            <Grid>
              <Path x:Name="PlayShape" Data="M6,3.5 L6,20.5 L20.5,12 Z" Width="22" Height="22" Stretch="Uniform"
                    Fill="#FFFFFF" />
              <Path x:Name="PauseShape" Data="M5,4 L9.5,4 L9.5,20 L5,20 Z M14.5,4 L19,4 L19,20 L14.5,20 Z"
                    Width="22" Height="22" Stretch="Uniform" Visibility="Collapsed" Fill="#FFFFFF" />
            </Grid>
          </Button>
          <Button x:Name="NextSong" Background="Transparent" BorderThickness="0" Padding="12,10" Margin="8,0,8,0"
                  Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" VerticalAlignment="Center">
            <StackPanel Orientation="Horizontal">
              <Path x:Name="NextShape" Data="M3,4 L3,20 L16.5,12 Z M18,4 L21,4 L21,20 L18,20 Z" Width="20" Height="20"
                    Stretch="Uniform" VerticalAlignment="Center" Margin="0,0,8,0"
                    Fill="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
              <TextBlock x:Name="NextWords" Text="下一首" FontSize="14" VerticalAlignment="Center" />
            </StackPanel>
          </Button>
          <Button x:Name="Watch" Background="Transparent" BorderThickness="0" Padding="12,10" VerticalAlignment="Center"
                  Foreground="#0050EF">
            <StackPanel Orientation="Horizontal">
              <Path x:Name="WatchShape" Data="M2,5 L22,5 L22,18 L2,18 Z M8,21 L16,21 M10.5,8.5 L10.5,14.5 L15,11.5 Z"
                    Width="20" Height="20" Stretch="Uniform" StrokeThickness="2" StrokeStartLineCap="Square"
                    StrokeEndLineCap="Square" StrokeLineJoin="Miter" VerticalAlignment="Center" Margin="0,0,8,0"
                    Stroke="#0050EF" />
              <TextBlock x:Name="WatchWords" Text="看视频" FontSize="14" VerticalAlignment="Center" />
            </StackPanel>
          </Button>
        </StackPanel>
      </StackPanel>
    </Grid>
    <!-- The way back sits in the main column: the queue column is what folds to nothing, and a
         button in it would fold with it and never be hit again. -->
    <Button x:Name="Unfold" HorizontalAlignment="Right" VerticalAlignment="Top" Background="Transparent"
            BorderThickness="0" Padding="10,8" Visibility="Collapsed"
            Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}">
      <Path Data="M15,5 L8,12 L15,19" Width="16" Height="16" Stretch="Uniform" StrokeThickness="2"
            StrokeStartLineCap="Square" StrokeEndLineCap="Square" StrokeLineJoin="Miter"
            Stroke="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    </Button>
    <Grid x:Name="QueuePane" Grid.Column="1" Margin="24,0,0,0">
      <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
      <Grid>
        <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
        <TextBlock x:Name="QueueHead" Text="接下来" FontSize="16" FontWeight="Light" Margin="0,0,0,10"
                   Foreground="{ThemeResource SystemControlForegroundBaseHighBrush}" />
        <Button x:Name="Fold" Grid.Column="1" Background="Transparent" BorderThickness="0" Padding="10,8"
                VerticalAlignment="Top" Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}">
          <Path Data="M9,5 L16,12 L9,19" Width="16" Height="16" Stretch="Uniform" StrokeThickness="2"
                StrokeStartLineCap="Square" StrokeEndLineCap="Square" StrokeLineJoin="Miter"
                Stroke="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </Button>
      </Grid>
      <ScrollViewer Grid.Row="1"><StackPanel x:Name="Queue" /></ScrollViewer>
    </Grid>
  </Grid>
</Grid>"##;

thread_local! {
    /// The window on show, while it is, and the page it took the place of.
    static WINDOW_ROOT: RefCell<Option<crate::xaml::IFrameworkElement>> = const { RefCell::new(None) };
    static BEFORE: RefCell<Option<crate::xaml::IFrameworkElement>> = const { RefCell::new(None) };
    /// Set while the clock writes the slider's value, so that is not read as a drag.
    static SYNCING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// When the bar was last dragged: the clock leaves it alone for a while after, or it
    /// would pull the thumb back under the pointer.
    static DRAGGED: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// The mode the queue is in, as last read: the clock shows it without asking the store
    /// again every half second.
    static MODE: std::cell::Cell<Option<PlaylistMode>> = const { std::cell::Cell::new(None) };
    /// The pictures the window and the panel show. They are held here, as on a page of their
    /// own: dropped with the call that asked for them, the sharp copy could never be handed
    /// over and the rough one would stay on screen.
    static COVERS: crate::ui::rows::Covers = crate::ui::rows::Covers::default();
}

/// Turns the window into the player.
pub fn open_window() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    // The page the window took the place of: 退出 puts it back.
    if let Ok(previous) = shell.current_page() {
        BEFORE.with_borrow_mut(|slot| *slot = Some(previous));
    }
    let root = crate::ui::load(WINDOW)?;
    shell.set_page(&root)?;
    shell.immersive(true)?;
    WINDOW_ROOT.with_borrow_mut(|slot| *slot = Some(root.clone()));
    // Every button is wired, and one that cannot be says so instead of doing nothing.
    report(crate::ui::on_click(&root, "Close", close_window), "Close");
    report(crate::ui::on_click(&root, "Watch", watch_window), "Watch");
    report(crate::ui::on_click(&root, "PlayPause", toggle), "PlayPause");
    report(crate::ui::on_click(&root, "NextSong", || step(true)), "NextSong");
    report(crate::ui::on_click(&root, "Prev", || step(false)), "Prev");
    report(crate::ui::on_click(&root, "Mode", cycle_mode), "Mode");
    report(crate::ui::on_click(&root, "Fold", || fold(true)), "Fold");
    report(crate::ui::on_click(&root, "Unfold", || fold(false)), "Unfold");
    style_buttons(&root);
    if let Ok(bar) = crate::ui::named::<crate::xaml::IRangeBase>(&root, "Bar")
        && let Ok(revoker) =
            bar.ValueChanged(move |_: Ref<IInspectable>, _: Ref<crate::xaml::RangeBaseValueChangedEventArgs>| {
                drag_to();
            })
    {
        keep(revoker);
    }
    fill_window();
    MODE.set(None);
    read_mode();
    tick();
    Ok(())
}

/// Keeps the clock up to date while the window is on show. The liveness check runs on the UI
/// thread: thread-locals read from a tokio worker are always empty.
fn tick() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let dispatcher = shell.dispatcher.clone();
    backend.spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let (done, wait) = tokio::sync::oneshot::channel();
            crate::ui::post(&dispatcher, move || {
                paint();
                let _ = done.send(WINDOW_ROOT.with_borrow(|slot| slot.is_none()));
            });
            if wait.await.unwrap_or(true) {
                return;
            }
        }
    });
}

/// 退出: the window goes and the page listening started from comes back as it was left.
pub fn close_window() {
    leave(false);
}

/// 看视频: the video being heard, on its own page.
fn watch_window() {
    leave(true);
}

/// Gives the window back. `to_heard` goes to what is being heard; otherwise the page the window
/// took the place of comes back, as it was.
fn leave(to_heard: bool) {
    let Some(shell) = shell() else { return };
    // The element lets go of the player while the window is still there: taken away holding a
    // closed player it freezes the app. This has to come before WINDOW_ROOT is cleared, or the
    // detach inside `stop` finds no window and does nothing.
    detach();
    WINDOW_ROOT.with_borrow_mut(|slot| *slot = None);
    MODE.set(None);
    let _ = shell.immersive(false);
    let heard = NOW.with_borrow(Clone::clone);
    let previous = BEFORE.with_borrow_mut(|slot| slot.take());
    // Leaving ends listening: the sound, the queue and where it got to all go.
    stop();
    if !to_heard && let Some(previous) = previous {
        // Its video is paused where listening found it, its progress untouched.
        let _ = shell.set_page(&previous);
        return;
    }
    if let Some(bvid) = heard {
        crate::ui::player::show(&bvid, "", "");
    }
}

fn toggle() {
    let Some(player) = sound() else {
        say("播放器没就绪");
        return;
    };
    let playing =
        media::session_of(&player).and_then(|s| s.PlaybackState()).is_ok_and(|s| s == MediaPlaybackState::Playing);
    let outcome = if playing { player.Pause() } else { player.Play() };
    if let Err(problem) = outcome {
        say(&format!("按不动：{problem}"));
        return;
    }
    say(if playing { "已暂停" } else { "继续播放" });
    paint();
}

/// Fills the window in with what the queue says is on now, and what comes after.
fn fill_window() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let now = core.services.listening.current();
        let queued = core.services.listening.items().unwrap_or_default();
        crate::ui::post(&dispatcher, move || {
            let Some(root) = WINDOW_ROOT.with_borrow(Clone::clone) else { return };
            if let Some(card) = &now {
                let _ = crate::ui::set_text(&root, "Song", &card.title);
                let _ = crate::ui::set_text(&root, "Who", &card.owner.name);
                // The same picture twice: the small one, and the one the frost is laid over.
                show_cover(&root, "Cover", &card.cover, 340);
                show_cover(&root, "StageCover", &card.cover, 680);
            }
            paint_queue(&root, &queued, now.as_ref().map(|card| card.bvid.clone()));
            paint();
        });
    });
}

/// Shows `cover` in the picture called `name`, decoded `width` px wide.
fn show_cover(root: &crate::xaml::IFrameworkElement, name: &str, cover: &str, width: i32) {
    let Ok(image) = crate::ui::named::<crate::xaml::IImage>(root, name) else { return };
    COVERS.with(|covers| {
        let _ = covers.show(&image, cover, width);
    });
}

/// Lists the videos after the one playing.
fn paint_queue(root: &crate::xaml::IFrameworkElement, items: &[rubi_model::VideoCard], on: Option<String>) {
    let Ok(panel) = crate::ui::named::<crate::xaml::IPanel>(root, "Queue") else { return };
    let Ok(children) = panel.Children().and_then(|c| c.cast::<windows_collections::IVector<crate::xaml::UIElement>>())
    else {
        return;
    };
    let _ = children.Clear();
    let _ = crate::ui::set_text(root, "QueueHead", &format!("接下来 · {} 首", items.len()));
    for item in items.iter().take(60) {
        let bright = on.as_deref() == Some(item.bvid.as_str());
        // The one being heard is blue and heavy; the rest take the theme's own word colour.
        let look = if bright { r##" Foreground="#0050EF" FontWeight="SemiBold""## } else { "" };
        let xaml = format!(
            r##"<TextBlock xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                  xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Text="{}" FontSize="13" Margin="0,0,0,8"
                  TextTrimming="CharacterEllipsis"{} />"##,
            crate::state::settings::escape(&item.title),
            look
        );
        if let Ok(line) = crate::ui::load(&xaml)
            && let Ok(element) = line.cast::<crate::xaml::UIElement>()
        {
            // A tap plays that video.
            let aimed = item.bvid.clone();
            if let Ok(revoker) =
                element.Tapped(move |_: Ref<IInspectable>, _: Ref<crate::xaml::TappedRoutedEventArgs>| {
                    jump_to(&aimed);
                })
            {
                keep(revoker);
            }
            let _ = children.Append(&element);
        }
    }
}

/// Plays vid now, and refills the window.
fn jump_to(bvid: &str) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (bvid, dispatcher, core) = (bvid.to_owned(), shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let Ok(card) = core.services.listening.jump(&bvid).await else { return };
        let quality = core.services.preferences.settings().playback.preferred_quality;
        if let Ok(playback) = core.services.videos.playback(&card.bvid, quality).await
            && let Ok(source) = audio_source(&playback)
        {
            let (title, id) = (card.title.clone(), card.bvid.clone());
            crate::ui::post(&dispatcher, move || {
                play(&id, &title, source);
                fill_window();
            });
        }
    });
}

/// Where the slider was dragged to.
fn drag_to() {
    if SYNCING.get() {
        return;
    }
    DRAGGED.set(
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0),
    );
    let Some(root) = WINDOW_ROOT.with_borrow(Clone::clone) else { return };
    let Ok(bar) = crate::ui::named::<crate::xaml::IRangeBase>(&root, "Bar") else { return };
    let Ok(position) = bar.Value() else { return };
    let Some(player) = sound() else { return };
    let Ok(session) = media::session_of(&player) else { return };
    let Ok(total) = session.NaturalDuration() else { return };
    let whole = bar.Maximum().unwrap_or(0.0);
    if whole <= 0.0 {
        return;
    }
    let share = (position / whole).clamp(0.0, 1.0);
    let at = windows_time::TimeSpan { duration: (total.duration as f64 * share) as i64 };
    if let Err(problem) = session.SetPosition(at) {
        toast(&format!("跳转失败：{problem}"));
    }
}

/// Steps the mode on: 顺序, 单曲循环, 随机, and round again.
fn cycle_mode() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let now = core.services.listening.mode().unwrap_or_default();
        // 列表循环 is not one of them: listening is a list that goes on, and the three cover it.
        let next = match now {
            PlaylistMode::Sequential | PlaylistMode::LoopAll => PlaylistMode::LoopOne,
            PlaylistMode::LoopOne => PlaylistMode::Shuffle,
            PlaylistMode::Shuffle => PlaylistMode::Sequential,
        };
        let done = core.services.listening.set_mode(next);
        let after = core.services.listening.mode().unwrap_or(now);
        crate::ui::post(&dispatcher, move || {
            if done.is_err() {
                toast("模式没能改");
            }
            MODE.set(Some(after));
            paint();
        });
    });
}

/// What the mode reads as.
fn mode_label(mode: PlaylistMode) -> &'static str {
    match mode {
        PlaylistMode::Sequential => "顺序",
        PlaylistMode::LoopAll => "列表循环",
        PlaylistMode::LoopOne => "单曲循环",
        PlaylistMode::Shuffle => "随机",
    }
}

/// Keeps the clock and the progress bar up to date.
fn paint() {
    let Some(root) = WINDOW_ROOT.with_borrow(Clone::clone) else { return };
    if let Some(mode) = MODE.get() {
        write_mode(&root, mode);
    }
    let Some(player) = sound() else { return };
    let Ok(session) = media::session_of(&player) else { return };
    let at = session.Position().map(|p| p.duration as f64 / 10_000_000.0).unwrap_or(0.0);
    let end = session.NaturalDuration().map(|p| p.duration as f64 / 10_000_000.0).unwrap_or(0.0);
    let quiet = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
        .saturating_sub(DRAGGED.get())
        > 1500;
    if end > 0.0
        && quiet
        && let Ok(bar) = crate::ui::named::<crate::xaml::IRangeBase>(&root, "Bar")
    {
        SYNCING.set(true);
        let _ = bar.SetMaximum(end);
        let _ = bar.SetValue(at);
        SYNCING.set(false);
    }
    let clock = format!("{} / {}", mmss(at), mmss(end));
    let _ = crate::ui::set_text(&root, "Clock", &clock);
    let playing = session.PlaybackState().is_ok_and(|s| s == MediaPlaybackState::Playing);
    show_part(&root, "PlayShape", !playing);
    show_part(&root, "PauseShape", playing);
}

/// Writes the mode on its button: one of the three shapes, or the word in the words style.
fn write_mode(root: &crate::xaml::IFrameworkElement, mode: PlaylistMode) {
    let words = shell().is_some_and(|s| s.prefs().words);
    let (sequential, looping, shuffled) = match mode {
        PlaylistMode::Shuffle => (false, false, true),
        PlaylistMode::LoopOne | PlaylistMode::LoopAll => (false, true, false),
        PlaylistMode::Sequential => (true, false, false),
    };
    show_part(root, "ModeSeq", !words && sequential);
    show_part(root, "ModeLoop", !words && looping);
    show_part(root, "ModeShuffle", !words && shuffled);
    let _ = crate::ui::set_text(root, "ModeWords", mode_label(mode));
}

/// Reads the mode once and shows it.
fn read_mode() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let Ok(mode) = core.services.listening.mode() else { return };
        crate::ui::post(&dispatcher, move || {
            MODE.set(Some(mode));
            paint();
        });
    });
}

fn mmss(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}
