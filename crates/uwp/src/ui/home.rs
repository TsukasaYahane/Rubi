//! 首页: 推荐, 热门, 排行, a 分区 to pick and a 影视 专区 to pick. Each keeps its own page, so it comes
//! back where it was left.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use rubi_core::{HOME_ZONES, PGC_ZONES};
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::layout::WindowClass;
use crate::ui::card_grid::{CardGrid, Source};
use crate::ui::pgc_zone::PgcZone;
use crate::ui::shell::{Tab, shell, show_tab};
use crate::ui::{REFRESH_BUTTON, keep, load, load_object, named, on_click, set_text, style_refresh};
use crate::xaml::{
    Color, FrameworkElement, IContentControl, IControl, IFlyoutBase, IFrameworkElement, IMenuFlyout, IMenuFlyoutItem,
    IPanel, IUIElement, PropertyValue, RoutedEventArgs, SolidColorBrush, UIElement, Visibility,
};
use windows_collections::IVector;

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };
const ZONES_TAB: usize = 3;
const PGC_TAB: usize = 4;

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" />
    <RowDefinition Height="Auto" />
    <RowDefinition Height="Auto" />
    <RowDefinition Height="*" />
  </Grid.RowDefinitions>
  <Grid Margin="16,8,16,8">
    <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
    <StackPanel Orientation="Horizontal">
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
      <Button x:Name="Tab0" Content="推荐" /><Button x:Name="Tab1" Content="热门" />
      <Button x:Name="Tab2" Content="排行" /><Button x:Name="Tab3" Content="分区" /><Button x:Name="Tab4" Content="影视" />
    </StackPanel>
    <TextBlock x:Name="Status" Grid.Column="1" FontSize="13" VerticalAlignment="Center" TextTrimming="CharacterEllipsis"
               Visibility="Collapsed"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    <!--refresh-->
    <Button x:Name="Portrait" Grid.Column="4" Background="Transparent" BorderThickness="0"
            Foreground="#0050EF" Padding="8,0" Margin="8,0,0,0" VerticalAlignment="Center" MinWidth="0"
            MinHeight="0" Height="30" FontSize="13">
      <StackPanel Orientation="Horizontal">
        <FontIcon Glyph="&#xE8EA;" FontSize="14" />
        <TextBlock x:Name="PortraitLabel" Text="竖屏" FontSize="13" Margin="6,0,0,0" />
      </StackPanel>
    </Button>
    <Button x:Name="Columns" Grid.Column="5" Background="Transparent" BorderThickness="0"
            Padding="8,0" Margin="8,0,0,0" VerticalAlignment="Center" FontSize="13" MinWidth="0"
            MinHeight="0" Height="30" Visibility="Collapsed" />
    <Button x:Name="Search" Grid.Column="3" Visibility="Collapsed" Background="Transparent" BorderThickness="0"
            Padding="8,2" VerticalAlignment="Center"><FontIcon Glyph="&#xE721;" FontSize="14" /></Button>
  </Grid>
  <Grid x:Name="FeedNoteRow" Grid.Row="1" Margin="16,0,16,8" Background="Transparent">
    <TextBlock x:Name="FeedNote" FontSize="12" TextWrapping="NoWrap" TextTrimming="CharacterEllipsis"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </Grid>
  <StackPanel x:Name="ZoneModes" Grid.Row="2" Orientation="Horizontal" Margin="16,0,16,8"
              Visibility="Collapsed">
    <Button x:Name="ZoneFeed" Content="推荐" Background="Transparent" BorderThickness="0" Padding="0,2"
            Margin="0,0,18,0" FontSize="14" />
    <Button x:Name="ZoneRank" Content="排行" Background="Transparent" BorderThickness="0" Padding="0,2"
            FontSize="14" />
  </StackPanel>
  <Grid x:Name="Grids" Grid.Row="3" />
</Grid>"##;

fn tab_source(tab: usize) -> Option<Source> {
    match tab {
        0 => Some(Source::Recommended),
        1 => Some(Source::Popular),
        2 => Some(Source::Ranking),
        _ => None,
    }
}

pub struct HomePage {
    root: IFrameworkElement,
    grids: [Rc<CardGrid>; 4],
    current: Cell<usize>,
    /// The zone 分区 shows, and whether it shows that zone's ranking.
    zone_region: Cell<u32>,
    zone_rank: Cell<bool>,
    /// The 影视 专区 opened so far, by season type, and the one 影视 shows.
    pgc: RefCell<HashMap<u32, Rc<PgcZone>>>,
    pgc_on: Cell<u32>,
}

