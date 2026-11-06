//! A 直播 room: the picture with danmaku over it and beside it. Gift banners, popups and effects
//! stay out, as the core's filter holds them back by default.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rubi_core::{LiveChannel, LiveFilter};
use rubi_model::{Danmaku, LiveEvent, LiveProtocol};
use tokio::sync::oneshot::channel as oneshot;
use windows_collections::IVector;
use windows_core::{Interface, Ref, Result};

use crate::backend::Backend;
use crate::state::layout::WindowClass;
use crate::state::library::count;
use crate::state::live::CHAT_MOST;
use crate::state::quality::Offer;
use crate::ui::control_bar::{self, ControlBar};
use crate::ui::danmaku::{Host, Overlay};
use crate::ui::quality;
use crate::ui::rows::{add_rows, on_row};
use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load, media, named, on_click, on_enter, post, set_text, soon};
use crate::xaml::{
    Color, ElementTheme, FocusState, FrameworkElement, Grid, GridLength, GridUnitType, IColumnDefinition, IControl,
    IFontIcon, IFrameworkElement, IFrameworkElement2, IItemsControl, IMediaPlayer, IMediaPlayerElement, IPanel,
    IRowDefinition, ITextBlock, ITextBox, IUIElement, MediaPlaybackState, MediaPlayer, MediaSource,
    SizeChangedEventArgs, SolidColorBrush, Thickness, VirtualKey, Visibility,
};

/// Width of the danmaku list beside the picture.
const CHAT_WIDTH: f64 = 320.0;
/// How far ahead of now a new danmaku starts, so it does not appear mid-way.
const DANMAKU_LEAD_MS: u32 = 200;
/// How often a new stream is looked at, and how many times before giving up on it.
const STANDBY_EVERY: Duration = Duration::from_millis(100);
const STANDBY_LOOKS: u32 = 150;
/// The icons of what is shown over the picture.
const OFF_AIR_GLYPH: &str = "\u{E93E}";
const FAILED_GLYPH: &str = "\u{E7BA}";

/// Bumped when a room opens or is left: answers for another are dropped.
static RUN: AtomicU64 = AtomicU64::new(0);

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.ColumnDefinitions>
    <ColumnDefinition Width="*" /><ColumnDefinition x:Name="ChatColumn" Width="320" />
  </Grid.ColumnDefinitions>
  <Grid.RowDefinitions>
    <RowDefinition x:Name="MainRow" Height="*" /><RowDefinition x:Name="ChatRow" Height="0" />
  </Grid.RowDefinitions>
  <Grid>
    <Grid.RowDefinitions>
      <RowDefinition x:Name="VideoRow" Height="*" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <Grid x:Name="Stage" Background="Black" RequestedTheme="Dark">
      <MediaPlayerElement x:Name="Video" AreTransportControlsEnabled="False" Stretch="Uniform" />
      <Canvas x:Name="Danmaku" IsHitTestVisible="False" />
      <StackPanel x:Name="Notice" Visibility="Collapsed" HorizontalAlignment="Center" VerticalAlignment="Center"
                  IsHitTestVisible="False">
        <FontIcon x:Name="NoticeIcon" FontSize="44" Foreground="#CCFFFFFF" HorizontalAlignment="Center" />
        <TextBlock x:Name="NoticeText" FontSize="15" Foreground="White" Margin="24,12,24,0" TextWrapping="Wrap"
                   TextAlignment="Center" />
      </StackPanel>
      <Button x:Name="Back" VerticalAlignment="Top" Background="Transparent" BorderThickness="0" Padding="12,8"
              Foreground="White"><FontIcon Glyph="&#xE72B;" FontSize="16" /></Button>
      <!--controls-->
    </Grid>
    <ContentControl x:Name="SendBelowVideo" Grid.Row="1" IsTabStop="False" HorizontalContentAlignment="Stretch">
      <Grid x:Name="SendBar" Padding="12,6">
        <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
        <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
        <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" /></Grid.RowDefinitions>
        <TextBox x:Name="SendText" PlaceholderText="发个友善的弹幕…" MaxLength="30" BorderThickness="0,0,0,1"
                 Background="Transparent" MinHeight="36" VerticalContentAlignment="Center" />
        <Button x:Name="SendButton" Grid.Column="1" Background="Transparent" BorderThickness="0" Margin="8,0,0,0"
                Width="36" Height="36" Padding="0">
          <FontIcon Glyph="&#xE724;" FontSize="18" /></Button>
        <TextBlock x:Name="SendNote" Grid.Row="1" Grid.ColumnSpan="2" FontSize="12" Margin="0,4,0,0" TextWrapping="Wrap"
                   Visibility="Collapsed" IsTextSelectionEnabled="True" />
      </Grid>
    </ContentControl>
    <StackPanel x:Name="Info" Grid.Row="2" Padding="16,10">
      <TextBlock x:Name="Title" FontSize="18" FontWeight="Light" TextTrimming="CharacterEllipsis" />
      <TextBlock x:Name="Meta" FontSize="12" TextWrapping="Wrap"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    </StackPanel>
  </Grid>
  <Border x:Name="ChatPane" Grid.Column="1" BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}"
          Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid>
      <Grid.RowDefinitions>
        <RowDefinition Height="Auto" /><RowDefinition Height="*" /><RowDefinition Height="Auto" />
      </Grid.RowDefinitions>
      <TextBlock Text="弹幕" FontSize="18" FontWeight="Light" Margin="14,8,14,4" />
      <ListView x:Name="Chat" Grid.Row="1" SelectionMode="None" IsItemClickEnabled="False">
        <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
        <ListView.ItemsPanel><ItemsPanelTemplate>
          <ItemsStackPanel ItemsUpdatingScrollMode="KeepLastItemInView" />
        </ItemsPanelTemplate></ListView.ItemsPanel>
        <ListView.ItemContainerStyle>
          <Style TargetType="ListViewItem">
            <Setter Property="MinHeight" Value="0" />
            <Setter Property="Padding" Value="14,3" />
            <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          </Style>
        </ListView.ItemContainerStyle>
        <ListView.ItemTemplate><DataTemplate>
          <Grid><TextBlock FontSize="13" TextWrapping="Wrap" /></Grid>
        </DataTemplate></ListView.ItemTemplate>
      </ListView>
      <Border Grid.Row="2" BorderThickness="0,1,0,0" BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
        <ContentControl x:Name="SendBelowChat" IsTabStop="False" HorizontalContentAlignment="Stretch" />
      </Border>
    </Grid>
  </Border>
