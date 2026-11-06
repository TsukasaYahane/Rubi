//! 播放: one video through the system player. The manifest comes from `state`; the stream
//! requests carry the headers B站 asks for through the player's own HTTP client.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use rubi_core::{MAX_FREE_QN, Playback, SendDanmaku, pick_streams};
use tokio::sync::oneshot::channel as oneshot;
use windows_collections::IVector;
use windows_core::{Error, HRESULT, Interface, Ref, Result};

use crate::state::handover::{Handover, Sample, Step};
use crate::state::layout::WindowClass;
use crate::state::manifest::dash_manifest;
use crate::state::panes;
use crate::state::quality::Offer;
use crate::state::settings::quality_patch;
use crate::ui::control_bar::{self, ControlBar};
use crate::ui::danmaku::{Host, Overlay};
use crate::ui::feed;
use crate::ui::keep;
use crate::ui::media;
use crate::ui::player_actions::Actions;
use crate::ui::player_side::Side;
use crate::ui::quality;
use crate::ui::shell::{shell, toast};
use crate::ui::splitter;
use crate::ui::watch;
use crate::ui::{load, named, on_click, on_enter, post, set_text, soon};
use crate::xaml::{
    AdaptiveMediaSource, Color, DataWriter, FocusManager, FocusState, FrameworkElement, Grid, GridLength, GridUnitType,
    HorizontalAlignment, IButtonBase, IColumnDefinition, IContentControl, IControl, IDataWriter, IFontIcon,
    IFrameworkElement, IFrameworkElement4, IInputStream, IMediaPlaybackSession, IMediaPlayer, IMediaPlayer3,
    IMediaPlayerElement, IMediaPlayerFailedEventArgs, IOutputStream, IPanel, IRandomAccessStream, IRangeBase,
    IRangeBaseValueChangedEventArgs, IRowDefinition, ITextBox, IUIElement, InMemoryRandomAccessStream,
    MediaPlaybackSession, MediaPlaybackState, MediaPlayer, MediaPlayerFailedEventArgs, MediaSource,
    RangeBaseValueChangedEventArgs, SizeChangedEventArgs, TappedRoutedEventArgs, UIElement, Uri, VerticalAlignment,
    VirtualKey, Visibility,
};

/// Quality asked for; the core caps it to what the account may play.
const DASH_TYPE: &str = "application/dash+xml";
/// The player's 100 ns units.
const TICKS_PER_SECOND: i64 = 10_000_000;
/// How often the bar shows the position.
const POSITION_EVERY: Duration = Duration::from_millis(500);
/// How long the seek bar is left alone after the hand moves it.
const SEEK_HOLD: Duration = Duration::from_millis(1_200);
/// How long the hand must rest before the video goes there.
const SEEK_SETTLE: Duration = Duration::from_millis(150);
/// How often a handover between qualities is looked at.
const HANDOVER_EVERY: Duration = Duration::from_millis(50);
/// What the arrow keys move by.
const SEEK_STEP: i64 = 5 * TICKS_PER_SECOND;
/// Width of the side pane beside the 竖屏 picture.
const FEED_PANE: f64 = 340.0;
/// How long the page must hold still before the video takes its new size.
const VIDEO_SETTLE: Duration = Duration::from_millis(250);
/// How long entering or leaving fullscreen goes on changing the window's size.
const FULLSCREEN_SETTLE: Duration = Duration::from_secs(1);

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.ColumnDefinitions>
    <ColumnDefinition x:Name="MainColumn" Width="62*" />
    <ColumnDefinition Width="Auto" />
    <ColumnDefinition x:Name="SideColumn" Width="38*" />
  </Grid.ColumnDefinitions>
  <Grid.RowDefinitions>
    <RowDefinition x:Name="MainRow" Height="*" />
    <RowDefinition x:Name="SideRow" Height="0" />
  </Grid.RowDefinitions>
  <Grid x:Name="Main">
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" />
      <RowDefinition x:Name="VideoRow" Height="*" />
      <RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <Button x:Name="Back" Grid.Row="0" Background="Transparent" BorderThickness="0" Padding="12,8">
      <StackPanel Orientation="Horizontal">
        <FontIcon Glyph="&#xE72B;" FontSize="16" /><TextBlock Text="返回" Margin="10,0,0,0" />
      </StackPanel>
    </Button>
    <MediaPlayerElement x:Name="Video" Grid.Row="1" AreTransportControlsEnabled="False" Stretch="Uniform" />
    <Canvas x:Name="Danmaku" Grid.Row="1" IsHitTestVisible="False" />
    <!--controls-->
    <ContentControl x:Name="SendBelowVideo" Grid.Row="2" IsTabStop="False" HorizontalContentAlignment="Stretch">
    <Grid x:Name="SendBar" Background="#111111" Padding="12,6" RequestedTheme="Dark">
      <Grid.ColumnDefinitions>
        <ColumnDefinition Width="*" />
        <ColumnDefinition Width="Auto" />
      </Grid.ColumnDefinitions>
      <TextBox x:Name="SendText" PlaceholderText="发个友善的弹幕…" MaxLength="100" BorderThickness="0,0,0,1"
               Background="Transparent" Foreground="White" />
      <Button x:Name="SendButton" Grid.Column="1" Background="Transparent" BorderThickness="0" Margin="8,0,0,0"
              Foreground="White">
        <FontIcon Glyph="&#xE724;" FontSize="18" />
      </Button>
    </Grid>
    </ContentControl>
    <StackPanel x:Name="Info" Grid.Row="3" Margin="16,10,16,6">
      <TextBlock x:Name="Title" FontSize="20" FontWeight="Light" TextWrapping="Wrap" MaxLines="2" />
      <TextBlock x:Name="Meta" FontSize="12" Margin="0,4,0,0" TextWrapping="Wrap" MaxLines="3"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    </StackPanel>
    <ScrollViewer x:Name="ActionRow" Grid.Row="4" HorizontalScrollMode="Auto" VerticalScrollMode="Disabled"
                  HorizontalScrollBarVisibility="Hidden" VerticalScrollBarVisibility="Disabled">
      <StackPanel Orientation="Horizontal" Margin="6,0,6,8">
        <StackPanel.Resources>
          <Style TargetType="Button">
            <Setter Property="Background" Value="Transparent" />
            <Setter Property="BorderThickness" Value="0" />
            <Setter Property="Padding" Value="10,6" />
            <Setter Property="MinWidth" Value="64" />
          </Style>
        </StackPanel.Resources>
        <Button x:Name="ActLike"><StackPanel>
          <FontIcon Glyph="&#xE8E1;" FontSize="20" /><TextBlock x:Name="LikeText" Text="赞" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActCoin"><StackPanel>
          <FontIcon Glyph="&#xE8C7;" FontSize="20" /><TextBlock x:Name="CoinText" Text="币" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel>
          <Button.Flyout><MenuFlyout>
            <MenuFlyoutItem x:Name="Coin1" Text="投 1 枚" /><MenuFlyoutItem x:Name="Coin2" Text="投 2 枚" />
          </MenuFlyout></Button.Flyout></Button>
        <Button x:Name="ActFav"><StackPanel>
          <FontIcon Glyph="&#xE734;" FontSize="20" /><TextBlock x:Name="FavText" Text="藏" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActLater"><StackPanel>
          <FontIcon Glyph="&#xE823;" FontSize="20" /><TextBlock x:Name="LaterText" Text="稍后" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActDanmaku"><StackPanel>
          <FontIcon Glyph="&#xE8BD;" FontSize="20" /><TextBlock x:Name="DanmakuText" Text="弹幕 开" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActFollow"><StackPanel>
          <FontIcon Glyph="&#xE8FA;" FontSize="20" /><TextBlock x:Name="FollowText" Text="关注" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
      </StackPanel>
    </ScrollViewer>
  </Grid>
  <Thumb x:Name="SideBar" Grid.Column="1" Width="5" Background="Transparent"><!--bar--></Thumb>
  <Grid x:Name="Side" Grid.Column="2" Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" />
      <RowDefinition Height="*" />
    </Grid.RowDefinitions>
    <StackPanel Orientation="Horizontal" Margin="4,6,4,2">
      <Button x:Name="TabRelated" Content="推荐" Background="Transparent" BorderThickness="0" FontSize="20"
              FontWeight="Light" Padding="12,2" />
      <Button x:Name="TabComments" Content="评论" Background="Transparent" BorderThickness="0" FontSize="20"
              FontWeight="Light" Padding="12,2" />
    </StackPanel>
    <!--lists-->
  </Grid>