impl HomePage {
    pub fn new() -> Result<Rc<Self>> {
        crate::ui::clip::prime();
        let root = load(&LAYOUT.replace("<!--refresh-->", REFRESH_BUTTON))?;
        let told = |tab: usize| {
            move || {
                if let Some(home) = shell().and_then(|s| s.home_page()) {
                    home.grid_spoke(tab);
                }
            }
        };
        let grids = [
            CardGrid::new(told(0))?,
            CardGrid::new(told(1))?,
            CardGrid::new(told(2))?,
            CardGrid::new(told(ZONES_TAB))?,
        ];
        let holder = named::<IPanel>(&root, "Grids")?.Children()?.cast::<IVector<UIElement>>()?;
        for grid in &grids {
            holder.Append(&grid.element().cast::<UIElement>()?)?;
        }
        let page = Rc::new(Self {
            root,
            grids,
            current: Cell::new(0),
            zone_region: Cell::new(0),
            zone_rank: Cell::new(false),
            pgc: RefCell::default(),
            pgc_on: Cell::new(0),
        });
        for tab in 0..=PGC_TAB {
            let weak = Rc::downgrade(&page);
            on_click(&page.root, &format!("Tab{tab}"), move || {
                if let Some(page) = weak.upgrade() {
                    let _ = page.select(tab);
                }
            })?;
        }
        let weak = Rc::downgrade(&page);
        on_click(&page.root, "Refresh", move || {
            if let Some(page) = weak.upgrade() {
                let _ = page.refresh();
            }
        })?;
        let feeding = Rc::downgrade(&page);
        on_click(&page.root, "ZoneFeed", move || {
            if let Some(page) = feeding.upgrade() {
                let _ = page.show_zone(false);
            }
        })?;
        let ranking = Rc::downgrade(&page);
        on_click(&page.root, "ZoneRank", move || {
            if let Some(page) = ranking.upgrade() {
                let _ = page.show_zone(true);
            }
        })?;
        let turning = Rc::downgrade(&page);
        on_click(&page.root, "Columns", move || {
            if let Some(page) = turning.upgrade() {
                let _ = page.toggle_columns();
            }
        })?;
        on_click(&page.root, "Search", || show_tab(Tab::Search))?;
        // The status line doubles as the way into 被过滤内容; a missing one must not keep the page
        // from being made.
        if let Ok(status) = named::<crate::xaml::IUIElement>(&page.root, "FeedNoteRow") {
            if let Ok(revoker) = status.Tapped(move |_: Ref<IInspectable>, _: Ref<crate::xaml::TappedRoutedEventArgs>| {
                crate::ui::filtered::open();
            }) {
                keep(revoker);
            }
        }
        on_click(&page.root, "Portrait", || {
            let _ = crate::ui::feed::open_from_pool();
        })?;
        page.show_tab()?;
        Ok(page)
    }

    pub fn root(&self) -> &IFrameworkElement {
        &self.root
    }

    pub fn say(&self, text: &str) {
        let _ = set_text(&self.root, "FeedNote", text);
    }

    /// Fills the feed the first time.
    pub fn start(&self) {
        if self.grids[0].source().is_none() {
            let _ = self.grids[0].open(Source::Recommended);
        }
    }

    /// Starts over with what the account now sees, after signing in.
    pub fn reload(&self) {
        let Some(backend) = shell().and_then(|s| s.backend.get().cloned()) else { return };
        backend.services.home.new_session();
        for grid in &self.grids {
            let _ = grid.forget();
        }
        let _ = self.reopen();
    }

    /// Reads the list on show again.
    fn refresh(&self) -> Result<()> {
        if let Some(zone) = self.pgc_shown() {
            zone.refresh();
            return Ok(());
        }
        self.grids[self.current.get()].refresh()
    }

    /// The 影视 专区 on show, if 影视 is.
    fn pgc_shown(&self) -> Option<Rc<PgcZone>> {
        if self.current.get() != PGC_TAB {
            return None;
        }
        self.pgc.borrow().get(&self.pgc_on.get()).cloned()
    }

