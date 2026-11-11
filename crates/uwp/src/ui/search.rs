//! 搜索: 热搜 until something is searched, then 综合, 视频 and UP 主. The page lives on, so coming
//! back from a result finds it as it was.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use rubi_core::PgcHit;
use rubi_model::EsportsMatch;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::pgc::paid;
use crate::state::search::hit_line;
use crate::ui::card_grid::{CardGrid, Source};
use crate::ui::match_rows;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{keep, load, named, on_click, on_enter, pgc_season, post, set_text, up};
use crate::xaml::{
    Color, IBorder, IContentControl, IControl, IFrameworkElement, IImage, IItemsControl, IItemsControl2,
    IItemsWrapGrid, IPanel, ITextBlock, ITextBox, IUIElement, PropertyValue, SizeChangedEventArgs, SolidColorBrush,
    UIElement, Visibility,
};

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };
const FACE_DECODE_WIDTH: i32 = 80;
/// 热搜 words asked for.
const HOT_WORDS: u32 = 20;
/// UPs 综合 shows above the videos.
const TOP_HITS: usize = 2;
/// From this wide 热搜 has two columns.
const HOT_TWO_COLUMNS_FROM: f64 = 600.0;
const HOT_ROW: f64 = 40.0;
const TAB_LABELS: [&str; 3] = ["综合", "视频", "UP 主"];
/// 影视 asked for, and how many show before 展开.
const PGC_MOST: usize = 20;
const PGC_FOLDED: usize = 3;
const PGC_DECODE_WIDTH: i32 = 120;
/// Matches shown before the fold.
const MATCHES_FOLDED: usize = 3;
const RED: Color = Color { a: 255, r: 0xE8, g: 0x11, b: 0x23 };

const USER_ROW: &str = r##"
<ListView.ItemTemplate><DataTemplate>
  <Grid>
    <Grid.ColumnDefinitions><ColumnDefinition Width="50" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
    <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" /></Grid.RowDefinitions>
    <Image Grid.RowSpan="2" Width="40" Height="40" Stretch="UniformToFill" HorizontalAlignment="Left" />
    <TextBlock Grid.Column="1" FontSize="14" TextTrimming="CharacterEllipsis" />
    <TextBlock Grid.Column="1" Grid.Row="1" FontSize="11"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </Grid>