</Grid>"##;

struct Room {
    root: IFrameworkElement,
    room_id: u64,
    opened: Instant,
    player: RefCell<Option<MediaPlayer>>,
    source: RefCell<Option<MediaSource>>,
    overlay: RefCell<Option<Rc<Overlay>>>,
    channel: Arc<Mutex<Option<Arc<LiveChannel>>>>,
    chat: RefCell<Vec<String>>,
    next_danmaku: Cell<u64>,
    full: Cell<bool>,
    bar: Rc<ControlBar>,
    standby: RefCell<Option<(MediaPlayer, MediaSource, Offer)>>,
    send_bar: Cell<SendPlace>,
    /// The qualities on offer.
    offer: RefCell<Offer>,
}

thread_local! {
    static ROOM: RefCell<Option<Rc<Room>>> = const { RefCell::new(None) };
}

fn room() -> Option<Rc<Room>> {
    ROOM.with_borrow(Clone::clone)
}

fn run() -> u64 {
    RUN.load(Ordering::Relaxed)
}

fn overlay_of(asked: u64) -> Option<Rc<Overlay>> {
    if asked != run() {
        return None;
    }
    room().and_then(|r| r.overlay.borrow().clone())
}

/// Leaves the room on show, if there is one.
pub fn leave() {
    RUN.fetch_add(1, Ordering::Relaxed);
    let Some(room) = ROOM.with_borrow_mut(Option::take) else { return };
    if let Some((player, source, _)) = room.standby.borrow_mut().take() {
        media::close(Some(player), Some(source));
    }
    room.bar.keep_volume();
    let _ = set_full(&room, false);
    let video = named::<IMediaPlayerElement>(&room.root, "Video");
    let _ = video.and_then(|v| v.SetMediaPlayer(None::<&MediaPlayer>));
    media::close(room.player.borrow_mut().take(), room.source.borrow_mut().take());
    let channel = room.channel.lock().unwrap_or_else(|e| e.into_inner()).take();
    if let (Some(channel), Some(backend)) = (channel, shell().and_then(|s| s.backend.get().cloned())) {
        backend.spawn(async move { channel.close().await });
    }
    // Leaving may come from an event of this very page: it goes when that is over.
    let old = room.root.clone();
    soon(&Rc::default(), move || {
        let _ = &old;
    });
}