    /// Follows the interface style.
    pub fn style(&self) -> Result<()> {
        style_refresh(&self.root)?;
        // The words style keeps to type: no frames around its two buttons.
        let words = shell().is_some_and(|s| s.prefs().words);
        let edge = crate::xaml::Thickness {
            left: if words { 0.0 } else { 1.0 },
            top: if words { 0.0 } else { 1.0 },
            right: if words { 0.0 } else { 1.0 },
            bottom: if words { 0.0 } else { 1.0 },
        };
        for name in ["Portrait", "Columns"] {
            named::<IControl>(&self.root, name)?.SetBorderThickness(edge)?;
        }
        let narrow = shell().is_some_and(|s| s.class() == WindowClass::Narrow);
        let shown = if narrow { Visibility::Visible } else { Visibility::Collapsed };
        named::<IUIElement>(&self.root, "Search")?.SetVisibility(shown)?;
        // A narrow window keeps 推荐, 热门 and 影视 in the row.
        let wide_only = if narrow { Visibility::Collapsed } else { Visibility::Visible };
        named::<IUIElement>(&self.root, "Tab2")?.SetVisibility(wide_only)?;
        named::<IUIElement>(&self.root, "Tab3")?.SetVisibility(wide_only)?;
        // A narrow window keeps the 竖屏 entry as its icon alone.
        named::<IUIElement>(&self.root, "PortraitLabel")?.SetVisibility(wide_only)?;
        // Only a narrow window chooses between one column and two.
        named::<IUIElement>(&self.root, "Columns")?.SetVisibility(if narrow {
            Visibility::Visible
        } else {
            Visibility::Collapsed
        })?;
        // The button says what a click gives, not what is on.
        named::<IUIElement>(&self.root, "ZoneModes")?.SetVisibility(if self.current.get() == ZONES_TAB {
            Visibility::Visible
        } else {
            Visibility::Collapsed
        })?;
        let tint = |on: bool| SolidColorBrush::CreateInstanceWithColor(if on { COBALT } else { DIM });
        named::<IControl>(&self.root, "ZoneFeed")?.SetForeground(&tint(!self.zone_rank.get())?)?;
        named::<IControl>(&self.root, "ZoneRank")?.SetForeground(&tint(self.zone_rank.get())?)?;
        let single = shell().is_some_and(|s| s.prefs().single_column);
        let word = if single { "双列" } else { "单列" };
        named::<IContentControl>(&self.root, "Columns")?.SetContent(&PropertyValue::CreateString(word)?)
    }

    fn reopen(&self) -> Result<()> {
        let tab = self.current.get();
        if tab == PGC_TAB {
            return self.show_tab();
        }
        if let Some(source) = tab_source(tab).filter(|_| self.grids[tab].source().is_none()) {
            self.grids[tab].open(source)?;
        }
        self.show_tab()
    }

    /// 分区 only offers its menu: the page on show stays until a 分区 is picked.
    fn select(self: &Rc<Self>, tab: usize) -> Result<()> {
        if tab == ZONES_TAB {
            return self.pick_zone();
        }
        if tab == PGC_TAB {
            return self.pick_pgc();
        }
        self.current.set(tab);
        self.reopen()
    }

    fn show_tab(&self) -> Result<()> {
        let current = self.current.get();
        for (tab, grid) in self.grids.iter().enumerate() {
            let on = tab == current;
            let shown = if on { Visibility::Visible } else { Visibility::Collapsed };
            grid.element().cast::<IUIElement>()?.SetVisibility(shown)?;
            let color = if on { COBALT } else { DIM };
            named::<IControl>(&self.root, &format!("Tab{tab}"))?
                .SetForeground(&SolidColorBrush::CreateInstanceWithColor(color)?)?;
        }
        let pgc = current == PGC_TAB;
        for (season_type, zone) in self.pgc.borrow().iter() {
            let on = pgc && *season_type == self.pgc_on.get();
            let shown = if on { Visibility::Visible } else { Visibility::Collapsed };
            zone.element().cast::<IUIElement>()?.SetVisibility(shown)?;
        }
        named::<IControl>(&self.root, "Tab4")?.SetForeground(&SolidColorBrush::CreateInstanceWithColor(if pgc {
            COBALT
        } else {
            DIM
        })?)?;
        let said = match self.pgc_shown() {
            Some(zone) => zone.said(),
            None if pgc => String::new(),
            None => self.status_of(current),
        };
        self.say(&said);
        Ok(())
    }

    /// Switches a narrow window between one column and two.
    fn toggle_columns(self: &Rc<Self>) -> Result<()> {
        let Some(shell) = shell() else { return Ok(()) };
        let mut prefs = shell.prefs();
        prefs.single_column = !prefs.single_column;
        shell.set_prefs(prefs)?;
        for grid in &self.grids {
            grid.resized();
        }
        self.style()
    }

    /// What the status line says for `tab`: what went wrong, else what the last refresh did.
    fn status_of(&self, tab: usize) -> String {
        let grid = &self.grids[tab];
        let said = grid.said();
        if !said.is_empty() {
            return said;
        }
        match grid.note() {
            Some(note) => {
                format!("新增 {} 条 · 过滤 {} 条 · 重复 {} 条", note.kept, note.blocked, note.dup)
            }
            None => String::new(),
        }
    }

