//! A comment area: its comments by heat or by time, each thread drawn as a fishbone tree that
//! opens in full or folds to its first comment, with a box to post a comment or answer one. The
//! video page, the 关注 page and a page of its own all show it.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use rubi_core::{CommentArea, ThreadView};
use rubi_model::{Comment, ThreadRow};
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::comments::{Act, Branch, Foot, Mode, Shape, Thread, body};
use crate::state::library::{age, count};
use crate::ui::card_menu::{self, Choice};
use crate::ui::rows::on_row;
use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load, named, on_click, on_enter, post, set_text};
use crate::xaml::{
    Color, IButtonBase, IContentControl, IControl, IFrameworkElement, IItemsControl, IKeyRoutedEventArgs,
    IListViewBase, IPanel, IPropertyValue, ITextBlock, ITextBox, IUIElement, KeyRoutedEventArgs, PropertyValue,
    RoutedEventArgs, SolidColorBrush, Thickness, UIElement, VirtualKey, Visibility,
};

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
/// Emotes in a comment, square, a little over the text's height.
const EMOTE_SIZE: f64 = 20.0;
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };
/// Rows from the end at which more load.
const LOAD_AHEAD: usize = 6;
/// Step in from one level to the next, and where the first level's text starts.
const LEVEL: f64 = 20.0;
const FIRST_LEVEL: f64 = 22.0;
/// Where a level's line runs within its step, and how far down the tick meets the name.
const LINE_AT: f64 = 6.0;
const TICK_AT: f64 = 16.0;

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
        <Setter Property="VerticalContentAlignment" Value="Stretch" />
        <Setter Property="Padding" Value="16,0" />
        <Setter Property="MinHeight" Value="0" />
      </Style>
    </ListView.ItemContainerStyle>
    <ListView.ItemTemplate>
      <DataTemplate>
        <Grid>
          <Grid IsHitTestVisible="False">
            <Grid.Resources>
              <Style TargetType="Rectangle">
                <Setter Property="Width" Value="1" />
                <Setter Property="HorizontalAlignment" Value="Left" />
                <Setter Property="Visibility" Value="Collapsed" />
                <Setter Property="Fill" Value="{ThemeResource SystemControlForegroundBaseLowBrush}" />
              </Style>
            </Grid.Resources>
            <Rectangle /><Rectangle /><Rectangle /><Rectangle /><Rectangle /><Rectangle />
            <Rectangle /><Rectangle VerticalAlignment="Top" Height="16" />
            <Rectangle VerticalAlignment="Top" Width="10" Height="1" Margin="0,16,0,0" />
          </Grid>
          <StackPanel Padding="0,6,0,6">
            <StackPanel Orientation="Horizontal">
              <TextBlock FontSize="13" FontWeight="SemiBold" Foreground="#0050EF" />
              <Border BorderBrush="#0050EF" BorderThickness="1" Padding="3,0" Margin="8,0,0,0" VerticalAlignment="Center"
                      Visibility="Collapsed"><TextBlock Text="UP" FontSize="10" Foreground="#0050EF" /></Border>
              <Border BorderBrush="#0050EF" BorderThickness="1" Padding="3,0" Margin="8,0,0,0" VerticalAlignment="Center"
                      Visibility="Collapsed"><TextBlock Text="置顶" FontSize="10" Foreground="#0050EF" /></Border>
              <TextBlock FontSize="11" Margin="8,0,0,0" VerticalAlignment="Center"
                         Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
            </StackPanel>
            <RichTextBlock FontSize="14" TextWrapping="Wrap" Margin="0,2,0,0" IsTextSelectionEnabled="True" />
            <StackPanel Orientation="Horizontal" Margin="0,3,0,0">
              <StackPanel.Resources>
                <Style TargetType="Button">
                  <Setter Property="Background" Value="Transparent" />
                  <Setter Property="BorderThickness" Value="0" />
                  <Setter Property="Padding" Value="0" />
                  <Setter Property="MinWidth" Value="0" />
                  <Setter Property="MinHeight" Value="0" />
                  <Setter Property="AllowFocusOnInteraction" Value="False" />
                </Style>
              </StackPanel.Resources>
              <Button><TextBlock FontSize="12" /></Button>
              <TextBlock FontSize="12" Margin="14,0,0,0" VerticalAlignment="Center"
                         Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
              <Button Margin="14,0,0,0"><TextBlock Text="回复" FontSize="12"
                      Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" /></Button>
              <Button Margin="14,0,0,0" Visibility="Collapsed"><TextBlock FontSize="12" Foreground="#0050EF" /></Button>
            </StackPanel>
          </StackPanel>
          <StackPanel Orientation="Horizontal" Margin="22,2,0,10" Visibility="Collapsed">
            <StackPanel.Resources>
              <Style TargetType="Button">
                <Setter Property="Background" Value="Transparent" />
                <Setter Property="BorderThickness" Value="0" />
                <Setter Property="Padding" Value="0" />
                <Setter Property="MinWidth" Value="0" />
                <Setter Property="MinHeight" Value="0" />
                <Setter Property="Margin" Value="0,0,20,0" />
                <Setter Property="AllowFocusOnInteraction" Value="False" />
              </Style>
            </StackPanel.Resources>
            <Button><TextBlock FontSize="13" Foreground="#0050EF" /></Button>
            <Button><TextBlock FontSize="13" Foreground="#0050EF" /></Button>
            <TextBlock Text="正在展开…" FontSize="13"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          </StackPanel>
        </Grid>
      </DataTemplate>
    </ListView.ItemTemplate>
  </ListView>
  <Grid Grid.Row="3" Padding="16,6" BorderThickness="0,1,0,0"
        BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" /></Grid.RowDefinitions>
    <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
    <StackPanel x:Name="ReplyTarget" Grid.ColumnSpan="2" Orientation="Horizontal" Margin="0,0,0,4"
                Visibility="Collapsed">
      <TextBlock Text="回复" FontSize="12" VerticalAlignment="Center"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      <TextBlock x:Name="ReplyName" FontSize="12" FontWeight="SemiBold" Foreground="#0050EF" Margin="6,0,0,0"
                 VerticalAlignment="Center" />
      <Button x:Name="CancelReply" Background="Transparent" BorderThickness="0" Padding="8,0" MinHeight="0"
              VerticalAlignment="Center" AutomationProperties.Name="取消回复"><FontIcon Glyph="&#xE711;" FontSize="10" /></Button>
    </StackPanel>
    <TextBox x:Name="CommentText" Grid.Row="1" PlaceholderText="发一条友善的评论" MaxLength="1000" />
    <Button x:Name="SendComment" Grid.Row="1" Grid.Column="1" Content="发送" Margin="8,0,0,0" Background="#0050EF"
            Foreground="White" BorderThickness="0" Padding="14,5" />
  </Grid>