</Grid>"##;

const LISTS: &str = r##"    <GridView x:Name="Related" Grid.Row="1" SelectionMode="None" IsItemClickEnabled="True">
      <GridView.ItemContainerTransitions><TransitionCollection /></GridView.ItemContainerTransitions>
      <GridView.ItemContainerStyle>
        <Style TargetType="GridViewItem">
          <Setter Property="Margin" Value="0" />
          <Setter Property="Padding" Value="12,6" />
          <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          <Setter Property="VerticalContentAlignment" Value="Top" />
        </Style>
      </GridView.ItemContainerStyle>
      <GridView.ItemsPanel>
        <ItemsPanelTemplate><ItemsWrapGrid Orientation="Horizontal" /></ItemsPanelTemplate>
      </GridView.ItemsPanel>
    </GridView>
    <ListView x:Name="Comments" Grid.Row="1" SelectionMode="None" Visibility="Collapsed">
      <ListView.ItemContainerStyle>
        <Style TargetType="ListViewItem">
          <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          <Setter Property="Padding" Value="16,8" />
        </Style>
      </ListView.ItemContainerStyle>
      <ListView.ItemTemplate>
        <DataTemplate>
          <StackPanel>
            <TextBlock FontSize="12" Foreground="#0050EF" />
            <TextBlock FontSize="14" TextWrapping="Wrap" Margin="0,3,0,0" />
            <TextBlock FontSize="11" Margin="0,3,0,0"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          </StackPanel>
        </DataTemplate>
      </ListView.ItemTemplate>
    </ListView>
"##;

/// Bumped when a video opens or the page is left: an answer for an earlier one is dropped.
static RUN: AtomicU64 = AtomicU64::new(0);

thread_local! {
    /// The player of the page on show; on the UI thread.
    static PLAYING: RefCell<Option<MediaPlayer>> = const { RefCell::new(None) };
    static PAUSED_FOR_BLUR: Cell<bool> = const { Cell::new(false) };
    /// What `PLAYING` plays: closing it lets go of its connections.
    static SOURCE: RefCell<Option<MediaSource>> = const { RefCell::new(None) };
    /// The page on show, while there is one.
    static PAGE: RefCell<Option<Page>> = const { RefCell::new(None) };
}

