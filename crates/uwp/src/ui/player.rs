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
use crate::state::library::count;
use crate::state::manifest::dash_manifest;
use crate::state::panes;
use crate::state::quality::Offer;
use crate::state::settings::{Prefs, quality_patch};
use crate::ui::control_bar::{self, ControlBar};
use crate::ui::danmaku::{Host, Overlay};
use crate::ui::feed;
use crate::ui::filling::Filling;
use crate::ui::keep;
use crate::ui::media;
use crate::ui::player_actions::Actions;
use crate::ui::player_side::{Pane, Side};
use crate::ui::quality;
use crate::ui::queue;
use crate::ui::shell::{shell, toast};
use crate::ui::splitter;
use crate::ui::twin_video::{SECOND_ELEMENT, TwinVideo};
use crate::ui::up;
use crate::ui::watch;
use crate::ui::{load, named, on_click, on_enter, pip, post, series, set_text, soon};
use crate::xaml::{
    AdaptiveMediaSource, Color, DataWriter, FocusManager, FocusState, FrameworkElement, Grid, GridLength, GridUnitType,
    IButtonBase, IColumnDefinition, IContentControl, IControl, IDataWriter, IFontIcon, IFrameworkElement,
    IFrameworkElement4, IInputStream, IMediaPlaybackSession, IMediaPlayer, IMediaPlayer3, IMediaPlayerFailedEventArgs,
    IOutputStream, IPanel, IRandomAccessStream, IRangeBase, IRangeBaseValueChangedEventArgs, IRowDefinition,
    ITextBlock, ITextBox, IUIElement, InMemoryRandomAccessStream, MediaPlaybackSession, MediaPlaybackState,
    MediaPlayer, MediaPlayerFailedEventArgs, MediaSource, PropertyValue, RangeBaseValueChangedEventArgs,
    SizeChangedEventArgs, TappedRoutedEventArgs, Thickness, UIElement, Uri, VirtualKey, Visibility,
};
use crate::xaml::{
    CompositionStretch, HoldingRoutedEventArgs, HoldingState, IBorder, IHoldingRoutedEventArgs, IPointer,
    IPointerRoutedEventArgs, PointerDeviceType, PointerRoutedEventArgs, SolidColorBrush,
};
use rubi_core::{Season, VideoPart};

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
/// The send bar's height in the 竖屏推流, until it has been measured.
const FEED_SEND_HEIGHT: f64 = 44.0;

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
  <ScrollViewer x:Name="MainScroll" VerticalScrollMode="Auto" VerticalScrollBarVisibility="Auto"
                HorizontalScrollMode="Disabled" HorizontalScrollBarVisibility="Disabled">
  <Grid x:Name="Main">
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" />
      <RowDefinition x:Name="VideoRow" Height="360" />
      <RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <Button x:Name="Back" Grid.Row="0" Background="Transparent" BorderThickness="0" Padding="12,8">
      <StackPanel Orientation="Horizontal">
        <FontIcon Glyph="&#xE72B;" FontSize="16" /><TextBlock Text="返回" Margin="10,0,0,0" />
      </StackPanel>
    </Button>
    <Border Grid.Row="1" Background="Black" />
    <Grid x:Name="Video" Grid.Row="1" Background="Transparent" />
    <Grid Grid.Row="1" <!--second--> />
    <Canvas x:Name="Danmaku" Grid.Row="1" IsHitTestVisible="False" />
    <Button x:Name="ResumeHint" Grid.Row="1" Visibility="Collapsed" HorizontalAlignment="Left" VerticalAlignment="Bottom"
            Margin="12,0,0,76" Background="#99000000" Foreground="White" BorderThickness="0" Padding="12,6" FontSize="13" />
    <!--controls-->
    <ContentControl x:Name="SendBelowVideo" Grid.Row="2" IsTabStop="False" HorizontalContentAlignment="Stretch">
    <Grid x:Name="SendBar" Background="#111111" Padding="12,6" RequestedTheme="Dark">
      <Grid.ColumnDefinitions>
        <ColumnDefinition Width="Auto" />
        <ColumnDefinition Width="*" />
        <ColumnDefinition Width="Auto" />
      </Grid.ColumnDefinitions>
            <Button x:Name="SendStyle" Background="Transparent" BorderThickness="0" Margin="0,0,8,0" Padding="6,4"
              Foreground="White" VerticalAlignment="Center" AutomationProperties.Name="弹幕样式">
        <StackPanel Orientation="Horizontal">
          <Border x:Name="SendSwatch" Width="12" Height="12" Background="White" BorderBrush="#80FFFFFF" BorderThickness="1"
                  VerticalAlignment="Center" />
          <TextBlock x:Name="SendStyleText" Text="滚动" FontSize="12" Margin="6,0,0,0" VerticalAlignment="Center" />
        </StackPanel>
      </Button>
      <TextBox x:Name="SendText" Grid.Column="1" PlaceholderText="发个友善的弹幕…" MaxLength="100" BorderThickness="0,0,0,1"
               Background="Transparent" Foreground="White" />
      <Button x:Name="SendButton" Grid.Column="2" Background="Transparent" BorderThickness="0" Margin="8,0,0,0"
              Foreground="White">
        <FontIcon Glyph="&#xE724;" FontSize="18" />
      </Button>
    </Grid>
    </ContentControl>
    <StackPanel x:Name="Below" Grid.Row="3">
    <StackPanel x:Name="Info" Grid.Row="3" Margin="16,10,16,6">
      <TextBlock x:Name="Title" FontSize="20" FontWeight="Light" TextWrapping="Wrap" MaxLines="2" />
      <Button x:Name="UpLink" Visibility="Collapsed" Background="Transparent" BorderThickness="0" Padding="0,2"
              FontSize="13" Foreground="#0050EF" />
      <TextBlock x:Name="Meta" FontSize="12" Margin="0,4,0,0" TextWrapping="Wrap" MaxLines="3"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      <Button x:Name="MoreDetail" Visibility="Collapsed" Content="展开详情 ▾" Background="Transparent"
              BorderThickness="0" Padding="0,2" FontSize="12" Foreground="#0050EF" />
      <StackPanel x:Name="Detail" Visibility="Collapsed" Padding="10,8" Margin="0,4,0,0"
                  Background="{ThemeResource SystemControlBackgroundBaseLowBrush}">
        <TextBlock x:Name="DetailDesc" FontSize="13" TextWrapping="Wrap" IsTextSelectionEnabled="True" />
        <TextBlock x:Name="DetailIds" FontSize="12" Margin="0,6,0,0" IsTextSelectionEnabled="True"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        <TextBlock x:Name="DetailCounts" FontSize="12" Margin="0,2,0,0"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      </StackPanel>
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
        <Button x:Name="ActLike" ToolTipService.ToolTip="长按或右键：一键三连"><StackPanel>
          <FontIcon Glyph="&#xE8E1;" FontSize="20" /><TextBlock x:Name="LikeText" Text="赞" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActCoin"><StackPanel>
          <FontIcon Glyph="&#xE8C7;" FontSize="20" /><TextBlock x:Name="CoinText" Text="币" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel>
          <Button.Flyout><MenuFlyout>
            <MenuFlyoutItem x:Name="Coin1" Text="投 1 枚" /><MenuFlyoutItem x:Name="Coin2" Text="投 2 枚" />
          <MenuFlyoutSeparator /><ToggleMenuFlyoutItem x:Name="CoinLike" Text="同时点赞" IsChecked="True" />
          </MenuFlyout></Button.Flyout></Button>
        <Button x:Name="ActFav"><StackPanel>
          <FontIcon Glyph="&#xE734;" FontSize="20" /><TextBlock x:Name="FavText" Text="藏" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActShare"><StackPanel>
          <FontIcon Glyph="&#xE72D;" FontSize="20" /><TextBlock Text="分享" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActLater"><StackPanel>
          <FontIcon Glyph="&#xE823;" FontSize="20" /><TextBlock x:Name="LaterText" Text="稍后" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActDanmaku"><StackPanel>
          <FontIcon Glyph="&#xE8BD;" FontSize="20" /><TextBlock x:Name="DanmakuText" Text="弹幕 开" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActFollow"><StackPanel>
          <FontIcon Glyph="&#xE8FA;" FontSize="20" /><TextBlock x:Name="FollowText" Text="关注" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
      </StackPanel>
    </ScrollViewer>
    <StackPanel x:Name="Series" Grid.Row="5" Margin="16,2,16,8">
      <StackPanel x:Name="PartsBlock" Visibility="Collapsed" Margin="0,6,0,0">
        <Grid>
          <Grid.ColumnDefinitions>
            <ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" />
            <ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" />
          </Grid.ColumnDefinitions>
          <TextBlock Text="分 P" FontSize="13" FontWeight="SemiBold" VerticalAlignment="Center" />
          <TextBlock x:Name="PartCount" Grid.Column="1" FontSize="12" Margin="10,0,0,0" VerticalAlignment="Center"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          <Button x:Name="AutoNext" Grid.Column="3" Background="Transparent" BorderThickness="0" Padding="8,2" FontSize="12" />
          <Button x:Name="PartsExpand" Grid.Column="4" Background="Transparent" BorderThickness="0" Padding="8,2" FontSize="12"
                  Foreground="#0050EF" />
        </Grid>
        <ScrollViewer x:Name="PartsScroll" Margin="0,6,0,0" VerticalScrollMode="Disabled" VerticalScrollBarVisibility="Disabled"
                      HorizontalScrollMode="Auto" HorizontalScrollBarVisibility="Auto" />
        <ContentControl x:Name="PartsAll" Margin="0,6,0,0" Visibility="Collapsed" IsTabStop="False"
                        HorizontalContentAlignment="Stretch" />
      </StackPanel>
      <StackPanel x:Name="SeasonBlock" Visibility="Collapsed" Margin="0,10,0,0">
        <Grid>
          <Grid.ColumnDefinitions>
            <ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" />
            <ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" />
          </Grid.ColumnDefinitions>
          <TextBlock x:Name="SeasonTitle" FontSize="13" FontWeight="SemiBold" VerticalAlignment="Center"
                     TextTrimming="CharacterEllipsis" MaxWidth="360" />
          <TextBlock x:Name="SeasonCount" Grid.Column="1" FontSize="12" Margin="10,0,0,0" VerticalAlignment="Center"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          <Button x:Name="SeasonSub" Grid.Column="3" Visibility="Collapsed" Background="Transparent" BorderBrush="#0050EF"
                  BorderThickness="1" Padding="10,2" FontSize="12" Foreground="#0050EF" Margin="0,0,6,0" />
          <Button x:Name="SeasonExpand" Grid.Column="4" Background="Transparent" BorderThickness="0" Padding="8,2"
                  FontSize="12" Foreground="#0050EF" />
        </Grid>
        <Button x:Name="SeasonNext" Background="Transparent" BorderThickness="0" Padding="0,4" FontSize="12"
                Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        <ScrollViewer x:Name="SeasonList" Visibility="Collapsed" MaxHeight="220" HorizontalScrollMode="Disabled"
                      VerticalScrollMode="Auto" VerticalScrollBarVisibility="Auto" />
      </StackPanel>
    </StackPanel>
    </StackPanel>
  </Grid>
  </ScrollViewer>
  <Thumb x:Name="SideBar" Grid.Column="1" <!--bar-attrs-->><!--bar--></Thumb>
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
      <Button x:Name="TabQueue" Visibility="Collapsed" Background="Transparent" BorderThickness="0" FontSize="20"
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
    <Border x:Name="CommentsHost" Grid.Row="1" Visibility="Collapsed" />
    <ListView x:Name="Queue" Grid.Row="1" SelectionMode="None" IsItemClickEnabled="True" Visibility="Collapsed">
      <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
      <ListView.ItemContainerStyle>
        <Style TargetType="ListViewItem">
          <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          <Setter Property="Padding" Value="16,8" />
        </Style>
      </ListView.ItemContainerStyle>
      <ListView.ItemTemplate>
        <DataTemplate>
          <StackPanel>
            <TextBlock FontSize="14" TextTrimming="CharacterEllipsis" />
            <TextBlock FontSize="11" Margin="0,2,0,0" TextTrimming="CharacterEllipsis"
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
    /// The episode the next opening plays, if it is one.
    static NEXT_EPISODE: Cell<Option<(u64, u64)>> = const { Cell::new(None) };
    /// The comment the next video opens at, from a notice.
    static NEXT_FOCUS: Cell<Option<rubi_model::CommentRef>> = const { Cell::new(None) };
    /// What `PLAYING` plays: closing it lets go of its connections.
    static SOURCE: RefCell<Option<MediaSource>> = const { RefCell::new(None) };
    /// Where the next opening of a video starts, in 100 ns ticks: back from the small window.
    static NEXT_START: RefCell<Option<(String, i64)>> = const { RefCell::new(None) };
    /// The speed chosen, kept from video to video this run.
    static SPEED: Cell<f64> = const { Cell::new(1.0) };
    /// How the picture fits, kept this run.
    static FIT: Cell<CompositionStretch> = const { Cell::new(CompositionStretch::Uniform) };
    /// A held press plays faster while it lasts; the click that ends it does not pause.
    static HOLD_TURN: Cell<u64> = const { Cell::new(0) };
    static HELD: Cell<bool> = const { Cell::new(false) };
    static SKIP_TAP: Cell<bool> = const { Cell::new(false) };
    /// How a danmaku sent goes: mode (1 scroll, 5 top, 4 bottom) and colour.
    static SEND_MODE: Cell<u8> = const { Cell::new(1) };
    static SEND_COLOR: Cell<u32> = const { Cell::new(0xFF_FFFF) };
    /// Where the video was left, as (part, ms), while the hint offers it.
    static RESUME: Cell<Option<(u64, u64)>> = const { Cell::new(None) };
    /// 播完自动连播下一个, this run.
    static AUTO_NEXT: Cell<bool> = const { Cell::new(true) };
    /// How far each part was watched this run (ms), by cid.
    static PART_PROGRESS: RefCell<std::collections::HashMap<u64, u64>> = RefCell::default();
    static PARTS_OPEN: Cell<bool> = const { Cell::new(false) };
    static SEASON_OPEN: Cell<bool> = const { Cell::new(false) };
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
    /// What reopens it from the small window.
    title: String,
    uploader: String,
    /// Its parts and its 合集, once known; whether the 合集 is subscribed, once asked.
    parts: Rc<RefCell<Vec<VideoPart>>>,
    /// The video's card, once its details have come.
    card: Rc<RefCell<Option<rubi_model::VideoCard>>>,
    season: Rc<RefCell<Option<Season>>>,
    subscribed: Rc<Cell<Option<bool>>>,
    cid: Rc<Cell<u64>>,
    /// The 竖屏推流 layout, not the ordinary one.
    feed: bool,
    /// The 影视 episode it plays, as (ep id, cid); its streams come by the episode, not the video.
    episode: Option<(u64, u64)>,
    /// The UP, once known.
    up: Rc<RefCell<(u64, String)>>,
    /// The qualities on offer.
    offer: Rc<RefCell<Offer>>,
    bar: Rc<ControlBar>,
    video: Rc<TwinVideo>,
    /// A new quality waiting to take over.
    standby: Rc<RefCell<Option<Standby>>>,
    /// The send bar sits in the control bar.
    send_in_full: Rc<Cell<bool>>,
    /// Until when the seek bar is left to the hand on it.
    seek_held: Rc<Cell<Instant>>,
    /// The position is being shown, not chosen.
    showing_position: Rc<Cell<bool>>,
    seeks: Arc<AtomicU64>,
    filling: Rc<Filling>,
}

