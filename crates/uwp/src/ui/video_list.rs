//! A page of videos: 历史记录, 稍后再看, or a 收藏夹's contents. Loads as it scrolls, and the ones that
//! allow it can be ticked and taken out.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::backend::Backend;
use crate::state::library::{Row, Shelf, row_size, video_link};
use crate::state::queue::Entry;
use crate::ui::card_menu::{self, Choice};
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{feed, keep, load, named, on_click, post, queue, set_text};
use crate::xaml::{
    IContentControl, IFrameworkElement, IImage, IItemsControl, IItemsControl2, IItemsWrapGrid, IListViewBase,
    IPropertyValue, ISelector, ITextBlock, IUIElement, ListViewSelectionMode, PropertyValue, SelectionChangedEventArgs,
    SizeChangedEventArgs, UIElement, Visibility,
};

const COVER_DECODE_WIDTH: i32 = 320;
/// Rows from the end at which more load.
const LOAD_AHEAD: usize = 8;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
    <RowDefinition Height="*" /><RowDefinition Height="Auto" />
  </Grid.RowDefinitions>
  <Button x:Name="Back" Visibility="Collapsed" Background="Transparent" BorderThickness="0" Padding="16,6"
          Foreground="#0050EF" />
  <Grid Grid.Row="1" Margin="16,12,8,2">
    <Grid.ColumnDefinitions>
      <ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" />
    </Grid.ColumnDefinitions>
    <TextBlock x:Name="Title" FontSize="20" FontWeight="Light" TextTrimming="CharacterEllipsis" />
    <Button x:Name="PlayAll" Grid.Column="1" Visibility="Collapsed" Background="#0050EF" Foreground="White"
            BorderThickness="0" Padding="12,4" Margin="0,0,8,0">
      <StackPanel Orientation="Horizontal">
        <FontIcon Glyph="&#xE768;" FontSize="14" /><TextBlock Text="全部播放" Margin="8,0,0,0" />
      </StackPanel>
    </Button>
    <Button x:Name="Manage" Grid.Column="2" Visibility="Collapsed" Background="Transparent" BorderThickness="0"
            Padding="8,2" Foreground="#0050EF" />
  </Grid>
  <TextBlock x:Name="Meta" Grid.Row="2" FontSize="12" Margin="16,0,16,6" TextWrapping="Wrap"
             Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  <GridView x:Name="List" Grid.Row="3" SelectionMode="None" IsItemClickEnabled="True" Padding="0">
    <GridView.ItemContainerTransitions><TransitionCollection /></GridView.ItemContainerTransitions>
    <GridView.ItemContainerStyle>
      <Style TargetType="GridViewItem">
        <Setter Property="Margin" Value="0" />
        <Setter Property="Padding" Value="0" />
        <Setter Property="HorizontalContentAlignment" Value="Stretch" />
        <Setter Property="VerticalContentAlignment" Value="Stretch" />
      </Style>
    </GridView.ItemContainerStyle>
    <GridView.ItemTemplate><DataTemplate>
      <Grid Margin="16,8,8,8">
        <Grid.ColumnDefinitions><ColumnDefinition Width="128" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
        <Grid.RowDefinitions>
          <RowDefinition Height="*" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
        </Grid.RowDefinitions>
        <Image Grid.RowSpan="3" Stretch="UniformToFill" />
        <TextBlock Grid.Column="1" Margin="10,0,0,0" FontSize="13" TextWrapping="Wrap" MaxLines="2"
                   TextTrimming="CharacterEllipsis" />
        <TextBlock Grid.Column="1" Grid.Row="1" Margin="10,0,0,0" FontSize="11" TextTrimming="CharacterEllipsis"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        <TextBlock Grid.Column="1" Grid.Row="2" Margin="10,0,0,0" FontSize="11" TextTrimming="CharacterEllipsis"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      </Grid>
    </DataTemplate></GridView.ItemTemplate>
  </GridView>
  <StackPanel x:Name="Bar" Grid.Row="4" Orientation="Horizontal" Visibility="Collapsed" Padding="8,4"
              BorderThickness="0,1,0,0" BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <Button x:Name="Remove" Background="Transparent" BorderThickness="0" Padding="8,4" Foreground="#0050EF" />
    <Button x:Name="Clear" Content="清除已看完" Visibility="Collapsed" Background="Transparent" BorderThickness="0"
            Padding="8,4" Foreground="#0050EF" />
    <Button x:Name="MoveTo" Content="移动到…" Visibility="Collapsed" Background="Transparent" BorderThickness="0"
            Padding="8,4" Foreground="#0050EF" />
    <Button x:Name="CopyTo" Content="复制到…" Visibility="Collapsed" Background="Transparent" BorderThickness="0"
            Padding="8,4" Foreground="#0050EF" />
  </StackPanel>