/// Opens room `room_id`.
pub fn open(room_id: u64, title: &str, uploader: &str) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    crate::ui::player::leave();
    crate::ui::me::leave();
    leave();
    let root = load(&LAYOUT.replace("<!--controls-->", &control_bar::markup("")))?;
    set_text(&root, "Title", title)?;
    set_text(&root, "Meta", &format!("{uploader} · 正在打开…"))?;
    let shown = Rc::new(Room {
        root: root.clone(),
        room_id,
        opened: Instant::now(),
        player: RefCell::default(),
        source: RefCell::default(),
        overlay: RefCell::default(),
        channel: Arc::default(),
        chat: RefCell::default(),
        next_danmaku: Cell::new(1),
        full: Cell::new(false),
        bar: ControlBar::attach(&root, "Stage", &["Back"], run, || room().map(|r| r.bar.clone()), set_sound)?,
        standby: RefCell::default(),
        send_bar: Cell::new(SendPlace::BelowVideo),
        offer: RefCell::default(),
    });
    ROOM.with_borrow_mut(|r| *r = Some(shown.clone()));
    on_click(&root, "Back", go_back)?;
    on_click(&root, "Full", toggle_full)?;
    on_click(&root, "DanmakuSwitch", toggle_danmaku)?;
    on_click(&root, "Quality", choose_quality)?;
    on_click(&root, "SendButton", send)?;
    on_enter(&root, "SendText", send)?;
    let chat = named::<IFrameworkElement>(&root, "Chat")?;
    on_row(&chat, |at, parts| {
        let Some(room) = room() else { return Ok(()) };
        let lines = room.chat.borrow();
        let Some(line) = lines.get(at) else { return Ok(()) };
        parts.GetAt(0)?.cast::<ITextBlock>()?.SetText(line)
    })?;
    keep(root.SizeChanged(|sender: Ref<windows_core::IInspectable>, _: Ref<SizeChangedEventArgs>| {
        if let Ok(root) = sender.ok().and_then(|s| s.cast::<IFrameworkElement>()) {
            let _ = apply_layout(&root);
        }
    })?);
    shell.set_page(&root)?;
    apply_layout(&root)?;

    let opening = run();
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    set_text(&root, "Meta", uploader)?;
    backend.spawn(async move {
        let started = source_at(&core, room_id, None).await;
        post(&dispatcher, move || {
            if opening == run() {
                let _ = playing(started);
            }
        });
    });
    listen(room_id, opening);
    Ok(())
}

/// Why nothing plays.
enum Stop {
    OffAir,
    Failed(String),
}

/// Says over the picture why nothing plays.
fn notice(root: &IFrameworkElement, glyph: &str, text: &str) -> Result<()> {
    named::<IFontIcon>(root, "NoticeIcon")?.SetGlyph(glyph)?;
    set_text(root, "NoticeText", text)?;
    named::<IUIElement>(root, "Notice")?.SetVisibility(Visibility::Visible)
}

/// The room's stream at `qn`, the best on offer when `None`, and what else is on offer.
async fn source_at(core: &Backend, room_id: u64, qn: Option<u32>) -> std::result::Result<(MediaSource, Offer), Stop> {
    let (play, headers) = core.services.live.play(room_id, qn).await.map_err(|e| Stop::Failed(e.to_string()))?;
    if !play.live {
        return Err(Stop::OffAir);
    }
    let offer = Offer { options: play.qualities.clone(), current: play.qn };
    let mut made = Err(Stop::Failed("这个直播间没有系统能播放的流".to_owned()));
    for stream in play.streams.iter().filter(|s| s.protocol == LiveProtocol::Hls) {
        made = media::stream_source(&stream.url, &headers).map_err(|e| Stop::Failed(e.message()));
        if made.is_ok() {
            break;
        }
    }
    made.map(|source| (source, offer))
}

fn choose_quality() {
    if let Some(room) = room() {
        let _ = quality::open(&room.root, &room.offer.borrow(), switch_quality);
    }
}

