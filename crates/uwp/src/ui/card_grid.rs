//! A virtualized grid of video cards, columns following the window. Rows are filled as they come
//! on screen, which also stands in for data binding: nothing Rust defines has to be known to XAML.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use rubi_model::{LiveRoom, VideoCard};
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::backend::Backend;
use crate::state::layout::grid_cell;
use crate::state::library::{room_link, video_link};
use crate::state::live::room_line;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{card_menu, feed, keep, live_room, load, post};
use crate::xaml::{
    Color, IBorder, IFrameworkElement, IImage, IItemsControl, IItemsControl2, IItemsWrapGrid, ITextBlock, IUIElement,
    SizeChangedEventArgs, SolidColorBrush, Stretch, UIElement, Visibility,
};

/// Cards asked for at a time.
const PAGE: usize = 30;
/// Candidates fetched per source when the feed fills.
const PER_SOURCE: usize = 20;
/// Rows from the end at which the next page loads.
const LOAD_AHEAD: usize = 8;
const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const GREY: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };
/// How much of a cover shows while the room is off air.
const OFF_AIR_OPACITY: f64 = 0.45;
/// Width covers are decoded at, in pixels.
const COVER_DECODE_WIDTH: i32 = 480;

const GRID: &str = r##"<GridView xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        Padding="9,0,9,12" SelectionMode="None" IsItemClickEnabled="True">
  <GridView.ItemContainerTransitions><TransitionCollection /></GridView.ItemContainerTransitions>
  <GridView.ItemContainerStyle>
    <Style TargetType="GridViewItem">
      <Setter Property="Margin" Value="0" />
      <Setter Property="Padding" Value="0" />
      <Setter Property="HorizontalContentAlignment" Value="Stretch" />
      <Setter Property="VerticalContentAlignment" Value="Top" />
    </Style>
  </GridView.ItemContainerStyle>
  <GridView.ItemsPanel>
    <ItemsPanelTemplate><ItemsWrapGrid Orientation="Horizontal" /></ItemsPanelTemplate>
  </GridView.ItemsPanel>
  <GridView.ItemTemplate>
    <DataTemplate>
      <Grid Margin="7">
        <Grid.RowDefinitions>
          <RowDefinition Height="*" />
          <RowDefinition Height="Auto" />
          <RowDefinition Height="Auto" />
        </Grid.RowDefinitions>
        <Image Stretch="UniformToFill" />
        <TextBlock Grid.Row="1" FontSize="13" MaxLines="2" TextWrapping="Wrap" TextTrimming="CharacterEllipsis"
                   Height="38" Margin="0,6,0,0" />
        <TextBlock Grid.Row="2" FontSize="12" MaxLines="1" TextTrimming="CharacterEllipsis" Height="18"
                   Margin="0,2,0,0" Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        <Border Background="#0050EF" Padding="5,1" Margin="4" HorizontalAlignment="Left" VerticalAlignment="Top"
                Visibility="Collapsed"><TextBlock Text="直播中" FontSize="11" Foreground="White" /></Border>
      </Grid>
    </DataTemplate>
  </GridView.ItemTemplate>
</GridView>"##;

/// What a card opens.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    Video { bvid: String, owner: String },
    Room { id: u64, owner: String },
}

/// One card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    key: String,
    title: String,
    line: String,
    cover: String,
    /// A room, and whether it is on air.
    live: Option<bool>,
    target: Target,
}

impl Tile {
    fn video(card: VideoCard) -> Self {
        Self {
            key: card.bvid.clone(),
            line: format!("{} · {} 播放", card.owner.name, card.stats.view),
            title: card.title,
            cover: card.cover,
            live: None,
            target: Target::Video { bvid: card.bvid, owner: card.owner.name },
        }
    }

    fn room(room: LiveRoom) -> Self {
        Self {
            key: room.room_id.to_string(),
            line: room_line(&room),
            title: room.title,
            cover: room.cover,
            live: Some(room.live),
            target: Target::Room { id: room.room_id, owner: room.uname },
        }
    }
}

/// What a grid shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Recommended,
    Popular,
    Ranking,
    /// A zone's feed, by its v2 main region.
    Zone(u32),
    /// Live rooms: the recommended ones, or a parent area's.
    Rooms(Option<u32>),
    /// Rooms of followed UPs, those on air first.
    FollowedRooms,
}