/// The page on show and its parts.
#[derive(Clone)]
struct Page {
    root: IFrameworkElement,
    side: Rc<Side>,
    actions: Rc<Actions>,
    overlay: Rc<RefCell<Option<Rc<Overlay>>>>,
    /// What 发弹幕 needs: the video, and its part once known.
    bvid: String,
    cid: Rc<Cell<u64>>,
    /// The 竖屏推流 layout, not the ordinary one.
    feed: bool,
    /// The qualities on offer.
    offer: Rc<RefCell<Offer>>,
    bar: Rc<ControlBar>,
    /// A new quality waiting to take over.
    standby: Rc<RefCell<Option<Standby>>>,
    /// The send bar sits in the control bar.
    send_in_full: Rc<Cell<bool>>,
    /// Until when the seek bar is left to the hand on it.
    seek_held: Rc<Cell<Instant>>,
    /// The position is being shown, not chosen.
    showing_position: Rc<Cell<bool>>,
    seeks: Arc<AtomicU64>,
    /// Filling the screen.
    full: Rc<Cell<bool>>,
    /// The video element is held at its size while the page resizes around it.
    held: Rc<Cell<bool>>,
    /// Bumped by each size change: only the last one lets the video go.
    resizes: Arc<AtomicU64>,
    /// Until when size changes are not held: entering or leaving fullscreen is one change.
    hold_from: Rc<Cell<Instant>>,
}

fn page() -> Option<Page> {
    PAGE.with(|p| p.borrow().clone())
}

/// Which opening of the page is current; answers for another are dropped.
pub fn run() -> u64 {
    RUN.load(Ordering::Relaxed)
}

/// The side pane, if opening `run` is still on show.
pub fn side(run: u64) -> Option<Rc<Side>> {
    if run != self::run() {
        return None;
    }
    page().map(|p| p.side)
}

/// The action row, if opening `run` is still on show.
pub fn actions(run: u64) -> Option<Rc<Actions>> {
    if run != self::run() {
        return None;
    }
    page().map(|p| p.actions)
}

/// The danmaku layer, if opening `run` is still on show.
pub fn overlay(run: u64) -> Option<Rc<Overlay>> {
    if run != self::run() {
        return None;
    }
    page()?.overlay.borrow().clone()
}

/// The picture in the middle, the side pane beside it: always on a wide window, on request on a
/// smaller one.
fn apply_feed_layout(root: &IFrameworkElement, wide: bool, full: bool) -> Result<()> {
    let pane = (wide || feed::panel_open()) && !full;
    let width = GridLength { value: if pane { FEED_PANE } else { 0.0 }, grid_unit_type: GridUnitType::Pixel };
    named::<IColumnDefinition>(root, "SideColumn")?.SetWidth(width)?;
    let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
    named::<IUIElement>(root, "Side")?.SetVisibility(shown(pane))?;
    for part in ["Info", "ActionRow"] {
        named::<IUIElement>(root, part)?.SetVisibility(shown(!full))?;
    }
    place_send_bar(root, full, Color { a: 0x66, r: 0, g: 0, b: 0 })
}

/// In the control bar in fullscreen, under the picture otherwise.
fn place_send_bar(root: &IFrameworkElement, full: bool, normally: Color) -> Result<()> {
    let Some(page) = page() else { return Ok(()) };
    if page.send_in_full.replace(full) == full {
        return Ok(());
    }
    let (from, to) = if full { ("SendBelowVideo", "SendInFull") } else { ("SendInFull", "SendBelowVideo") };
    let color = if full { Color { a: 0, r: 0, g: 0, b: 0 } } else { normally };
    control_bar::move_send_bar(root, from, to, color)
}

/// Keeps the video element at its size until the page stops resizing. The system player draws on
/// a surface of its own, and resizing that many times in a row can leave it blank on screen; the
/// side pane is opaque, so the page can move freely over a video that waits.
fn hold_video() -> Result<()> {
    let (Some(page), Some(shell)) = (page(), shell()) else { return Ok(()) };
    if PLAYING.with(|p| p.borrow().is_none()) || Instant::now() < page.hold_from.get() {
        return Ok(());
    }
    if !page.held.replace(true) {
        let video = named::<IFrameworkElement>(&page.root, "Video")?;
        let (width, height) = (video.ActualWidth()?, video.ActualHeight()?);
        video.SetHorizontalAlignment(HorizontalAlignment::Left)?;
        video.SetVerticalAlignment(VerticalAlignment::Top)?;
        video.SetWidth(width)?;
        video.SetHeight(height)?;
    }
    let turn = page.resizes.fetch_add(1, Ordering::Relaxed) + 1;
    let (resizes, dispatcher, run) = (page.resizes.clone(), shell.dispatcher.clone(), self::run());
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    backend.spawn(async move {
        tokio::time::sleep(VIDEO_SETTLE).await;
        if resizes.load(Ordering::Relaxed) == turn {
            post(&dispatcher, move || {
                if run == self::run() {
                    let _ = release_video();
                }
            });
        }
    });
    Ok(())
}

/// Lets the video take its place again, on a fresh surface.
fn release_video() -> Result<()> {
    let Some(page) = page() else { return Ok(()) };
    if !page.held.replace(false) {
        return Ok(());
    }
    let video = named::<IFrameworkElement>(&page.root, "Video")?;
    video.SetWidth(f64::NAN)?;
    video.SetHeight(f64::NAN)?;
    video.SetHorizontalAlignment(HorizontalAlignment::Stretch)?;
    video.SetVerticalAlignment(VerticalAlignment::Stretch)?;
    let element = video.cast::<IMediaPlayerElement>()?;
    if let Some(player) = PLAYING.with(|p| p.borrow().clone()) {
        element.SetMediaPlayer(None::<&MediaPlayer>)?;
        element.SetMediaPlayer(&player)?;
    }
    Ok(())
}

