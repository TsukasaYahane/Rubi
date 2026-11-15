//! 听视频: the audio track alone, carrying on with the next video when one ends.

use std::cell::RefCell;

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::ui::media;
use rubi_core::PlaylistMode;

use crate::ui::shell::{shell, toast};
use crate::ui::{keep, post};
use crate::xaml::{IMediaPlayer, MediaPlaybackState, MediaPlayer};

thread_local! {
    /// Set while the next video is being fetched, so one ending moves on once.
    static ADVANCING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };

    /// The player making the sound, and the video it is playing.
    static HEARD: RefCell<Option<(MediaPlayer, String)>> = const { RefCell::new(None) };
}

/// Plays `bvid`'s audio, then whatever the listening queue holds next.
pub fn start(bvid: &str) {
    crate::ui::player::stop_video();
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (bvid, dispatcher, core) = (bvid.to_owned(), shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let begun = core.services.listening.start_related(&bvid).await;
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
            Err(problem) => {
                say(&format!("取音轨失败：{problem}"));
                format!("取音轨失败：{problem}")
            }
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

/// Says when a button could not be wired at all.
fn report(outcome: windows_core::Result<()>, name: &str) {
    if let Err(problem) = outcome {
        say(&format!("{name} 绑不上：{problem}"));
    }
}

/// Says what is going on in the window itself: a toast would be under the page it opened.
fn say(text: &str) {
    let Some(root) = WINDOW_ROOT.with_borrow(Clone::clone) else { return };
    let _ = crate::ui::set_text(&root, "Who", text);
}

fn play(bvid: &str, title: &str, url: &str, headers: &[(String, String)]) {
    stop();
    let _ = headers;
    let source = match media::file_source(url) {
        Ok(source) => source,
        Err(problem) => {
            say(&format!("打不开音轨：{problem}"));
            toast(&format!("打不开音轨：{problem}"));
            return;
        }
    };
    let player = match media::player(&source) {
        Ok(player) => player,
        Err(problem) => {
            say(&format!("打不开播放器：{problem}"));
            toast(&format!("打不开播放器：{problem}"));
            return;
        }
    };
    if let Err(problem) = player.Play() {
        say(&format!("放不出来：{problem}"));
        toast(&format!("放不出来：{problem}"));
        return;
    }
    // The next video starts when this one ends. MediaEnded is not raised for a single audio
    // file played through an adaptive source, so the session's own state is watched instead.
    if let Ok(session) = media::session_of(&player) {
        let watched = session.clone();
        if let Ok(revoker) = session.PlaybackStateChanged(
            move |_: Ref<crate::xaml::MediaPlaybackSession>, _: Ref<IInspectable>| {
                let finished = watched
                    .cast::<crate::xaml::IMediaPlaybackSession>()
                    .and_then(|s| s.PlaybackState())
                    .is_ok_and(|state| state == crate::xaml::MediaPlaybackState::None);
                if finished && !ADVANCING.replace(true) {
                    advance();
                }
            },
        ) {
            keep(revoker);
        }
    }

    HEARD.with_borrow_mut(|slot| *slot = Some((player, bvid.to_owned())));
    let _ = title;
    fill_window();
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
    crate::ui::on_click(&root, "Mode", cycle_mode).ok();
    if let Ok(bar) = crate::ui::named::<crate::xaml::IRangeBase>(&root, "Bar")
        && let Ok(revoker) = bar.ValueChanged(move |_: Ref<IInspectable>, _: Ref<crate::xaml::RangeBaseValueChangedEventArgs>| {
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
                fill_window();
            });
        }
    });
}


// ---------------------------------------------------------------------------------------------
// The listening window: the whole window becomes a player, with no navigation.