    fn pick_zone(self: &Rc<Self>) -> Result<()> {
        let items: String =
            HOME_ZONES.iter().map(|(name, _, _)| format!(r##"<MenuFlyoutItem Text="{name}" />"##)).collect();
        let menu = load_object(&format!(
            r##"<MenuFlyout xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">{items}</MenuFlyout>"##
        ))?;
        let listed = menu.cast::<IMenuFlyout>()?.Items()?;
        for (at, (name, region, _)) in HOME_ZONES.iter().enumerate() {
            let weak = Rc::downgrade(self);
            let (name, region) = (*name, *region);
            let item = listed.GetAt(at as u32)?.cast::<IMenuFlyoutItem>()?;
            keep(item.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                if let Some(page) = weak.upgrade() {
                    let _ = page.open_zone(name, region);
                }
            })?);
        }
        menu.cast::<IFlyoutBase>()?.ShowAt(&named::<FrameworkElement>(&self.root, "Tab3")?)
    }

    fn pick_pgc(self: &Rc<Self>) -> Result<()> {
        let items: String =
            PGC_ZONES.iter().map(|(name, _)| format!(r##"<MenuFlyoutItem Text="{name}" />"##)).collect();
        let menu = load_object(&format!(
            r##"<MenuFlyout xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">{items}</MenuFlyout>"##
        ))?;
        let listed = menu.cast::<IMenuFlyout>()?.Items()?;
        for (at, (name, season_type)) in PGC_ZONES.iter().enumerate() {
            let weak = Rc::downgrade(self);
            let (name, season_type) = (*name, *season_type);
            let item = listed.GetAt(at as u32)?.cast::<IMenuFlyoutItem>()?;
            keep(item.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                if let Some(page) = weak.upgrade() {
                    let _ = page.open_pgc(name, season_type);
                }
            })?);
        }
        menu.cast::<IFlyoutBase>()?.ShowAt(&named::<FrameworkElement>(&self.root, "Tab4")?)
    }

    /// Shows 影视 专区 `season_type`, made the first time.
    fn open_pgc(&self, name: &str, season_type: u32) -> Result<()> {
        named::<IContentControl>(&self.root, "Tab4")?.SetContent(&PropertyValue::CreateString(name)?)?;
        let made = self.pgc.borrow().get(&season_type).cloned();
        let zone = match made {
            Some(zone) => zone,
            None => {
                let zone = PgcZone::new(season_type, name, || {
                    if let Some(home) = shell().and_then(|s| s.home_page()) {
                        home.grid_spoke(PGC_TAB);
                    }
                })?;
                named::<IPanel>(&self.root, "Grids")?
                    .Children()?
                    .cast::<IVector<UIElement>>()?
                    .Append(&zone.element().cast::<UIElement>()?)?;
                self.pgc.borrow_mut().insert(season_type, zone.clone());
                zone
            }
        };
        self.current.set(PGC_TAB);
        self.pgc_on.set(season_type);
        self.show_tab()?;
        zone.start();
        Ok(())
    }

    fn open_zone(&self, name: &str, region: u32) -> Result<()> {
        named::<IContentControl>(&self.root, "Tab3")?.SetContent(&PropertyValue::CreateString(name)?)?;
        self.current.set(ZONES_TAB);
        self.zone_region.set(region);
        let source = self.zone_source();
        if self.grids[ZONES_TAB].source() != Some(source.clone()) {
            self.grids[ZONES_TAB].open(source)?;
        }
        self.show_tab()
    }

    /// What 分区 should show: the zone's feed, or its ranking.
    fn zone_source(&self) -> Source {
        let region = self.zone_region.get();
        if self.zone_rank.get() { Source::ZoneRanking(region) } else { Source::Zone(region) }
    }

    /// Switches 分区 between its feed and its ranking.
    fn show_zone(&self, ranking: bool) -> Result<()> {
        if self.zone_rank.get() == ranking {
            return Ok(());
        }
        self.zone_rank.set(ranking);
        let source = self.zone_source();
        if self.grids[ZONES_TAB].source() != Some(source.clone()) {
            self.grids[ZONES_TAB].open(source)?;
        }
        self.show_tab()
    }

    /// The grid of `tab` has something new to say.
    fn grid_spoke(&self, tab: usize) {
        if tab != self.current.get() {
            return;
        }
        match self.pgc_shown() {
            Some(zone) => self.say(&zone.said()),
            None if tab == PGC_TAB => {}
            None => self.say(&self.status_of(tab)),
        }
    }

    /// Sets the cells of the grid on show.
    pub fn resized(&self) {
        match self.pgc_shown() {
            Some(zone) => {
                let _ = zone.fit();
            }
            None if self.current.get() == PGC_TAB => {}
            None => self.grids[self.current.get()].resized(),
        }
    }
}
