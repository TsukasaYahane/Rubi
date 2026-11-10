//! A comment area: its comments by heat or by time, with a box to post a new one. The video page,
//! the 关注 page and a page of its own all show it.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use rubi_core::CommentArea;
use rubi_model::Comment;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Result};

use crate::ui::rows::{add_rows, on_row};
use crate::ui::shell::shell;
use crate::ui::{load, named, on_click, on_enter, post, set_text};
use crate::xaml::{
    Color, IControl, IFrameworkElement, IItemsControl, ITextBlock, ITextBox, IUIElement, SolidColorBrush, UIElement,
    Visibility,
};

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };
/// Rows from the end at which more load.
const LOAD_AHEAD: usize = 6;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" /><RowDefinition Height="Auto" />
  </Grid.RowDefinitions>
  <Grid Margin="16,6,8,2">
    <Grid.ColumnDefinitions>
      <ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" />
      <ColumnDefinition Width="Auto" />
    </Grid.ColumnDefinitions>
    <TextBlock x:Name="CommentCount" FontSize="16" VerticalAlignment="Center" />
    <Button x:Name="ByHeat" Grid.Column="1" Content="按热度" Background="Transparent" BorderThickness="0" Padding="6,2"
            FontSize="12" />
    <Button x:Name="ByTime" Grid.Column="2" Content="按时间" Background="Transparent" BorderThickness="0" Padding="6,2"
            FontSize="12" />
    <Button x:Name="CloseComments" Grid.Column="3" Visibility="Collapsed" Background="Transparent" BorderThickness="0"
            Padding="8,2"><FontIcon Glyph="&#xE711;" FontSize="12" /></Button>
  </Grid>
  <TextBlock x:Name="CommentStatus" Grid.Row="1" FontSize="12" Margin="16,2" TextWrapping="Wrap"
             Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  <ListView x:Name="CommentRows" Grid.Row="2" SelectionMode="None">
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
          <TextBlock FontSize="12" Foreground="#0050EF" />
          <TextBlock FontSize="14" TextWrapping="Wrap" Margin="0,3,0,0" IsTextSelectionEnabled="True" />
          <TextBlock FontSize="11" Margin="0,3,0,0"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </StackPanel>
      </DataTemplate>
    </ListView.ItemTemplate>
  </ListView>
  <Grid Grid.Row="3" Padding="16,6" BorderThickness="0,1,0,0"
        BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
    <TextBox x:Name="CommentText" PlaceholderText="发一条友善的评论" MaxLength="1000" />
    <Button x:Name="SendComment" Grid.Column="1" Content="发送" Margin="8,0,0,0" Background="#0050EF"
            Foreground="White" BorderThickness="0" Padding="14,5" />
  </Grid>
</Grid>"##;

/// A comment as a row: replies carry how deep they sit.
struct Row {
    depth: u32,
    comment: Comment,
}

thread_local! {
    static LISTS: RefCell<HashMap<u64, Weak<CommentList>>> = RefCell::default();
}

fn list_of(id: u64) -> Option<Rc<CommentList>> {
    LISTS.with_borrow(|l| l.get(&id).and_then(Weak::upgrade))
}

pub struct CommentList {
    id: u64,
    root: IFrameworkElement,
    area: Cell<Option<CommentArea>>,
    /// How many the area holds, as its owner knows it.
    total: Cell<u64>,
    by_time: Cell<bool>,
    rows: RefCell<Vec<Row>>,
    next: RefCell<Option<String>>,
    loading: Cell<bool>,
    sending: Cell<bool>,
    /// Bumped when the list starts over: answers for an earlier one are dropped.
    turn: Cell<u64>,
    close: RefCell<Option<Box<dyn Fn()>>>,
}

