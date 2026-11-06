//! 竖屏推流: portrait videos one after another. It is the video page in another layout, so the
//! actions, danmaku, fullscreen and comments are the same ones; this adds moving through the feed
//! and deciding which videos open here.

use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use rubi_core::portrait_of;
use windows_core::{IInspectable, Ref, Result};

use crate::state::feed::{BATCH, Feed, Item};
use crate::ui::keep;
use crate::ui::shell::shell;
use crate::ui::{named, on_click, player, post, set_text};
use crate::xaml::{IFrameworkElement, IUIElement, PointerRoutedEventArgs};

/// How long a video's shape is looked up before it opens anyway.
const LOOKUP: Duration = Duration::from_millis(1_500);
/// Wheel notches closer together than this count as one move.
const WHEEL_GAP: Duration = Duration::from_millis(450);

/// The video page's elements, by the same names, around a centred picture.
pub const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.ColumnDefinitions>
    <ColumnDefinition x:Name="MainColumn" Width="*" />
    <ColumnDefinition Width="Auto" />
    <ColumnDefinition x:Name="SideColumn" Width="340" />
  </Grid.ColumnDefinitions>
  <Grid.RowDefinitions>
    <RowDefinition x:Name="MainRow" Height="*" />
    <RowDefinition x:Name="SideRow" Height="0" />
  </Grid.RowDefinitions>
  <Grid x:Name="Main" Background="Black" RequestedTheme="Dark">
    <Grid.RowDefinitions>
      <RowDefinition x:Name="VideoRow" Height="*" />
      <RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <MediaPlayerElement x:Name="Video" Grid.RowSpan="4" AreTransportControlsEnabled="False" Stretch="Uniform" />
    <Canvas x:Name="Danmaku" Grid.RowSpan="4" IsHitTestVisible="False" />
    <Grid x:Name="TopBar" VerticalAlignment="Top">
      <Grid.ColumnDefinitions>
        <ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" />
      </Grid.ColumnDefinitions>
      <Button x:Name="Back" Background="Transparent" BorderThickness="0" Padding="12,8" Foreground="White">
        <FontIcon Glyph="&#xE72B;" FontSize="16" />
      </Button>
      <TextBlock x:Name="Position" Grid.Column="1" FontSize="12" Foreground="White" VerticalAlignment="Center" />
      <Button x:Name="Normal" Grid.Column="2" Content="普通页" FontSize="12" Foreground="White" Padding="8,4"
              Background="#59000000" BorderThickness="0" />
    </Grid>
    <StackPanel x:Name="ActionRow" Grid.RowSpan="4" HorizontalAlignment="Right" VerticalAlignment="Center"
                Margin="0,0,4,0">
      <StackPanel.Resources>
        <Style TargetType="Button">
          <Setter Property="Background" Value="Transparent" />
          <Setter Property="BorderThickness" Value="0" />
          <Setter Property="Foreground" Value="White" />
          <Setter Property="Padding" Value="0" />
          <Setter Property="Width" Value="56" />
          <Setter Property="Height" Value="52" />
        </Style>
        <Style TargetType="TextBlock">
          <Setter Property="FontSize" Value="11" />
          <Setter Property="HorizontalAlignment" Value="Center" />
          <Setter Property="Foreground" Value="White" />
        </Style>
      </StackPanel.Resources>
      <Button x:Name="Prev"><FontIcon Glyph="&#xE70E;" FontSize="20" /></Button>
      <Button x:Name="ActLike"><StackPanel><FontIcon Glyph="&#xE8E1;" FontSize="24" /><TextBlock x:Name="LikeText" Text="赞" /></StackPanel></Button>
      <Button x:Name="ActCoin"><StackPanel><FontIcon Glyph="&#xE8C7;" FontSize="24" /><TextBlock x:Name="CoinText" Text="币" /></StackPanel>
        <Button.Flyout><MenuFlyout>
          <MenuFlyoutItem x:Name="Coin1" Text="投 1 枚" /><MenuFlyoutItem x:Name="Coin2" Text="投 2 枚" />
        </MenuFlyout></Button.Flyout></Button>
      <Button x:Name="ActFav"><StackPanel><FontIcon Glyph="&#xE734;" FontSize="24" /><TextBlock x:Name="FavText" Text="藏" /></StackPanel></Button>
      <Button x:Name="ActComment"><StackPanel><FontIcon Glyph="&#xE90A;" FontSize="24" /><TextBlock Text="评论" /></StackPanel></Button>
      <Button x:Name="ActLater"><StackPanel><FontIcon Glyph="&#xE823;" FontSize="24" /><TextBlock x:Name="LaterText" Text="稍后" /></StackPanel></Button>
      <Button x:Name="ActDanmaku"><StackPanel><FontIcon Glyph="&#xE8BD;" FontSize="24" /><TextBlock x:Name="DanmakuText" Text="弹幕 开" /></StackPanel></Button>
      <Button x:Name="ActFollow"><StackPanel><FontIcon Glyph="&#xE8FA;" FontSize="24" /><TextBlock x:Name="FollowText" Text="关注" /></StackPanel></Button>
      <Button x:Name="Next"><FontIcon Glyph="&#xE70D;" FontSize="20" /></Button>
    </StackPanel>
    <StackPanel x:Name="Info" Grid.Row="2" Margin="12,0,72,8" VerticalAlignment="Bottom">
      <TextBlock x:Name="Title" FontSize="15" Foreground="White" TextWrapping="Wrap" MaxLines="2" />
      <TextBlock x:Name="Meta" FontSize="12" Margin="0,3,0,0" Foreground="#CCFFFFFF" TextWrapping="Wrap" MaxLines="2" />
    </StackPanel>
    <ContentControl x:Name="SendBelowVideo" Grid.Row="3" IsTabStop="False" HorizontalContentAlignment="Stretch"
                    Margin="0,0,0,88">
    <Grid x:Name="SendBar" Background="#66000000" Padding="12,6">
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
    <!--controls-->
  </Grid>
  <Thumb x:Name="SideBar" Grid.Column="1" Width="0" Visibility="Collapsed"><!--bar--></Thumb>
  <Grid x:Name="Side" Grid.Column="2" Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" />
      <RowDefinition Height="*" />
    </Grid.RowDefinitions>
    <StackPanel Orientation="Horizontal" Margin="4,6,4,2">
      <Button x:Name="TabComments" Content="评论" Background="Transparent" BorderThickness="0" FontSize="20"
              FontWeight="Light" Padding="12,2" />
      <Button x:Name="TabRelated" Content="推荐" Background="Transparent" BorderThickness="0" FontSize="20"
              FontWeight="Light" Padding="12,2" />
    </StackPanel>
    <!--lists-->
  </Grid>