impl Source {
    /// Page `page` of the list, from 1, and whether another may follow.
    async fn fetch(self, core: &Backend, page: u32) -> rubi_core::Result<(Vec<Tile>, bool)> {
        let services = &core.services;
        let videos = |cards: Vec<VideoCard>| cards.into_iter().map(Tile::video).collect::<Vec<_>>();
        match self {
            Self::Recommended => {
                if page == 1 {
                    let _ = services.account.ensure_device_ids().await;
                    services.home.refresh(PER_SOURCE).await?;
                }
                let cards = services.home.next_page(PAGE)?.into_iter().map(|r| r.card).collect();
                Ok((videos(cards), true))
            }
            Self::Popular => {
                let cards = services.discover.popular(page).await?;
                let more = !cards.is_empty();
                Ok((videos(cards), more))
            }
            Self::Ranking => Ok((videos(services.discover.ranking().await?), false)),
            Self::Zone(region) => {
                let cards = services.discover.zone_feed(region, page).await?;
                let more = !cards.is_empty();
                Ok((videos(cards), more))
            }
            Self::Rooms(None) => {
                let rooms = services.live.recommended().await?;
                Ok((rooms.into_iter().map(Tile::room).collect(), false))
            }
            Self::FollowedRooms => {
                let (mut rooms, more) = services.live.followed(page).await?;
                rooms.sort_by_key(|r| !r.live);
                Ok((rooms.into_iter().map(Tile::room).collect(), more))
            }
            Self::Rooms(Some(area)) => {
                let (rooms, more) = services.live.area_rooms(area, page).await?;
                Ok((rooms.into_iter().map(Tile::room).collect(), more))
            }
        }
    }

    fn working(self) -> &'static str {
        if self == Self::Recommended { "正在获取推荐…" } else { "正在读取…" }
    }
}

/// A page of cards and whether another may follow, or why there is none.
type Answer = std::result::Result<(Vec<Tile>, bool), String>;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static GRIDS: RefCell<HashMap<u64, Weak<CardGrid>>> = RefCell::default();
}

pub struct CardGrid {
    id: u64,
    /// Told when the grid has something new to say.
    spoke: Box<dyn Fn()>,
    grid: IFrameworkElement,
    source: Cell<Option<Source>>,
    cards: RefCell<Vec<Tile>>,
    seen: RefCell<HashSet<String>>,
    page: Cell<u32>,
    ended: Cell<bool>,
    loading: Cell<bool>,
    /// Bumped when the grid starts over: answers for an earlier one are dropped.
    turn: Cell<u64>,
    /// What the grid last had to say, shown while its tab is.
    said: RefCell<String>,
    /// The cells have been set for the grid's width.
    sized: Cell<bool>,
    /// The grid width and columns the cells were last set for.
    fitted: Cell<(f64, u32)>,
    covers: Covers,
}

