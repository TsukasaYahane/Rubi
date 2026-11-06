//! 首页: 推荐, 热门, 排行 and a 分区 to pick. Each tab keeps its own grid, so it comes back where it
//! was left.

use std::cell::Cell;
use std::rc::Rc;

use rubi_core::HOME_ZONES;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::layout::WindowClass;
use crate::ui::card_grid::{CardGrid, Source};
use crate::ui::shell::{Tab, shell, show_tab};
use crate::ui::{REFRESH_BUTTON, keep, load, load_object, named, on_click, set_text, style_refresh};
use crate::xaml::{
    Color, FrameworkElement, IContentControl, IControl, IFlyoutBase, IFrameworkElement, IMenuFlyout, IMenuFlyoutItem,
    IPanel, IUIElement, PropertyValue, RoutedEventArgs, SolidColorBrush, UIElement, Visibility,
};
use windows_collections::IVector;

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };
const TAB_LABELS: [&str; 4] = ["推荐", "热门", "排行", "分区"];
const ZONES_TAB: usize = 3;

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" />
    <RowDefinition Height="*" />
  </Grid.RowDefinitions>
  <Grid Margin="16,8,16,8">
    <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
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
      <Button x:Name="Tab2" Content="排行" /><Button x:Name="Tab3" Content="分区" />
    </StackPanel>
    <TextBlock x:Name="Status" Grid.Column="1" FontSize="13" VerticalAlignment="Center" TextTrimming="CharacterEllipsis"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    <!--refresh-->
    <Button x:Name="Search" Grid.Column="3" Visibility="Collapsed" Background="Transparent" BorderThickness="0"
            Padding="8,2" VerticalAlignment="Center"><FontIcon Glyph="&#xE721;" FontSize="14" /></Button>
  </Grid>
  <Grid x:Name="Grids" Grid.Row="1" />
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
}

impl HomePage {
    pub fn new() -> Result<Rc<Self>> {
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
        let page = Rc::new(Self { root, grids, current: Cell::new(0) });
        for (tab, _) in TAB_LABELS.iter().enumerate() {
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
        on_click(&page.root, "Search", || show_tab(Tab::Search))?;
        page.show_tab()?;
        Ok(page)
    }

    pub fn root(&self) -> &IFrameworkElement {
        &self.root
    }

    pub fn say(&self, text: &str) {
        let _ = set_text(&self.root, "Status", text);
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
        self.grids[self.current.get()].refresh()
    }

    /// Follows the interface style.
    pub fn style(&self) -> Result<()> {
        style_refresh(&self.root)?;
        let narrow = shell().is_some_and(|s| s.class() == WindowClass::Narrow);
        let shown = if narrow { Visibility::Visible } else { Visibility::Collapsed };
        named::<IUIElement>(&self.root, "Search")?.SetVisibility(shown)
    }

    fn reopen(&self) -> Result<()> {
        let tab = self.current.get();
        if let Some(source) = tab_source(tab).filter(|_| self.grids[tab].source().is_none()) {
            self.grids[tab].open(source)?;
        }
        self.show_tab()
    }

    fn select(self: &Rc<Self>, tab: usize) -> Result<()> {
        self.current.set(tab);
        self.reopen()?;
        if tab == ZONES_TAB {
            self.pick_zone()?;
        }
        Ok(())
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
        let waiting = current == ZONES_TAB && self.grids[current].source().is_none();
        self.say(&if waiting { "选一个分区".to_owned() } else { self.grids[current].said() });
        Ok(())
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

    fn open_zone(&self, name: &str, region: u32) -> Result<()> {
        named::<IContentControl>(&self.root, "Tab3")?.SetContent(&PropertyValue::CreateString(name)?)?;
        self.grids[ZONES_TAB].open(Source::Zone(region))
    }

    /// The grid of `tab` has something new to say.
    fn grid_spoke(&self, tab: usize) {
        if tab == self.current.get() {
            self.say(&self.grids[tab].said());
        }
    }

    /// Sets the cells of the grid on show.
    pub fn resized(&self) {
        self.grids[self.current.get()].resized();
    }
}