/// Goes on at quality `qn`.
fn switch_quality(qn: u32) {
    let (Some(room), Some(shell)) = (room(), shell()) else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core, room_id, opening) = (shell.dispatcher.clone(), backend.clone(), room.room_id, run());
    backend.spawn(async move {
        let ready = source_at(&core, room_id, Some(qn)).await;
        post(&dispatcher, move || {
            if opening == run() {
                let _ = quality_ready(ready);
            }
        });
    });
}

/// The new stream plays out of sight until it is under way, then takes over.
fn quality_ready(ready: std::result::Result<(MediaSource, Offer), Stop>) -> Result<()> {
    let Some(room) = room() else { return Ok(()) };
    let (Some(shell), Ok((source, offer))) = (shell(), ready) else {
        toast("切换清晰度失败");
        return Ok(());
    };
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    if room.standby.borrow().is_some() {
        return Ok(());
    }
    let player = media::standby(&source, true)?;
    *room.standby.borrow_mut() = Some((player, source, offer));
    let (dispatcher, opening) = (shell.dispatcher.clone(), run());
    backend.spawn(async move {
        for _ in 0..STANDBY_LOOKS {
            tokio::time::sleep(STANDBY_EVERY).await;
            let (ready, wait) = oneshot();
            post(&dispatcher, move || {
                let _ = ready.send(opening == run() && standby_ready());
            });
            if wait.await.unwrap_or(true) {
                return;
            }
        }
        post(&dispatcher, move || {
            if opening == run()
                && let Some(room) = self::room()
                && let Some((player, source, _)) = room.standby.borrow_mut().take()
            {
                media::close(Some(player), Some(source));
                toast("切换清晰度失败");
            }
        });
    });
    Ok(())
}

/// Swaps the streams once the new one plays; whether the wait is over.
fn standby_ready() -> bool {
    let Some(room) = room() else { return true };
    let playing = room.standby.borrow().as_ref().is_none_or(|(player, _, _)| {
        media::session_of(player).and_then(|s| s.PlaybackState()).is_ok_and(|s| s == MediaPlaybackState::Playing)
    });
    if !playing {
        return false;
    }
    let Some((player, source, offer)) = room.standby.borrow_mut().take() else { return true };
    let swapped = (|| {
        named::<IMediaPlayerElement>(&room.root, "Video")?.SetMediaPlayer(&player)?;
        let old = room.player.borrow_mut().replace(player);
        let old_source = room.source.borrow_mut().replace(source);
        room.bar.apply_sound()?;
        media::close(old, old_source);
        quality::show(&room.root, &offer)?;
        *room.offer.borrow_mut() = offer;
        Ok::<(), windows_core::Error>(())
    })();
    if swapped.is_err() {
        toast("切换清晰度失败");
    }
    true
}

fn playing(source: std::result::Result<(MediaSource, Offer), Stop>) -> Result<()> {
    let Some(room) = room() else { return Ok(()) };
    let (source, offer) = match source {
        Ok(ready) => ready,
        Err(Stop::OffAir) => return notice(&room.root, OFF_AIR_GLYPH, "主播现在没有开播"),
        Err(Stop::Failed(problem)) => return notice(&room.root, FAILED_GLYPH, &format!("无法播放：{problem}")),
    };
    let player = media::play(&named::<IMediaPlayerElement>(&room.root, "Video")?, &source)?;
    player.cast::<IMediaPlayer>()?.Play()?;
    *room.player.borrow_mut() = Some(player);
    *room.source.borrow_mut() = Some(source);
    room.bar.apply_sound()?;
    quality::show(&room.root, &offer)?;
    *room.offer.borrow_mut() = offer;
    let opened = room.opened;
    let overlay = Overlay::attach(
        &room.root,
        Box::new(move || opened.elapsed().as_millis() as u32),
        Host { run, find: overlay_of },
    )?;
    *room.overlay.borrow_mut() = Some(overlay);
    Ok(())
}

/// Reads the room's messages until it is left.
fn listen(room_id: u64, opening: u64) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let Some(shown) = room() else { return };
    let (dispatcher, core, held) = (shell.dispatcher.clone(), backend.clone(), shown.channel.clone());
    backend.spawn(async move {
        let Ok(channel) = core.services.live.channel(room_id, LiveFilter::default()).await else {
            post(&dispatcher, move || {
                if opening == run()
                    && let Some(room) = room()
                {
                    let _ = set_text(&room.root, "Online", "弹幕连接失败");
                }
            });
            return;
        };
        let channel = Arc::new(channel);
        *held.lock().unwrap_or_else(|e| e.into_inner()) = Some(channel.clone());
        if opening != run() {
            channel.close().await;
            return;
        }
        while let Ok(Some(batch)) = channel.next().await {
            post(&dispatcher, move || {
                if opening == run() {
                    let _ = heard(batch.events);
                }
            });
        }
    });
}