fn page() -> Option<Page> {
    PAGE.with(|p| p.borrow().clone())
}

/// Which opening of the page is current; answers for another are dropped.
pub fn run() -> u64 {
    RUN.load(Ordering::Relaxed)
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
    place_send_bar(root, full, Color { a: 0x66, r: 0, g: 0, b: 0 })?;
    // The send bar keeps the very bottom; the control bar sits on it, or at the bottom in
    // fullscreen, where the send bar moves into it.
    let lift =
        if full { 0.0 } else { named::<IFrameworkElement>(root, "SendBar")?.ActualHeight()?.max(FEED_SEND_HEIGHT) };
    named::<IFrameworkElement>(root, "ControlBar")?.SetMargin(Thickness {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: lift,
    })
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

fn drag_side(change: f64) {
    let (Some(page), Some(shell)) = (page(), shell()) else { return };
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
    let _ = page().map(|p| p.filling.toggle(false).and_then(|changed| filled(&p, changed)));
}

/// 窗口全屏: the video fills the window, the screen left as it is.
fn toggle_window_full() {
    let _ = page().map(|p| p.filling.toggle(true).and_then(|changed| filled(&p, changed)));
}

fn leave_full() -> Result<()> {
    page().map_or(Ok(()), |p| p.filling.leave().and_then(|changed| filled(&p, changed)))
}

/// The page follows a change of [`Filling`].
fn filled(page: &Page, changed: bool) -> Result<()> {
    if !changed {
        return Ok(());
    }
    named::<IFontIcon>(&page.root, "WindowFullIcon")?.SetGlyph(page.filling.window_glyph())?;
    apply_layout(&page.root)?;
    page.bar.set_full(page.filling.screen())?;
    page.bar.wake()
}

/// The window left fullscreen by itself (the system's own control): the page follows.
pub fn window_changed() {
    let _ = page().map(|p| p.filling.follow_system().and_then(|changed| filled(&p, changed)));
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
    if key == VirtualKey::Escape && page.filling.on() {
        let _ = leave_full();
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
    let full = page().is_some_and(|p| p.filling.on());
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
    for part in ["Back", "Below", "Side"] {
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
    fit_video(root)
}

/// The picture's height, fixed whatever opens under it: in fullscreen the whole column; on a wide
/// window what the column leaves beside the back button, the send bar, the title and the actions;
/// on a narrow one 16:9 of its width. The page scrolls to whatever goes further.
fn fit_video(root: &IFrameworkElement) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let full = page().is_some_and(|p| p.filling.on());
    let wide = shell.class() == WindowClass::Wide || full;
    let height = |name: &str| named::<IFrameworkElement>(root, name).and_then(|e| e.ActualHeight()).unwrap_or(0.0);
    let scroll = named::<crate::xaml::IScrollViewer>(root, "MainScroll")?;
    let viewport = scroll.cast::<IFrameworkElement>()?.ActualHeight()?;
    let video = if full {
        viewport
    } else if wide {
        let detail_open =
            named::<IUIElement>(root, "Detail").and_then(|d| d.Visibility()).is_ok_and(|v| v == Visibility::Visible);
        // Title and actions as they are with nothing opened under the title.
        let detail = if detail_open { height("Detail") + 4.0 } else { 0.0 };
        let under = height("Back") + height("SendBelowVideo") + height("Info") - detail + height("ActionRow");
        (viewport - under).max(200.0)
    } else {
        (root.ActualWidth()? * 9.0 / 16.0).floor()
    };
    let row = named::<IRowDefinition>(root, "VideoRow")?;
    if (row.ActualHeight().unwrap_or(0.0) - video).abs() > 0.5 {
        row.SetHeight(GridLength { value: video.max(1.0), grid_unit_type: GridUnitType::Pixel })?;
    }
    let mode = if full { crate::xaml::ScrollMode::Disabled } else { crate::xaml::ScrollMode::Auto };
    scroll.SetVerticalScrollMode(mode)?;
    if full {
        scroll.cast::<crate::xaml::IScrollViewer2>()?.ChangeViewWithOptionalAnimation(None, Some(0.0), None, true)?;
    }
    Ok(())
}

/// Draws the parts and the 合集.
fn show_series(page: &Page) {
    let parts = page.parts.borrow();
    let progress = PART_PROGRESS.with_borrow(Clone::clone);
    let _ = series::show_parts(&page.root, &parts, page.cid.get(), &progress, PARTS_OPEN.get(), AUTO_NEXT.get());
    let season = page.season.borrow();
    let _ = series::show_season(&page.root, season.as_ref(), &page.bvid, SEASON_OPEN.get(), page.subscribed.get());
}

/// Whether the 合集 is subscribed, when signed in.
fn ask_subscribed(page: &Page) {
    let Some(id) = page.season.borrow().as_ref().map(|s| s.id) else { return };
    let (Some(shell), Some(backend)) = (shell(), shell().and_then(|s| s.backend.get().cloned())) else { return };
    if !backend.services.account.is_logged_in() {
        return;
    }
    let (dispatcher, core, run) = (shell.dispatcher.clone(), backend.clone(), self::run());
    backend.spawn(async move {
        let on = core.services.seasons.is_subscribed(id).await.ok();
        post(&dispatcher, move || {
            if let Some(page) = (run == self::run()).then(self::page).flatten() {
                page.subscribed.set(on);
                show_series(&page);
            }
        });
    });
}

fn toggle_subscribe() {
    let Some(page) = page() else { return };
    let (Some(id), Some(was)) = (page.season.borrow().as_ref().map(|s| s.id), page.subscribed.get()) else { return };
    let (Some(shell), Some(backend)) = (shell(), shell().and_then(|s| s.backend.get().cloned())) else { return };
    page.subscribed.set(Some(!was));
    show_series(&page);
    let (dispatcher, core, run) = (shell.dispatcher.clone(), backend.clone(), self::run());
    backend.spawn(async move {
        let done = core.services.seasons.subscribe(id, !was).await;
        post(&dispatcher, move || {
            let Some(page) = (run == self::run()).then(self::page).flatten() else { return };
            match done {
                Ok(()) => toast(if was { "已取消订阅" } else { "已订阅合集" }),
                Err(problem) => {
                    page.subscribed.set(Some(was));
                    show_series(&page);
                    toast(&format!("订阅失败：{problem}"));
                }
            }
        });
    });
}

/// Opens the 合集's video after this one; false when there is none.
fn season_step() -> bool {
    let Some(page) = page() else { return false };
    let next = page.season.borrow().as_ref().and_then(|season| {
        let all = series::episodes(season);
        let at = all.iter().position(|e| e.bvid == page.bvid)?;
        all.get(at + 1).map(|e| (e.bvid.clone(), e.title.clone()))
    });
    match next {
        Some((bvid, title)) => {
            let _ = open(false, &bvid, &title, &page.uploader);
            true
        }
        None => false,
    }
}

/// A video of this 合集, chosen from its list.
pub fn open_episode_of_season(bvid: &str, title: &str) {
    let uploader = page().map(|p| p.uploader.clone()).unwrap_or_default();
    let _ = open(false, bvid, title, &uploader);
}

/// 播完自动连播下一个: the next part, else the 合集's next video.
fn auto_next() {
    if !AUTO_NEXT.get() {
        return;
    }
    let Some(page) = page() else { return };
    let next_part = {
        let parts = page.parts.borrow();
        parts.iter().position(|p| p.cid == page.cid.get()).and_then(|at| parts.get(at + 1)).map(|p| p.cid)
    };
    match next_part {
        Some(cid) => play_part(cid, Some(0)),
        None => {
            season_step();
        }
    }
}

/// Plays part `cid` of the video on show, from `start_ms` or, when `None`, from where it was left
/// this run.
pub fn play_part(cid: u64, start_ms: Option<u64>) {
    let Some(page) = page() else { return };
    let (Some(shell), Some(backend)) = (shell(), shell().and_then(|s| s.backend.get().cloned())) else { return };
    if cid == page.cid.get() && start_ms.is_none() {
        return;
    }
    hide_resume();
    // Where this part got to, for coming back to it.
    if let Some((_, at_s)) = progress() {
        PART_PROGRESS.with_borrow_mut(|m| m.insert(page.cid.get(), at_s * 1000));
    }
    let start = start_ms.or_else(|| PART_PROGRESS.with_borrow(|m| m.get(&cid).copied())).unwrap_or(0);
    // The page lets go of this part: picture, player, danmaku.
    if let Some(standby) = page.standby.borrow_mut().take() {
        let _ = page.video.clear_next();
        media::close(Some(standby.player), Some(standby.source));
    }
    page.video.clear();
    media::close(PLAYING.with(|p| p.borrow_mut().take()), SOURCE.with(|s| s.borrow_mut().take()));
    page.overlay.borrow_mut().take();
    if let Ok(canvas) = named::<IPanel>(&page.root, "Danmaku") {
        let _ = canvas.Children().and_then(|c| c.Clear());
    }
    let _ = page.bar.set_marks(Vec::new());
    page.cid.set(cid);
    show_series(&page);
    if start > 0 {
        NEXT_START.with_borrow_mut(|n| *n = Some((page.bvid.clone(), start as i64 * 10_000)));
    }
    let (dispatcher, core, run, bvid, uploader) =
        (shell.dispatcher.clone(), backend.clone(), self::run(), page.bvid.clone(), page.uploader.clone());
    backend.spawn(async move {
        let quality = core.services.preferences.settings().playback.preferred_quality.min(MAX_FREE_QN);
        let source = match core.services.videos.playback_part(&bvid, cid, quality).await {
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
            if run == self::run() {
                let _ = shown(source, &uploader);
            }
        });
    });
}

/// 分享: links, the share card, or the system's sheet.
fn choose_share() {
    let Some(page) = page() else { return };
    let Ok(button) = named::<FrameworkElement>(&page.root, "ActShare") else { return };
    let card = page.card.borrow().clone();
    let title = card.as_ref().map_or_else(|| page.title.clone(), |c| c.title.clone());
    let url = format!("https://www.bilibili.com/video/{}", page.bvid);
    let titled = format!("【{title}】 {url}");
    let (link, shared) = (url.clone(), crate::ui::sharing::Shared::Link { title: title.clone(), url });
    let mut choices: Vec<crate::ui::card_menu::Choice> = vec![
        ("复制链接".to_owned(), Rc::new(move || crate::ui::card_menu::copy_text(&link, "已复制链接"))),
        ("复制标题和链接".to_owned(), Rc::new(move || crate::ui::card_menu::copy_text(&titled, "已复制标题和链接"))),
    ];
    if let Some(card) = card {
        let made = crate::ui::share_card::Card {
            bvid: card.bvid.clone(),
            title: card.title.clone(),
            uploader: card.owner.name.clone(),
            plays: count(card.stats.view),
            cover: card.cover.clone(),
        };
        let made = RefCell::new(Some(made));
        choices.push((
            "分享卡片…".to_owned(),
            Rc::new(move || {
                if let Some(made) = made.borrow_mut().take() {
                    let _ = crate::ui::share_card::open(made);
                }
            }),
        ));
    }
    let (bvid, named_as) = (page.bvid.clone(), title.clone());
    choices.push((
        "发给好友…".to_owned(),
        Rc::new(move || {
            let _ = crate::ui::share_friend::open(&bvid, &named_as);
        }),
    ));
    choices.push(("系统分享…".to_owned(), Rc::new(move || crate::ui::sharing::share(shared.clone()))));
    let _ = crate::ui::card_menu::show_under(&button, choices);
}

/// The line under the title, and what 展开详情 shows.
fn show_detail(root: &IFrameworkElement, card: &rubi_model::VideoCard) -> Result<()> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    let age = crate::state::library::age(card.pubdate, now);
    let line = if age.is_empty() {
        format!("{} 播放", count(card.stats.view))
    } else {
        format!("{} 播放 · {age}", count(card.stats.view))
    };
    set_text(root, "Meta", &line)?;
    set_text(root, "DetailDesc", if card.desc.trim().is_empty() { "（没有简介）" } else { card.desc.trim() })?;
    set_text(root, "DetailIds", &format!("{} · av{}", card.bvid, card.aid))?;
    set_text(root, "DetailCounts", &format!("评论 {} · 弹幕 {}", count(card.stats.reply), count(card.stats.danmaku)))?;
    named::<IUIElement>(root, "MoreDetail")?.SetVisibility(Visibility::Visible)
}

