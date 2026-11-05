//! 首页: the recommended videos in a virtualized grid, columns following the window. Rows are
//! filled as they come on screen, which also stands in for data binding: nothing Rust defines
//! has to be known to XAML.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rubi_model::VideoCard;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::layout::grid_cell;
use crate::ui::shell::shell;
use crate::ui::{load, named, player, post, set_text};
use crate::xaml::{
    BitmapImage, ContainerContentChangingEventArgs, IBitmapImage, IContentControl2, IFrameworkElement, IImage,
    IItemClickEventArgs, IItemsControl, IItemsControl2, IItemsWrapGrid, IListViewBase, IListViewBase2, IPanel,
    IPropertyValue, ITextBlock, ItemClickEventArgs, ListViewBase, PropertyValue, SizeChangedEventArgs, UIElement, Uri,
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
  <GridView x:Name="Grid" Grid.Row="1" Padding="16,0,0,12" SelectionMode="None" IsItemClickEnabled="True">
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
        <StackPanel Margin="0,0,10,10">
          <Image Stretch="UniformToFill" Height="120" />
          <TextBlock FontSize="13" MaxLines="2" TextWrapping="Wrap" TextTrimming="CharacterEllipsis"
                     Height="38" Margin="0,5,0,0" />
          <TextBlock FontSize="11" MaxLines="1" TextTrimming="CharacterEllipsis"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </StackPanel>
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
    cover_height: Cell<f64>,
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
            cover_height: Cell::new(0.0),
        });

        let filling = Rc::downgrade(&page);
        let revoker = grid.cast::<IListViewBase2>()?.ContainerContentChanging(
            move |_: Ref<ListViewBase>, args: Ref<ContainerContentChangingEventArgs>| {
                if let (Some(page), Ok(args)) = (filling.upgrade(), args.ok()) {
                    let _ = page.fill_row(args);
                }
            },
        )?;
        std::mem::forget(revoker);

        let clicking = Rc::downgrade(&page);
        let revoker =
            grid.cast::<IListViewBase>()?.ItemClick(move |_: Ref<IInspectable>, args: Ref<ItemClickEventArgs>| {
                if let (Some(page), Ok(args)) = (clicking.upgrade(), args.ok()) {
                    let _ = page.open(args);
                }
            })?;
        std::mem::forget(revoker);

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
                self.say("");
                let _ = self.append(cards);
            }
            Err(problem) => self.say(&format!("获取失败：{problem}")),
        }
    }

    fn append(&self, cards: Vec<VideoCard>) -> Result<()> {
        let items = self.grid.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?;
        let mut held = self.cards.borrow_mut();
        for card in cards {
            items.Append(&PropertyValue::CreateInt32(held.len() as i32)?)?;
            held.push(card);
        }
        Ok(())
    }

    /// Sets the cells for the grid's width and the window's columns.
    pub fn resized(&self) {
        let _ = self.apply_cell();
    }

    fn apply_cell(&self) -> Result<()> {
        let Some(shell) = shell() else { return Ok(()) };
        let cell = grid_cell(self.grid.ActualWidth()?, shell.class().columns());
        let panel = self.grid.cast::<IItemsControl2>()?.ItemsPanelRoot()?.cast::<IItemsWrapGrid>()?;
        panel.SetMaximumRowsOrColumns(shell.class().columns() as i32)?;
        panel.SetItemWidth(cell.width)?;
        panel.SetItemHeight(cell.height)?;
        self.cover_height.set(cell.cover_height);
        Ok(())
    }

    fn fill_row(&self, args: &ContainerContentChangingEventArgs) -> Result<()> {
        if args.InRecycleQueue()? {
            return Ok(());
        }
        if self.cover_height.get() == 0.0 {
            self.apply_cell()?;
        }
        let at = args.ItemIndex()? as usize;
        let row = args.ItemContainer()?.cast::<IContentControl2>()?.ContentTemplateRoot()?;
        let parts = row.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
        let cards = self.cards.borrow();
        let Some(card) = cards.get(at) else { return Ok(()) };

        let cover = parts.GetAt(0)?;
        cover.cast::<IFrameworkElement>()?.SetHeight(self.cover_height.get())?;
        let bitmap = BitmapImage::CreateInstanceWithUriSource(&Uri::CreateUri(&card.cover)?)?;
        bitmap.cast::<IBitmapImage>()?.SetDecodePixelWidth(COVER_DECODE_WIDTH)?;
        cover.cast::<IImage>()?.SetSource(&bitmap)?;

        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&card.title)?;
        parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&format!("{} · {} 播放", card.owner.name, card.stats.view))?;

        let near_end = at + LOAD_AHEAD >= cards.len();
        drop(cards);
        if near_end {
            self.load_page(false);
        }
        Ok(())
    }

    fn open(&self, args: &ItemClickEventArgs) -> Result<()> {
        let at = args.cast::<IItemClickEventArgs>()?.ClickedItem()?.cast::<IPropertyValue>()?.GetInt32()? as usize;
        if let Some(card) = self.cards.borrow().get(at) {
            player::show(&card.bvid, &card.title, &card.owner.name)?;
        }
        Ok(())
    }
}