</Grid>"##;

/// Where a list's videos come from, and how they leave it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    History,
    Later,
    Folder(u64),
}

impl Source {
    fn manages(self) -> bool {
        self != Self::History
    }

    /// One page, and what to ask for the next one with.
    async fn fetch(self, core: &Backend, cursor: Option<String>) -> rubi_core::Result<(Vec<Row>, Option<String>)> {
        let now = now();
        match self {
            Self::History => {
                let (entries, next) = core.services.history.page(cursor.as_deref()).await?;
                Ok((entries.into_iter().map(|e| Row::watched(e, now)).collect(), next))
            }
            Self::Later => {
                let entries = core.services.watch_later.list().await?;
                Ok((entries.into_iter().map(Row::saved_for_later).collect(), None))
            }
            Self::Folder(id) => {
                let page = cursor.and_then(|c| c.parse().ok()).unwrap_or(1);
                let (cards, more) = core.services.favorites.items(id, page).await?;
                let next = (more && !cards.is_empty()).then(|| (page + 1).to_string());
                Ok((cards.into_iter().map(|c| Row::favorite(c, now)).collect(), next))
            }
        }
    }

    async fn remove(self, core: &Backend, aids: &[u64]) -> rubi_core::Result<()> {
        match self {
            Self::History => Ok(()),
            Self::Later => core.services.watch_later.remove(aids).await,
            Self::Folder(id) => core.services.favorites.remove_from_folder(id, aids).await,
        }
    }
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs() as i64)
}

pub struct VideoList {
    id: u64,
    root: IFrameworkElement,
    list: IFrameworkElement,
    source: Cell<Source>,
    /// The fixed line a 收藏夹 shows; the others count their videos.
    line: RefCell<String>,
    rows: RefCell<Vec<Row>>,
    next: RefCell<Option<String>>,
    ended: Cell<bool>,
    loading: Cell<bool>,
    /// Bumped when the list starts over: answers for an earlier one are dropped.
    turn: Cell<u64>,
    managing: Cell<bool>,
    busy: Cell<bool>,
    covers: Covers,
    fitted: Cell<bool>,
    back: RefCell<Option<Box<dyn Fn()>>>,
    /// The folders videos can go to, by id and title.
    targets: RefCell<Vec<(u64, String)>>,
    /// Told when videos went from one folder to another: from, to, how many, and whether copied.
    transfer_heard: RefCell<Option<Box<dyn Fn(u64, u64, usize, bool)>>>,
}

thread_local! {
    static LISTS: RefCell<HashMap<u64, Weak<VideoList>>> = RefCell::default();
}

fn list_of(id: u64) -> Option<Rc<VideoList>> {
    LISTS.with_borrow(|l| l.get(&id).and_then(Weak::upgrade))
}

impl VideoList {
    pub fn new(title: &str, source: Source) -> Result<Rc<Self>> {
        let root = load(LAYOUT)?;
        let list = named::<IFrameworkElement>(&root, "List")?;
        let shown = Rc::new(Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            root,
            list: list.clone(),
            source: Cell::new(source),
            line: RefCell::default(),
            rows: RefCell::default(),
            next: RefCell::default(),
            ended: Cell::new(false),
            loading: Cell::new(false),
            turn: Cell::new(0),
            managing: Cell::new(false),
            busy: Cell::new(false),
            covers: Covers::default(),
            fitted: Cell::new(false),
            back: RefCell::default(),
            targets: RefCell::default(),
            transfer_heard: RefCell::default(),
        });
        LISTS.with_borrow_mut(|l| l.insert(shown.id, Rc::downgrade(&shown)));
        set_text(&shown.root, "Title", title)?;