fn toggle_detail() {
    let Some(page) = page() else { return };
    let Ok(detail) = named::<IUIElement>(&page.root, "Detail") else { return };
    let open = detail.Visibility().is_ok_and(|v| v == Visibility::Collapsed);
    let _ = detail.SetVisibility(if open { Visibility::Visible } else { Visibility::Collapsed });
    if let Ok(button) = named::<IContentControl>(&page.root, "MoreDetail") {
        let label = if open { "收起详情 ▴" } else { "展开详情 ▾" };
        let _ = PropertyValue::CreateString(label).and_then(|l| button.SetContent(&l));
    }
}

/// How long the 上次看到 hint stays, in seconds.
const RESUME_FOR: u32 = 5;

/// Offers to pick up where the video was left: `at_ms` into part `cid`.
fn offer_resume(cid: u64, at_ms: u64) {
    let Some(page) = page() else { return };
    RESUME.with(|r| r.set(Some((cid, at_ms))));
    let _ = named::<IUIElement>(&page.root, "ResumeHint").and_then(|h| h.SetVisibility(Visibility::Visible));
    count_resume(RESUME_FOR, self::run());
}

fn count_resume(left: u32, run: u64) {
    let Some(page) = page().filter(|_| run == self::run()) else { return };
    let Some((_, at_ms)) = RESUME.with(Cell::get) else { return };
    if left == 0 {
        hide_resume();
        return;
    }
    let text = format!("上次看到 {} · 点此继续　{left} 秒后隐藏", clock((at_ms / 1000) as i64));
    if let Ok(button) = named::<IContentControl>(&page.root, "ResumeHint") {
        let _ = PropertyValue::CreateString(&text).and_then(|t| button.SetContent(&t));
    }
    let (Some(shell), Some(backend)) = (shell(), shell().and_then(|s| s.backend.get().cloned())) else { return };
    let dispatcher = shell.dispatcher.clone();
    backend.spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        post(&dispatcher, move || count_resume(left - 1, run));
    });
}

