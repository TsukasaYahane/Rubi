//! 直播: rooms by area, and 赛事. The page lives on while a room is open, so coming back finds the
//! same place.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rubi_model::LiveArea;
use windows_collections::IVector;
use windows_core::{Interface, Result};

use crate::state::settings::escape;
use crate::ui::card_grid::{CardGrid, Source};
use crate::ui::schedule::Schedule;
use crate::ui::shell::shell;
use crate::ui::{REFRESH_BUTTON, load, named, on_click, post, set_text, style_refresh};
use crate::xaml::{
    Color, IContentControl, IControl, IFrameworkElement, IPanel, IUIElement, SolidColorBrush, UIElement, Visibility,
};

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };
const CLEAR: Color = Color { a: 0, r: 0, g: 0, b: 0 };
const WHITE: Color = Color { a: 255, r: 255, g: 255, b: 255 };
const TABS: usize = 3;
const FOLLOWED_TAB: usize = 1;
const SCHEDULE_TAB: usize = 2;
/// Top areas shown beside 推荐.
const AREAS_SHOWN: usize = 12;

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
  </Grid.RowDefinitions>
  <Grid Margin="16,8,16,4">
    <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
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
      <Button x:Name="Tab0" Content="直播" /><Button x:Name="Tab1" Content="关注" />
      <Button x:Name="Tab2" Content="赛事" />
    </StackPanel>
    <TextBlock x:Name="Status" Grid.Column="1" FontSize="13" VerticalAlignment="Center" TextTrimming="CharacterEllipsis"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    <!--refresh-->
  </Grid>
  <ScrollViewer Grid.Row="1" HorizontalScrollBarVisibility="Hidden" HorizontalScrollMode="Enabled"
                VerticalScrollMode="Disabled" VerticalScrollBarVisibility="Disabled">
    <ContentControl x:Name="Chips" IsTabStop="False" Margin="16,2,16,8" />
  </ScrollViewer>
  <Grid x:Name="Lists" Grid.Row="2" />
</Grid>"##;

/// A row of labels of which one is on, `Chip0`, `Chip1`… under the returned element.
pub fn chips(labels: &[String]) -> Result<IFrameworkElement> {
    let buttons: String = labels
        .iter()
        .enumerate()
        .map(|(at, label)| {
            format!(
                r##"<Button x:Name="Chip{at}" Content="{}" FontSize="13" Padding="10,3" Margin="0,0,6,0" BorderThickness="1"
                     BorderBrush="{{ThemeResource SystemControlForegroundBaseLowBrush}}" Background="Transparent" />"##,
                escape(label)
            )
        })
        .collect();
    load(&format!(
        r##"<StackPanel xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
             xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Orientation="Horizontal">{buttons}</StackPanel>"##
    ))
}

/// Paints which of `count` chips is on.
pub fn mark_chip(row: &IFrameworkElement, count: usize, on: usize) -> Result<()> {
    for at in 0..count {
        let chip = named::<IControl>(row, &format!("Chip{at}"))?;
        chip.SetBackground(&SolidColorBrush::CreateInstanceWithColor(if at == on { COBALT } else { CLEAR })?)?;
        if at == on {
            chip.SetForeground(&SolidColorBrush::CreateInstanceWithColor(WHITE)?)?;
        } else {
            chip.cast::<crate::xaml::IDependencyObject>()?.ClearValue(&crate::xaml::Control::ForegroundProperty()?)?;
        }
    }
    Ok(())
}

pub struct LivePage {
    root: IFrameworkElement,
    rooms: Rc<CardGrid>,
    followed: Rc<CardGrid>,
    schedule: Rc<Schedule>,
    tab: Cell<usize>,
    areas: RefCell<Vec<LiveArea>>,
    /// The row of area chips, once the areas are known.
    area_chips: RefCell<Option<IFrameworkElement>>,
}

impl LivePage {
    fn new() -> Result<Rc<Self>> {
        let root = load(&LAYOUT.replace("<!--refresh-->", REFRESH_BUTTON))?;
        let rooms = CardGrid::new(|| {
            if let Some(page) = held() {
                page.show_status();
            }
        })?;
        let followed = CardGrid::new(|| {
            if let Some(page) = held() {
                page.show_status();
            }
        })?;
        let schedule = Schedule::new()?;
        let lists = named::<IPanel>(&root, "Lists")?.Children()?.cast::<IVector<UIElement>>()?;
        lists.Append(&rooms.element().cast::<UIElement>()?)?;
        lists.Append(&followed.element().cast::<UIElement>()?)?;
        lists.Append(&schedule.root().cast::<UIElement>()?)?;
        let page = Rc::new(Self {
            root,
            rooms,
            followed,
            schedule,
            tab: Cell::new(0),
            areas: RefCell::default(),
            area_chips: RefCell::default(),
        });
        let weak = Rc::downgrade(&page);
        on_click(&page.root, "Refresh", move || {
            if let Some(page) = weak.upgrade() {
                let _ = page.refresh_list();
            }
        })?;
        for tab in 0..TABS {
            let weak = Rc::downgrade(&page);
            on_click(&page.root, &format!("Tab{tab}"), move || {
                if let Some(page) = weak.upgrade() {
                    let _ = page.select(tab);
                }
            })?;
        }
        Ok(page)
    }