impl CommentList {
    pub fn new() -> Result<Rc<Self>> {
        let root = load(LAYOUT)?;
        let shown = Rc::new(Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            root: root.clone(),
            area: Cell::new(None),
            total: Cell::new(0),
            by_time: Cell::new(false),
            rows: RefCell::default(),
            next: RefCell::default(),
            loading: Cell::new(false),
            sending: Cell::new(false),
            turn: Cell::new(0),
            close: RefCell::default(),
        });
        LISTS.with_borrow_mut(|l| l.insert(shown.id, Rc::downgrade(&shown)));
        let filling = Rc::downgrade(&shown);
        on_row(&named(&root, "CommentRows")?, move |at, parts| {
            filling.upgrade().map_or(Ok(()), |s| s.fill(at, parts))
        })?;
        for (name, by_time) in [("ByHeat", false), ("ByTime", true)] {
            let weak = Rc::downgrade(&shown);
            on_click(&root, name, move || {
                if let Some(shown) = weak.upgrade()
                    && shown.by_time.replace(by_time) != by_time
                {
                    let _ = shown.restart();
                }
            })?;
        }
        let weak = Rc::downgrade(&shown);
        on_click(&root, "CloseComments", move || {
            if let Some(shown) = weak.upgrade()
                && let Some(close) = shown.close.borrow().as_ref()
            {
                close();
            }
        })?;
        let weak = Rc::downgrade(&shown);
        let send = move || {
            if let Some(shown) = weak.upgrade() {
                let _ = shown.send();
            }
        };
        on_click(&root, "SendComment", send.clone())?;
        on_enter(&root, "CommentText", send)?;
        shown.mark()?;
        Ok(shown)
    }

    pub fn element(&self) -> &IFrameworkElement {
        &self.root
    }

    /// Offers a close button that does `close`.
    pub fn set_close(&self, close: impl Fn() + 'static) -> Result<()> {
        *self.close.borrow_mut() = Some(Box::new(close));
        named::<IUIElement>(&self.root, "CloseComments")?.SetVisibility(Visibility::Visible)
    }

    /// Shows `area`, which holds about `total` comments (0 when not known).
    pub fn show(&self, area: CommentArea, total: u64) -> Result<()> {
        if self.area.replace(Some(area)) == Some(area) && !self.rows.borrow().is_empty() {
            return Ok(());
        }
        self.total.set(total);
        self.restart()
    }

    fn restart(&self) -> Result<()> {
        self.mark()?;
        self.turn.set(self.turn.get() + 1);
        self.rows.borrow_mut().clear();
        *self.next.borrow_mut() = None;
        self.loading.set(false);
        self.items()?.Clear()?;
        self.say("正在读取…");
        self.load();
        Ok(())
    }

    fn items(&self) -> Result<IVector<IInspectable>> {
        named::<IItemsControl>(&self.root, "CommentRows")?.Items()?.cast()
    }

    fn say(&self, text: &str) {
        let _ = set_text(&self.root, "CommentStatus", text);
        let _ = named::<IUIElement>(&self.root, "CommentStatus")
            .and_then(|s| s.SetVisibility(if text.is_empty() { Visibility::Collapsed } else { Visibility::Visible }));
    }

    fn mark(&self) -> Result<()> {
        let total = self.total.get();
        set_text(&self.root, "CommentCount", &if total > 0 { format!("评论 {total}") } else { "评论".to_owned() })?;
        for (name, on) in [("ByHeat", !self.by_time.get()), ("ByTime", self.by_time.get())] {
            let brush = SolidColorBrush::CreateInstanceWithColor(if on { COBALT } else { DIM })?;
            named::<IControl>(&self.root, name)?.SetForeground(&brush)?;
        }
        Ok(())
    }

    fn load(&self) {
        let (Some(shell), Some(area)) = (shell(), self.area.get()) else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.loading.replace(true) {
            return;
        }
        let (id, turn, by_time, offset) = (self.id, self.turn.get(), self.by_time.get(), self.next.borrow().clone());
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let page = core.services.comments.page(area, by_time, offset.as_deref()).await.map_err(|e| e.to_string());
            let page = page.map(|p| {
                let pinned = p.pinned.into_iter().map(|comment| (0, comment));
                let threads = p.threads.into_iter().flat_map(|t| t.rows).map(|(row, comment)| (row.depth, comment));
                (pinned.chain(threads).collect::<Vec<_>>(), p.next_offset)
            });
            post(&dispatcher, move || {
                if let Some(shown) = list_of(id).filter(|s| s.turn.get() == turn) {
                    let _ = shown.took(page);
                }
            });
        });
    }

    fn took(&self, page: std::result::Result<(Vec<(u32, Comment)>, Option<String>), String>) -> Result<()> {
        self.loading.set(false);
        match page {
            Ok((rows, next)) => {
                let from = self.rows.borrow().len();
                self.rows.borrow_mut().extend(rows.into_iter().map(|(depth, comment)| Row { depth, comment }));
                *self.next.borrow_mut() = next;
                let to = self.rows.borrow().len();
                add_rows(&named(&self.root, "CommentRows")?, from, to)?;
                self.say(if to == 0 { "还没有评论" } else { "" });
            }
            Err(problem) => self.say(&format!("评论读取失败：{problem}")),
        }
        Ok(())
    }

    fn fill(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let rows = self.rows.borrow();
        let Some(row) = rows.get(at) else { return Ok(()) };
        let comment = &row.comment;
        let name = if row.depth > 0 { format!("↳ {}", comment.name) } else { comment.name.clone() };
        parts.GetAt(0)?.cast::<ITextBlock>()?.SetText(&name)?;
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&comment.text)?;
        let replies = if row.depth == 0 && comment.reply_count > 0 {
            format!(" · {} 条回复", comment.reply_count)
        } else {
            String::new()
        };
        parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&format!("赞 {}{replies}", comment.likes))?;
        let near_end = at + LOAD_AHEAD >= rows.len();
        drop(rows);
        if near_end && self.next.borrow().is_some() {
            self.load();
        }
        Ok(())
    }

    /// Posts the typed comment under the area; it shows at the top once B站 has it.
    fn send(&self) -> Result<()> {
        let (Some(shell), Some(area)) = (shell(), self.area.get()) else { return Ok(()) };
        let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
        let text = named::<ITextBox>(&self.root, "CommentText")?.Text()?.to_string();
        if text.trim().is_empty() {
            self.say("先写点什么");
            return Ok(());
        }
        if !backend.services.account.is_logged_in() {
            self.say("登录后才能评论");
            return Ok(());
        }
        if self.sending.replace(true) {
            return Ok(());
        }
        self.say("正在发送…");
        let (id, dispatcher, core) = (self.id, shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let sent = core.services.comments.reply(area, 0, 0, &text, &[]).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(shown) = list_of(id) {
                    let _ = shown.sent(sent.map(|(_, comment)| comment));
                }
            });
        });
        Ok(())
    }

    fn sent(&self, sent: std::result::Result<Comment, String>) -> Result<()> {
        self.sending.set(false);
        match sent {
            Ok(comment) => {
                named::<ITextBox>(&self.root, "CommentText")?.SetText("")?;
                self.rows.borrow_mut().insert(0, Row { depth: 0, comment });
                self.total.set(self.total.get() + 1);
                self.mark()?;
                let count = self.rows.borrow().len();
                self.items()?.Clear()?;
                add_rows(&named(&self.root, "CommentRows")?, 0, count)?;
                self.say("已发送");
            }
            Err(problem) => self.say(&format!("没有发出去：{problem}")),
        }
        Ok(())
    }
}