fn hide_resume() {
    RESUME.with(|r| r.set(None));
    if let Some(page) = page() {
        let _ = named::<IUIElement>(&page.root, "ResumeHint").and_then(|h| h.SetVisibility(Visibility::Collapsed));
    }
}

/// 点此继续: goes to where the video was left, in its part.
fn resume() {
    let Some((cid, at_ms)) = RESUME.with(Cell::get) else { return };
    hide_resume();
    let Some(page) = page() else { return };
    if cid == page.cid.get() || cid == 0 {
        if let Some(session) = playing_session() {
            let _ = session.SetPosition(windows_time::TimeSpan { duration: at_ms as i64 * 10_000 });
        }
    } else {
        play_part(cid, Some(at_ms));
    }
}

/// The speed a player plays at: the chosen one, or faster while a press is held.
fn rate() -> f64 {
    let chosen = SPEED.get();
    if HELD.get() { if chosen >= 2.0 { 3.0 } else { 2.0 } } else { chosen }
}

fn apply_speed(player: &MediaPlayer) {
    let _ = media::session_of(player).and_then(|s| s.SetPlaybackRate(rate()));
}

fn apply_speed_now() {
    PLAYING.with(|p| {
        if let Some(player) = p.borrow().as_ref() {
            apply_speed(player);
        }
    });
}