fn drag_side(change: f64) {
    let (Some(page), Some(shell)) = (page(), shell()) else { return };
    let _ = hold_video();
    let Ok(side) = named::<IFrameworkElement>(&page.root, "Side") else { return };
    let (now, whole) = (side.ActualWidth().unwrap_or(0.0), page.root.ActualWidth().unwrap_or(0.0));
    shell.set_side_width(Some(panes::side_width(now - change, whole)));
    let _ = apply_layout(&page.root);
}

fn reset_side() {
    let (Some(page), Some(shell)) = (page(), shell()) else { return };
    shell.set_side_width(None);
    let _ = apply_layout(&page.root);
    crate::ui::shell::save_widths();
}

/// Enters or leaves fullscreen. The video element stays where it is and only the page around
/// it folds away, so the picture does not restart its swap chain and the danmaku stay over it.
pub fn toggle_full() {
    let _ = set_full(!page().is_some_and(|p| p.full.get()));
}

fn set_full(on: bool) -> Result<()> {
    let (Some(page), Some(shell)) = (page(), shell()) else { return Ok(()) };
    if page.full.replace(on) == on {
        return Ok(());
    }
    page.hold_from.set(Instant::now() + FULLSCREEN_SETTLE);
    release_video()?;
    shell.fill_screen(on)?;
    apply_layout(&page.root)?;
    page.bar.set_full(on)
}

/// The window left fullscreen by itself (the system's own control): the page follows.
pub fn window_changed() {
    if page().is_some_and(|p| p.full.get()) && !shell().is_some_and(|s| s.is_filling_screen()) {
        let _ = set_full(false);
    }
}

/// Keys of the video page; false when the key is not one of them.
pub fn key(key: VirtualKey) -> bool {
    let Some(page) = page() else { return false };
    let typing = named::<IControl>(&page.root, "SendText")
        .and_then(|c| c.FocusState())
        .is_ok_and(|f| f != FocusState::Unfocused);
    if typing {
        return false;
    }
    if key == VirtualKey::Escape && page.full.get() {
        let _ = set_full(false);
        return true;
    }
    if !shell().is_some_and(|s| s.prefs().shortcuts) {
        return false;
    }
    match key {
        VirtualKey::Space if !focus_takes(key) => toggle_play(),
        VirtualKey::Left if !focus_takes(key) => seek_by(-SEEK_STEP),
        VirtualKey::Right if !focus_takes(key) => seek_by(SEEK_STEP),
        VirtualKey::F => toggle_full(),
        VirtualKey::D => toggle_danmaku(),
        VirtualKey::Up | VirtualKey::PageUp if page.feed => feed::step(-1),
        VirtualKey::Down | VirtualKey::PageDown if page.feed => feed::step(1),
        _ => return false,
    }
    true
}

/// Whether the focused control has its own use for `key`: a button for the space bar, a slider
/// for the arrows. Clicked buttons do not keep the focus, so this is someone moving by keyboard.
fn focus_takes(key: VirtualKey) -> bool {
    let Ok(focused) = FocusManager::GetFocusedElement() else { return false };
    if key == VirtualKey::Space { focused.cast::<IButtonBase>().is_ok() } else { focused.cast::<IRangeBase>().is_ok() }
}

fn playing_session() -> Option<IMediaPlaybackSession> {
    PLAYING.with(|p| {
        let session = p.borrow().as_ref()?.cast::<IMediaPlayer3>().and_then(|m| m.PlaybackSession()).ok()?;
        session.cast().ok()
    })
}

fn is_playing() -> bool {
    playing_session().and_then(|s| s.PlaybackState().ok()) == Some(MediaPlaybackState::Playing)
}

fn pause(on: bool) {
    PLAYING.with(|p| {
        if let Some(player) = p.borrow().as_ref().and_then(|m| m.cast::<IMediaPlayer>().ok()) {
            let _ = if on { player.Pause() } else { player.Play() };
        }
    });
}

/// Whether the video is playing, and how far into it, in seconds.
pub fn progress() -> Option<(bool, u64)> {
    let position = playing_session()?.Position().ok()?;
    Some((is_playing(), (position.duration / TICKS_PER_SECOND).max(0) as u64))
}

fn toggle_play() {
    pause(is_playing());
}

fn seek_by(step: i64) {
    let Some(session) = playing_session() else { return };
    if let Ok(position) = session.Position() {
        let _ = session.SetPosition(windows_time::TimeSpan { duration: (position.duration + step).max(0) });
    }
}

/// Pauses when the window loses the focus, and picks up again when it returns, if that is what
/// paused it.
pub fn focus_changed(active: bool) {
    if !active {
        if shell().is_some_and(|s| s.prefs().pause_on_blur) && is_playing() {
            pause(true);
            PAUSED_FOR_BLUR.set(true);
        }
    } else if PAUSED_FOR_BLUR.replace(false) {
        pause(false);
    }
}

/// Shows a line under the title.
pub fn say(text: &str) {
    if let Some(page) = page() {
        let _ = set_text(&page.root, "Meta", text);
    }
}

/// Follows a new window class, if the page is on show.
pub fn resized() {
    if let Some(page) = page() {
        let _ = apply_layout(&page.root);
    }
}