const WINDOW: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#000000">
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <Grid Margin="24,16,24,8">
    <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
    <Button x:Name="Close" Content="‹ 退出" Background="Transparent" BorderThickness="0"
            Foreground="#FFFFFF" FontSize="15" Padding="0" />
    <TextBlock Grid.Column="1" Text="听视频" FontSize="15" Margin="16,0,0,0" VerticalAlignment="Center"
               Foreground="#A6A6A6" />
  </Grid>
  <Grid Grid.Row="1" Margin="24,0,24,24">
    <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="340" /></Grid.ColumnDefinitions>
    <StackPanel VerticalAlignment="Center" HorizontalAlignment="Center" MaxWidth="560">
      <Border Width="300" Height="300" HorizontalAlignment="Center" Background="#1c1c1c">
        <Image x:Name="Cover" Stretch="UniformToFill" />
      </Border>
      <TextBlock x:Name="Song" Margin="0,22,0,0" FontSize="20" FontWeight="SemiBold" TextAlignment="Center"
                 TextWrapping="Wrap" MaxLines="2" TextTrimming="CharacterEllipsis" Foreground="#FFFFFF" />
      <TextBlock x:Name="Who" Margin="0,8,0,0" FontSize="13" TextAlignment="Center" TextTrimming="CharacterEllipsis"
                 Foreground="#A6A6A6" />
      <Grid Margin="0,20,0,0">
        <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
        <Slider x:Name="Bar" Minimum="0" Maximum="1" StepFrequency="1" Margin="0,0,0,0"
                VerticalAlignment="Center" Foreground="#0050EF" Background="#262626" />
        <TextBlock x:Name="Clock" Grid.Column="1" Margin="12,0,0,0" FontSize="11" Foreground="#A6A6A6"
                   FontFamily="Consolas" VerticalAlignment="Center" />
      </Grid>
      <StackPanel Orientation="Horizontal" HorizontalAlignment="Center" Margin="0,22,0,0">
        <Button x:Name="Mode" Width="96" Background="Transparent" BorderThickness="0" Foreground="#A6A6A6"
                FontSize="13" VerticalAlignment="Center" />
        <Button x:Name="Prev" Content="上一首" Width="72" Background="Transparent" BorderThickness="0"
                Foreground="#A6A6A6" FontSize="14" Margin="8,0,16,0" />
        <Button x:Name="PlayPause" Width="72" Height="72" Background="#0050EF" BorderThickness="0"
                Foreground="#FFFFFF" FontSize="22" />
        <Button x:Name="NextSong" Content="下一首" Width="72" Background="Transparent" BorderThickness="0"
                Foreground="#A6A6A6" FontSize="14" Margin="16,0,8,0" />
        <Button x:Name="Watch" Content="看视频" Width="96" Background="Transparent" BorderThickness="0"
                Foreground="#0050EF" FontSize="14" VerticalAlignment="Center" />
      </StackPanel>
    </StackPanel>
    <Grid Grid.Column="1" Margin="24,0,0,0">
      <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
      <TextBlock x:Name="QueueHead" Text="接下来" FontSize="16" FontWeight="Light" Foreground="#FFFFFF"
                 Margin="0,0,0,10" />
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
}

/// Turns the window into the player.
pub fn open_window() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    // What was on show is put back when the window is given back.
    if let Ok(previous) = shell.current_page() {
        BEFORE.with_borrow_mut(|slot| *slot = Some(previous));
    }
    let root = crate::ui::load(WINDOW)?;
    shell.set_page(&root)?;
    shell.immersive(true)?;
    WINDOW_ROOT.with_borrow_mut(|slot| *slot = Some(root.clone()));
    // Every button is wired, and one that cannot be says so instead of doing nothing.
    report(crate::ui::on_click(&root, "Close", close_window), "Close");
    report(crate::ui::on_click(&root, "Watch", close_window), "Watch");
    report(crate::ui::on_click(&root, "PlayPause", toggle), "PlayPause");
    report(crate::ui::on_click(&root, "NextSong", || step(true)), "NextSong");
    report(crate::ui::on_click(&root, "Prev", || step(false)), "Prev");
    report(crate::ui::on_click(&root, "Mode", cycle_mode), "Mode");
    fill_window();
    tick();
    Ok(())
}

/// Keeps the clock up to date while the window is on show.
fn tick() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let dispatcher = shell.dispatcher.clone();
    backend.spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let alive = WINDOW_ROOT.with_borrow(|slot| slot.is_some());
            if !alive {
                return;
            }
            let _ = crate::ui::post(&dispatcher, paint);
        }
    });
}

/// Gives the window back.
pub fn close_window() {
    let Some(shell) = shell() else { return };
    WINDOW_ROOT.with_borrow_mut(|slot| *slot = None);
    let _ = shell.immersive(false);
    // The page that was there before comes back: showing the player again would rebuild it, and
    // the window content would keep the listening one.
    if let Some(previous) = BEFORE.with_borrow_mut(|slot| slot.take()) {
        let _ = shell.set_page(&previous);
        return;
    }
    if let Some((_, bvid)) = HEARD.with_borrow(Clone::clone) {
        crate::ui::player::show(&bvid, "", "");
    }
}