fn speed_label(rate: f64) -> String {
    if rate.fract() == 0.0 { format!("{rate:.1}×") } else { format!("{rate}×") }
}

fn set_speed_label(root: &IFrameworkElement) {
    if let Ok(button) = named::<IContentControl>(root, "Speed") {
        let _ = PropertyValue::CreateString(&speed_label(SPEED.get())).and_then(|l| button.SetContent(&l));
    }
}

/// 倍速: a menu under the button.
fn choose_speed() {
    let Some(page) = page() else { return };
    let Ok(button) = named::<FrameworkElement>(&page.root, "Speed") else { return };
    let choices: Vec<crate::ui::card_menu::Choice> = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0]
        .into_iter()
        .map(|r| {
            let mark = if r == SPEED.get() { "● " } else { "" };
            let pick: Rc<dyn Fn()> = Rc::new(move || {
                SPEED.set(r);
                apply_speed_now();
                if let Some(page) = self::page() {
                    set_speed_label(&page.root);
                }
            });
            (format!("{mark}{}", speed_label(r)), pick)
        })
        .collect();
    let _ = crate::ui::card_menu::show_under(&button, choices);
}

/// Plays faster while a press on the picture is held: the mouse after a moment, a finger or pen
/// held.
fn hold_for_speed(video: &IUIElement) -> Result<()> {
    keep(video.PointerPressed(|sender: Ref<windows_core::IInspectable>, args: Ref<PointerRoutedEventArgs>| {
        let (Ok(sender), Ok(args)) = (sender.ok(), args.ok()) else { return };
        let _ = start_hold(sender, args);
    })?);
    for event in [0, 1] {
        let ended = move |sender: Ref<windows_core::IInspectable>, args: Ref<PointerRoutedEventArgs>| {
            let (Ok(sender), Ok(args)) = (sender.ok(), args.ok()) else { return };
            end_hold();
            let pointer = args.cast::<IPointerRoutedEventArgs>().and_then(|a| a.Pointer());
            if let (Ok(element), Ok(pointer)) = (sender.cast::<IUIElement>(), pointer) {
                let _ = element.ReleasePointerCapture(&pointer);
            }
        };
        keep(if event == 0 { video.PointerReleased(ended)? } else { video.PointerCaptureLost(ended)? });
    }
    keep(video.Holding(|_: Ref<windows_core::IInspectable>, args: Ref<HoldingRoutedEventArgs>| {
        let state = args.ok().and_then(|a| a.cast::<IHoldingRoutedEventArgs>()?.HoldingState());
        match state {
            Ok(HoldingState::Started) => held(),
            Ok(_) => end_hold(),
            Err(_) => {}
        }
    })?);
    Ok(())
}

/// How long a mouse press is held before it plays faster.
const HOLD_AFTER: std::time::Duration = std::time::Duration::from_millis(400);