</Grid>"##;
const PLACEHOLDER: &str = "发一条友善的评论";

/// A comment the list holds: a pinned one on its own, or a thread.
enum Block {
    Pinned(Comment),
    Thread(Thread),
}

impl Block {
    fn thread_of(&self, root: u64) -> Option<&Thread> {
        match self {
            Block::Thread(thread) if thread.root() == root => Some(thread),
            _ => None,
        }
    }
}

/// A row of the list, known by a key the row's item carries, which stays when rows come and go
/// around it.
enum Line {
    Comment { key: i32, root: Option<u64>, row: ThreadRow, comment: Comment, shape: Shape },
    Foot { key: i32, root: u64, foot: Foot },
}

impl Line {
    fn key(&self) -> i32 {
        match self {
            Line::Comment { key, .. } | Line::Foot { key, .. } => *key,
        }
    }
}

/// Who a reply being written answers.
#[derive(Clone)]
struct Target {
    root: u64,
    rpid: u64,
    name: String,
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
    blocks: RefCell<Vec<Block>>,
    /// The rows on show, block after block, and how many each block has.
    lines: RefCell<Vec<Line>>,
    counts: RefCell<Vec<usize>>,
    next_key: Cell<i32>,
    next: RefCell<Option<String>>,
    loading: Cell<bool>,
    sending: Cell<bool>,
    target: RefCell<Option<Target>>,
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
            blocks: RefCell::default(),
            lines: RefCell::default(),
            counts: RefCell::default(),
            next_key: Cell::new(0),
            next: RefCell::default(),
            loading: Cell::new(false),
            sending: Cell::new(false),
            target: RefCell::default(),
            turn: Cell::new(0),
            close: RefCell::default(),
        });
        LISTS.with_borrow_mut(|l| l.insert(shown.id, Rc::downgrade(&shown)));
        let id = shown.id;
        let rows = named::<IFrameworkElement>(&root, "CommentRows")?;
        on_row(&rows, move |at, parts| list_of(id).map_or(Ok(()), |s| s.fill(at, parts)))?;
        card_menu::attach_choices(&rows, move |key| list_of(id).and_then(|s| s.choices(key as i32)))?;
        for (name, by_time) in [("ByHeat", false), ("ByTime", true)] {
            on_click(&root, name, move || {
                if let Some(shown) = list_of(id)
                    && shown.by_time.replace(by_time) != by_time
                {
                    let _ = shown.restart();
                }
            })?;
        }
        on_click(&root, "CloseComments", move || {
            if let Some(shown) = list_of(id)
                && let Some(close) = shown.close.borrow().as_ref()
            {
                close();
            }
        })?;
        let send = move || {
            if let Some(shown) = list_of(id) {
                let _ = shown.send();
            }
        };
        on_click(&root, "SendComment", send.clone())?;
        on_enter(&root, "CommentText", send)?;
        on_click(&root, "CancelReply", move || {
            if let Some(shown) = list_of(id) {
                let _ = shown.set_target(None);
            }
        })?;
        keep(named::<IUIElement>(&root, "CommentText")?.KeyDown(
            move |_: Ref<IInspectable>, args: Ref<KeyRoutedEventArgs>| {
                if let Ok(args) = args.ok().and_then(|a| a.cast::<IKeyRoutedEventArgs>())
                    && args.Key().is_ok_and(|k| k == VirtualKey::Escape)
                    && let Some(shown) = list_of(id)
                    && shown.target.borrow().is_some()
                {
                    let _ = args.SetHandled(true);
                    let _ = shown.set_target(None);
                }
            },
        )?);
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
        if self.area.replace(Some(area)) == Some(area) && !self.blocks.borrow().is_empty() {
            return Ok(());
        }
        self.total.set(total);
        self.restart()
    }

    fn restart(&self) -> Result<()> {
        self.mark()?;
        self.turn.set(self.turn.get() + 1);
        self.blocks.borrow_mut().clear();
        self.lines.borrow_mut().clear();
        self.counts.borrow_mut().clear();
        *self.next.borrow_mut() = None;
        self.loading.set(false);
        self.set_target(None)?;
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
            post(&dispatcher, move || {
                if let Some(shown) = list_of(id).filter(|s| s.turn.get() == turn) {
                    let _ = shown.took(page);
                }
            });
        });
    }

    fn took(&self, page: std::result::Result<rubi_core::CommentsPage, String>) -> Result<()> {
        self.loading.set(false);
        match page {
            Ok(page) => {
                let blocks = page
                    .pinned
                    .into_iter()
                    .map(Block::Pinned)
                    .chain(page.threads.into_iter().map(|view| Block::Thread(Thread::new(view))));
                for block in blocks {
                    self.push_block(block)?;
                }
                *self.next.borrow_mut() = page.next_offset;
                self.say(if self.lines.borrow().is_empty() { "还没有评论" } else { "" });
            }
            Err(problem) => self.say(&format!("评论读取失败：{problem}")),
        }
        Ok(())
    }

    fn key(&self) -> i32 {
        let key = self.next_key.get();
        self.next_key.set(key.wrapping_add(1));
        key
    }

    /// The rows `block` shows.
    fn lines_of(&self, block: &Block) -> Vec<Line> {
        match block {
            Block::Pinned(comment) => {
                let row = ThreadRow { rpid: comment.rpid, depth: 0, has_later_sibling: false, reply_to: None };
                let shape = Shape { depth: 0, later: false, through: Vec::new(), branch: Branch::None };
                vec![Line::Comment { key: self.key(), root: None, row, comment: comment.clone(), shape }]
            }
            Block::Thread(thread) => {
                let root = thread.root();
                let mut lines: Vec<Line> = thread
                    .shown()
                    .into_iter()
                    .map(|(at, shape)| {
                        let (row, comment) = thread.view.rows[at].clone();
                        Line::Comment { key: self.key(), root: Some(root), row, comment, shape }
                    })
                    .collect();
                let foot = thread.foot();
                if foot != Foot::None {
                    lines.push(Line::Foot { key: self.key(), root, foot });
                }
                lines
            }
        }
    }

    fn push_block(&self, block: Block) -> Result<()> {
        let lines = self.lines_of(&block);
        let items = self.items()?;
        for line in &lines {
            items.Append(&PropertyValue::CreateInt32(line.key())?)?;
        }
        self.counts.borrow_mut().push(lines.len());
        self.lines.borrow_mut().extend(lines);
        self.blocks.borrow_mut().push(block);
        Ok(())
    }

    /// Puts `block` in at `at`, among the blocks.
    fn insert_block(&self, at: usize, block: Block) -> Result<()> {
        let start: usize = self.counts.borrow()[..at].iter().sum();
        let lines = self.lines_of(&block);
        let items = self.items()?;
        for (offset, line) in lines.iter().enumerate() {
            items.InsertAt((start + offset) as u32, &PropertyValue::CreateInt32(line.key())?)?;
        }
        self.counts.borrow_mut().insert(at, lines.len());
        self.lines.borrow_mut().splice(start..start, lines);
        self.blocks.borrow_mut().insert(at, block);
        Ok(())
    }

    /// Shows block `at` again after it changed.
    fn redraw(&self, at: usize) -> Result<()> {
        let start: usize = self.counts.borrow()[..at].iter().sum();
        let old = self.counts.borrow()[at];
        let lines = self.lines_of(&self.blocks.borrow()[at]);
        let items = self.items()?;
        for _ in 0..old {
            items.RemoveAt(start as u32)?;
        }
        for (offset, line) in lines.iter().enumerate() {
            items.InsertAt((start + offset) as u32, &PropertyValue::CreateInt32(line.key())?)?;
        }
        self.counts.borrow_mut()[at] = lines.len();
        self.lines.borrow_mut().splice(start..start + old, lines);
        Ok(())
    }

    fn thread_at(&self, root: u64) -> Option<usize> {
        self.blocks.borrow().iter().position(|b| b.thread_of(root).is_some())
    }

    /// The block holding comment `rpid`.
    fn block_with(&self, rpid: u64) -> Option<usize> {
        self.blocks.borrow().iter().position(|b| match b {
            Block::Pinned(c) => c.rpid == rpid,
            Block::Thread(t) => t.view.rows.iter().any(|(row, _)| row.rpid == rpid),
        })
    }

    fn line_by_key(&self, key: i32) -> Option<usize> {
        self.lines.borrow().iter().position(|l| l.key() == key)
    }

    fn fill(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        self.wire(parts)?;
        let lines = self.lines.borrow();
        let Some(line) = lines.get(at) else { return Ok(()) };
        let (lines_part, body_part, foot_part) =
            (parts.GetAt(0)?.cast::<IPanel>()?, parts.GetAt(1)?.cast::<IPanel>()?, parts.GetAt(2)?.cast::<IPanel>()?);
        match line {
            Line::Comment { root, row, comment, shape, .. } => {
                draw_lines(&lines_part, shape)?;
                body_part.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)?;
                foot_part.cast::<IUIElement>()?.SetVisibility(Visibility::Collapsed)?;
                let indent = if shape.depth == 0 { 0.0 } else { f64::from(shape.depth - 1) * LEVEL + FIRST_LEVEL };
                body_part.cast::<IFrameworkElement>()?.SetMargin(Thickness { left: indent, ..Thickness::default() })?;
                let total = root
                    .filter(|_| shape.depth == 0)
                    .and_then(|r| {
                        self.blocks.borrow().iter().find_map(|b| b.thread_of(r).map(|t| t.view.total_replies))
                    })
                    .unwrap_or(0);
                fill_comment(&body_part, row, comment, shape.branch, root.is_none(), total)?;
            }
            Line::Foot { foot, .. } => {
                draw_lines(&lines_part, &Shape { depth: 0, later: false, through: Vec::new(), branch: Branch::None })?;
                body_part.cast::<IUIElement>()?.SetVisibility(Visibility::Collapsed)?;
                foot_part.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)?;
                fill_foot(&foot_part, foot)?;
            }
        }
        let near_end = at + LOAD_AHEAD >= lines.len();
        drop(lines);
        if near_end && self.next.borrow().is_some() {
            self.load();
        }
        Ok(())
    }

    /// Wires the buttons of a row the first time it is filled: rows are reused, and a button
    /// finds its row by the key its item carries.
    fn wire(&self, parts: &IVector<UIElement>) -> Result<()> {
        let marker = parts.GetAt(0)?.cast::<IFrameworkElement>()?;
        if marker.Tag().is_ok_and(|t| t.cast::<IPropertyValue>().is_ok()) {
            return Ok(());
        }
        marker.SetTag(&PropertyValue::CreateInt32(1)?)?;
        let id = self.id;
        let meta = children(&children(&parts.GetAt(1)?.cast::<IPanel>()?)?.GetAt(2)?.cast::<IPanel>()?)?;
        let foot = children(&parts.GetAt(2)?.cast::<IPanel>()?)?;
        let buttons = [
            (meta.GetAt(0)?, Press::Like),
            (meta.GetAt(2)?, Press::Reply),
            (meta.GetAt(3)?, Press::Branch),
            (foot.GetAt(0)?, Press::Foot(0)),
            (foot.GetAt(1)?, Press::Foot(1)),
        ];
        for (button, press) in buttons {
            keep(button.cast::<IButtonBase>()?.Click(move |sender: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                let key = sender
                    .ok()
                    .and_then(|s| s.cast::<IFrameworkElement>()?.DataContext()?.cast::<IPropertyValue>()?.GetInt32());
                if let (Ok(key), Some(shown)) = (key, list_of(id)) {
                    let _ = shown.pressed(key, press);
                }
            })?);
        }
        Ok(())
    }

    fn pressed(&self, key: i32, press: Press) -> Result<()> {
        let Some(at) = self.line_by_key(key) else { return Ok(()) };
        let lines = self.lines.borrow();
        match (&lines[at], press) {
            (Line::Comment { row, comment, .. }, Press::Like) => {
                let (rpid, on) = (row.rpid, !comment.liked);
                drop(lines);
                self.like(rpid, on)
            }
            (Line::Comment { root: Some(root), row, .. }, Press::Branch) => {
                let (root, rpid) = (*root, row.rpid);
                drop(lines);
                let Some(at) = self.thread_at(root) else { return Ok(()) };
                if let Some(Block::Thread(thread)) = self.blocks.borrow_mut().get_mut(at) {
                    thread.toggle_branch(rpid);
                }
                self.redraw(at)
            }
            (Line::Comment { root, row, comment, .. }, Press::Reply) => {
                let target = Target { root: root.unwrap_or(row.rpid), rpid: row.rpid, name: comment.name.clone() };
                drop(lines);
                self.reply_to(target)
            }
            (Line::Foot { root, foot: Foot::Acts(acts), .. }, Press::Foot(which)) => {
                let (root, act) = (*root, acts.get(which).map(|(a, _)| *a));
                drop(lines);
                match act {
                    Some(act) => self.act(root, act),
                    None => Ok(()),
                }
            }
            _ => Ok(()),
        }
    }

    /// The menu for the row with `key`: none for a thread's foot.
    fn choices(&self, key: i32) -> Option<Vec<Choice>> {
        let at = self.line_by_key(key)?;
        let lines = self.lines.borrow();
        let Line::Comment { root, row, comment, .. } = &lines[at] else { return None };
        let (id, rpid, liked, text) = (self.id, row.rpid, comment.liked, comment.text.clone());
        let target = Target { root: root.unwrap_or(row.rpid), rpid, name: comment.name.clone() };
        Some(vec![
            (
                "回复".to_owned(),
                Rc::new(move || {
                    if let Some(shown) = list_of(id) {
                        let _ = shown.reply_to(target.clone());
                    }
                }),
            ),
            (
                if liked { "取消点赞" } else { "点赞" }.to_owned(),
                Rc::new(move || {
                    if let Some(shown) = list_of(id) {
                        let _ = shown.like(rpid, !liked);
                    }
                }),
            ),
            ("复制".to_owned(), Rc::new(move || card_menu::copy_text(&text, "已复制评论"))),
        ])
    }

    fn logged_in(&self, refused: &str) -> bool {
        let signed = shell().and_then(|s| s.backend.get().cloned()).is_some_and(|b| b.services.account.is_logged_in());
        if !signed {
            toast(refused);
        }
        signed
    }

    fn reply_to(&self, target: Target) -> Result<()> {
        if !self.logged_in("登录后才能回复") {
            return Ok(());
        }
        self.set_target(Some(target))?;
        named::<IControl>(&self.root, "CommentText")?.Focus(crate::xaml::FocusState::Programmatic)?;
        Ok(())
    }

    fn set_target(&self, target: Option<Target>) -> Result<()> {
        let placeholder = target.as_ref().map_or(PLACEHOLDER.to_owned(), |t| format!("回复 @{}", t.name));
        set_text(&self.root, "ReplyName", &target.as_ref().map_or(String::new(), |t| format!("@{}", t.name)))?;
        named::<IUIElement>(&self.root, "ReplyTarget")?.SetVisibility(if target.is_some() {
            Visibility::Visible
        } else {
            Visibility::Collapsed
        })?;
        named::<crate::xaml::ITextBox2>(&self.root, "CommentText")?.SetPlaceholderText(&placeholder)?;
        *self.target.borrow_mut() = target;
        Ok(())
    }

    /// Opens, reads or folds a thread.
    fn act(&self, root: u64, act: Act) -> Result<()> {
        let Some(at) = self.thread_at(root) else { return Ok(()) };
        match act {
            Act::Collapse => self.set_mode(at, Mode::Collapsed),
            Act::Expand => self.set_mode(at, Mode::Open),
            Act::More => {
                let reading = match self.blocks.borrow_mut().get_mut(at) {
                    Some(Block::Thread(thread)) if thread.needs_reading() => true,
                    Some(Block::Thread(thread)) => {
                        thread.more();
                        false
                    }
                    _ => return Ok(()),
                };
                if !reading {
                    return self.redraw(at);
                }
                self.set_mode(at, Mode::Loading)?;
                let (Some(shell), Some(area)) = (shell(), self.area.get()) else { return Ok(()) };
                let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
                let (id, turn, dispatcher, core) =
                    (self.id, self.turn.get(), shell.dispatcher.clone(), backend.clone());
                backend.spawn(async move {
                    let read = core.services.comments.thread(area, root, usize::MAX).await.map_err(|e| e.to_string());
                    post(&dispatcher, move || {
                        if let Some(shown) = list_of(id).filter(|s| s.turn.get() == turn) {
                            let _ = shown.read_whole(root, read);
                        }
                    });
                });
                Ok(())
            }
        }
    }

    fn set_mode(&self, at: usize, mode: Mode) -> Result<()> {
        if let Some(Block::Thread(thread)) = self.blocks.borrow_mut().get_mut(at) {
            thread.mode = mode;
        }
        self.redraw(at)
    }

    fn read_whole(&self, root: u64, read: std::result::Result<ThreadView, String>) -> Result<()> {
        let Some(at) = self.thread_at(root) else { return Ok(()) };
        if let Some(Block::Thread(thread)) = self.blocks.borrow_mut().get_mut(at) {
            match read {
                Ok(view) => thread.read_whole(view),
                Err(problem) => {
                    thread.mode = Mode::Open;
                    toast(&format!("展开失败：{problem}"));
                }
            }
        }
        self.redraw(at)
    }

    /// Likes `rpid` (or takes it back) at once, and undoes it if B站 says no.
    fn like(&self, rpid: u64, on: bool) -> Result<()> {
        if !self.logged_in("登录后才能点赞") {
            return Ok(());
        }
        self.mark_liked(rpid, on)?;
        let (Some(shell), Some(area)) = (shell(), self.area.get()) else { return Ok(()) };
        let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
        let (id, dispatcher, core) = (self.id, shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let done = core.services.comments.like(area, rpid, on).await.map_err(|e| e.to_string());
            if let Err(problem) = done {
                post(&dispatcher, move || {
                    toast(&format!("点赞失败：{problem}"));
                    if let Some(shown) = list_of(id) {
                        let _ = shown.mark_liked(rpid, !on);
                    }
                });
            }
        });
        Ok(())
    }

    fn mark_liked(&self, rpid: u64, on: bool) -> Result<()> {
        let Some(at) = self.block_with(rpid) else { return Ok(()) };
        {
            let mut blocks = self.blocks.borrow_mut();
            let comment = match &mut blocks[at] {
                Block::Pinned(c) => Some(c),
                Block::Thread(t) => t.view.rows.iter_mut().map(|(_, c)| c).find(|c| c.rpid == rpid),
            };
            if let Some(c) = comment
                && c.liked != on
            {
                c.liked = on;
                c.likes = if on { c.likes + 1 } else { c.likes.saturating_sub(1) };
            }
        }
        self.redraw(at)
    }

    /// Posts the typed text: an answer to the comment being replied to, or a new comment.
    fn send(&self) -> Result<()> {
        let (Some(shell), Some(area)) = (shell(), self.area.get()) else { return Ok(()) };
        let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
        let text = named::<ITextBox>(&self.root, "CommentText")?.Text()?.to_string();
        if text.trim().is_empty() {
            toast("先写点什么");
            return Ok(());
        }
        let target = self.target.borrow().clone();
        if !self.logged_in(if target.is_some() { "登录后才能回复" } else { "登录后才能评论" }) {
            return Ok(());
        }
        if self.sending.replace(true) {
            return Ok(());
        }
        let (id, dispatcher, core) = (self.id, shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let (root, parent) = target.as_ref().map_or((0, 0), |t| (t.root, t.rpid));
            let sent = core.services.comments.reply(area, root, parent, &text, &[]).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(shown) = list_of(id) {
                    let _ = shown.sent(target, sent.map(|(_, comment)| comment));
                }
            });
        });
        Ok(())
    }

    fn sent(&self, target: Option<Target>, sent: std::result::Result<Comment, String>) -> Result<()> {
        self.sending.set(false);
        let comment = match sent {
            Ok(comment) => comment,
            Err(problem) => {
                toast(&format!("没有发出去：{problem}"));
                return Ok(());
            }
        };
        named::<ITextBox>(&self.root, "CommentText")?.SetText("")?;
        let rpid = comment.rpid;
        match target {
            Some(target) => {
                let Some(at) = self.thread_at(target.root) else { return Ok(()) };
                if let Some(Block::Thread(thread)) = self.blocks.borrow_mut().get_mut(at) {
                    thread.add_reply(target.rpid, comment);
                }
                self.set_target(None)?;
                self.redraw(at)?;
                toast("已回复");
            }
            None => {
                let row = ThreadRow { rpid, depth: 0, has_later_sibling: false, reply_to: None };
                let view = ThreadView { root_rpid: rpid, rows: vec![(row, comment)], hidden: 0, total_replies: 0 };
                let after_pinned = self.blocks.borrow().iter().take_while(|b| matches!(b, Block::Pinned(_))).count();
                self.insert_block(after_pinned, Block::Thread(Thread::new(view)))?;
                self.total.set(self.total.get() + 1);
                self.mark()?;
                self.say("");
                toast("已发送");
            }
        }
        self.show_comment(rpid)
    }

    /// Scrolls to the row of comment `rpid`.
    fn show_comment(&self, rpid: u64) -> Result<()> {
        let key = self.lines.borrow().iter().find_map(|l| match l {
            Line::Comment { key, row, .. } if row.rpid == rpid => Some(*key),
            _ => None,
        });
        let Some(key) = key else { return Ok(()) };
        let Some(at) = self.line_by_key(key) else { return Ok(()) };
        let item = self.items()?.GetAt(at as u32)?;
        named::<IListViewBase>(&self.root, "CommentRows")?.ScrollIntoView(&item)
    }
}