fn toggle() {
    let heard = HEARD.with_borrow(Clone::clone);
    let Some((player, _)) = heard else {
        let queued = shell()
            .and_then(|s| s.backend.get().cloned())
            .map(|core| core.services.listening.items().map(|i| i.len()).unwrap_or(0))
            .unwrap_or(0);
        toast(&format!("播放器还没准备好 · 队列 {queued} 首"));
        return;
    };
    let playing = media::session_of(&player)
        .and_then(|s| s.PlaybackState())
        .is_ok_and(|state| state == MediaPlaybackState::Playing);
    let outcome = if playing { player.Pause() } else { player.Play() };
    if let Err(problem) = outcome {
        toast(&format!("暂停/播放失败：{problem}"));
    }
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
                if let Ok(image) = crate::ui::named::<crate::xaml::IImage>(&root, "Cover") {
                    let covers = crate::ui::rows::Covers::default();
                    let _ = covers.show(&image, &card.cover, 300);
                }
            }
            paint_queue(&root, &queued, now.as_ref().map(|card| card.bvid.clone()));
            paint();
        });
    });
}

/// Lists the videos after the one playing.
fn paint_queue(root: &crate::xaml::IFrameworkElement, items: &[rubi_model::VideoCard], on: Option<String>) {
    let Ok(panel) = crate::ui::named::<crate::xaml::IPanel>(root, "Queue") else { return };
    let Ok(children) = panel.Children().and_then(|c| c.cast::<windows_collections::IVector<crate::xaml::UIElement>>()) else {
        return;
    };
    let _ = children.Clear();
    let _ = crate::ui::set_text(root, "QueueHead", &format!("接下来 · {} 首", items.len()));
    for item in items.iter().take(60) {
        let bright = on.as_deref() == Some(item.bvid.as_str());
        let xaml = format!(
            r##"<TextBlock xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                  xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Text="{}" FontSize="13" Margin="0,0,0,8"
                  TextTrimming="CharacterEllipsis" Foreground="{}" />"##,
            crate::state::settings::escape(&item.title),
            if bright { "#7EA6FF" } else { "#A6A6A6" }
        );
        if let Ok(line) = crate::ui::load(&xaml)
            && let Ok(element) = line.cast::<crate::xaml::UIElement>()
        {
            // A tap plays that video.
            let aimed = item.bvid.clone();
            if let Ok(revoker) = element.Tapped(move |_: Ref<IInspectable>, _: Ref<crate::xaml::TappedRoutedEventArgs>| {
                jump_to(&aimed);
            }) {
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
            && let Some(stream) = playback.url.audio.first()
        {
            let (url, headers, title, id) =
                (stream.url.clone(), playback.headers.clone(), card.title.clone(), card.bvid.clone());
            crate::ui::post(&dispatcher, move || {
                play(&id, &title, &url, &headers);
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
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
    );
    let Some(root) = WINDOW_ROOT.with_borrow(Clone::clone) else { return };
    let Ok(bar) = crate::ui::named::<crate::xaml::IRangeBase>(&root, "Bar") else { return };
    let Ok(position) = bar.Value() else { return };
    let heard = HEARD.with_borrow(Clone::clone);
    let Some((player, _)) = heard else { return };
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

/// Steps the mode on: in order, around, one again, at random.
fn cycle_mode() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let now = core.services.listening.mode().unwrap_or_default();
        let next = match now {
            PlaylistMode::Sequential => PlaylistMode::LoopAll,
            PlaylistMode::LoopAll => PlaylistMode::LoopOne,
            PlaylistMode::LoopOne => PlaylistMode::Shuffle,
            PlaylistMode::Shuffle => PlaylistMode::Sequential,
        };
        let done = core.services.listening.set_mode(next);
        crate::ui::post(&dispatcher, move || {
            if done.is_err() {
                toast("模式没能改");
            }
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
    let _ = root;
    show_mode(&root);
    let heard = HEARD.with_borrow(Clone::clone);
    let Some((player, _)) = heard else { return };
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
    let playing = session
        .PlaybackState()
        .is_ok_and(|state| state == MediaPlaybackState::Playing);
    let label = if playing { "❚❚" } else { "▶" };
    if let Ok(value) = crate::xaml::PropertyValue::CreateString(label) {
        if let Ok(button) = crate::ui::named::<crate::xaml::IContentControl>(&root, "PlayPause") {
            let _ = button.SetContent(&value);
        }
    }
}

/// Says which mode the queue is in.
fn show_mode(root: &crate::xaml::IFrameworkElement) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let Ok(mode) = core.services.listening.mode() else { return };
        crate::ui::post(&dispatcher, move || {
            let Some(root) = WINDOW_ROOT.with_borrow(Clone::clone) else { return };
            let _ = crate::ui::set_text(&root, "Mode", mode_label(mode));
        });
    });
}

fn mmss(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}