fn start_hold(sender: &windows_core::IInspectable, args: &PointerRoutedEventArgs) -> Result<()> {
    SKIP_TAP.set(false);
    let pointer = args.cast::<IPointerRoutedEventArgs>()?.Pointer()?;
    if pointer.cast::<IPointer>()?.PointerDeviceType()? != PointerDeviceType::Mouse
        || !args.GetCurrentPoint(None::<&UIElement>)?.Properties()?.IsLeftButtonPressed()?
    {
        return Ok(());
    }
    sender.cast::<IUIElement>()?.CapturePointer(&pointer)?;
    let turn = HOLD_TURN.get() + 1;
    HOLD_TURN.set(turn);
    let Some(shell) = shell() else { return Ok(()) };
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    let dispatcher = shell.dispatcher.clone();
    backend.spawn(async move {
        tokio::time::sleep(HOLD_AFTER).await;
        post(&dispatcher, move || {
            if HOLD_TURN.get() == turn {
                held();
            }
        });
    });
    Ok(())
}

fn held() {
    if HELD.replace(true) {
        return;
    }
    apply_speed_now();
    toast(&format!("▶▶ {} 倍速中", speed_label(rate())));
}

fn end_hold() {
    HOLD_TURN.set(HOLD_TURN.get() + 1);
    if HELD.replace(false) {
        SKIP_TAP.set(true);
        apply_speed_now();
        toast(&format!("恢复 {}", speed_label(SPEED.get())));
    }
}

/// 播放设置: the picture's fit and how danmaku show.
fn open_settings() {
    let Some(page) = page() else { return };
    let Ok(button) = named::<FrameworkElement>(&page.root, "PlaySettings") else { return };
    let look = page
        .overlay
        .borrow()
        .as_ref()
        .map(|o| o.look())
        .unwrap_or_else(|| shell().map(|s| s.prefs().danmaku).unwrap_or_default());
    let _ = crate::ui::play_settings::show(&button, Some(FIT.get()), look, None, |change| {
        let Some(page) = self::page() else { return };
        match change {
            crate::ui::play_settings::Change::Fit(fit) => {
                FIT.set(fit);
                page.video.set_stretch(fit);
            }
            crate::ui::play_settings::Change::Look(look) => {
                if let Some(overlay) = page.overlay.borrow().as_ref() {
                    let _ = overlay.set_look(look);
                }
                if let Some(shell) = shell() {
                    let _ = shell.set_prefs(Prefs { danmaku: look, ..shell.prefs() });
                }
            }
        }
    });
}

/// The 样式 button: the colour and where a danmaku sent goes.
fn show_send_style(root: &IFrameworkElement) {
    let color = SEND_COLOR.get();
    let color = Color { a: 255, r: (color >> 16) as u8, g: (color >> 8) as u8, b: color as u8 };
    if let Ok(swatch) = named::<IBorder>(root, "SendSwatch") {
        let _ = SolidColorBrush::CreateInstanceWithColor(color).and_then(|b| swatch.SetBackground(&b));
    }
    let label = crate::ui::play_settings::SEND_MODES.iter().find(|m| m.1 == SEND_MODE.get()).map_or("滚动", |m| m.0);
    let _ = set_text(root, "SendStyleText", label);
}

fn choose_send_style() {
    let Some(page) = page() else { return };
    let Ok(button) = named::<FrameworkElement>(&page.root, "SendStyle") else { return };
    let _ = crate::ui::play_settings::show_send_style(&button, SEND_MODE.get(), SEND_COLOR.get(), |mode, color| {
        SEND_MODE.set(mode);
        SEND_COLOR.set(color);
        if let Some(page) = self::page() {
            show_send_style(&page.root);
        }
    });
}

/// Moves the picture into the small window, or brings it back.
fn toggle_pip() {
    let Some(page) = page() else { return };
    if pip::holds_video(&page.bvid) {
        pip::back();
        return;
    }
    if PLAYING.with(|p| p.borrow().is_none()) {
        return;
    }
    if let Some(standby) = page.standby.borrow_mut().take() {
        let _ = page.video.clear_next();
        media::close(Some(standby.player), Some(standby.source));
    }
    // The page's elements let go before the small window takes the player.
    page.video.clear();
    let (Some(player), Some(source)) =
        (PLAYING.with(|p| p.borrow_mut().take()), SOURCE.with(|s| s.borrow_mut().take()))
    else {
        return;
    };
    let _ = leave_full();
    toast("在小窗播放");
    let video = pip::Video { bvid: page.bvid.clone(), title: page.title.clone(), uploader: page.uploader.clone() };
    if let Err(problem) = pip::open(pip::Owner::Video(video), player, source) {
        toast(&format!("小窗打不开：{}", problem.message()));
    }
}

/// Takes the picture back from the small window, if video `bvid` is on show and has none.
pub fn take_back(bvid: &str, player: &MediaPlayer, source: &MediaSource) -> bool {
    let Some(page) = page().filter(|p| p.bvid == bvid) else { return false };
    if PLAYING.with(|p| p.borrow().is_some()) || page.video.show(player).is_err() {
        return false;
    }
    PLAYING.with(|p| *p.borrow_mut() = Some(player.clone()));
    SOURCE.with(|s| *s.borrow_mut() = Some(source.clone()));
    let _ = page.bar.apply_sound();
    apply_speed(player);
    true
}

/// Opens video `bvid` again, starting `at` (100 ns ticks) in: back from the small window.
pub fn open_at(bvid: &str, title: &str, uploader: &str, at: i64) -> Result<()> {
    NEXT_START.with_borrow_mut(|n| *n = Some((bvid.to_owned(), at)));
    open(false, bvid, title, uploader)
}

/// Stops the video and lets its player go.
pub fn leave() {
    watch::stop();
    let _ = leave_full();
    PAUSED_FOR_BLUR.set(false);
    let waiting = page().and_then(|p| {
        p.bar.keep_volume();
        p.standby.borrow_mut().take()
    });
    RUN.fetch_add(1, Ordering::Relaxed);
    // The element lets go first: it must not be left holding a closed player.
    if let Some(page) = page() {
        page.video.clear();
        // Leaving may come from an event of this very page: it goes when that is over.
        let old = page.root.clone();
        soon(&Rc::default(), move || {
            let _ = &old;
        });
    }
    PAGE.with(|p| *p.borrow_mut() = None);
    if let Some(standby) = waiting {
        media::close(Some(standby.player), Some(standby.source));
    }
    media::close(PLAYING.with(|p| p.borrow_mut().take()), SOURCE.with(|s| s.borrow_mut().take()));
}

/// The page on show, for what lives beside it.
pub fn root() -> Option<IFrameworkElement> {
    page().map(|p| p.root)
}

/// Opens the 影视 episode `ep_id` / `cid`, whose video is `bvid`.
pub fn show_episode(bvid: &str, title: &str, uploader: &str, ep_id: u64, cid: u64) -> Result<()> {
    NEXT_EPISODE.set(Some((ep_id, cid)));
    open(false, bvid, title, uploader)
}