/// A button of a row, and what pressing it means.
#[derive(Clone, Copy)]
enum Press {
    Like,
    Reply,
    /// Folds a reply's answers away, or shows them.
    Branch,
    /// The thread foot's first or second action.
    Foot(usize),
}

fn children(panel: &IPanel) -> Result<IVector<UIElement>> {
    panel.Children()?.cast()
}

fn show(element: &UIElement, on: bool) -> Result<()> {
    element.cast::<IUIElement>()?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })
}

/// The fishbone lines of a row: the levels passing by, its own line down from its parent, and the
/// tick to its name.
fn draw_lines(panel: &IPanel, shape: &Shape) -> Result<()> {
    let rects = children(panel)?;
    let x = |level: u32| f64::from(level - 1) * LEVEL + LINE_AT;
    for level in 1..=6u32 {
        let rect = rects.GetAt(level - 1)?;
        let through = shape.through.contains(&level);
        if through {
            rect.cast::<IFrameworkElement>()?.SetMargin(Thickness { left: x(level), ..Thickness::default() })?;
        }
        show(&rect, through)?;
    }
    let (down, stub, tick) = (rects.GetAt(6)?, rects.GetAt(7)?, rects.GetAt(8)?);
    let own = shape.depth > 0;
    for (rect, on, top) in [(&down, own && shape.later, 0.0), (&stub, own && !shape.later, 0.0), (&tick, own, TICK_AT)]
    {
        if on {
            let margin = Thickness { left: x(shape.depth), top, ..Thickness::default() };
            rect.cast::<IFrameworkElement>()?.SetMargin(margin)?;
        }
        show(rect, on)?;
    }
    Ok(())
}

