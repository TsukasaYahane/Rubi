//! A virtualized grid of video cards, columns following the window. Rows are filled as they come
//! on screen, which also stands in for data binding: nothing Rust defines has to be known to XAML.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use rubi_model::VideoCard;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::backend::Backend;
use crate::state::layout::grid_cell;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{feed, keep, load, post};
use crate::xaml::{
    IFrameworkElement, IImage, IItemsControl, IItemsControl2, IItemsWrapGrid, ITextBlock, SizeChangedEventArgs,
    UIElement,
};

/// Cards asked for at a time.
const PAGE: usize = 30;
/// Candidates fetched per source when the feed fills.
const PER_SOURCE: usize = 20;
/// Rows from the end at which the next page loads.
const LOAD_AHEAD: usize = 8;
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
      </Grid>
    </DataTemplate>
  </GridView.ItemTemplate>
</GridView>"##;

/// What a grid shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Recommended,
    Popular,
    Ranking,
    /// A zone's feed, by its v2 main region.
    Zone(u32),
}

impl Source {
    /// Page `page` of the list, from 1, and whether another may follow.
    async fn fetch(self, core: &Backend, page: u32) -> rubi_core::Result<(Vec<VideoCard>, bool)> {
        let services = &core.services;
        match self {
            Self::Recommended => {
                if page == 1 {
                    let _ = services.account.ensure_device_ids().await;
                    services.home.refresh(PER_SOURCE).await?;
                }
                let cards = services.home.next_page(PAGE)?.into_iter().map(|r| r.card).collect();
                Ok((cards, true))
            }
            Self::Popular => {
                let cards = services.discover.popular(page).await?;
                let more = !cards.is_empty();
                Ok((cards, more))
            }
            Self::Ranking => Ok((services.discover.ranking().await?, false)),
            Self::Zone(region) => {
                let cards = services.discover.zone_feed(region, page).await?;
                let more = !cards.is_empty();
                Ok((cards, more))
            }
        }
    }

    fn working(self) -> &'static str {
        if self == Self::Recommended { "正在获取推荐…" } else { "正在读取…" }
    }
}

/// A page of cards and whether another may follow, or why there is none.
pub type Answer = std::result::Result<(Vec<VideoCard>, bool), String>;

pub struct CardGrid {
    /// Which tab of the home page this is.
    tab: usize,
    grid: IFrameworkElement,
    source: Cell<Option<Source>>,
    cards: RefCell<Vec<VideoCard>>,
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
    pub fn new(tab: usize) -> Result<Rc<Self>> {
        let grid = load(GRID)?;
        let shown = Rc::new(Self {
            tab,
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
        let filling = Rc::downgrade(&shown);
        on_row(&grid, move |at, parts| filling.upgrade().map_or(Ok(()), |s| s.fill_row(at, parts)))?;
        let clicking = Rc::downgrade(&shown);
        on_row_click(&grid, move |at| {
            if let Some(shown) = clicking.upgrade() {
                let _ = shown.open_row(at);
            }
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
        if let Some(home) = shell().and_then(|s| s.home_page()) {
            home.grid_spoke(self.tab);
        }
    }

    fn load_page(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let Some(source) = self.source.get() else { return };
        if self.ended.get() || self.loading.replace(true) {
            return;
        }
        let (tab, turn, page) = (self.tab, self.turn.get(), self.page.get());
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let answer = source.fetch(&core, page).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(home) = crate::ui::shell::shell().and_then(|s| s.home_page()) {
                    home.grid_answered(tab, turn, answer);
                }
            });
        });
    }

    pub fn answered(&self, turn: u64, answer: Answer) -> Result<()> {
        if turn != self.turn.get() {
            return Ok(());
        }
        self.loading.set(false);
        match answer {
            Ok((cards, more)) => {
                self.ended.set(!more);
                self.page.set(self.page.get() + 1);
                let mut seen = self.seen.borrow_mut();
                let fresh: Vec<VideoCard> = cards.into_iter().filter(|c| seen.insert(c.bvid.clone())).collect();
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
        cover.cast::<IImage>()?.SetSource(&self.covers.get(&card.cover, COVER_DECODE_WIDTH)?)?;

        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&card.title)?;
        parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&format!("{} · {} 播放", card.owner.name, card.stats.view))?;

        let near_end = at + LOAD_AHEAD >= cards.len();
        drop(cards);
        if near_end {
            self.load_page();
        }
        Ok(())
    }

    fn open_row(&self, at: usize) -> Result<()> {
        if let Some(card) = self.cards.borrow().get(at) {
            feed::route(&card.bvid, &card.title, &card.owner.name)?;
        }
        Ok(())
    }
}