        let sizing = Rc::downgrade(&shown);
        keep(list.SizeChanged(move |_: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
            if let Some(shown) = sizing.upgrade() {
                let _ = shown.fit();
            }
        })?);
        let filling = Rc::downgrade(&shown);
        on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |s| s.fill(at, parts)))?;
        let opening = Rc::downgrade(&shown);
        on_row_click(&list, move |at| {
            let Some(shown) = opening.upgrade() else { return };
            let row = shown.rows.borrow().get(at).cloned();
            if let Some(row) = row {
                let _ = feed::route(&row.bvid, &row.title, &row.owner);
            }
        })?;
        let asking = Rc::downgrade(&shown);
        card_menu::attach(&list, move |at| {
            let shown = asking.upgrade()?;
            let rows = shown.rows.borrow();
            let row = rows.get(at)?;
            let later = (shown.source.get() != Source::Later).then(|| row.bvid.clone());
            Some(card_menu::Target { later, link: video_link(&row.bvid), recommended: None })
        })?;
        let counting = Rc::downgrade(&shown);
        keep(list.cast::<ISelector>()?.SelectionChanged(
            move |_: Ref<IInspectable>, _: Ref<SelectionChangedEventArgs>| {
                if let Some(shown) = counting.upgrade() {
                    let _ = shown.show_count();
                }
            },
        )?);
        shown.wire("Back", |s| {
            if let Some(back) = s.back.borrow().as_ref() {
                back();
            }
        })?;
        shown.wire("Manage", |s| {
            let _ = s.set_managing(!s.managing.get());
        })?;
        shown.wire("Remove", Self::remove_picked)?;
        shown.wire("PlayAll", |s| {
            let entries = s
                .rows
                .borrow()
                .iter()
                .map(|r| Entry { bvid: r.bvid.clone(), title: r.title.clone(), owner: r.owner.clone() })
                .collect();
            let _ = queue::start(Shelf::Later.label(), entries, 0);
        })?;
        shown.wire("Clear", Self::clear_finished)?;
        shown.wire("MoveTo", |s| {
            let _ = s.pick_target(false);
        })?;
        shown.wire("CopyTo", |s| {
            let _ = s.pick_target(true);
        })?;
        Ok(shown)
    }

    fn wire(self: &Rc<Self>, name: &str, act: impl Fn(&Rc<Self>) + 'static) -> Result<()> {
        let weak = Rc::downgrade(self);
        on_click(&self.root, name, move || {
            if let Some(shown) = weak.upgrade() {
                act(&shown);
            }
        })
    }

    pub fn root(&self) -> &IFrameworkElement {
        &self.root
    }

    /// Shows a way back above the list, or none.
    pub fn set_back(&self, label: Option<&str>, back: impl Fn() + 'static) -> Result<()> {
        *self.back.borrow_mut() = Some(Box::new(back));
        if let Some(label) = label {
            self.label("Back", &format!("← {label}"))?;
        }
        self.show(&self.root, "Back", label.is_some())
    }

    /// The folders a folder's videos can be moved or copied to.
    pub fn set_targets(&self, targets: Vec<(u64, String)>) {
        *self.targets.borrow_mut() = targets;
    }

    /// Says whom to tell when videos go to another folder.
    pub fn on_transfer(&self, heard: impl Fn(u64, u64, usize, bool) + 'static) {
        *self.transfer_heard.borrow_mut() = Some(Box::new(heard));
    }

    fn label(&self, name: &str, text: &str) -> Result<()> {
        named::<IContentControl>(&self.root, name)?.SetContent(&PropertyValue::CreateString(text)?)
    }

    fn show(&self, root: &IFrameworkElement, name: &str, on: bool) -> Result<()> {
        named::<IUIElement>(root, name)?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })
    }

    /// Starts over from another source.
    pub fn open(self: &Rc<Self>, source: Source, title: &str, line: &str) -> Result<()> {
        self.source.set(source);
        *self.line.borrow_mut() = line.to_owned();
        set_text(&self.root, "Title", title)?;
        self.reload()
    }

    /// Starts over.
    pub fn reload(self: &Rc<Self>) -> Result<()> {
        self.set_managing(false)?;
        self.turn.set(self.turn.get() + 1);
        self.rows.borrow_mut().clear();
        *self.next.borrow_mut() = None;
        self.ended.set(false);
        self.loading.set(false);
        self.reset_items()?;
        self.show(&self.root, "Manage", false)?;
        self.show(&self.root, "PlayAll", false)?;
        set_text(&self.root, "Meta", "正在读取…")?;
        self.load();
        Ok(())
    }

    fn reset_items(&self) -> Result<()> {
        self.items()?.Clear()?;
        add_rows(&self.list, 0, self.rows.borrow().len())
    }

    fn items(&self) -> Result<IVector<IInspectable>> {
        self.list.cast::<IItemsControl>()?.Items()?.cast()
    }

    fn load(self: &Rc<Self>) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.ended.get() || self.loading.replace(true) {
            return;
        }
        let (id, turn, source) = (self.id, self.turn.get(), self.source.get());
        let (cursor, dispatcher, core) = (self.next.borrow().clone(), shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let page = source.fetch(&core, cursor).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(shown) = list_of(id).filter(|s| s.turn.get() == turn) {
                    let _ = shown.took(page);
                }
            });
        });
    }

    fn took(&self, page: std::result::Result<(Vec<Row>, Option<String>), String>) -> Result<()> {
        self.loading.set(false);
        match page {
            Ok((rows, next)) => {
                self.ended.set(next.is_none());
                *self.next.borrow_mut() = next;
                let from = self.rows.borrow().len();
                self.rows.borrow_mut().extend(rows);
                let to = self.rows.borrow().len();
                add_rows(&self.list, from, to)?;
                self.show(&self.root, "Manage", self.source.get().manages() && to > 0)?;
                self.show_play_all()?;
                self.show_count()
            }
            Err(problem) => {
                self.ended.set(true);
                set_text(&self.root, "Meta", &format!("读取失败：{problem}"))
            }
        }
    }

    fn fill(self: &Rc<Self>, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        if !self.fitted.get() {
            self.fit()?;
        }
        let wants_more = {
            let rows = self.rows.borrow();
            let Some(row) = rows.get(at) else { return Ok(()) };
            self.covers.show(&parts.GetAt(0)?.cast::<IImage>()?, &row.cover, COVER_DECODE_WIDTH)?;
            parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&row.title)?;
            parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&row.owner)?;
            parts.GetAt(3)?.cast::<ITextBlock>()?.SetText(&row.line)?;
            at + LOAD_AHEAD >= rows.len()
        };
        if wants_more {
            self.load();
        }
        Ok(())
    }

    fn fit(&self) -> Result<()> {
        let width = self.list.ActualWidth()?;
        let Ok(panel) = self.list.cast::<IItemsControl2>().and_then(|l| l.ItemsPanelRoot()) else { return Ok(()) };
        let panel = panel.cast::<IItemsWrapGrid>()?;
        let (columns, cell_width, cell_height) = row_size(width);
        panel.SetMaximumRowsOrColumns(columns as i32)?;
        panel.SetItemWidth(cell_width)?;
        panel.SetItemHeight(cell_height)?;
        self.fitted.set(true);
        Ok(())
    }

    /// 稍后再看 plays through from the first, outside managing.
    fn show_play_all(&self) -> Result<()> {
        let on = self.source.get() == Source::Later && !self.managing.get() && !self.rows.borrow().is_empty();
        self.show(&self.root, "PlayAll", on)
    }

    fn picked(&self) -> Result<Vec<usize>> {
        let items = self.list.cast::<IListViewBase>()?.SelectedItems()?;
        (0..items.Size()?).map(|at| Ok(items.GetAt(at)?.cast::<IPropertyValue>()?.GetInt32()? as usize)).collect()
    }

    fn show_count(&self) -> Result<()> {
        let fixed = self.line.borrow().clone();
        let total = self.rows.borrow().len();
        let mut line = match self.source.get() {
            Source::Later => format!("{total} 个视频"),
            Source::Folder(_) => fixed,
            Source::History if total == 0 => "还没有历史记录".to_owned(),
            Source::History => String::new(),
        };
        if self.managing.get() {
            line = format!("{line} · 已选 {}", self.picked()?.len());
        }
        set_text(&self.root, "Meta", &line)
    }

    fn set_managing(&self, on: bool) -> Result<()> {
        self.managing.set(on);
        let list = self.list.cast::<IListViewBase>()?;
        list.SetSelectionMode(if on { ListViewSelectionMode::Multiple } else { ListViewSelectionMode::None })?;
        list.SetIsItemClickEnabled(!on)?;
        let source = self.source.get();
        let manage = match (on, source) {
            (true, _) => "完成",
            (false, Source::Later) => "管理",
            (false, _) => "编辑",
        };
        self.label("Manage", manage)?;
        self.label("Remove", if source == Source::Later { "移除选中" } else { "移出收藏夹" })?;
        self.show(&self.root, "Bar", on)?;
        self.show_play_all()?;
        self.show(&self.root, "Clear", on && source == Source::Later)?;
        let folder = matches!(source, Source::Folder(_));
        self.show(&self.root, "MoveTo", on && folder)?;
        self.show(&self.root, "CopyTo", on && folder)?;
        self.show_count()
    }

    /// Takes the ticked videos out of the list, here and on B站.
    fn remove_picked(self: &Rc<Self>) {
        let Ok(picked) = self.picked() else { return };
        let rows = self.rows.borrow();
        let aids: Vec<u64> = picked.iter().filter_map(|at| rows.get(*at)).map(|r| r.aid).collect();
        drop(rows);
        if !aids.is_empty() {
            self.take_out(aids, "已移除");
        }
    }

    /// Offers the other folders under the button for the ticked videos.
    fn pick_target(self: &Rc<Self>, keep: bool) -> Result<()> {
        let Source::Folder(from) = self.source.get() else { return Ok(()) };
        let rows = self.rows.borrow();
        let aids: Vec<u64> = self.picked()?.iter().filter_map(|at| rows.get(*at)).map(|r| r.aid).collect();
        drop(rows);
        if aids.is_empty() {
            return set_text(&self.root, "Meta", "先选中要移动或复制的视频");
        }
        let choices: Vec<Choice> = self
            .targets
            .borrow()
            .iter()
            .filter(|(id, _)| *id != from)
            .map(|(to, title)| {
                let (weak, aids, to, title) = (Rc::downgrade(self), aids.clone(), *to, title.clone());
                let label = title.clone();
                let act: Rc<dyn Fn()> = Rc::new(move || {
                    if let Some(shown) = weak.upgrade() {
                        shown.transfer(from, to, &title, aids.clone(), keep);
                    }
                });
                (label, act)
            })
            .collect();
        if choices.is_empty() {
            return set_text(&self.root, "Meta", "没有别的收藏夹");
        }
        card_menu::show_under(&named(&self.root, if keep { "CopyTo" } else { "MoveTo" })?, choices)
    }

    fn transfer(self: &Rc<Self>, from: u64, to: u64, title: &str, aids: Vec<u64>, keep: bool) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.busy.replace(true) {
            return;
        }
        let said = format!("{}{title}", if keep { "已复制到 " } else { "已移动到 " });
        let (id, dispatcher, core) = (self.id, shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let done = core.services.favorites.transfer(from, to, &aids, keep).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                let Some(shown) = list_of(id) else { return };
                let worked = done.is_ok();
                let count = aids.len();
                let _ = if keep {
                    shown.busy.set(false);
                    match done {
                        Ok(()) => shown.set_managing(false).and_then(|()| {
                            let line = named::<ITextBlock>(&shown.root, "Meta")?.Text()?;
                            set_text(&shown.root, "Meta", &format!("{line} · {said}"))
                        }),
                        Err(problem) => set_text(&shown.root, "Meta", &format!("操作失败：{problem}")),
                    }
                } else {
                    shown.gone(&aids, done, &said)
                };
                if worked && let Some(heard) = shown.transfer_heard.borrow().as_ref() {
                    heard(from, to, count, keep);
                }
            });
        });
    }

    fn clear_finished(self: &Rc<Self>) {
        let aids: Vec<u64> = self.rows.borrow().iter().filter(|r| r.finished).map(|r| r.aid).collect();
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.busy.replace(true) {
            return;
        }
        let (id, dispatcher, core) = (self.id, shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let done = core.services.watch_later.clear_watched().await.map_err(|e| e.to_string());
            finish(&dispatcher, id, aids, done, "已清除看完的视频");
        });
    }

    fn take_out(self: &Rc<Self>, aids: Vec<u64>, said: &'static str) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.busy.replace(true) {
            return;
        }
        let (id, source, dispatcher, core) = (self.id, self.source.get(), shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let done = source.remove(&core, &aids).await.map_err(|e| e.to_string());
            finish(&dispatcher, id, aids, done, said);
        });
    }

    /// The videos leave at once: B站 may go on listing them for a while, so it is not asked again.
    fn gone(&self, aids: &[u64], done: std::result::Result<(), String>, said: &str) -> Result<()> {
        self.busy.set(false);
        match done {
            Ok(()) => {
                self.rows.borrow_mut().retain(|r| !aids.contains(&r.aid));
                self.reset_items()?;
                self.show_play_all()?;
                self.show_count()?;
                let line = named::<ITextBlock>(&self.root, "Meta")?.Text()?;
                set_text(&self.root, "Meta", &format!("{line} · {said}"))
            }
            Err(problem) => set_text(&self.root, "Meta", &format!("操作失败：{problem}")),
        }
    }
}

fn finish(
    dispatcher: &crate::xaml::CoreDispatcher,
    id: u64,
    aids: Vec<u64>,
    done: std::result::Result<(), String>,
    said: &'static str,
) {
    post(dispatcher, move || {
        if let Some(shown) = list_of(id) {
            let _ = shown.gone(&aids, done, said);
        }
    });
}