</Grid>"##;

thread_local! {
    static FEED: RefCell<Feed> = RefCell::default();
    /// The side pane on a window too small to keep it open.
    static PANEL_OPEN: Cell<bool> = const { Cell::new(false) };
    static LAST_WHEEL: Cell<Option<Instant>> = const { Cell::new(None) };
    static LOADING: Cell<bool> = const { Cell::new(false) };
}

pub fn panel_open() -> bool {
    PANEL_OPEN.get()
}

/// Opens `bvid`: in the feed when it is a portrait video, on the video page otherwise. A list that
/// did not say what shape it is gets a quick look at its details.
pub fn route(bvid: &str, title: &str, uploader: &str) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    let item = Item { bvid: bvid.to_owned(), title: title.to_owned(), uploader: uploader.to_owned() };
    match backend.services.videos.known_portrait(bvid) {
        Some(true) => return start(item),
        Some(false) => return player::show(bvid, title, uploader),
        None => {}
    }
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let lookup = core.services.videos.portrait(&item.bvid);
        let portrait = tokio::time::timeout(LOOKUP, lookup).await.ok().and_then(|p| p.ok()).unwrap_or(false);
        post(&dispatcher, move || {
            let _ = if portrait { start(item) } else { player::show(&item.bvid, &item.title, &item.uploader) };
        });
    });
    Ok(())
}

fn start(first: Item) -> Result<()> {
    FEED.set(Feed::start(first));
    PANEL_OPEN.set(false);
    open_current()
}

fn open_current() -> Result<()> {
    let Some(item) = FEED.with_borrow(|f| f.current().cloned()) else { return Ok(()) };
    player::open(true, &item.bvid, &item.title, &item.uploader)?;
    show_position();
    load_more();
    Ok(())
}

fn show_position() {
    let at = FEED.with_borrow(Feed::position);
    if let Some(root) = player::root() {
        let _ = set_text(&root, "Position", &format!("竖屏推流 · 第 {at} 条"));
    }
}

/// Moves `by` videos on through the feed.
pub fn step(by: isize) {
    if FEED.with_borrow_mut(|f| f.step(by).is_some()) {
        let _ = open_current();
    } else if by > 0 {
        player::say("后面暂时没有竖屏视频了");
        load_more();
    }
}

/// Adds portrait videos related to the last ones.
fn load_more() {
    let (wanted, sources) = FEED.with_borrow(|f| (f.wants_more(), f.sources()));
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    if !wanted || LOADING.replace(true) {
        return;
    }
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let mut found = Vec::new();
        for source in sources {
            for card in core.services.discover.related(&source).await.unwrap_or_default() {
                if portrait_of(&card.raw) == Some(true) {
                    found.push(Item { bvid: card.bvid, title: card.title, uploader: card.owner.name });
                }
            }
            if found.len() >= BATCH {
                break;
            }
        }
        post(&dispatcher, move || {
            LOADING.set(false);
            FEED.with_borrow_mut(|f| f.add(found));
        });
    });
}

/// Wires what only the feed has.
pub fn attach(root: &IFrameworkElement) -> Result<()> {
    on_click(root, "Prev", || step(-1))?;
    on_click(root, "Next", || step(1))?;
    on_click(root, "Normal", open_normally)?;
    on_click(root, "ActComment", toggle_panel)?;
    let revoker = named::<IUIElement>(root, "Main")?.PointerWheelChanged(
        |_: Ref<IInspectable>, args: Ref<PointerRoutedEventArgs>| {
            if let Ok(args) = args.ok() {
                let _ = wheel(args);
            }
        },
    )?;
    keep(revoker);
    Ok(())
}

fn wheel(args: &PointerRoutedEventArgs) -> Result<()> {
    let delta = args.GetCurrentPoint(None::<&crate::xaml::UIElement>)?.Properties()?.MouseWheelDelta()?;
    let now = Instant::now();
    if delta != 0 && LAST_WHEEL.get().is_none_or(|last| now - last >= WHEEL_GAP) {
        LAST_WHEEL.set(Some(now));
        step(if delta < 0 { 1 } else { -1 });
    }
    Ok(())
}

fn open_normally() {
    if let Some(item) = FEED.with_borrow(|f| f.current().cloned()) {
        let _ = player::show(&item.bvid, &item.title, &item.uploader);
    }
}

fn toggle_panel() {
    PANEL_OPEN.set(!PANEL_OPEN.get());
    player::resized();
}