/// Opens the video with its comments at `focus`, as a notice asks.
pub fn show_comment(bvid: &str, title: &str, focus: rubi_model::CommentRef) -> Result<()> {
    NEXT_FOCUS.set(Some(focus));
    open(false, bvid, title, "")
}

/// Opens the ordinary video page.
pub fn show(bvid: &str, title: &str, uploader: &str) -> Result<()> {
    open(false, bvid, title, uploader)
}

/// Opens a video, in the 竖屏推流 layout when `feed`.
pub fn open(feed: bool, bvid: &str, title: &str, uploader: &str) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    // An opened video feeds the local recommendation, as on Android.
    if let Some(backend) = shell.backend.get().cloned() {
        let (core, clicked) = (backend.clone(), bvid.to_owned());
        backend.spawn(async move {
            let _ = core.services.home.record(&clicked, rubi_model::Signal::Click).await;
        });
    }
    let episode = NEXT_EPISODE.take();
    queue::opening(bvid);
    leave();
    crate::ui::me::leave();
    let layout = if feed { feed::LAYOUT } else { LAYOUT };
    let place = if feed { r#"Grid.RowSpan="4""# } else { r#"Grid.Row="1""# };
    let root = load(
        &splitter::fill(layout)
            .replace("<!--lists-->", LISTS)
            .replace("<!--controls-->", &control_bar::markup(place))
            .replace("<!--second-->", SECOND_ELEMENT),
    )?;
    named::<IUIElement>(&root, "SeekRow")?.SetVisibility(Visibility::Visible)?;
    named::<IUIElement>(&root, "PlayPause")?.SetVisibility(Visibility::Visible)?;
    set_text(&root, "Title", title)?;
    let opening = if uploader.is_empty() { "正在打开…".to_owned() } else { format!("{uploader} · 正在打开…") };
    set_text(&root, "Meta", &opening)?;
    on_click(&root, "Back", go_back)?;
    settle_buttons(&root)?;
    if feed {
        feed::attach(&root)?;
    }
    splitter::attach(&root, "SideBar", drag_side, crate::ui::shell::save_widths, reset_side)?;
    let pane = Side::attach(&root)?;
    if feed {
        pane.select(Pane::Comments)?;
    }
    if let Some(focus) = NEXT_FOCUS.take() {
        pane.focus_comment(focus)?;
    }
    let actions = Actions::attach(&root, bvid)?;
    let (shown_overlay, cid) = (Rc::new(RefCell::new(None)), Rc::new(Cell::new(0)));
    let opened = Page {
        root: root.clone(),
        side: pane,
        actions,
        overlay: shown_overlay,
        bvid: bvid.to_owned(),
        title: title.to_owned(),
        uploader: uploader.to_owned(),
        parts: Rc::default(),
        card: Rc::default(),
        season: Rc::default(),
        subscribed: Rc::default(),
        cid,
        feed,
        episode,
        up: Rc::default(),
        offer: Rc::default(),
        bar: ControlBar::attach(
            &root,
            "Video",
            if feed { &["TopBar"] } else { &[] },
            run,
            || page().map(|p| p.bar),
            set_sound,
        )?,
        video: TwinVideo::new(&root, run, || page().map(|p| p.video))?,
        standby: Rc::default(),
        send_in_full: Rc::default(),
        seek_held: Rc::new(Cell::new(Instant::now())),
        showing_position: Rc::default(),
        seeks: Arc::default(),
        filling: Rc::default(),
    };
    PAGE.with(|p| *p.borrow_mut() = Some(opened));
    on_click(&root, "ActDanmaku", toggle_danmaku)?;
    on_click(&root, "Full", toggle_full)?;
    on_click(&root, "WindowFull", toggle_window_full)?;
    named::<IUIElement>(&root, "WindowFull")?.SetVisibility(Visibility::Visible)?;
    on_click(&root, "Quality", choose_quality)?;
    on_click(&root, "Pip", toggle_pip)?;
    if !feed {
        for name in ["Info", "ActionRow", "MainScroll"] {
            keep(named::<IFrameworkElement>(&root, name)?.SizeChanged(
                |_: Ref<windows_core::IInspectable>, _: Ref<SizeChangedEventArgs>| {
                    if let Some(page) = page() {
                        let _ = fit_video(&page.root);
                    }
                },
            )?);
        }
        on_click(&root, "ActShare", choose_share)?;
        on_click(&root, "MoreDetail", toggle_detail)?;
        on_click(&root, "ResumeHint", resume)?;
        on_click(&root, "AutoNext", || {
            AUTO_NEXT.set(!AUTO_NEXT.get());
            if let Some(page) = page() {
                show_series(&page);
            }
        })?;
        on_click(&root, "PartsExpand", || {
            PARTS_OPEN.set(!PARTS_OPEN.get());
            if let Some(page) = page() {
                show_series(&page);
            }
        })?;
        on_click(&root, "SeasonExpand", || {
            SEASON_OPEN.set(!SEASON_OPEN.get());
            if let Some(page) = page() {
                show_series(&page);
            }
        })?;
        on_click(&root, "SeasonNext", || {
            let _ = season_step();
        })?;
        on_click(&root, "SeasonSub", toggle_subscribe)?;
    }
    on_click(&root, "Speed", choose_speed)?;
    on_click(&root, "PlaySettings", open_settings)?;
    on_click(&root, "SendStyle", choose_send_style)?;
    for name in ["Speed", "PlaySettings"] {
        named::<IUIElement>(&root, name)?.SetVisibility(Visibility::Visible)?;
    }
    set_speed_label(&root);
    show_send_style(&root);
    if let Some(page) = page() {
        page.video.set_stretch(FIT.get());
    }
    hold_for_speed(&named::<IUIElement>(&root, "Video")?)?;
    named::<IUIElement>(&root, "Pip")?.SetVisibility(Visibility::Visible)?;
    on_click(&root, "UpLink", open_up)?;
    on_click(&root, "PlayPause", toggle_play)?;
    on_click(&root, "Prev", || queue::step(-1))?;
    on_click(&root, "Next", || queue::step(1))?;
    if let Some(queued) = queue::view() {
        let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
        named::<IUIElement>(&root, "Prev")?.SetVisibility(shown(queued.has_previous()))?;
        named::<IUIElement>(&root, "Next")?.SetVisibility(shown(queued.has_next()))?;
    }
    on_click(&root, "DanmakuSwitch", toggle_danmaku)?;
    keep(named::<IUIElement>(&root, "Video")?.Tapped(
        |_: Ref<windows_core::IInspectable>, args: Ref<TappedRoutedEventArgs>| {
            // A finger's tap is the bar's: it shows or hides it. The click ending a held press
            // does not pause either.
            if !crate::ui::control_bar::is_touch(&args) && !SKIP_TAP.take() {
                toggle_play();
            }
        },
    )?);
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
        // What the first screen needs goes out first: the picture is fetched on its own task and
        // this only adds what the title row shows. The second screen's reads wait their turn, so
        // they never hold up the page.
        let detail = services.videos.detail(&info_bvid).await.ok();
        let after = info_dispatcher.clone();
        let slow = page.clone();
        post(&info_dispatcher, move || {
            let Some(page) = (run == self::run()).then(page).flatten() else { return };
            if let Some(info) = detail {
                let card = info.card;
                *page.parts.borrow_mut() = info.parts;
                *page.season.borrow_mut() = info.season;
                show_series(&page);
                ask_subscribed(&page);
                *page.card.borrow_mut() = Some(card.clone());
                page.side.set_aid(card.aid, card.stats.reply);
                page.actions.set_counts(card.stats.like, card.stats.coin, card.stats.favorite);
                let _ = set_text(&page.root, "Title", &card.title);
                let _ = show_up_link(&page, card.owner.mid, &card.owner.name);
                let _ = show_detail(&page.root, &card);
            }
        });
        let wants_resume = episode.is_none() && !feed;
        let signed_in = services.account.is_logged_in();
        let (related, resume, relation) = tokio::join!(
            services.discover.related(&info_bvid),
            async {
                if wants_resume { services.videos.resume_point(&info_bvid).await } else { None }
            },
            async {
                if signed_in { services.actions.relation(&info_bvid).await.ok() } else { None }
            },
        );
        let related = related.unwrap_or_default();
        post(&after, move || {
            let Some(page) = (run == self::run()).then(slow.clone()).flatten() else { return };
            if let Some(relation) = relation {
                page.actions.set_relation(relation);
            }
            if let Some(point) = resume {
                offer_resume(point.cid, point.position_ms);
            }
            let _ = page.side.set_related(related);
        });
    });
    backend.spawn(async move {
        let quality = core.services.preferences.settings().playback.preferred_quality.min(MAX_FREE_QN);
        let playback = match episode {
            Some((ep_id, cid)) => core.services.pgc.play(ep_id, cid, quality).await,
            None => core.services.videos.playback(&bvid, quality).await,
        };
        let source = match playback {
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
    let plain = ["Back", "Full", "SendButton", "Prev", "Next", "Normal", "TabComments", "TabRelated", "TabQueue"];
    let acting = ["ActLike", "ActCoin", "ActFav", "ActShare", "ActLater", "ActDanmaku", "ActFollow", "ActComment"];
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

fn show_up_link(page: &Page, mid: u64, name: &str) -> Result<()> {
    *page.up.borrow_mut() = (mid, name.to_owned());
    named::<IContentControl>(&page.root, "UpLink")?.SetContent(&PropertyValue::CreateString(name)?)?;
    named::<IUIElement>(&page.root, "UpLink")?.SetVisibility(Visibility::Visible)
}

/// Opens the UP's page, coming back to this video from the start.
/// Says how to come back to the video on show from the page opened next.
pub fn remember_here() {
    let (Some(page), Some(shell)) = (page(), shell()) else { return };
    let uploader = page.up.borrow().1.clone();
    let (bvid, title) = (page.bvid.clone(), named::<ITextBlock>(&page.root, "Title").and_then(|t| t.Text()));
    let (title, feed) = (title.map(|t| t.to_string()).unwrap_or_default(), page.feed);
    let episode = page.episode;
    shell.remember(move || {
        let _ = match episode {
            Some((ep_id, cid)) => show_episode(&bvid, &title, &uploader, ep_id, cid),
            None => open(feed, &bvid, &title, &uploader),
        };
    });
}

fn open_up() {
    let Some(page) = page() else { return };
    let (mid, name) = page.up.borrow().clone();
    remember_here();
    crate::ui::shell::leave_pages();
    let _ = up::open(mid, &name);
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
    let episode = page.episode;
    backend.spawn(async move {
        let _ = core.services.preferences.patch_settings(&quality_patch(qn).to_string());
        let playback = match episode {
            Some((ep_id, cid)) => core.services.pgc.play(ep_id, cid, qn).await,
            None => core.services.videos.playback(&bvid, qn).await,
        };
        let source = match playback {
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
    page.video.prepare(&player)?;
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
    report_failures(&player)?;
    report_end(&player)?;
    let control = player.cast::<IMediaPlayer>()?;
    control.SetIsLoopingEnabled(page.feed)?;
    if playing {
        control.Play()?;
    }
    let old = PLAYING.with(|p| p.borrow_mut().replace(player));
    let old_source = SOURCE.with(|s| s.borrow_mut().replace(source));
    page.video.switch(old, old_source)?;
    page.bar.apply_sound()?;
    quality::show(&page.root, &offer)?;
    *page.offer.borrow_mut() = offer;
    Ok(())
}

fn shown(opened: std::result::Result<Opened, String>, uploader: &str) -> Result<()> {
    let Some(page) = page() else { return Ok(()) };
    match opened {
        Ok(Opened { source, cid, duration_ms, offer }) => {
            play(&source)?;
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
            let whole_ms = f64::from(duration_ms.max(1));
            overlay.watch_marks(move |marks| {
                if let Some(page) = self::page() {
                    let marks = marks.iter().map(|m| (f64::from(m.target_ms) / whole_ms, m.label.clone())).collect();
                    let _ = page.bar.set_marks(marks);
                }
            });
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
    let request =
        SendDanmaku { bvid: page.bvid.clone(), cid, progress_ms, text, mode: SEND_MODE.get(), color: SEND_COLOR.get() };
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
/// The video's end moves a queue on to the next one.
fn report_end(player: &MediaPlayer) -> Result<()> {
    let Some(dispatcher) = shell().map(|s| s.dispatcher.clone()) else { return Ok(()) };
    let run = RUN.load(Ordering::Relaxed);
    let revoker =
        player.cast::<IMediaPlayer>()?.MediaEnded(move |_: Ref<MediaPlayer>, _: Ref<windows_core::IInspectable>| {
            post(&dispatcher, move || {
                if RUN.load(Ordering::Relaxed) == run {
                    if queue::view().is_some() {
                        queue::ended();
                    } else {
                        auto_next();
                    }
                }
            });
        })?;
    keep(revoker);
    Ok(())
}

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

fn play(source: &MediaSource) -> Result<MediaPlaybackSession> {
    let player = media::player(source)?;
    if let Some(page) = page() {
        page.video.show(&player)?;
    }
    report_failures(&player)?;
    report_end(&player)?;
    if page().is_some_and(|p| p.feed) {
        player.cast::<IMediaPlayer>()?.SetIsLoopingEnabled(true)?;
    }
    player.cast::<IMediaPlayer>()?.Play()?;
    let session = player.cast::<IMediaPlayer3>()?.PlaybackSession()?;
    apply_speed(&player);
    let start = NEXT_START.take().filter(|(bvid, _)| page().is_some_and(|p| &p.bvid == bvid));
    if let Some((_, at)) = start {
        let _ = session.cast::<IMediaPlaybackSession>()?.SetPosition(windows_time::TimeSpan { duration: at });
    }
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