/// Beside the video on a wide window; under it otherwise, the video keeping 16:9.
fn apply_layout(root: &IFrameworkElement) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let full = page().is_some_and(|p| p.full.get());
    if page().is_some_and(|p| p.feed) {
        return apply_feed_layout(root, shell.class() == WindowClass::Wide, full);
    }
    let wide = shell.class() == WindowClass::Wide || full;
    let star = |value| GridLength { value, grid_unit_type: GridUnitType::Star };
    let pixels = |value| GridLength { value, grid_unit_type: GridUnitType::Pixel };
    let auto = GridLength { value: 1.0, grid_unit_type: GridUnitType::Auto };
    let dragged = shell.side_width().map(|w| panes::side_width(w, root.ActualWidth().unwrap_or(0.0)));
    named::<IColumnDefinition>(root, "MainColumn")?.SetWidth(match (wide, dragged) {
        (true, None) => star(62.0),
        _ => star(1.0),
    })?;
    named::<IColumnDefinition>(root, "SideColumn")?.SetWidth(if full {
        pixels(0.0)
    } else if wide {
        dragged.map_or(star(38.0), pixels)
    } else {
        pixels(0.0)
    })?;
    place_send_bar(root, full, Color { a: 255, r: 0x11, g: 0x11, b: 0x11 })?;
    for part in ["Back", "Info", "ActionRow", "Side"] {
        let shown = if full { Visibility::Collapsed } else { Visibility::Visible };
        named::<IUIElement>(root, part)?.SetVisibility(shown)?;
    }
    let bar = if wide && !full { Visibility::Visible } else { Visibility::Collapsed };
    named::<IUIElement>(root, "SideBar")?.SetVisibility(bar)?;
    named::<IRowDefinition>(root, "MainRow")?.SetHeight(if wide { star(1.0) } else { auto })?;
    named::<IRowDefinition>(root, "SideRow")?.SetHeight(if wide { pixels(0.0) } else { star(1.0) })?;
    let side = named::<FrameworkElement>(root, "Side")?;
    Grid::SetColumn(&side, if wide { 2 } else { 0 })?;
    Grid::SetRow(&side, if wide { 0 } else { 1 })?;
    let video = if wide { star(1.0) } else { pixels((root.ActualWidth()? * 9.0 / 16.0).floor().max(1.0)) };
    named::<IRowDefinition>(root, "VideoRow")?.SetHeight(video)
}

/// Stops the video and lets its player go.
pub fn leave() {
    watch::stop();
    let _ = set_full(false);
    PAUSED_FOR_BLUR.set(false);
    if let Some(page) = page() {
        page.bar.keep_volume();
        if let Some(standby) = page.standby.borrow_mut().take() {
            media::close(Some(standby.player), Some(standby.source));
        }
    }
    RUN.fetch_add(1, Ordering::Relaxed);
    // The element lets go first: it must not be left holding a closed player.
    if let Some(page) = page() {
        let video = named::<IMediaPlayerElement>(&page.root, "Video");
        let _ = video.and_then(|v| v.SetMediaPlayer(None::<&MediaPlayer>));
        // Leaving may come from an event of this very page: it goes when that is over.
        let old = page.root.clone();
        soon(&Rc::default(), move || {
            let _ = &old;
        });
    }
    PAGE.with(|p| *p.borrow_mut() = None);
    media::close(PLAYING.with(|p| p.borrow_mut().take()), SOURCE.with(|s| s.borrow_mut().take()));
}

/// The page on show, for what lives beside it.
pub fn root() -> Option<IFrameworkElement> {
    page().map(|p| p.root)
}

/// Opens the ordinary video page.
pub fn show(bvid: &str, title: &str, uploader: &str) -> Result<()> {
    open(false, bvid, title, uploader)
}

