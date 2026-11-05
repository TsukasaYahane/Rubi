//! The video page's 推荐 / 评论: beside the video on a wide window, under it otherwise.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rubi_core::CommentArea;
use rubi_model::{Comment, VideoCard};
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::panes::{SideLayout, side_layout};
use crate::ui::feed;
use crate::ui::keep;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{load_object, named, on_click, player, post};
use crate::xaml::{
    DataTemplate, IControl, IFrameworkElement, IImage, IItemsControl, IItemsControl2, IItemsWrapGrid, ITextBlock,
    IUIElement, SizeChangedEventArgs, SolidColorBrush, UIElement, Visibility,
};

const COVER_DECODE_WIDTH: i32 = 320;
/// Rows from the end at which more comments load.
const LOAD_AHEAD: usize = 6;

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

/// A comment as a row: replies carry how deep they sit.
struct CommentRow {
    depth: u32,
    comment: Comment,
}

pub struct Side {
    root: IFrameworkElement,
    aid: Cell<u64>,
    related: RefCell<Vec<VideoCard>>,
    comments: RefCell<Vec<CommentRow>>,
    next_comments: RefCell<Option<String>>,
    comments_opened: Cell<bool>,
    loading: Cell<bool>,
    /// The row template in use, so it changes only when the pane crosses a width.
    large_rows: Cell<Option<bool>>,
    /// The list's panel exists only once it has rows: until then the fit has to be tried again.
    fitted: Cell<bool>,
    covers: Covers,
    /// The layout the rows were last set to.
    layout: Cell<Option<SideLayout>>,
}

impl Side {
    pub fn attach(root: &IFrameworkElement) -> Result<Rc<Self>> {
        let side = Rc::new(Self {
            root: root.clone(),
            aid: Cell::new(0),
            related: RefCell::default(),
            comments: RefCell::default(),
            next_comments: RefCell::new(None),
            comments_opened: Cell::new(false),
            loading: Cell::new(false),
            large_rows: Cell::new(None),
            fitted: Cell::new(false),
            covers: Covers::default(),
            layout: Cell::new(None),
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

        let comments = named::<IFrameworkElement>(root, "Comments")?;
        let filling = Rc::downgrade(&side);
        on_row(&comments, move |at, parts| filling.upgrade().map_or(Ok(()), |s| s.fill_comment(at, parts)))?;

        let tab = Rc::downgrade(&side);
        on_click(root, "TabRelated", move || {
            if let Some(side) = tab.upgrade() {
                let _ = side.show_tab(false);
            }
        })?;
        let tab = Rc::downgrade(&side);
        on_click(root, "TabComments", move || {
            if let Some(side) = tab.upgrade() {
                let _ = side.show_tab(true);
            }
        })?;
        side.mark_tabs(false)?;
        Ok(side)
    }

    /// The video's aid is known: comments can load.
    pub fn set_aid(&self, aid: u64) {
        self.aid.set(aid);
        if self.comments_opened.get() {
            self.load_comments();
        }
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

    pub fn show_tab(&self, comments: bool) -> Result<()> {
        let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
        named::<IUIElement>(&self.root, "Related")?.SetVisibility(shown(!comments))?;
        named::<IUIElement>(&self.root, "Comments")?.SetVisibility(shown(comments))?;
        self.mark_tabs(comments)?;
        if comments && !self.comments_opened.replace(true) {
            self.load_comments();
        }
        Ok(())
    }

    fn mark_tabs(&self, comments: bool) -> Result<()> {
        let tint = |on| {
            use crate::xaml::Color;
            let color =
                if on { Color { a: 255, r: 0, g: 0x50, b: 0xEF } } else { Color { a: 255, r: 0x76, g: 0x76, b: 0x76 } };
            SolidColorBrush::CreateInstanceWithColor(color)
        };
        named::<IControl>(&self.root, "TabRelated")?.SetForeground(&tint(!comments)?)?;
        named::<IControl>(&self.root, "TabComments")?.SetForeground(&tint(comments)?)
    }

    fn load_comments(&self) {
        let (Some(shell), aid) = (shell(), self.aid.get()) else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if aid == 0 || self.loading.replace(true) {
            return;
        }
        let offset = self.next_comments.borrow().clone();
        let dispatcher = shell.dispatcher.clone();
        let core = backend.clone();
        let run = player::run();
        backend.spawn(async move {
            let area = CommentArea { kind: 1, oid: aid };
            let page = core.services.comments.page(area, false, offset.as_deref()).await.map_err(|e| e.to_string());
            let page = page.map(|p| {
                let pinned = p.pinned.into_iter().map(|comment| (0, comment));
                let threads = p.threads.into_iter().flat_map(|t| t.rows).map(|(row, comment)| (row.depth, comment));
                (pinned.chain(threads).collect::<Vec<_>>(), p.next_offset)
            });
            post(&dispatcher, move || {
                if let Some(side) = player::side(run) {
                    side.comments_loaded(page);
                }
            });
        });
    }

    fn comments_loaded(&self, page: std::result::Result<(Vec<(u32, Comment)>, Option<String>), String>) {
        self.loading.set(false);
        match page {
            Ok((rows, next)) => {
                let from = self.comments.borrow().len();
                self.comments
                    .borrow_mut()
                    .extend(rows.into_iter().map(|(depth, comment)| CommentRow { depth, comment }));
                *self.next_comments.borrow_mut() = next;
                let to = self.comments.borrow().len();
                if let Ok(list) = named(&self.root, "Comments") {
                    let _ = add_rows(&list, from, to);
                }
            }
            Err(problem) => player::say(&format!("评论读取失败：{problem}")),
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

    fn fill_comment(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let rows = self.comments.borrow();
        let Some(row) = rows.get(at) else { return Ok(()) };
        let name = if row.depth > 0 { format!("↳ {}", row.comment.name) } else { row.comment.name.clone() };
        parts.GetAt(0)?.cast::<ITextBlock>()?.SetText(&name)?;
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&row.comment.text)?;
        parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&format!("赞 {}", row.comment.likes))?;
        let near_end = at + LOAD_AHEAD >= rows.len();
        drop(rows);
        if near_end && self.next_comments.borrow().is_some() {
            self.load_comments();
        }
        Ok(())
    }
}
