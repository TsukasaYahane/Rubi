//! The video page's 推荐 / 评论: beside the video on a wide window, under it otherwise.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rubi_core::CommentArea;
use rubi_model::VideoCard;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::panes::{SideLayout, side_layout};
use crate::state::queue::Queue;
use crate::ui::comment_list::CommentList;
use crate::ui::feed;
use crate::ui::keep;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::{load_object, named, on_click, queue};
use crate::xaml::{
    DataTemplate, IBorder, IContentControl, IControl, IFrameworkElement, IImage, IItemsControl, IItemsControl2,
    IItemsWrapGrid, ITextBlock, IUIElement, PropertyValue, SizeChangedEventArgs, SolidColorBrush, UIElement,
    Visibility,
};

const COVER_DECODE_WIDTH: i32 = 320;

/// A related video with its cover beside the words: a narrow pane.
const SMALL_ROW: &str = r##"<DataTemplate xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">
  <Grid>
    <Grid.ColumnDefinitions><ColumnDefinition Width="128" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
    <Grid.RowDefinitions><RowDefinition Height="*" /><RowDefinition Height="Auto" /></Grid.RowDefinitions>
    <Image Grid.RowSpan="2" Stretch="UniformToFill" />
    <TextBlock Grid.Column="1" Margin="10,0,0,0" FontSize="13" TextWrapping="Wrap" MaxLines="2"
               TextTrimming="CharacterEllipsis" />
    <TextBlock Grid.Column="1" Grid.Row="1" Margin="10,0,0,0" FontSize="12" TextTrimming="CharacterEllipsis"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </Grid></DataTemplate>"##;

/// With the cover above the words: a wide pane.
const LARGE_ROW: &str = r##"<DataTemplate xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">
  <Grid>
    <Grid.RowDefinitions>
      <RowDefinition Height="*" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <Image Stretch="UniformToFill" />
    <TextBlock Grid.Row="1" Height="36" Margin="0,5,0,0" FontSize="13" TextWrapping="Wrap" MaxLines="2"
               TextTrimming="CharacterEllipsis" />
    <TextBlock Grid.Row="2" FontSize="12" TextTrimming="CharacterEllipsis"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </Grid></DataTemplate>"##;

/// What the pane shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Related,
    Comments,
    Queue,
}

pub struct Side {
    root: IFrameworkElement,
    aid: Cell<u64>,
    related: RefCell<Vec<VideoCard>>,
    replies: Cell<u64>,
    comments: Rc<CommentList>,
    comments_opened: Cell<bool>,
    /// The row template in use, so it changes only when the pane crosses a width.
    large_rows: Cell<Option<bool>>,
    /// The list's panel exists only once it has rows: until then the fit has to be tried again.
    fitted: Cell<bool>,
    covers: Covers,
    /// The layout the rows were last set to.
    layout: Cell<Option<SideLayout>>,
    /// The queue on play, as the video opened.
    queue: Option<Queue>,
}

impl Side {
    pub fn attach(root: &IFrameworkElement) -> Result<Rc<Self>> {
        let side = Rc::new(Self {
            root: root.clone(),
            aid: Cell::new(0),
            related: RefCell::default(),
            replies: Cell::new(0),
            comments: CommentList::new()?,
            comments_opened: Cell::new(false),
            large_rows: Cell::new(None),
            fitted: Cell::new(false),
            covers: Covers::default(),
            layout: Cell::new(None),
            queue: queue::view(),
        });

        let related = named::<IFrameworkElement>(root, "Related")?;
        let sizing = Rc::downgrade(&side);
        let revoker = related.SizeChanged(move |_: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
            // At once, for the reason in `HomePage::resized`.
            if let Some(side) = sizing.upgrade() {
                let _ = side.fit_rows();
            }
        })?;
        keep(revoker);
        let filling = Rc::downgrade(&side);
        on_row(&related, move |at, parts| filling.upgrade().map_or(Ok(()), |s| s.fill_related(at, parts)))?;
        let opening = Rc::downgrade(&side);
        on_row_click(&related, move |at| {
            let Some(side) = opening.upgrade() else { return };
            let card = side.related.borrow().get(at).cloned();
            if let Some(card) = card {
                let _ = feed::route(&card.bvid, &card.title, &card.owner.name);
            }
        })?;

        named::<IBorder>(root, "CommentsHost")?.SetChild(&side.comments.element().cast::<UIElement>()?)?;

