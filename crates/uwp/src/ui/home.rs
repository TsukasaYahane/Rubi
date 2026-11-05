//! 首页: the recommended videos in a virtualized grid, columns following the window. Rows are
//! filled as they come on screen, which also stands in for data binding: nothing Rust defines
//! has to be known to XAML.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rubi_model::VideoCard;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::layout::grid_cell;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{load, named, player, post, set_text, soon};
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

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" />
    <RowDefinition Height="*" />
  </Grid.RowDefinitions>
  <StackPanel Orientation="Horizontal" Margin="16,8,16,8">
    <TextBlock Text="推荐" FontSize="20" FontWeight="Light" />
    <TextBlock x:Name="Status" Margin="16,0,0,0" FontSize="13" VerticalAlignment="Center"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </StackPanel>
  <GridView x:Name="Grid" Grid.Row="1" Padding="9,0,9,12" SelectionMode="None" IsItemClickEnabled="True">
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
  </GridView>
</Grid>"##;

pub struct HomePage {
    root: IFrameworkElement,
    grid: IFrameworkElement,
    cards: RefCell<Vec<VideoCard>>,
    loading: Cell<bool>,
    /// The feed has been filled from the network.
    started: Cell<bool>,
    /// The cells have been set for the grid's width.
    sized: Cell<bool>,
    /// The grid width and columns the cells were last set for.
    fitted: Cell<(f64, u32)>,
    fit_pending: Rc<Cell<bool>>,
    covers: Covers,
}

impl HomePage {
    pub fn new() -> Result<Rc<Self>> {
        let root = load(LAYOUT)?;
        let grid = named::<IFrameworkElement>(&root, "Grid")?;
        let page = Rc::new(Self {
            root,
            grid: grid.clone(),
            cards: RefCell::default(),
            loading: Cell::new(false),
            started: Cell::new(false),
            sized: Cell::new(false),
            fitted: Cell::new((0.0, 0)),
            fit_pending: Rc::default(),
            covers: Covers::default(),
        });

        let filling = Rc::downgrade(&page);
        on_row(&grid, move |at, parts| filling.upgrade().map_or(Ok(()), |p| p.fill_row(at, parts)))?;
        let clicking = Rc::downgrade(&page);
        on_row_click(&grid, move |at| {
            if let Some(page) = clicking.upgrade() {
                let _ = page.open(at);
            }
        })?;

        let sizing = Rc::downgrade(&page);
        let revoker = grid.SizeChanged(move |_: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
            if let Some(page) = sizing.upgrade() {
                page.resized();
            }
        })?;
        std::mem::forget(revoker);
        Ok(page)
    }

    pub fn root(&self) -> &IFrameworkElement {
        &self.root
    }

    pub fn say(&self, text: &str) {
        let _ = set_text(&self.root, "Status", text);
    }

    /// Fills the feed the first time and shows the first page.
    pub fn start(&self) {
        if !self.started.replace(true) {
            self.say("正在获取推荐…");
            self.load_page(true);
        }
    }

    /// Starts over with what the account now sees, after signing in.
    pub fn reload(&self) {
        let Some(backend) = shell().and_then(|s| s.backend.get().cloned()) else { return };
        backend.services.home.new_session();
        self.cards.borrow_mut().clear();
        if let Ok(items) = self.grid.cast::<IItemsControl>().and_then(|l| l.Items()) {
            let _ = items.cast::<IVector<IInspectable>>().and_then(|v| v.Clear());
        }
        self.started.set(true);
        self.say("正在获取推荐…");
        self.load_page(true);
    }

    fn load_page(&self, fill: bool) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.loading.replace(true) {
            return;
        }
        let dispatcher = shell.dispatcher.clone();
        let core = backend.clone();
        backend.spawn(async move {
            let services = &core.services;
            let filled = if fill {
                let _ = services.account.ensure_device_ids().await;
                services.home.refresh(PER_SOURCE).await.map(|_| ()).map_err(|e| e.to_string())
            } else {
                Ok(())
            };
            let page = filled.and_then(|()| services.home.next_page(PAGE).map_err(|e| e.to_string()));
            post(&dispatcher, move || {
                if let Some(home) = crate::ui::shell::shell().and_then(|s| s.home_page()) {
                    home.loaded(page.map(|p| p.into_iter().map(|r| r.card).collect()));
                }
            });
        });
    }

    fn loaded(&self, page: std::result::Result<Vec<VideoCard>, String>) {
        self.loading.set(false);
        match page {
            Ok(cards) => {
                let nothing = cards.is_empty() && self.cards.borrow().is_empty();
                self.say(if nothing { "暂时没有内容，稍后再打开" } else { "" });
                let _ = self.append(cards);
            }
            Err(problem) => self.say(&format!("获取失败：{problem}")),
        }
    }

    fn append(&self, cards: Vec<VideoCard>) -> Result<()> {
        let from = self.cards.borrow().len();
        self.cards.borrow_mut().extend(cards);
        add_rows(&self.grid, from, self.cards.borrow().len())
    }

    /// Sets the cells for the grid's width and the window's columns.
    pub fn resized(self: &Rc<Self>) {
        let page = Rc::downgrade(self);
        soon(&self.fit_pending, move || {
            if let Some(page) = page.upgrade() {
                let _ = page.apply_cell();
            }
        });
    }

    fn apply_cell(&self) -> Result<()> {
        let Some(shell) = shell() else { return Ok(()) };
        let width = self.grid.ActualWidth()?;
        let columns = shell.class().columns(width);
        if self.fitted.get() == (width, columns) {
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
            self.load_page(false);
        }
        Ok(())
    }

    fn open(&self, at: usize) -> Result<()> {
        if let Some(card) = self.cards.borrow().get(at) {
            player::show(&card.bvid, &card.title, &card.owner.name)?;
        }
        Ok(())
    }
}