</DataTemplate></ListView.ItemTemplate>"##;

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
    <RowDefinition Height="*" />
  </Grid.RowDefinitions>
  <Grid Margin="16,8,16,8">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
    <TextBox x:Name="Query" PlaceholderText="搜索视频和 UP 主" MaxLength="100" />
    <Button x:Name="Go" Grid.Column="1" Content="搜索" Background="Transparent" BorderThickness="0" Margin="8,0,0,0"
            Foreground="#0050EF" />
  </Grid>
  <StackPanel x:Name="Tabs" Grid.Row="1" Orientation="Horizontal" Margin="16,0,16,4" Visibility="Collapsed">
    <StackPanel.Resources>
      <Style TargetType="Button">
        <Setter Property="Background" Value="Transparent" />
        <Setter Property="BorderThickness" Value="0" />
        <Setter Property="Padding" Value="0" />
        <Setter Property="Margin" Value="0,0,18,0" />
        <Setter Property="FontSize" Value="20" />
        <Setter Property="FontWeight" Value="Light" />
      </Style>
    </StackPanel.Resources>
    <Button x:Name="Tab0" Content="综合" /><Button x:Name="Tab1" Content="视频" /><Button x:Name="Tab2" Content="UP 主" />
  </StackPanel>
  <TextBlock x:Name="Status" Grid.Row="2" FontSize="12" Margin="16,0,16,4" TextWrapping="Wrap"
             Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  <Grid x:Name="HotPane" Grid.Row="3">
    <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
    <TextBlock Text="热搜" FontSize="16" FontWeight="Light" Margin="16,4,16,4" />
    <GridView x:Name="Hot" Grid.Row="1" SelectionMode="None" IsItemClickEnabled="True" Padding="16,0">
      <GridView.ItemContainerTransitions><TransitionCollection /></GridView.ItemContainerTransitions>
      <GridView.ItemContainerStyle>
        <Style TargetType="GridViewItem">
          <Setter Property="Margin" Value="0" />
          <Setter Property="Padding" Value="0" />
          <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          <Setter Property="VerticalContentAlignment" Value="Center" />
        </Style>
      </GridView.ItemContainerStyle>
      <GridView.ItemsPanel><ItemsPanelTemplate><ItemsWrapGrid Orientation="Horizontal" /></ItemsPanelTemplate></GridView.ItemsPanel>
      <GridView.ItemTemplate><DataTemplate>
        <StackPanel Orientation="Horizontal">
          <TextBlock FontSize="14" Foreground="#0050EF" Width="30" />
          <TextBlock FontSize="14" TextTrimming="CharacterEllipsis" />
        </StackPanel>
      </DataTemplate></GridView.ItemTemplate>
    </GridView>
  </Grid>
  <Grid x:Name="Results" Grid.Row="3" Visibility="Collapsed">
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
    </Grid.RowDefinitions>
    <StackPanel>
    <StackPanel x:Name="MatchPart" Visibility="Collapsed">
      <Grid Margin="16,2,16,2">
        <TextBlock Text="赛事" FontSize="15" />
        <Button x:Name="MatchMore" HorizontalAlignment="Right" Background="Transparent" BorderThickness="0"
                Padding="0" Foreground="#0050EF" FontSize="12" Visibility="Collapsed" />
      </Grid>
      <!--matches-->
    </StackPanel>
    <StackPanel x:Name="PgcPart" Visibility="Collapsed">
      <TextBlock Text="影视" FontSize="15" Margin="16,2,16,2" />
      <ListView x:Name="Pgc" SelectionMode="None" IsItemClickEnabled="True" ScrollViewer.VerticalScrollMode="Disabled">
        <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
        <ListView.ItemContainerStyle>
          <Style TargetType="ListViewItem">
            <Setter Property="Padding" Value="16,6" />
            <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          </Style>
        </ListView.ItemContainerStyle>
        <ListView.ItemTemplate><DataTemplate>
          <Grid>
            <Grid.ColumnDefinitions><ColumnDefinition Width="70" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
            <Grid.RowDefinitions>
              <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
              <RowDefinition Height="Auto" />
            </Grid.RowDefinitions>
            <Grid Grid.RowSpan="4" Width="60" Height="80" HorizontalAlignment="Left" VerticalAlignment="Top"
                  Background="{ThemeResource SystemControlBackgroundBaseLowBrush}">
              <Image Stretch="UniformToFill" />
              <Border HorizontalAlignment="Right" VerticalAlignment="Top" Padding="4,0">
                <TextBlock FontSize="9" Foreground="White" />
              </Border>
            </Grid>
            <TextBlock Grid.Column="1" FontSize="14" TextTrimming="CharacterEllipsis" />
            <TextBlock Grid.Column="1" Grid.Row="1" FontSize="11" TextTrimming="CharacterEllipsis"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
            <TextBlock Grid.Column="1" Grid.Row="2" FontSize="12" Foreground="#E0A000" />
            <TextBlock Grid.Column="1" Grid.Row="3" FontSize="11" Foreground="#0050EF" Text="查看剧集 ›" />
          </Grid>
        </DataTemplate></ListView.ItemTemplate>
      </ListView>
      <Button x:Name="PgcMore" Background="Transparent" BorderThickness="0" Padding="16,2" Foreground="#0050EF"
              FontSize="12" Visibility="Collapsed" />
    </StackPanel>
    </StackPanel>
    <ListView x:Name="TopUsers" Grid.Row="1" SelectionMode="None" IsItemClickEnabled="True"
              ScrollViewer.VerticalScrollMode="Disabled"><!--users--></ListView>
    <Grid x:Name="Videos" Grid.Row="2" />
    <ListView x:Name="Users" Grid.Row="2" SelectionMode="None" IsItemClickEnabled="True" Visibility="Collapsed"><!--users--></ListView>
  </Grid>
</Grid>"##;

/// An UP in the results.
struct Hit {
    mid: u64,
    name: String,
    face: String,
    line: String,
}

