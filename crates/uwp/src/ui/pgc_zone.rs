//! A 影视 专区 inside 首页: the 番剧 release timeline, filters and a wall of posters. Each 专区 keeps
//! its own page, so coming back finds the same filters and place.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use rubi_core::{PgcConditions, PgcDay, PgcSeason};
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::pgc::{paid, poster_cell, poster_url, season_line};
use crate::state::settings::escape;
use crate::ui::card_menu::{self, Choice};
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{keep, load, named, on_click, pgc_season, post};
use crate::xaml::{
    Color, FrameworkElement, IBorder, IContentControl, IFrameworkElement, IImage, IItemsControl, IItemsControl2,
    IItemsWrapGrid, IPanel, ITextBlock, IUIElement, SizeChangedEventArgs, SolidColorBrush, UIElement, Visibility,
};

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const RED: Color = Color { a: 255, r: 0xE8, g: 0x11, b: 0x23 };
const POSTER_DECODE_WIDTH: i32 = 300;
/// Releases shown under a day of the timeline.
const RELEASES_SHOWN: usize = 6;
/// Rows from the end at which more load.
const LOAD_AHEAD: usize = 10;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
  </Grid.RowDefinitions>
  <ContentControl x:Name="TimelineHost" IsTabStop="False" HorizontalContentAlignment="Stretch" Margin="16,0,16,6" />
  <ContentControl x:Name="FilterHost" Grid.Row="1" IsTabStop="False" HorizontalContentAlignment="Stretch"
                  Margin="16,0,16,6" />
  <GridView x:Name="Posters" Grid.Row="2" SelectionMode="None" IsItemClickEnabled="True" Padding="16,0,4,0">
    <GridView.ItemContainerTransitions><TransitionCollection /></GridView.ItemContainerTransitions>
    <GridView.ItemContainerStyle>
      <Style TargetType="GridViewItem">
        <Setter Property="Margin" Value="0" />
        <Setter Property="Padding" Value="0,0,12,12" />
        <Setter Property="HorizontalContentAlignment" Value="Stretch" />
        <Setter Property="VerticalContentAlignment" Value="Stretch" />
      </Style>
    </GridView.ItemContainerStyle>
    <GridView.ItemsPanel><ItemsPanelTemplate><ItemsWrapGrid Orientation="Horizontal" /></ItemsPanelTemplate></GridView.ItemsPanel>
    <GridView.ItemTemplate><DataTemplate>
      <Grid>
        <Grid.RowDefinitions>
          <RowDefinition Height="*" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
          <RowDefinition Height="Auto" />
        </Grid.RowDefinitions>
        <Grid Background="{ThemeResource SystemControlBackgroundBaseLowBrush}">
          <Image Stretch="UniformToFill" />
          <Border HorizontalAlignment="Right" VerticalAlignment="Top" Padding="5,1">
            <TextBlock FontSize="10" Foreground="White" />
          </Border>
        </Grid>
        <TextBlock Grid.Row="1" FontSize="13" Margin="0,4,0,0" TextTrimming="CharacterEllipsis" />
        <TextBlock Grid.Row="2" FontSize="11" Foreground="#E0A000" TextTrimming="CharacterEllipsis" />
        <TextBlock Grid.Row="3" FontSize="11" TextTrimming="CharacterEllipsis"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      </Grid>
    </DataTemplate></GridView.ItemTemplate>
  </GridView>
</Grid>"##;

const CHIP: &str = r##"<Button x:Name="{name}" FontSize="13" Padding="10,3" Margin="0,0,6,6" BorderThickness="1"
  Background="Transparent" BorderBrush="{ThemeResource SystemControlForegroundBaseMediumLowBrush}">
  <StackPanel Orientation="Horizontal"><TextBlock Text="{label}" />
  <FontIcon Glyph="&#xE70D;" FontSize="10" Margin="6,0,0,0" VerticalAlignment="Center" /></StackPanel></Button>"##;

thread_local! {
    static ZONES: RefCell<HashMap<u64, Weak<PgcZone>>> = RefCell::default();
}

fn zone_of(id: u64) -> Option<Rc<PgcZone>> {
    ZONES.with_borrow(|z| z.get(&id).and_then(Weak::upgrade))
}

pub struct PgcZone {
    id: u64,
    root: IFrameworkElement,
    posters: IFrameworkElement,
    season_type: u32,
    name: String,
    seasons: RefCell<Vec<PgcSeason>>,
    page: Cell<u32>,
    more: Cell<bool>,
    loading: Cell<bool>,
    /// Bumped when the wall starts over: answers for an earlier one are dropped.
    turn: Cell<u64>,
    conditions: RefCell<PgcConditions>,
    /// The filters picked, as (field, value); absent is 全部.
    picked: RefCell<Vec<(String, String)>>,
    order: RefCell<String>,
    /// The timeline's releases by their buttons' order: (season id, title).
    releases: RefCell<Vec<(u64, String)>>,
    said: RefCell<String>,
    spoke: Box<dyn Fn()>,
    covers: Covers,
    fitted: Cell<(f64, u32)>,
}

