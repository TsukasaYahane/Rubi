//! 竖屏推流: portrait videos one after another. It is the video page in another layout, so the
//! actions, danmaku, fullscreen and comments are the same ones; this adds moving through the feed
//! and deciding which videos open here. It starts from a portrait video opened anywhere, its
//! related videos following; or from the 竖屏 entry on the home page, the recommendation pool's
//! portrait videos following.

use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use rubi_core::portrait_of;
use windows_core::{IInspectable, Ref, Result};

use crate::state::feed::{BATCH, Feed, Item};
use crate::ui::keep;
use crate::ui::shell::{shell, toast};
use crate::ui::{load, named, on_click, player, post, set_text};
use crate::xaml::{IFontIcon, IFrameworkElement, IUIElement, PointerRoutedEventArgs, Visibility};

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
    <Grid x:Name="Video" Grid.RowSpan="4" Background="Transparent" />
    <Grid Grid.RowSpan="4" <!--second--> />
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
    <StackPanel x:Name="Info" Grid.Row="2" Margin="12,0,72,88" VerticalAlignment="Bottom">
      <Button x:Name="UpLink" Visibility="Collapsed" Background="Transparent" BorderThickness="0" Padding="0,2"
              FontSize="13" Foreground="#6E9BFF" />
      <TextBlock x:Name="Title" FontSize="15" Foreground="White" TextWrapping="Wrap" MaxLines="2" />
      <TextBlock x:Name="Meta" FontSize="12" Margin="0,3,0,0" Foreground="#CCFFFFFF" TextWrapping="Wrap" MaxLines="2" />
    </StackPanel>
    <ContentControl x:Name="SendBelowVideo" Grid.Row="3" IsTabStop="False" HorizontalContentAlignment="Stretch"
                    VerticalAlignment="Bottom">
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
    /// The feed was entered from 首页 and takes its videos from the recommendation pool.
    static FROM_POOL: Cell<bool> = const { Cell::new(false) };
}

/// The page shown while the first portrait videos are picked.
const PICKING: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="Black" RequestedTheme="Dark">
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <Grid>
    <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
    <Button x:Name="Back" Background="Transparent" BorderThickness="0" Padding="12,8" Foreground="White">
      <StackPanel Orientation="Horizontal">
        <FontIcon Glyph="&#xE72B;" FontSize="16" /><TextBlock Text="返回" Margin="10,0,0,0" />
      </StackPanel>
    </Button>
    <TextBlock Grid.Column="1" Text="竖屏推流 · 来自推荐" FontSize="12" Foreground="#B3FFFFFF"
               HorizontalAlignment="Right" VerticalAlignment="Center" Margin="0,0,16,0" />
  </Grid>
  <StackPanel Grid.Row="1" HorizontalAlignment="Center" VerticalAlignment="Center" MaxWidth="420" Margin="24,0">
    <FontIcon x:Name="PickIcon" Glyph="&#xE8EA;" FontSize="44" Foreground="#D9FFFFFF" />
    <TextBlock x:Name="PickTitle" Text="正在挑选竖屏视频…" FontSize="16" Foreground="White" Margin="0,14,0,0"
               HorizontalAlignment="Center" />
    <TextBlock x:Name="PickLine" Text="从推荐池里挑，找到第一个就开始播放" FontSize="12" Foreground="#B3FFFFFF"
               Margin="0,6,0,0" HorizontalAlignment="Center" TextWrapping="Wrap" TextAlignment="Center" />
    <ProgressBar x:Name="PickBusy" IsIndeterminate="True" Width="180" Margin="0,16,0,0" Foreground="#0050EF" />
    <StackPanel x:Name="PickChoices" Orientation="Horizontal" HorizontalAlignment="Center" Margin="0,16,0,0"
                Visibility="Collapsed">
      <Button x:Name="PickAgain" Content="继续等" Foreground="White" BorderBrush="White" BorderThickness="1"
              Background="Transparent" Padding="14,4" Margin="0,0,8,0" />
      <Button x:Name="PickHome" Content="返回首页" Foreground="White" BorderBrush="White" BorderThickness="1"
              Background="Transparent" Padding="14,4" />
    </StackPanel>
  </StackPanel>
</Grid>"##;

/// Glyphs for the page's states: nothing found yet, and held back by risk control.
const NONE_GLYPH: &str = "\u{E9CE}";
const HELD_GLYPH: &str = "\u{E916}";

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
    FROM_POOL.set(false);
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
    let from = if FROM_POOL.get() { " · 来自推荐" } else { "" };
    if let Some(root) = player::root() {
        let _ = set_text(&root, "Position", &format!("竖屏推流{from} · 第 {at} 条"));
    }
}