fn heard(events: Vec<LiveEvent>) -> Result<()> {
    let Some(room) = room() else { return Ok(()) };
    let list = named::<IFrameworkElement>(&room.root, "Chat")?;
    let now = room.opened.elapsed().as_millis() as u32 + DANMAKU_LEAD_MS;
    let mut fresh = Vec::new();
    let from = room.chat.borrow().len();
    for event in events {
        match event {
            LiveEvent::Danmaku { uname, text, color } => {
                room.chat.borrow_mut().push(format!("{uname}：{text}"));
                let id = room.next_danmaku.replace(room.next_danmaku.get() + 1);
                fresh.push(Danmaku {
                    id,
                    progress_ms: now,
                    mode: 1,
                    font_size: 25,
                    color,
                    content: text,
                    ..Danmaku::default()
                });
            }
            LiveEvent::Online { count: viewers } => {
                set_text(&room.root, "Online", &format!("{} 人气", count(viewers)))?
            }
            LiveEvent::Status { live: false } => notice(&room.root, OFF_AIR_GLYPH, "直播已结束")?,
            _ => {}
        }
    }
    let to = room.chat.borrow().len();
    if to > CHAT_MOST {
        room.chat.borrow_mut().drain(..CHAT_MOST / 2);
        let items = list.cast::<IItemsControl>()?.Items()?.cast::<IVector<windows_core::IInspectable>>()?;
        items.Clear()?;
        add_rows(&list, 0, room.chat.borrow().len())?;
    } else {
        add_rows(&list, from, to)?;
    }
    if let Some(overlay) = room.overlay.borrow().as_ref() {
        overlay.add_live(fresh);
    }
    Ok(())
}

/// The list beside the picture on a wide window, below it otherwise.
fn apply_layout(root: &IFrameworkElement) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let full = room().is_some_and(|r| r.full.get());
    let wide = shell.class() == WindowClass::Wide;
    let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
    let pixels = |value| GridLength { value, grid_unit_type: GridUnitType::Pixel };
    let fill = GridLength { value: 1.0, grid_unit_type: GridUnitType::Star };
    let auto = GridLength { value: 0.0, grid_unit_type: GridUnitType::Auto };
    let (chat_column, main_row, chat_row, video_row) = if full {
        (pixels(0.0), fill, pixels(0.0), fill)
    } else if wide {
        (pixels(CHAT_WIDTH), fill, pixels(0.0), fill)
    } else {
        (pixels(0.0), auto, fill, pixels((root.ActualWidth()? * 9.0 / 16.0).floor()))
    };
    named::<IColumnDefinition>(root, "ChatColumn")?.SetWidth(chat_column)?;
    named::<IRowDefinition>(root, "MainRow")?.SetHeight(main_row)?;
    named::<IRowDefinition>(root, "ChatRow")?.SetHeight(chat_row)?;
    named::<IRowDefinition>(root, "VideoRow")?.SetHeight(video_row)?;
    let pane = named::<FrameworkElement>(root, "ChatPane")?;
    Grid::SetColumn(&pane, if wide { 1 } else { 0 })?;
    Grid::SetRow(&pane, if wide { 0 } else { 1 })?;
    named::<IUIElement>(root, "ChatPane")?.SetVisibility(shown(!full))?;
    named::<IUIElement>(root, "Info")?.SetVisibility(shown(!full))?;
    move_send_bar(
        root,
        match (full, wide) {
            (true, _) => SendPlace::InFull,
            (false, true) => SendPlace::BelowChat,
            (false, false) => SendPlace::BelowVideo,
        },
    )
}

/// Where the send bar sits.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SendPlace {
    BelowVideo,
    BelowChat,
    InFull,
}

impl SendPlace {
    fn host(self) -> &'static str {
        match self {
            Self::BelowVideo => "SendBelowVideo",
            Self::BelowChat => "SendBelowChat",
            Self::InFull => "SendInFull",
        }
    }
}