impl PgcZone {
    /// The page of 专区 `season_type`, called `name`; `spoke` hears when it has something to say.
    pub fn new(season_type: u32, name: &str, spoke: impl Fn() + 'static) -> Result<Rc<Self>> {
        let root = load(LAYOUT)?;
        let posters = named::<IFrameworkElement>(&root, "Posters")?;
        let zone = Rc::new(Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            root,
            posters: posters.clone(),
            season_type,
            name: name.to_owned(),
            seasons: RefCell::default(),
            page: Cell::new(1),
            more: Cell::new(true),
            loading: Cell::new(false),
            turn: Cell::new(0),
            conditions: RefCell::default(),
            picked: RefCell::default(),
            order: RefCell::default(),
            releases: RefCell::default(),
            said: RefCell::default(),
            spoke: Box::new(spoke),
            covers: Covers::default(),
            fitted: Cell::new((0.0, 0)),
        });
        ZONES.with_borrow_mut(|z| z.insert(zone.id, Rc::downgrade(&zone)));
        let filling = Rc::downgrade(&zone);
        on_row(&posters, move |at, parts| filling.upgrade().map_or(Ok(()), |z| z.fill(at, parts)))?;
        let opening = Rc::downgrade(&zone);
        on_row_click(&posters, move |at| {
            if let Some(zone) = opening.upgrade() {
                let season = zone.seasons.borrow().get(at).map(|s| (s.season_id, s.title.clone()));
                if let Some((id, title)) = season {
                    let _ = pgc_season::open(id, &title);
                }
            }
        })?;
        let sizing = Rc::downgrade(&zone);
        keep(posters.SizeChanged(move |_: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
            if let Some(zone) = sizing.upgrade() {
                let _ = zone.fit();
            }
        })?);
        Ok(zone)
    }

    pub fn element(&self) -> &IFrameworkElement {
        &self.root
    }

    pub fn said(&self) -> String {
        self.said.borrow().clone()
    }

    fn say(&self, text: &str) {
        *self.said.borrow_mut() = text.to_owned();
        (self.spoke)();
    }

    /// Reads the timeline, the filters and the first posters, once.
    pub fn start(self: &Rc<Self>) {
        if !self.seasons.borrow().is_empty() || self.loading.get() {
            return;
        }
        self.load_frame();
    }

    /// Reads everything again.
    pub fn refresh(self: &Rc<Self>) {
        self.load_frame();
    }

    fn load_frame(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        self.say("正在读取…");
        let (id, season_type, dispatcher, core) =
            (self.id, self.season_type, shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let pgc = &core.services.pgc;
            let (days, conditions) = tokio::join!(pgc.timeline(season_type), pgc.conditions(season_type));
            let (days, conditions) = (days.unwrap_or_default(), conditions.unwrap_or_default());
            post(&dispatcher, move || {
                if let Some(zone) = zone_of(id) {
                    if let Err(problem) = zone.took_frame(days, conditions) {
                        zone.say(&format!("显示失败：{}", problem.message()));
                    }
                }
            });
        });
    }

    fn took_frame(self: &Rc<Self>, days: Vec<PgcDay>, conditions: PgcConditions) -> Result<()> {
        if self.order.borrow().is_empty() {
            *self.order.borrow_mut() = conditions.orders.first().map(|(value, _)| value.clone()).unwrap_or_default();
        }
        *self.conditions.borrow_mut() = conditions;
        self.show_timeline(&days)?;
        self.show_filters()?;
        self.restart()
    }

    fn show_timeline(&self, days: &[PgcDay]) -> Result<()> {
        let host = named::<IContentControl>(&self.root, "TimelineHost")?;
        if days.is_empty() {
            let note = format!(
                r##"<TextBlock xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" FontSize="12"
                Text="{}没有放送时间表，排序选「更新时间」可看近期上线"
                Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />"##,
                escape(&self.name)
            );
            return host.SetContent(&load(&note)?.cast::<UIElement>()?);
        }
        let mut releases = Vec::new();
        let mut columns = String::new();
        for day in days {
            let mut lines = String::new();
            for release in day.releases.iter().take(RELEASES_SHOWN) {
                let at = releases.len();
                releases.push((release.season_id, release.title.clone()));
                lines.push_str(&format!(
                    r##"<Button x:Name="Rel{at}" Background="Transparent" BorderThickness="0" Padding="0,2"
                      HorizontalAlignment="Stretch" HorizontalContentAlignment="Left"><TextBlock FontSize="12"
                      TextTrimming="CharacterEllipsis"><Run Text="{}" Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
                      <Run Text="{}" /> <Run Text="{}" Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" /></TextBlock></Button>"##,
                    escape(&release.time),
                    escape(&release.title),
                    escape(&release.index)
                ));
            }
            let (label, tint, back) = if day.today {
                ("今天".to_owned(), "#0050EF", "#1A0050EF")
            } else {
                (day.date.clone(), "", "Transparent")
            };
            let tint = if tint.is_empty() { String::new() } else { format!(r#" Foreground="{tint}""#) };
            columns.push_str(&format!(
                r##"<Border Width="170" Padding="8,4" Background="{back}" BorderThickness="0,0,1,0"
                  BorderBrush="{{ThemeResource SystemControlForegroundBaseLowBrush}}"><StackPanel>
                  <TextBlock Text="{}" FontSize="13"{tint} />{lines}</StackPanel></Border>"##,
                escape(&label)
            ));
        }
        let timeline = load(&format!(
            r##"<StackPanel xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
              <TextBlock Text="近期上映" FontSize="16" FontWeight="Light" Margin="0,0,0,4" />
              <ScrollViewer HorizontalScrollMode="Enabled" HorizontalScrollBarVisibility="Auto"
                VerticalScrollMode="Disabled" VerticalScrollBarVisibility="Disabled" MaxHeight="190"
                BorderThickness="1" BorderBrush="{{ThemeResource SystemControlForegroundBaseLowBrush}}">
                <StackPanel Orientation="Horizontal">{columns}</StackPanel></ScrollViewer></StackPanel>"##
        ))?;
        for (at, (season, title)) in releases.iter().cloned().enumerate() {
            on_click(&timeline, &format!("Rel{at}"), move || {
                let _ = pgc_season::open(season, &title);
            })?;
        }
        *self.releases.borrow_mut() = releases;
        host.SetContent(&timeline.cast::<UIElement>()?)
    }

    fn show_filters(self: &Rc<Self>) -> Result<()> {
        let conditions = self.conditions.borrow();
        let picked = self.picked.borrow();
        let mut chips = String::new();
        for (at, filter) in conditions.filters.iter().enumerate() {
            let chosen = picked
                .iter()
                .find(|(field, _)| *field == filter.field)
                .and_then(|(_, value)| filter.values.iter().find(|(v, _)| v == value))
                .map(|(_, name)| name.clone());
            let label = match chosen {
                Some(name) => format!("{}：{name}", filter.name),
                None => filter.name.clone(),
            };
            chips.push_str(&CHIP.replace("{name}", &format!("Filter{at}")).replace("{label}", &escape(&label)));
        }
        let order = conditions
            .orders
            .iter()
            .find(|(value, _)| *value == *self.order.borrow())
            .map_or_else(|| "排序".to_owned(), |(_, name)| name.clone());
        chips.push_str(&CHIP.replace("{name}", "Order").replace("{label}", &escape(&order)));
        let count = conditions.filters.len();
        drop((conditions, picked));
        let row = load(&format!(
            r##"<ScrollViewer xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
                HorizontalScrollMode="Enabled" HorizontalScrollBarVisibility="Hidden" VerticalScrollMode="Disabled"
                VerticalScrollBarVisibility="Disabled"><StackPanel Orientation="Horizontal">{chips}</StackPanel></ScrollViewer>"##
        ))?;
        for at in 0..count {
            let weak = Rc::downgrade(self);
            let chip = row.clone();
            on_click(&row, &format!("Filter{at}"), move || {
                if let Some(zone) = weak.upgrade() {
                    let _ = zone.offer_filter(&chip, at);
                }
            })?;
        }
        let weak = Rc::downgrade(self);
        let chip = row.clone();
        on_click(&row, "Order", move || {
            if let Some(zone) = weak.upgrade() {
                let _ = zone.offer_orders(&chip);
            }
        })?;
        named::<IContentControl>(&self.root, "FilterHost")?.SetContent(&row.cast::<UIElement>()?)
    }

    fn offer_filter(self: &Rc<Self>, row: &IFrameworkElement, at: usize) -> Result<()> {
        let Some(filter) = self.conditions.borrow().filters.get(at).cloned() else { return Ok(()) };
        let current = self.picked.borrow().iter().find(|(f, _)| *f == filter.field).map(|(_, v)| v.clone());
        let choices: Vec<Choice> = filter
            .values
            .iter()
            .map(|(value, name)| {
                let mark = if current.as_deref() == Some(value.as_str()) { "● " } else { "" };
                let (weak, field, value) = (Rc::downgrade(self), filter.field.clone(), value.clone());
                let act: Rc<dyn Fn()> = Rc::new(move || {
                    if let Some(zone) = weak.upgrade() {
                        zone.picked.borrow_mut().retain(|(f, _)| *f != field);
                        zone.picked.borrow_mut().push((field.clone(), value.clone()));
                        let _ = zone.show_filters();
                        let _ = zone.restart();
                    }
                });
                (format!("{mark}{name}"), act)
            })
            .collect();
        card_menu::show_under(&named::<FrameworkElement>(row, &format!("Filter{at}"))?, choices)
    }

    fn offer_orders(self: &Rc<Self>, row: &IFrameworkElement) -> Result<()> {
        let current = self.order.borrow().clone();
        let choices: Vec<Choice> = self
            .conditions
            .borrow()
            .orders
            .iter()
            .map(|(value, name)| {
                let mark = if *value == current { "● " } else { "" };
                let (weak, value) = (Rc::downgrade(self), value.clone());
                let act: Rc<dyn Fn()> = Rc::new(move || {
                    if let Some(zone) = weak.upgrade() {
                        *zone.order.borrow_mut() = value.clone();
                        let _ = zone.show_filters();
                        let _ = zone.restart();
                    }
                });
                (format!("{mark}{name}"), act)
            })
            .collect();
        card_menu::show_under(&named::<FrameworkElement>(row, "Order")?, choices)
    }

    fn restart(&self) -> Result<()> {
        self.turn.set(self.turn.get() + 1);
        self.seasons.borrow_mut().clear();
        self.page.set(1);
        self.more.set(true);
        self.loading.set(false);
        self.posters.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        self.load();
        Ok(())
    }

    fn load(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if !self.more.get() || self.loading.replace(true) {
            return;
        }
        let (id, turn, season_type, page) = (self.id, self.turn.get(), self.season_type, self.page.get());
        let (filters, order) = (self.picked.borrow().clone(), self.order.borrow().clone());
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let found = core.services.pgc.index(season_type, &filters, &order, page).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(zone) = zone_of(id).filter(|z| z.turn.get() == turn) {
                    let _ = zone.took(found);
                }
            });
        });
    }

    fn took(&self, found: std::result::Result<(Vec<PgcSeason>, bool), String>) -> Result<()> {
        self.loading.set(false);
        match found {
            Ok((seasons, more)) => {
                self.more.set(more && !seasons.is_empty());
                self.page.set(self.page.get() + 1);
                let from = self.seasons.borrow().len();
                self.seasons.borrow_mut().extend(seasons);
                let to = self.seasons.borrow().len();
                add_rows(&self.posters, from, to)?;
                self.say(if to == 0 { "没有符合筛选的作品" } else { "" });
            }
            Err(problem) => {
                self.more.set(false);
                self.say(&format!("读取失败：{problem}"));
            }
        }
        Ok(())
    }

    /// Sets the poster cells for the width there is.
    pub fn fit(&self) -> Result<()> {
        let width = self.posters.ActualWidth()? - 20.0;
        if width <= 0.0 {
            return Ok(());
        }
        let (columns, cell, height) = poster_cell(width);
        if self.fitted.get() == (width, columns) {
            return Ok(());
        }
        let Ok(panel) = self.posters.cast::<IItemsControl2>().and_then(|l| l.ItemsPanelRoot()) else { return Ok(()) };
        let panel = panel.cast::<IItemsWrapGrid>()?;
        panel.SetMaximumRowsOrColumns(columns as i32)?;
        panel.SetItemWidth(cell)?;
        panel.SetItemHeight(height)?;
        self.fitted.set((width, columns));
        Ok(())
    }

    fn fill(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        if self.fitted.get().1 == 0 {
            self.fit()?;
        }
        let wants_more = {
            let seasons = self.seasons.borrow();
            let Some(season) = seasons.get(at) else { return Ok(()) };
            let poster = parts.GetAt(0)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
            let url = poster_url(&season.cover, POSTER_DECODE_WIDTH as u32);
            poster.GetAt(0)?.cast::<IImage>()?.SetSource(&self.covers.get(&url, POSTER_DECODE_WIDTH)?)?;
            let badge = poster.GetAt(1)?.cast::<IBorder>()?;
            badge.cast::<IUIElement>()?.SetVisibility(if season.badge.is_empty() {
                Visibility::Collapsed
            } else {
                Visibility::Visible
            })?;
            badge.SetBackground(&SolidColorBrush::CreateInstanceWithColor(if paid(&season.badge) {
                RED
            } else {
                COBALT
            })?)?;
            badge.Child()?.cast::<ITextBlock>()?.SetText(&season.badge)?;
            parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&season.title)?;
            parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&season_line(&season.score, &season.index_show))?;
            parts.GetAt(3)?.cast::<ITextBlock>()?.SetText(&season.order_text)?;
            at + LOAD_AHEAD >= seasons.len()
        };
        if wants_more {
            self.load();
        }
        Ok(())
    }
}