fn go_back() {
    if let Some(shell) = shell() {
        shell.back();
    }
}

/// A page of its own for `area`, with a way back.
pub fn open_page(area: CommentArea, total: u64) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let page = load(PAGE)?;
    let list = CommentList::new()?;
    named::<crate::xaml::IContentControl>(&page, "Host")?.SetContent(&list.element().cast::<UIElement>()?)?;
    on_click(&page, "Back", go_back)?;
    list.show(area, total)?;
    shell.set_page(&page)?;
    PAGE_LIST.with_borrow_mut(|p| *p = Some(list));
    Ok(())
}

thread_local! {
    /// The list of the comment page on show.
    static PAGE_LIST: RefCell<Option<Rc<CommentList>>> = const { RefCell::new(None) };
}

const PAGE: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <Button x:Name="Back" Background="Transparent" BorderThickness="0" Padding="12,8">
    <StackPanel Orientation="Horizontal">
      <FontIcon Glyph="&#xE72B;" FontSize="16" /><TextBlock Text="返回" Margin="10,0,0,0" />
    </StackPanel>
  </Button>
  <ContentControl x:Name="Host" Grid.Row="1" IsTabStop="False" HorizontalContentAlignment="Stretch"
                  VerticalContentAlignment="Stretch" MaxWidth="760" />
</Grid>"##;