impl CardGrid {
    pub fn new(spoke: impl Fn() + 'static) -> Result<Rc<Self>> {
        let grid = load(GRID)?;
        let shown = Rc::new(Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            spoke: Box::new(spoke),
            grid: grid.clone(),
            source: Cell::new(None),
            cards: RefCell::default(),
            seen: RefCell::default(),
            page: Cell::new(1),
            ended: Cell::new(false),
            loading: Cell::new(false),
            turn: Cell::new(0),
            said: RefCell::default(),
            sized: Cell::new(false),
            fitted: Cell::new((0.0, 0)),
            covers: Covers::default(),
        });
        GRIDS.with_borrow_mut(|g| g.insert(shown.id, Rc::downgrade(&shown)));
        let filling = Rc::downgrade(&shown);
        on_row(&grid, move |at, parts| filling.upgrade().map_or(Ok(()), |s| s.fill_row(at, parts)))?;
        let clicking = Rc::downgrade(&shown);
        on_row_click(&grid, move |at| {
            if let Some(shown) = clicking.upgrade() {
                let _ = shown.open_row(at);
            }
        })?;
        let asking = Rc::downgrade(&shown);
        card_menu::attach(&grid, move |at| {
            let shown = asking.upgrade()?;
            let cards = shown.cards.borrow();
            Some(match &cards.get(at)?.target {
                Target::Video { bvid, .. } => card_menu::Target { later: Some(bvid.clone()), link: video_link(bvid) },
                Target::Room { id, .. } => card_menu::Target { later: None, link: room_link(*id) },
            })
        })?;
        let sizing = Rc::downgrade(&shown);
        keep(grid.SizeChanged(move |_: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
            if let Some(shown) = sizing.upgrade() {
                shown.resized();
            }
        })?);
        Ok(shown)
    }

    pub fn element(&self) -> &IFrameworkElement {
        &self.grid
    }

    pub fn source(&self) -> Option<Source> {
        self.source.get()
    }

    pub fn said(&self) -> String {
        self.said.borrow().clone()
    }

    /// Starts over with `source`.
    pub fn open(self: &Rc<Self>, source: Source) -> Result<()> {
        self.source.set(Some(source));
        self.turn.set(self.turn.get() + 1);
        self.cards.borrow_mut().clear();
        self.seen.borrow_mut().clear();
        self.page.set(1);
        self.ended.set(false);
        self.loading.set(false);
        self.grid.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        self.say(source.working());
        self.load_page();
        Ok(())
    }

    /// Forgets what it showed, until it is opened again.
    pub fn forget(&self) -> Result<()> {
        self.source.set(None);
        self.turn.set(self.turn.get() + 1);
        self.cards.borrow_mut().clear();
        self.seen.borrow_mut().clear();
        self.said.borrow_mut().clear();
        self.grid.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()
    }

    pub fn say(&self, text: &str) {
        *self.said.borrow_mut() = text.to_owned();
        (self.spoke)();
    }

    fn load_page(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let Some(source) = self.source.get() else { return };
        if self.ended.get() || self.loading.replace(true) {
            return;
        }
        let (id, turn, page) = (self.id, self.turn.get(), self.page.get());
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let answer = source.fetch(&core, page).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(grid) = GRIDS.with_borrow(|g| g.get(&id).and_then(Weak::upgrade)) {
                    let _ = grid.answered(turn, answer);
                }
            });
        });
    }

    fn answered(&self, turn: u64, answer: Answer) -> Result<()> {
        if turn != self.turn.get() {
            return Ok(());
        }
        self.loading.set(false);
        match answer {
            Ok((cards, more)) => {
                self.ended.set(!more);
                self.page.set(self.page.get() + 1);
                let mut seen = self.seen.borrow_mut();
                let fresh: Vec<Tile> = cards.into_iter().filter(|c| seen.insert(c.key.clone())).collect();
                drop(seen);
                let from = self.cards.borrow().len();
                self.cards.borrow_mut().extend(fresh);
                let to = self.cards.borrow().len();
                self.say(if to == 0 { "暂时没有内容，稍后再打开" } else { "" });
                add_rows(&self.grid, from, to)
            }
            Err(problem) => {
                self.say(&format!("获取失败：{problem}"));
                Ok(())
            }
        }
    }

    /// Sets the cells for the grid's width and the window's columns.
    ///
    /// Done at once, inside the size change: left for later, a shrinking grid would first lay
    /// its old, wider cells out on fewer columns, and flash that frame.
    pub fn resized(&self) {
        let _ = self.apply_cell();
    }

    fn apply_cell(&self) -> Result<()> {
        let Some(shell) = shell() else { return Ok(()) };
        let width = self.grid.ActualWidth()?;
        let columns = shell.class().columns(width);
        if width <= 0.0 || self.fitted.get() == (width, columns) {
            return Ok(());
        }
        let cell = grid_cell(width, columns);
        let panel = self.grid.cast::<IItemsControl2>()?.ItemsPanelRoot()?.cast::<IItemsWrapGrid>()?;
        panel.SetMaximumRowsOrColumns(columns as i32)?;
        panel.SetItemWidth(cell.width)?;
        panel.SetItemHeight(cell.height)?;
        self.sized.set(true);
        self.fitted.set((width, columns));
        Ok(())
    }

    fn fill_row(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        if !self.sized.get() {
            self.apply_cell()?;
        }
        let cards = self.cards.borrow();
        let Some(card) = cards.get(at) else { return Ok(()) };

        let cover = parts.GetAt(0)?;
        // A room's cover comes in any shape, and shows whole.
        let fit = if card.live.is_some() { Stretch::Uniform } else { Stretch::UniformToFill };
        cover.cast::<IImage>()?.SetStretch(fit)?;
        // A card without a picture still has its words.
        if let Ok(picture) = self.covers.get(&card.cover, COVER_DECODE_WIDTH) {
            cover.cast::<IImage>()?.SetSource(&picture)?;
        }

        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&card.title)?;
        parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&card.line)?;
        let badge = parts.GetAt(3)?;
        badge.cast::<IUIElement>()?.SetVisibility(if card.live.is_some() {
            Visibility::Visible
        } else {
            Visibility::Collapsed
        })?;
        let on_air = card.live.unwrap_or(true);
        let tag = badge.cast::<IBorder>()?;
        let color = if on_air { COBALT } else { GREY };
        tag.SetBackground(&SolidColorBrush::CreateInstanceWithColor(color)?)?;
        tag.Child()?.cast::<ITextBlock>()?.SetText(if on_air { "直播中" } else { "未开播" })?;
        cover.cast::<IUIElement>()?.SetOpacity(if on_air { 1.0 } else { OFF_AIR_OPACITY })?;

        let near_end = at + LOAD_AHEAD >= cards.len();
        drop(cards);
        if near_end {
            self.load_page();
        }
        Ok(())
    }

    fn open_row(&self, at: usize) -> Result<()> {
        let Some(card) = self.cards.borrow().get(at).cloned() else { return Ok(()) };
        match card.target {
            Target::Video { bvid, owner } => feed::route(&bvid, &card.title, &owner),
            Target::Room { id, owner } => live_room::open(id, &card.title, &owner),
        }
    }
}