/// Opens a video, in the 竖屏推流 layout when `feed`.
pub fn open(feed: bool, bvid: &str, title: &str, uploader: &str) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    leave();
    crate::ui::me::leave();
    let layout = if feed { feed::LAYOUT } else { LAYOUT };
    let place = if feed { r#"Grid.RowSpan="4""# } else { r#"Grid.Row="1""# };
    let root = load(
        &layout
            .replace("<!--bar-->", splitter::BAR_TEMPLATE)
            .replace("<!--lists-->", LISTS)
            .replace("<!--controls-->", &control_bar::markup(place)),
    )?;
    named::<IUIElement>(&root, "SeekRow")?.SetVisibility(Visibility::Visible)?;
    named::<IUIElement>(&root, "PlayPause")?.SetVisibility(Visibility::Visible)?;
    set_text(&root, "Title", title)?;
    set_text(&root, "Meta", &format!("{uploader} · 正在打开…"))?;
    on_click(&root, "Back", go_back)?;
    settle_buttons(&root)?;
    if feed {
        feed::attach(&root)?;
    }
    splitter::attach(&root, "SideBar", drag_side, crate::ui::shell::save_widths, reset_side)?;
    let pane = Side::attach(&root)?;
    if feed {
        pane.show_tab(true)?;
    }
    let actions = Actions::attach(&root, bvid)?;
    let (shown_overlay, cid) = (Rc::new(RefCell::new(None)), Rc::new(Cell::new(0)));
    let opened = Page {
        root: root.clone(),
        side: pane,
        actions,
        overlay: shown_overlay,
        bvid: bvid.to_owned(),
        cid,
        feed,
        offer: Rc::default(),
        bar: ControlBar::attach(
            &root,
            "Video",
            if feed { &["TopBar"] } else { &[] },
            run,
            || page().map(|p| p.bar),
            set_sound,
        )?,
        standby: Rc::default(),
        send_in_full: Rc::default(),
        seek_held: Rc::new(Cell::new(Instant::now())),
        showing_position: Rc::default(),
        seeks: Arc::default(),
        full: Rc::new(Cell::new(false)),
        held: Rc::default(),
        resizes: Arc::default(),
        hold_from: Rc::new(Cell::new(Instant::now())),
    };
    PAGE.with(|p| *p.borrow_mut() = Some(opened));
    on_click(&root, "ActDanmaku", toggle_danmaku)?;
    on_click(&root, "Full", toggle_full)?;
    on_click(&root, "Quality", choose_quality)?;
    on_click(&root, "PlayPause", toggle_play)?;
    on_click(&root, "DanmakuSwitch", toggle_danmaku)?;
    keep(
        named::<IUIElement>(&root, "Video")?
            .Tapped(|_: Ref<windows_core::IInspectable>, _: Ref<TappedRoutedEventArgs>| toggle_play())?,
    );
    keep(named::<IRangeBase>(&root, "Seek")?.ValueChanged(
        |_: Ref<windows_core::IInspectable>, args: Ref<RangeBaseValueChangedEventArgs>| {
            let value = args.ok().and_then(|a| a.cast::<IRangeBaseValueChangedEventArgs>()?.NewValue());
            if let Ok(value) = value {
                seek_asked(value);
            }
        },
    )?);
    on_click(&root, "SendButton", send_danmaku)?;
    on_enter(&root, "SendText", send_danmaku)?;
    let pending = Rc::new(Cell::new(false));
    let revoker = root.SizeChanged(move |sender: Ref<windows_core::IInspectable>, _: Ref<SizeChangedEventArgs>| {
        // From the sender: a handler holding its own source would keep the page for good.
        let Ok(root) = sender.ok().and_then(|s| s.cast::<IFrameworkElement>()) else { return };
        let _ = hold_video();
        soon(&pending, move || {
            let _ = apply_layout(&root);
        });
    })?;
    keep(revoker);
    shell.set_page(&root)?;
    apply_layout(&root)?;

    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    let run = RUN.load(Ordering::Relaxed);
    let dispatcher = shell.dispatcher.clone();
    let core = backend.clone();
    let (bvid, uploader) = (bvid.to_owned(), uploader.to_owned());
    let (info_dispatcher, info_core, info_bvid) = (dispatcher.clone(), core.clone(), bvid.clone());
    backend.spawn(async move {
        let services = &info_core.services;
        let (detail, related) = tokio::join!(services.videos.detail(&info_bvid), services.discover.related(&info_bvid));
        let detail = detail.ok().map(|d| d.card);
        let related = related.unwrap_or_default();
        let relation =
            if services.account.is_logged_in() { services.actions.relation(&info_bvid).await.ok() } else { None };
        post(&info_dispatcher, move || {
            let Some(page) = (run == self::run()).then(page).flatten() else { return };
            if let Some(card) = detail {
                page.side.set_aid(card.aid);
                page.actions.set_counts(card.stats.like, card.stats.coin, card.stats.favorite);
                let _ = set_text(&page.root, "Title", &card.title);
                let _ = set_text(&page.root, "Meta", &format!("{} 播放 · {}", card.stats.view, card.desc));
            }
            if let Some(relation) = relation {
                page.actions.set_relation(relation);
            }
            let _ = page.side.set_related(related);
        });
    });
    backend.spawn(async move {
        let quality = core.services.preferences.settings().playback.preferred_quality.min(MAX_FREE_QN);
        let source = match core.services.videos.playback(&bvid, quality).await {
            Ok(playback) => open_source(&playback)
                .map(|source| Opened {
                    source,
                    cid: playback.cid,
                    duration_ms: playback.url.duration_ms as u32,
                    offer: offer_of(&playback),
                })
                .map_err(|e| e.message()),
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

/// The page's buttons do not take the focus when clicked, so the space bar goes on pausing the
/// video and never repeats the last button; in the words style they show no icons.
fn settle_buttons(root: &IFrameworkElement) -> Result<()> {
    let words = shell().is_some_and(|s| s.prefs().words);
    let plain = ["Back", "Full", "SendButton", "Prev", "Next", "Normal", "TabComments", "TabRelated"];
    let acting = ["ActLike", "ActCoin", "ActFav", "ActLater", "ActDanmaku", "ActFollow", "ActComment"];
    for name in plain.into_iter().chain(acting) {
        if let Ok(button) = named::<IFrameworkElement4>(root, name) {
            button.SetAllowFocusOnInteraction(false)?;
        }
    }
    if words {
        for name in acting {
            if let Ok(button) = named::<IContentControl>(root, name) {
                let parts = button.Content()?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
                parts.GetAt(0)?.cast::<IUIElement>()?.SetVisibility(Visibility::Collapsed)?;
            }
        }
    }
    Ok(())
}

fn go_back() {
    if let Some(shell) = shell() {
        shell.back();
    }
}

/// A video ready to play.
struct Opened {
    source: MediaSource,
    cid: u64,
    duration_ms: u32,
    offer: Offer,
}

fn offer_of(playback: &Playback) -> Offer {
    Offer { options: playback.url.accept.clone(), current: playback.url.quality }
}

fn choose_quality() {
    if let Some(page) = page() {
        let _ = quality::open(&page.root, &page.offer.borrow(), switch_quality);
    }
}

/// Goes on from the same place at quality `qn`, and keeps it as the default.
fn switch_quality(qn: u32) {
    let (Some(page), Some(shell)) = (page(), shell()) else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (bvid, dispatcher, core, run) = (page.bvid.clone(), shell.dispatcher.clone(), backend.clone(), self::run());
    backend.spawn(async move {
        let _ = core.services.preferences.patch_settings(&quality_patch(qn).to_string());
        let source = match core.services.videos.playback(&bvid, qn).await {
            Ok(playback) => open_source(&playback).map(|source| (source, offer_of(&playback))).map_err(|e| e.message()),
            Err(e) => Err(e.to_string()),
        };
        post(&dispatcher, move || {
            if run == self::run() {
                let _ = quality_ready(source);
            }
        });
    });
}

/// A new quality waiting, out of sight, to take over.
struct Standby {
    player: MediaPlayer,
    source: MediaSource,
    offer: Offer,
    handover: Handover,
}

fn quality_ready(source: std::result::Result<(MediaSource, Offer), String>) -> Result<()> {
    let (Some(page), Some(shell)) = (page(), shell()) else { return Ok(()) };
    let (source, offer) = match source {
        Ok(ready) => ready,
        Err(problem) => {
            toast(&format!("切换清晰度失败：{problem}"));
            return Ok(());
        }
    };
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    if page.standby.borrow().is_some() {
        media::close_source(Some(source));
        return Ok(());
    }
    let player = media::standby(&source, false)?;
    *page.standby.borrow_mut() = Some(Standby { player, source, offer, handover: Handover::default() });
    let (dispatcher, run) = (shell.dispatcher.clone(), self::run());
    backend.spawn(async move {
        while run == self::run() {
            tokio::time::sleep(HANDOVER_EVERY).await;
            let (done, wait) = oneshot();
            post(&dispatcher, move || {
                let _ = done.send(run != self::run() || handover_step());
            });
            if wait.await.unwrap_or(true) {
                return;
            }
        }
    });
    Ok(())
}

fn milliseconds(at: windows_time::TimeSpan) -> u64 {
    (at.duration / 10_000).max(0) as u64
}

/// Moves the handover on; whether it is over.
fn handover_step() -> bool {
    let Some(page) = page() else { return true };
    let Some(old) = playing_session() else { return true };
    let mut waiting = page.standby.borrow_mut();
    let Some(standby) = waiting.as_mut() else { return true };
    let Ok(new) = media::session_of(&standby.player) else { return true };
    let whole = new.NaturalDuration().map_or(0, |d| d.duration);
    let state = new.PlaybackState().unwrap_or(MediaPlaybackState::None);
    let sample = Sample {
        opened: whole > 0,
        settled: state == MediaPlaybackState::Paused,
        new_ms: new.Position().map_or(0, milliseconds),
        old_ms: old.Position().map_or(0, milliseconds),
        old_playing: is_playing(),
    };
    match standby.handover.step(sample) {
        Step::Wait => false,
        Step::SeekTo(ms) => {
            let _ = new.SetPosition(windows_time::TimeSpan { duration: ms as i64 * 10_000 });
            false
        }
        Step::Swap => {
            let Some(standby) = waiting.take() else { return true };
            drop(waiting);
            if take_over(&page, standby, sample.old_playing).is_err() {
                toast("切换清晰度失败");
            }
            true
        }
    }
}

fn take_over(page: &Page, standby: Standby, playing: bool) -> Result<()> {
    let Standby { player, source, offer, .. } = standby;
    named::<IMediaPlayerElement>(&page.root, "Video")?.SetMediaPlayer(&player)?;
    report_failures(&player)?;
    let control = player.cast::<IMediaPlayer>()?;
    control.SetIsLoopingEnabled(page.feed)?;
    if playing {
        control.Play()?;
    }
    let old = PLAYING.with(|p| p.borrow_mut().replace(player));
    let old_source = SOURCE.with(|s| s.borrow_mut().replace(source));
    page.bar.apply_sound()?;
    media::close(old, old_source);
    quality::show(&page.root, &offer)?;
    *page.offer.borrow_mut() = offer;
    Ok(())
}

fn shown(opened: std::result::Result<Opened, String>, uploader: &str) -> Result<()> {
    let Some(page) = page() else { return Ok(()) };
    match opened {
        Ok(Opened { source, cid, duration_ms, offer }) => {
            play(&page.root, &source)?;
            quality::show(&page.root, &offer)?;
            *page.offer.borrow_mut() = offer;
            page.cid.set(cid);
            watch::start(&page.bvid, cid);
            let overlay = Overlay::attach(
                &page.root,
                // Read each time: another quality brings another session.
                Box::new(|| {
                    playing_session().and_then(|s| s.Position().ok()).map_or(0, |at| (at.duration / 10_000) as u32)
                }),
                Host { run, find: overlay },
            )?;
            if let Some(shell) = shell()
                && let Some(backend) = shell.backend.get().cloned()
            {
                overlay.load(backend, shell.dispatcher.clone(), cid, duration_ms);
            }
            *page.overlay.borrow_mut() = Some(overlay);
            Ok(())
        }
        Err(problem) => set_text(&page.root, "Meta", &format!("{uploader} · 无法播放：{problem}")),
    }
}

fn toggle_danmaku() {
    let Some(page) = page() else { return };
    if let Some(overlay) = page.overlay.borrow().as_ref() {
        let on = overlay.toggle();
        let _ = set_text(&page.root, "DanmakuText", if on { "弹幕 开" } else { "弹幕 关" });
        let _ = page.bar.show_danmaku_state(on);
    }
}

fn set_sound(level: f64, muted: bool) {
    PLAYING.with(|p| {
        if let Some(player) = p.borrow().as_ref().and_then(|m| m.cast::<IMediaPlayer>().ok()) {
            let _ = player.SetVolume(level);
            let _ = player.SetIsMuted(muted);
        }
    });
}

/// The hand moved the seek bar: follows it once it rests.
fn seek_asked(seconds: f64) {
    let (Some(page), Some(shell)) = (page(), shell()) else { return };
    if page.showing_position.get() {
        return;
    }
    page.seek_held.set(Instant::now() + SEEK_HOLD);
    let Some(backend) = shell.backend.get().cloned() else { return };
    let turn = page.seeks.fetch_add(1, Ordering::Relaxed) + 1;
    let (seeks, dispatcher, run) = (page.seeks.clone(), shell.dispatcher.clone(), self::run());
    backend.spawn(async move {
        tokio::time::sleep(SEEK_SETTLE).await;
        if seeks.load(Ordering::Relaxed) == turn {
            post(&dispatcher, move || {
                if run == self::run()
                    && let Some(session) = playing_session()
                {
                    let at = windows_time::TimeSpan { duration: (seconds * TICKS_PER_SECOND as f64) as i64 };
                    let _ = session.SetPosition(at);
                }
            });
        }
    });
}

/// Shows on the bar how far the video is, for as long as this run lasts.
fn follow_position() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, run) = (shell.dispatcher.clone(), self::run());
    backend.spawn(async move {
        while run == self::run() {
            tokio::time::sleep(POSITION_EVERY).await;
            post(&dispatcher, move || {
                if run == self::run() {
                    let _ = show_position();
                }
            });
        }
    });
}

fn show_position() -> Result<()> {
    let (Some(page), Some(session)) = (page(), playing_session()) else { return Ok(()) };
    let seconds = |t: windows_time::TimeSpan| t.duration / TICKS_PER_SECOND;
    let (at, whole) = (seconds(session.Position()?), seconds(session.NaturalDuration()?));
    named::<IFontIcon>(&page.root, "PlayIcon")?.SetGlyph(if is_playing() { "\u{E769}" } else { "\u{E768}" })?;
    set_text(&page.root, "Time", &format!("{} / {}", clock(at), clock(whole)))?;
    if Instant::now() >= page.seek_held.get() {
        let seek = named::<IRangeBase>(&page.root, "Seek")?;
        page.showing_position.set(true);
        seek.SetMaximum(whole.max(1) as f64)?;
        seek.SetValue(at as f64)?;
        page.showing_position.set(false);
    }
    Ok(())
}

fn clock(seconds: i64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn send_danmaku() {
    let _ = send_typed();
}

fn send_typed() -> Result<()> {
    let (Some(page), Some(shell)) = (page(), shell()) else { return Ok(()) };
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    if !backend.services.account.is_logged_in() {
        crate::ui::shell::show_tab(crate::ui::shell::Tab::Me);
        return Ok(());
    }
    let field = named::<ITextBox>(&page.root, "SendText")?;
    let text = field.Text()?;
    let cid = page.cid.get();
    if text.trim().is_empty() || cid == 0 {
        return Ok(());
    }
    let position = PLAYING.with(|p| {
        p.borrow().as_ref().and_then(|player| {
            let session = player.cast::<IMediaPlayer3>().and_then(|m| m.PlaybackSession()).ok()?;
            session.cast::<IMediaPlaybackSession>().and_then(|s| s.Position()).ok()
        })
    });
    let progress_ms = position.map_or(0, |t| (t.duration / 10_000) as u32);
    field.SetText("")?;
    let request = SendDanmaku { bvid: page.bvid.clone(), cid, progress_ms, text, mode: 1, color: 0xFF_FFFF };
    let (dispatcher, run, core) = (shell.dispatcher.clone(), self::run(), backend.clone());
    backend.spawn(async move {
        let sent = core.services.danmaku.send(request).await.map_err(|e| e.to_string());
        post(&dispatcher, move || match sent {
            Ok(sent) => {
                if let Some(overlay) = overlay(run) {
                    overlay.add(sent);
                }
            }
            Err(problem) => toast(&format!("发送失败：{problem}")),
        });
    });
    Ok(())
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
    keep(revoker);
    Ok(())
}

fn play(root: &IFrameworkElement, source: &MediaSource) -> Result<MediaPlaybackSession> {
    let player = media::play(&named::<IMediaPlayerElement>(root, "Video")?, source)?;
    report_failures(&player)?;
    if page().is_some_and(|p| p.feed) {
        player.cast::<IMediaPlayer>()?.SetIsLoopingEnabled(true)?;
    }
    player.cast::<IMediaPlayer>()?.Play()?;
    let session = player.cast::<IMediaPlayer3>()?.PlaybackSession()?;
    PLAYING.with(|p| *p.borrow_mut() = Some(player));
    SOURCE.with(|s| *s.borrow_mut() = Some(source.clone()));
    if let Some(page) = page() {
        page.bar.apply_sound()?;
    }
    follow_position();
    Ok(session)
}

/// The media source for `playback`. Blocks on WinRT calls: never on the UI thread.
fn open_source(playback: &Playback) -> Result<MediaSource> {
    let (video, audio) = pick_streams(&playback.url);
    let manifest = video
        .and_then(|v| dash_manifest(playback.url.duration_ms, v, audio))
        .ok_or_else(|| Error::new(HRESULT(0x8000_4005_u32 as i32), "这个视频没有可用的 DASH 信息"))?;

    let client = media::client(&playback.headers)?;

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
    media::source_of(&created)
}