pub struct SearchPage {
    root: IFrameworkElement,
    videos: Rc<CardGrid>,
    hot: RefCell<Vec<String>>,
    hits: RefCell<Vec<Hit>>,
    term: RefCell<String>,
    hits_page: Cell<u32>,
    hits_ended: Cell<bool>,
    hits_loading: Cell<bool>,
    /// Bumped by each search: answers for an earlier one are dropped.
    turn: Cell<u64>,
    tab: Cell<usize>,
    hot_loaded: Cell<bool>,
    covers: Covers,
    /// 影视 the search found, and whether all of them show.
    pgc: RefCell<Vec<PgcHit>>,
    pgc_all: Cell<bool>,
    /// Matches the search found, and whether all of them show.
    matches: RefCell<Vec<EsportsMatch>>,
    matches_all: Cell<bool>,
}

impl SearchPage {
    fn new() -> Result<Rc<Self>> {
        let root = load(
            &LAYOUT.replace("<!--users-->", USER_ROW).replace("<!--matches-->", &match_rows::list("Matches", true)),
        )?;
        let videos = CardGrid::new(|| {
            if let Some(page) = held() {
                let _ = page.show_status();
            }
        })?;
        named::<IPanel>(&root, "Videos")?
            .Children()?
            .cast::<IVector<UIElement>>()?
            .Append(&videos.element().cast::<UIElement>()?)?;
        let page = Rc::new(Self {
            root,
            videos,
            hot: RefCell::default(),
            hits: RefCell::default(),
            term: RefCell::default(),
            hits_page: Cell::new(1),
            hits_ended: Cell::new(false),
            hits_loading: Cell::new(false),
            turn: Cell::new(0),
            tab: Cell::new(0),
            hot_loaded: Cell::new(false),
            covers: Covers::default(),
            pgc: RefCell::default(),
            pgc_all: Cell::new(false),
            matches: RefCell::default(),
            matches_all: Cell::new(false),
        });
        let weak = Rc::downgrade(&page);
        let submit = move || {
            if let Some(page) = weak.upgrade() {
                let _ = page.submit();
            }
        };
        on_click(&page.root, "Go", submit.clone())?;
        on_enter(&page.root, "Query", submit)?;
        for (tab, _) in TAB_LABELS.iter().enumerate() {
            let weak = Rc::downgrade(&page);
            on_click(&page.root, &format!("Tab{tab}"), move || {
                if let Some(page) = weak.upgrade() {
                    page.tab.set(tab);
                    let _ = page.show_tab();
                }
            })?;
        }
        page.wire_hot()?;
        page.wire_pgc()?;
        page.wire_matches()?;
        for (name, top) in [("TopUsers", true), ("Users", false)] {
            let list = named::<IFrameworkElement>(&page.root, name)?;
            let filling = Rc::downgrade(&page);
            on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |p| p.fill_hit(at, parts, top)))?;
            let opening = Rc::downgrade(&page);
            on_row_click(&list, move |at| {
                if let Some(page) = opening.upgrade() {
                    let hit = page.hits.borrow().get(at).map(|h| (h.mid, h.name.clone()));
                    if let Some((mid, name)) = hit {
                        let _ = up::open(mid, &name);
                    }
                }
            })?;
        }
        Ok(page)
    }

    fn wire_pgc(self: &Rc<Self>) -> Result<()> {
        let list = named::<IFrameworkElement>(&self.root, "Pgc")?;
        let filling = Rc::downgrade(self);
        on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |p| p.fill_pgc(at, parts)))?;
        let opening = Rc::downgrade(self);
        on_row_click(&list, move |at| {
            if let Some(page) = opening.upgrade() {
                let hit = page.pgc.borrow().get(at).map(|h| (h.season_id, h.title.clone()));
                if let Some((id, title)) = hit {
                    let _ = pgc_season::open(id, &title);
                }
            }
        })?;
        let folding = Rc::downgrade(self);
        on_click(&self.root, "PgcMore", move || {
            if let Some(page) = folding.upgrade() {
                page.pgc_all.set(!page.pgc_all.get());
                let _ = page.show_pgc();
            }
        })
    }

    fn wire_matches(self: &Rc<Self>) -> Result<()> {
        let list = named::<IFrameworkElement>(&self.root, "Matches")?;
        let filling = Rc::downgrade(self);
        on_row(&list, move |at, parts| {
            let Some(page) = filling.upgrade() else { return Ok(()) };
            let matches = page.matches.borrow();
            let Some(game) = matches.get(at) else { return Ok(()) };
            match_rows::fill(parts, game, true, &page.covers)
        })?;
        on_row_click(&list, |at| {
            let Some(game) = held().and_then(|p| p.matches.borrow().get(at).cloned()) else { return };
            let turn = held().map_or(0, |p| p.turn.get());
            match_rows::open(&game, move |changed| {
                let Some(page) = held().filter(|p| p.turn.get() == turn) else { return };
                if let Some(slot) = page.matches.borrow_mut().get_mut(at) {
                    *slot = changed;
                }
                let _ = page.show_matches();
            });
        })?;
        let folding = Rc::downgrade(self);
        on_click(&self.root, "MatchMore", move || {
            if let Some(page) = folding.upgrade() {
                page.matches_all.set(!page.matches_all.get());
                let _ = page.show_matches();
            }
        })
    }

    fn load_matches(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (term, turn, dispatcher, core) =
            (self.term.borrow().clone(), self.turn.get(), shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let found = core.services.esports.related(&term).await.unwrap_or_default();
            post(&dispatcher, move || {
                if let Some(page) = held().filter(|p| p.turn.get() == turn) {
                    *page.matches.borrow_mut() = found;
                    let _ = page.show_matches();
                }
            });
        });
    }

    /// The first few matches, or all of them once asked.
    fn show_matches(&self) -> Result<()> {
        let total = self.matches.borrow().len();
        let shown = if self.matches_all.get() { total } else { total.min(MATCHES_FOLDED) };
        let list = named::<IFrameworkElement>(&self.root, "Matches")?;
        list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        add_rows(&list, 0, shown)?;
        let more = named::<IContentControl>(&self.root, "MatchMore")?;
        let label = if self.matches_all.get() { "收起".to_owned() } else { format!("全部 {total} 场 ›") };
        more.SetContent(&PropertyValue::CreateString(&label)?)?;
        let folds = if total > MATCHES_FOLDED { Visibility::Visible } else { Visibility::Collapsed };
        more.cast::<IUIElement>()?.SetVisibility(folds)?;
        self.show_tab()
    }

    fn load_pgc(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (term, turn, dispatcher, core) =
            (self.term.borrow().clone(), self.turn.get(), shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let found = core.services.discover.search_pgc(&term, PGC_MOST).await.unwrap_or_default();
            post(&dispatcher, move || {
                if let Some(page) = held().filter(|p| p.turn.get() == turn) {
                    *page.pgc.borrow_mut() = found;
                    let _ = page.show_pgc();
                }
            });
        });
    }

    /// The first few 影视, or all of them once asked.
    fn show_pgc(&self) -> Result<()> {
        let total = self.pgc.borrow().len();
        let shown = if self.pgc_all.get() { total } else { total.min(PGC_FOLDED) };
        let list = named::<IFrameworkElement>(&self.root, "Pgc")?;
        list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        add_rows(&list, 0, shown)?;
        let more = named::<IContentControl>(&self.root, "PgcMore")?;
        let label = if self.pgc_all.get() { "收起".to_owned() } else { format!("展开全部 {total} 部") };
        more.SetContent(&PropertyValue::CreateString(&label)?)?;
        let folds = if total > PGC_FOLDED { Visibility::Visible } else { Visibility::Collapsed };
        more.cast::<IUIElement>()?.SetVisibility(folds)?;
        self.show_tab()
    }

    fn fill_pgc(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let hits = self.pgc.borrow();
        let Some(hit) = hits.get(at) else { return Ok(()) };
        let poster = parts.GetAt(0)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
        let _ = self.covers.show(&poster.GetAt(0)?.cast::<IImage>()?, &hit.cover, PGC_DECODE_WIDTH);
        let badge = poster.GetAt(1)?.cast::<IBorder>()?;
        let tint = if paid(&hit.badge) { RED } else { COBALT };
        badge.SetBackground(&SolidColorBrush::CreateInstanceWithColor(tint)?)?;
        badge.cast::<IUIElement>()?.SetVisibility(if hit.badge.is_empty() {
            Visibility::Collapsed
        } else {
            Visibility::Visible
        })?;
        badge.Child()?.cast::<ITextBlock>()?.SetText(&hit.badge)?;
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&hit.title)?;
        let meta = [hit.kind.as_str(), hit.styles.as_str(), hit.index_show.as_str()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
        parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&meta)?;
        let score = if hit.score.is_empty() { String::new() } else { format!("★ {}", hit.score) };
        parts.GetAt(3)?.cast::<ITextBlock>()?.SetText(&score)
    }

    fn wire_hot(self: &Rc<Self>) -> Result<()> {
        let list = named::<IFrameworkElement>(&self.root, "Hot")?;
        let filling = Rc::downgrade(self);
        on_row(&list, move |at, parts| {
            let Some(page) = filling.upgrade() else { return Ok(()) };
            let hot = page.hot.borrow();
            let Some(word) = hot.get(at) else { return Ok(()) };
            parts.GetAt(0)?.cast::<ITextBlock>()?.SetText(&(at + 1).to_string())?;
            parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(word)
        })?;
        let opening = Rc::downgrade(self);
        on_row_click(&list, move |at| {
            if let Some(page) = opening.upgrade() {
                let word = page.hot.borrow().get(at).cloned();
                if let Some(word) = word {
                    let _ = named::<ITextBox>(&page.root, "Query").and_then(|q| q.SetText(&word));
                    let _ = page.search(word);
                }
            }
        })?;
        let sizing = Rc::downgrade(self);
        keep(list.SizeChanged(move |_: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
            if let Some(page) = sizing.upgrade() {
                let _ = page.fit_hot();
            }
        })?);
        Ok(())
    }

    fn fit_hot(&self) -> Result<()> {
        let list = named::<IFrameworkElement>(&self.root, "Hot")?;
        let width = list.ActualWidth()? - 32.0;
        let Ok(panel) = list.cast::<IItemsControl2>().and_then(|l| l.ItemsPanelRoot()) else { return Ok(()) };
        let panel = panel.cast::<IItemsWrapGrid>()?;
        let columns = if width >= HOT_TWO_COLUMNS_FROM { 2 } else { 1 };
        panel.SetMaximumRowsOrColumns(columns)?;
        panel.SetItemWidth((width / f64::from(columns)).floor().max(1.0))?;
        panel.SetItemHeight(HOT_ROW)
    }

    fn load_hot(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.hot_loaded.replace(true) {
            return;
        }
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let Ok((words, hidden)) = core.services.discover.hot_words(HOT_WORDS).await else { return };
            let words: Vec<String> = words.into_iter().map(|w| w.keyword).collect();
            post(&dispatcher, move || {
                if let Some(page) = held() {
                    let _ = page.took_hot(words, hidden);
                }
            });
        });
    }

    fn took_hot(&self, words: Vec<String>, hidden: usize) -> Result<()> {
        let count = words.len();
        *self.hot.borrow_mut() = words;
        add_rows(&named(&self.root, "Hot")?, 0, count)?;
        self.fit_hot()?;
        if hidden > 0 && self.term.borrow().is_empty() {
            set_text(&self.root, "Status", &format!("已过滤推广热搜词 {hidden} 条"))?;
        }
        Ok(())
    }

    fn submit(self: &Rc<Self>) -> Result<()> {
        let typed = named::<ITextBox>(&self.root, "Query")?.Text()?.to_string();
        let term = typed.trim().to_owned();
        if term.is_empty() {
            return self.show_mode(false);
        }
        self.search(term)
    }

    fn search(self: &Rc<Self>, term: String) -> Result<()> {
        *self.term.borrow_mut() = term.clone();
        self.turn.set(self.turn.get() + 1);
        self.hits.borrow_mut().clear();
        self.hits_page.set(1);
        self.hits_ended.set(false);
        self.hits_loading.set(false);
        for name in ["TopUsers", "Users", "Pgc", "Matches"] {
            named::<IItemsControl>(&self.root, name)?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        }
        self.pgc.borrow_mut().clear();
        self.pgc_all.set(false);
        self.load_pgc();
        self.matches.borrow_mut().clear();
        self.matches_all.set(false);
        self.load_matches();
        self.tab.set(0);
        self.videos.open(Source::Search(Arc::from(term.as_str())))?;
        self.load_hits();
        self.show_mode(true)?;
        self.show_tab()
    }

    fn load_hits(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.hits_ended.get() || self.hits_loading.replace(true) {
            return;
        }
        let (term, page, turn) = (self.term.borrow().clone(), self.hits_page.get(), self.turn.get());
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let Ok(found) = core.services.discover.search_users(&term, page).await else { return };
            let more = found.has_more;
            let hits: Vec<(u64, String, String, String)> =
                found.users.into_iter().map(|u| (u.mid, u.name, u.face, hit_line(u.fans, u.videos))).collect();
            post(&dispatcher, move || {
                if let Some(page) = held() {
                    let _ = page.took_hits(turn, hits, more);
                }
            });
        });
    }

    fn took_hits(&self, turn: u64, hits: Vec<(u64, String, String, String)>, more: bool) -> Result<()> {
        if turn != self.turn.get() {
            return Ok(());
        }
        self.hits_loading.set(false);
        self.hits_ended.set(!more);
        self.hits_page.set(self.hits_page.get() + 1);
        let from = self.hits.borrow().len();
        self.hits.borrow_mut().extend(hits.into_iter().map(|(mid, name, face, line)| Hit { mid, name, face, line }));
        let to = self.hits.borrow().len();
        add_rows(&named(&self.root, "Users")?, from, to)?;
        if from < TOP_HITS {
            add_rows(&named(&self.root, "TopUsers")?, from, to.min(TOP_HITS))?;
        }
        Ok(())
    }

    fn fill_hit(&self, at: usize, parts: &IVector<UIElement>, top: bool) -> Result<()> {
        let wants_more = {
            let hits = self.hits.borrow();
            let Some(hit) = hits.get(at) else { return Ok(()) };
            let _ = self.covers.show(&parts.GetAt(0)?.cast::<IImage>()?, &hit.face, FACE_DECODE_WIDTH);
            parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&hit.name)?;
            parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&hit.line)?;
            !top && at + 6 >= hits.len()
        };
        if wants_more {
            self.load_hits();
        }
        Ok(())
    }

    fn show_mode(&self, results: bool) -> Result<()> {
        let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
        named::<IUIElement>(&self.root, "HotPane")?.SetVisibility(shown(!results))?;
        named::<IUIElement>(&self.root, "Results")?.SetVisibility(shown(results))?;
        named::<IUIElement>(&self.root, "Tabs")?.SetVisibility(shown(results))
    }

    fn show_tab(&self) -> Result<()> {
        let tab = self.tab.get();
        let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
        named::<IUIElement>(&self.root, "TopUsers")?.SetVisibility(shown(tab == 0))?;
        named::<IUIElement>(&self.root, "PgcPart")?.SetVisibility(shown(tab == 0 && !self.pgc.borrow().is_empty()))?;
        named::<IUIElement>(&self.root, "MatchPart")?
            .SetVisibility(shown(tab == 0 && !self.matches.borrow().is_empty()))?;
        named::<IUIElement>(&self.root, "Videos")?.SetVisibility(shown(tab != 2))?;
        named::<IUIElement>(&self.root, "Users")?.SetVisibility(shown(tab == 2))?;
        for at in 0..TAB_LABELS.len() {
            let color = if at == tab { COBALT } else { DIM };
            named::<IControl>(&self.root, &format!("Tab{at}"))?
                .SetForeground(&SolidColorBrush::CreateInstanceWithColor(color)?)?;
        }
        self.show_status()
    }

    fn show_status(&self) -> Result<()> {
        if !self.term.borrow().is_empty() {
            let said = if self.tab.get() == 2 { String::new() } else { self.videos.said() };
            set_text(&self.root, "Status", &said)?;
        }
        Ok(())
    }
}

thread_local! {
    static PAGE: RefCell<Option<Rc<SearchPage>>> = const { RefCell::new(None) };
}

fn held() -> Option<Rc<SearchPage>> {
    PAGE.with_borrow(Clone::clone)
}

/// Shows the search tab.
pub fn show() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let page = match held() {
        Some(page) => page,
        None => {
            let page = SearchPage::new()?;
            PAGE.with_borrow_mut(|p| *p = Some(page.clone()));
            page
        }
    };
    shell.set_page(&page.root)?;
    page.load_hot();
    Ok(())
}