    /// Reads the list on show again.
    fn refresh_list(&self) -> Result<()> {
        match self.tab.get() {
            0 => self.rooms.refresh(),
            FOLLOWED_TAB => self.followed.refresh(),
            _ => self.schedule.refresh(),
        }
    }

    fn select(&self, tab: usize) -> Result<()> {
        self.tab.set(tab);
        self.apply()
    }

    fn apply(&self) -> Result<()> {
        let tab = self.tab.get();
        let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
        self.rooms.element().cast::<IUIElement>()?.SetVisibility(shown(tab == 0))?;
        self.followed.element().cast::<IUIElement>()?.SetVisibility(shown(tab == FOLLOWED_TAB))?;
        self.schedule.root().cast::<IUIElement>()?.SetVisibility(shown(tab == SCHEDULE_TAB))?;
        for at in 0..TABS {
            let color = if at == tab { COBALT } else { DIM };
            named::<IControl>(&self.root, &format!("Tab{at}"))?
                .SetForeground(&SolidColorBrush::CreateInstanceWithColor(color)?)?;
        }
        let chips_host = named::<IContentControl>(&self.root, "Chips")?;
        chips_host.SetContent(None::<&windows_core::IInspectable>)?;
        if tab == 0 {
            if self.rooms.source().is_none() {
                self.rooms.open(Source::Rooms(None))?;
            }
            if let Some(row) = self.area_chips.borrow().as_ref() {
                chips_host.SetContent(&row.cast::<UIElement>()?)?;
            }
        } else if tab == FOLLOWED_TAB {
            if self.followed.source().is_none() && signed_in() {
                self.followed.open(Source::FollowedRooms)?;
            }
        } else {
            self.schedule.start()?;
            if let Some(row) = self.schedule.chips() {
                chips_host.SetContent(&row.cast::<UIElement>()?)?;
            }
        }
        self.show_status();
        Ok(())
    }

    fn show_status(&self) {
        let said = match self.tab.get() {
            0 => self.rooms.said(),
            FOLLOWED_TAB if !signed_in() => "登录后查看关注的直播".to_owned(),
            FOLLOWED_TAB => self.followed.said(),
            _ => self.schedule.said(),
        };
        let _ = set_text(&self.root, "Status", &said);
    }

    fn load_areas(self: &Rc<Self>) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if !self.areas.borrow().is_empty() {
            return;
        }
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let Ok(areas) = core.services.live.areas().await else { return };
            post(&dispatcher, move || {
                if let Some(page) = held() {
                    let _ = page.took_areas(areas);
                }
            });
        });
    }

    fn took_areas(self: &Rc<Self>, mut areas: Vec<LiveArea>) -> Result<()> {
        areas.truncate(AREAS_SHOWN);
        let mut labels = vec!["推荐".to_owned()];
        labels.extend(areas.iter().map(|a| a.name.clone()));
        let row = chips(&labels)?;
        mark_chip(&row, labels.len(), 0)?;
        for at in 0..labels.len() {
            let (weak, shown, count) = (Rc::downgrade(self), row.clone(), labels.len());
            let area = at.checked_sub(1).and_then(|a| areas.get(a)).map(|a| a.id);
            on_click(&row, &format!("Chip{at}"), move || {
                if let Some(page) = weak.upgrade() {
                    let _ = mark_chip(&shown, count, at);
                    let _ = page.rooms.open(Source::Rooms(area));
                }
            })?;
        }
        *self.areas.borrow_mut() = areas;
        *self.area_chips.borrow_mut() = Some(row);
        self.apply()
    }

    pub fn schedule_answered(
        &self,
        turn: u64,
        answer: std::result::Result<(Vec<rubi_model::EsportsMatch>, bool), String>,
    ) -> Result<()> {
        self.schedule.took(turn, answer)
    }

    pub fn schedule_match(&self, at: usize) -> Option<rubi_model::EsportsMatch> {
        self.schedule.at(at)
    }

    pub fn schedule_changed(&self, at: usize, game: rubi_model::EsportsMatch) {
        let _ = self.schedule.changed(at, game);
    }

    pub fn schedule_games(&self, games: Vec<rubi_model::EsportsGame>) -> Result<()> {
        self.schedule.took_games(games)
    }

    /// The schedule has something new to show.
    pub fn refresh(&self) {
        let _ = self.apply();
    }
}

thread_local! {
    static PAGE: RefCell<Option<Rc<LivePage>>> = const { RefCell::new(None) };
}

pub fn held() -> Option<Rc<LivePage>> {
    PAGE.with_borrow(Clone::clone)
}

fn signed_in() -> bool {
    shell().and_then(|s| s.backend.get().cloned()).is_some_and(|b| b.services.account.is_logged_in())
}

/// Forgets whose rooms were listed: the account changed.
pub fn forget() {
    if let Some(page) = held() {
        let _ = page.followed.forget();
    }
}

/// Shows the 直播 tab.
pub fn show() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let page = match held() {
        Some(page) => page,
        None => {
            let page = LivePage::new()?;
            PAGE.with_borrow_mut(|p| *p = Some(page.clone()));
            page
        }
    };
    style_refresh(&page.root)?;
    shell.set_page(&page.root)?;
    page.apply()?;
    page.load_areas();
    Ok(())
}

/// Sets the cells of the grid.
pub fn resized() {
    if let Some(page) = held() {
        page.rooms.resized();
        page.followed.resized();
        page.schedule.resized();
    }
}