/// The 竖屏 entry on 首页: picks the pool's portrait videos and plays the first as soon as there
/// is one, picking the rest as it plays.
pub fn open_from_pool() -> Result<()> {
    let Some(host) = shell() else { return Ok(()) };
    let root = load(PICKING)?;
    let back = || {
        if let Some(shell) = shell() {
            shell.back();
        }
    };
    on_click(&root, "Back", back)?;
    on_click(&root, "PickHome", back)?;
    on_click(&root, "PickAgain", || {
        let _ = pick_first();
    })?;
    host.set_page(&root)?;
    if let Some(backend) = host.backend.get() {
        backend.services.portrait.start();
    }
    pick_first()
}

/// Asks for the first portrait videos, the picking page on show.
fn pick_first() -> Result<()> {
    let Some(host) = shell() else { return Ok(()) };
    let Some(backend) = host.backend.get().cloned() else { return Ok(()) };
    let root = host.current_page()?;
    say_picking(&root, None)?;
    let (turn, dispatcher, core) = (crate::ui::shell::page_turn(), host.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let picked = core.services.portrait.next(1).await;
        post(&dispatcher, move || {
            if crate::ui::shell::page_turn() != turn {
                return;
            }
            let picked = match picked {
                Ok(picked) => picked,
                Err(problem) => {
                    if let Some(root) = shell().and_then(|s| s.current_page().ok()) {
                        let _ = say_picking(&root, Some((NONE_GLYPH, "挑选失败", &problem.to_string())));
                    }
                    return;
                }
            };
            let items: Vec<Item> = picked
                .cards
                .into_iter()
                .map(|c| Item { bvid: c.bvid, title: c.title, uploader: c.owner.name })
                .collect();
            let mut items = items.into_iter();
            let Some(first) = items.next() else {
                let note = if picked.limited {
                    (HELD_GLYPH, "B 站暂时限制了查询", "过一会儿再挑选；可以先返回首页")
                } else {
                    (NONE_GLYPH, "推荐池里暂时没有竖屏视频", "正在拉取新一批推荐并挑选，也会沿相关推荐找")
                };
                if let Some(root) = shell().and_then(|s| s.current_page().ok()) {
                    let _ = say_picking(&root, Some(note));
                }
                return;
            };
            if picked.limited {
                toast("B 站暂时限制了查询，已挑出的可以先看");
            }
            FEED.set(Feed::start(first));
            FEED.with_borrow_mut(|f| f.add(items));
            FROM_POOL.set(true);
            PANEL_OPEN.set(false);
            let _ = open_current();
        });
    });
    Ok(())
}

/// The picking page busy, or saying why it stopped.
fn say_picking(root: &IFrameworkElement, stopped: Option<(&str, &str, &str)>) -> Result<()> {
    let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
    named::<IUIElement>(root, "PickBusy")?.SetVisibility(shown(stopped.is_none()))?;
    named::<IUIElement>(root, "PickChoices")?.SetVisibility(shown(stopped.is_some()))?;
    let (glyph, title, line) =
        stopped.unwrap_or(("\u{E8EA}", "正在挑选竖屏视频…", "从推荐池里挑，找到第一个就开始播放"));
    named::<IFontIcon>(root, "PickIcon")?.SetGlyph(glyph)?;
    set_text(root, "PickTitle", title)?;
    set_text(root, "PickLine", line)
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

/// Adds portrait videos: from the recommendation pool when the feed began there, else related
/// to the last ones.
fn load_more() {
    let (wanted, sources) = FEED.with_borrow(|f| (f.wants_more(), f.sources()));
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    if !wanted || LOADING.replace(true) {
        return;
    }
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    if FROM_POOL.get() {
        backend.spawn(async move {
            let picked = core.services.portrait.next(BATCH).await;
            post(&dispatcher, move || {
                LOADING.set(false);
                let Ok(picked) = picked else { return };
                if picked.limited && picked.cards.is_empty() {
                    player::say("B 站暂时限制了查询，过一会儿再挑选");
                }
                let found =
                    picked.cards.into_iter().map(|c| Item { bvid: c.bvid, title: c.title, uploader: c.owner.name });
                FEED.with_borrow_mut(|f| f.add(found));
            });
        });
        return;
    }
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