/// Under the list it wears the page's colours and a box of its own; on the picture, the picture's.
fn move_send_bar(root: &IFrameworkElement, place: SendPlace) -> Result<()> {
    let Some(room) = room() else { return Ok(()) };
    let from = room.send_bar.replace(place);
    let color = match place {
        SendPlace::BelowVideo => Color { a: 255, r: 0x11, g: 0x11, b: 0x11 },
        _ => Color { a: 0, r: 0, g: 0, b: 0 },
    };
    if from != place {
        control_bar::move_send_bar(root, from.host(), place.host(), color)?;
    } else {
        named::<IPanel>(root, "SendBar")?.SetBackground(&SolidColorBrush::CreateInstanceWithColor(color)?)?;
    }
    let edge = if place == SendPlace::BelowChat { 1.0 } else { 0.0 };
    let line = Thickness { left: edge, top: edge, right: edge, bottom: 1.0 };
    named::<IControl>(root, "SendText")?.SetBorderThickness(line)?;
    let theme = if place == SendPlace::BelowChat { ElementTheme::Default } else { ElementTheme::Dark };
    named::<IFrameworkElement2>(root, "SendBar")?.SetRequestedTheme(theme)
}

fn set_sound(level: f64, muted: bool) {
    let Some(room) = room() else { return };
    if let Some(player) = room.player.borrow().as_ref().and_then(|p| p.cast::<IMediaPlayer>().ok()) {
        let _ = player.SetVolume(level);
        let _ = player.SetIsMuted(muted);
    }
}

fn go_back() {
    if let Some(shell) = shell() {
        shell.back();
    }
}

fn toggle_full() {
    if let Some(room) = room() {
        let _ = set_full(&room, !room.full.get());
    }
}

fn set_full(room: &Room, on: bool) -> Result<()> {
    if room.full.replace(on) == on {
        return Ok(());
    }
    let Some(shell) = shell() else { return Ok(()) };
    shell.fill_screen(on)?;
    apply_layout(&room.root)?;
    room.bar.set_full(on)
}

fn toggle_danmaku() {
    let Some(room) = room() else { return };
    let overlay = room.overlay.borrow().clone();
    if let Some(overlay) = overlay {
        let _ = room.bar.show_danmaku_state(overlay.toggle());
    }
}

/// The window left fullscreen by itself: the page follows.
pub fn window_changed() {
    if let (Some(room), Some(shell)) = (room(), shell())
        && room.full.get()
        && !shell.is_filling_screen()
    {
        let _ = set_full(&room, false);
    }
}

/// Keys of the room; false when the key is not one of them.
pub fn key(key: VirtualKey) -> bool {
    let Some(room) = room() else { return false };
    let typing = named::<IControl>(&room.root, "SendText")
        .and_then(|c| c.FocusState())
        .is_ok_and(|f| f != FocusState::Unfocused);
    if key == VirtualKey::Escape && room.full.get() {
        let _ = set_full(&room, false);
        return true;
    }
    if typing || !shell().is_some_and(|s| s.prefs().shortcuts) {
        return false;
    }
    match key {
        VirtualKey::F => toggle_full(),
        VirtualKey::D => toggle_danmaku(),
        _ => return false,
    }
    true
}

fn send() {
    let (Some(room), Some(shell)) = (room(), shell()) else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    if !backend.services.account.is_logged_in() {
        crate::ui::shell::show_tab(crate::ui::shell::Tab::Me);
        return;
    }
    let Ok(field) = named::<ITextBox>(&room.root, "SendText") else { return };
    let text = field.Text().map(|t| t.to_string()).unwrap_or_default();
    if text.trim().is_empty() {
        return;
    }
    let _ = field.SetText("");
    let (dispatcher, core, room_id, opening) = (shell.dispatcher.clone(), backend.clone(), room.room_id, run());
    backend.spawn(async move {
        let said = match core.services.live.send(room_id, &text).await {
            Ok(()) => "已发送".to_owned(),
            Err(problem) => format!("发送失败：{problem}"),
        };
        post(&dispatcher, move || {
            if opening == run()
                && let Some(room) = self::room()
            {
                // Under the box, where the eye is after sending, and wrapped: a reason can be long.
                let _ = set_text(&room.root, "SendNote", &said);
                let _ = named::<IUIElement>(&room.root, "SendNote").and_then(|n| n.SetVisibility(Visibility::Visible));
            }
        });
    });
}