fn fill_comment(
    body_part: &IPanel,
    row: &ThreadRow,
    comment: &Comment,
    branch: Branch,
    pinned: bool,
    replies: u32,
) -> Result<()> {
    let parts = children(body_part)?;
    let head = children(&parts.GetAt(0)?.cast::<IPanel>()?)?;
    head.GetAt(0)?.cast::<ITextBlock>()?.SetText(&comment.name)?;
    show(&head.GetAt(1)?, comment.is_up)?;
    show(&head.GetAt(2)?, pinned)?;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    head.GetAt(3)?.cast::<ITextBlock>()?.SetText(&age(comment.ctime, now))?;
    crate::ui::emote_text::show(&parts.GetAt(1)?.cast()?, &body(row, comment), &comment.emotes, EMOTE_SIZE)?;
    let meta = children(&parts.GetAt(2)?.cast::<IPanel>()?)?;
    let like = meta.GetAt(0)?.cast::<IContentControl>()?.Content()?.cast::<ITextBlock>()?;
    like.SetText(&format!("{} {}", if comment.liked { "已赞" } else { "赞" }, count(comment.likes)))?;
    like.SetForeground(&SolidColorBrush::CreateInstanceWithColor(if comment.liked { COBALT } else { DIM })?)?;
    let replied = meta.GetAt(1)?;
    replied.cast::<ITextBlock>()?.SetText(&format!("{replies} 条回复"))?;
    show(&replied, replies > 0)?;
    let fold = meta.GetAt(3)?;
    let label = match branch {
        Branch::None => None,
        Branch::Open(_) => Some("折叠".to_owned()),
        Branch::Folded(n) => Some(format!("展开 {n} 条")),
    };
    if let Some(label) = &label {
        fold.cast::<IContentControl>()?.Content()?.cast::<ITextBlock>()?.SetText(label)?;
    }
    show(&fold, label.is_some())
}

fn fill_foot(foot_part: &IPanel, foot: &Foot) -> Result<()> {
    let parts = children(foot_part)?;
    let acts: &[(Act, String)] = match foot {
        Foot::Acts(acts) => acts,
        _ => &[],
    };
    for slot in 0..2 {
        let button = parts.GetAt(slot)?;
        match acts.get(slot as usize) {
            Some((_, label)) => {
                button.cast::<IContentControl>()?.Content()?.cast::<ITextBlock>()?.SetText(label)?;
                show(&button, true)?;
            }
            None => show(&button, false)?,
        }
    }
    show(&parts.GetAt(2)?, *foot == Foot::Loading)
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
    named::<IContentControl>(&page, "Host")?.SetContent(&list.element().cast::<UIElement>()?)?;
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