        for (name, pane) in [("TabRelated", Pane::Related), ("TabComments", Pane::Comments), ("TabQueue", Pane::Queue)]
        {
            // The 竖屏推流 layout has no queue tab.
            if named::<IFrameworkElement>(root, name).is_err() {
                continue;
            }
            let tab = Rc::downgrade(&side);
            on_click(root, name, move || {
                if let Some(side) = tab.upgrade() {
                    let _ = side.select(pane);
                }
            })?;
        }
        side.mark_tabs(Pane::Related)?;
        side.show_queue()?;
        Ok(side)
    }

    /// The video's aid and how many comments it has are known: comments can load.
    pub fn set_aid(&self, aid: u64, replies: u64) {
        self.aid.set(aid);
        self.replies.set(replies);
        self.load_comments();
    }

    pub fn set_related(&self, cards: Vec<VideoCard>) -> Result<()> {
        let count = cards.len();
        *self.related.borrow_mut() = cards;
        add_rows(&named(&self.root, "Related")?, 0, count)
    }

    /// Lays the related videos out for the pane's width, live while it is dragged.
    fn fit_rows(&self) -> Result<()> {
        let list = named::<IFrameworkElement>(&self.root, "Related")?;
        let layout = side_layout(list.ActualWidth()?);
        if self.fitted.get() && self.layout.get() == Some(layout) {
            return Ok(());
        }
        self.fitted.set(false);
        if self.large_rows.replace(Some(layout.large)) != Some(layout.large) {
            let template = load_object(if layout.large { LARGE_ROW } else { SMALL_ROW })?;
            list.cast::<IItemsControl>()?.SetItemTemplate(&template.cast::<DataTemplate>()?)?;
        }
        // The panel goes while a new template settles: the next row filled comes back here.
        let Ok(panel) = list.cast::<IItemsControl2>().and_then(|l| l.ItemsPanelRoot()) else { return Ok(()) };
        let panel = panel.cast::<IItemsWrapGrid>()?;
        panel.SetMaximumRowsOrColumns(layout.columns as i32)?;
        panel.SetItemWidth(layout.cell_width)?;
        panel.SetItemHeight(layout.cell_height)?;
        self.layout.set(Some(layout));
        self.fitted.set(true);
        Ok(())
    }

    pub fn select(&self, pane: Pane) -> Result<()> {
        let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
        named::<IUIElement>(&self.root, "Related")?.SetVisibility(shown(pane == Pane::Related))?;
        named::<IUIElement>(&self.root, "CommentsHost")?.SetVisibility(shown(pane == Pane::Comments))?;
        if let Ok(list) = named::<IUIElement>(&self.root, "Queue") {
            list.SetVisibility(shown(pane == Pane::Queue))?;
        }
        self.mark_tabs(pane)?;
        if pane == Pane::Comments && !self.comments_opened.replace(true) {
            self.load_comments();
        }
        Ok(())
    }

    fn mark_tabs(&self, pane: Pane) -> Result<()> {
        let tint = |on| {
            use crate::xaml::Color;
            let color =
                if on { Color { a: 255, r: 0, g: 0x50, b: 0xEF } } else { Color { a: 255, r: 0x76, g: 0x76, b: 0x76 } };
            SolidColorBrush::CreateInstanceWithColor(color)
        };
        for (name, of) in [("TabRelated", Pane::Related), ("TabComments", Pane::Comments), ("TabQueue", Pane::Queue)] {
            if let Ok(tab) = named::<IControl>(&self.root, name) {
                tab.SetForeground(&tint(of == pane)?)?;
            }
        }
        Ok(())
    }

    /// With a queue on play, its tab shows and opens first.
    fn show_queue(self: &Rc<Self>) -> Result<()> {
        let (Some(queue), Ok(list)) = (&self.queue, named::<IFrameworkElement>(&self.root, "Queue")) else {
            return Ok(());
        };
        let Ok(tab) = named::<IContentControl>(&self.root, "TabQueue") else { return Ok(()) };
        tab.SetContent(&PropertyValue::CreateString(&queue.label())?)?;
        tab.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)?;
        let filling = Rc::downgrade(self);
        on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |s| s.fill_queue(at, parts)))?;
        on_row_click(&list, queue::go)?;
        add_rows(&list, 0, queue.entries().len())?;
        self.select(Pane::Queue)
    }

    fn fill_queue(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let Some(queue) = &self.queue else { return Ok(()) };
        let Some(entry) = queue.entries().get(at) else { return Ok(()) };
        let title = if at == queue.at() { format!("▶ {}", entry.title) } else { entry.title.clone() };
        parts.GetAt(0)?.cast::<ITextBlock>()?.SetText(&title)?;
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&entry.owner)
    }

    /// The comments load once both the aid is known and the tab has been opened.
    fn load_comments(&self) {
        let aid = self.aid.get();
        if aid > 0 && self.comments_opened.get() {
            let _ = self.comments.show(CommentArea { kind: 1, oid: aid }, self.replies.get());
        }
    }

    fn fill_related(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        if !self.fitted.get() {
            self.fit_rows()?;
        }
        let cards = self.related.borrow();
        let Some(card) = cards.get(at) else { return Ok(()) };
        parts.GetAt(0)?.cast::<IImage>()?.SetSource(&self.covers.get(&card.cover, COVER_DECODE_WIDTH)?)?;
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&card.title)?;
        parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&card.owner.name)
    }
}
