//! The window: navigation that follows its width, and the page on show.

use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::backend::Backend;
use crate::session;
use crate::state::layout::WindowClass;
use crate::ui::home::HomePage;
use crate::ui::{load, me, named, on_click, player, post};
use crate::xaml::{
    ApplicationData, Color, CoreDispatcher, IApplicationData, IContentControl, IControl, IFrameworkElement,
    IStorageItem, IUIElement, IWindow, SolidColorBrush, UIElement, Visibility, Window, WindowSizeChangedEventArgs,
};

const COBALT: Color = Color { a: 255, r: 0x00, g: 0x50, b: 0xEF };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Follow,
    Live,
    Me,
}

const TABS: [(Tab, &str); 4] = [(Tab::Home, "Home"), (Tab::Follow, "Follow"), (Tab::Live, "Live"), (Tab::Me, "Me")];

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
      Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
  <Grid.Resources>
    <Style x:Key="Nav" TargetType="Button">
      <Setter Property="Background" Value="Transparent" />
      <Setter Property="BorderThickness" Value="0" />
      <Setter Property="HorizontalAlignment" Value="Stretch" />
      <Setter Property="HorizontalContentAlignment" Value="Left" />
      <Setter Property="Height" Value="44" />
      <Setter Property="Padding" Value="14,0" />
    </Style>
    <Style x:Key="Bar" TargetType="Button" BasedOn="{StaticResource Nav}">
      <Setter Property="Height" Value="54" />
      <Setter Property="HorizontalContentAlignment" Value="Center" />
      <Setter Property="Padding" Value="0" />
    </Style>
  </Grid.Resources>
  <Grid.ColumnDefinitions>
    <ColumnDefinition Width="Auto" />
    <ColumnDefinition Width="*" />
  </Grid.ColumnDefinitions>
  <Grid.RowDefinitions>
    <RowDefinition Height="*" />
    <RowDefinition Height="Auto" />
  </Grid.RowDefinitions>
  <StackPanel x:Name="Rail" Grid.RowSpan="2" Padding="0,6,0,0"
              Background="{ThemeResource SystemControlBackgroundChromeMediumLowBrush}">
    <Button x:Name="RailHome" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon Glyph="&#xE80F;" FontSize="20" /><TextBlock x:Name="LabelHome" Margin="12,0,0,0" Text="首页" /></StackPanel></Button>
    <Button x:Name="RailFollow" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon Glyph="&#xEB51;" FontSize="20" /><TextBlock x:Name="LabelFollow" Margin="12,0,0,0" Text="关注" /></StackPanel></Button>
    <Button x:Name="RailLive" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon Glyph="&#xE93E;" FontSize="20" /><TextBlock x:Name="LabelLive" Margin="12,0,0,0" Text="直播" /></StackPanel></Button>
    <Button x:Name="RailMe" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon Glyph="&#xE77B;" FontSize="20" /><TextBlock x:Name="LabelMe" Margin="12,0,0,0" Text="我的" /></StackPanel></Button>
  </StackPanel>
  <ContentControl x:Name="Page" Grid.Column="1" HorizontalContentAlignment="Stretch"
                  VerticalContentAlignment="Stretch" IsTabStop="False" />
  <Grid x:Name="Bottom" Grid.Column="1" Grid.Row="1" BorderThickness="0,1,0,0"
        BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <Grid.ColumnDefinitions>
      <ColumnDefinition /><ColumnDefinition /><ColumnDefinition /><ColumnDefinition />
    </Grid.ColumnDefinitions>
    <Button x:Name="BarHome" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon Glyph="&#xE80F;" FontSize="20" /><TextBlock Text="首页" FontSize="11" /></StackPanel></Button>
    <Button x:Name="BarFollow" Grid.Column="1" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon Glyph="&#xEB51;" FontSize="20" /><TextBlock Text="关注" FontSize="11" /></StackPanel></Button>
    <Button x:Name="BarLive" Grid.Column="2" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon Glyph="&#xE93E;" FontSize="20" /><TextBlock Text="直播" FontSize="11" /></StackPanel></Button>
    <Button x:Name="BarMe" Grid.Column="3" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon Glyph="&#xE77B;" FontSize="20" /><TextBlock Text="我的" FontSize="11" /></StackPanel></Button>
  </Grid>
</Grid>"##;

const PLACEHOLDER: &str = r##"
<TextBlock xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
           Text="这一页还没做" FontSize="20" FontWeight="Light" Margin="16" />"##;

pub struct Shell {
    pub dispatcher: CoreDispatcher,
    pub backend: OnceCell<Arc<Backend>>,
    root: IFrameworkElement,
    page: IContentControl,
    class: Cell<WindowClass>,
    tab: Cell<Tab>,
    home: RefCell<Option<Rc<HomePage>>>,
}

thread_local! {
    static SHELL: RefCell<Option<Rc<Shell>>> = const { RefCell::new(None) };
}

/// The shell; on the UI thread once the window is up.
pub fn shell() -> Option<Rc<Shell>> {
    SHELL.with(|s| s.borrow().clone())
}

/// Builds the window, then opens the core in the background.
pub fn start() -> Result<()> {
    let window = Window::Current()?.cast::<IWindow>()?;
    let root = load(LAYOUT)?;
    let page = named::<IContentControl>(&root, "Page")?;
    let dispatcher = window.Dispatcher()?;
    let shell = Rc::new(Shell {
        dispatcher: dispatcher.clone(),
        backend: OnceCell::new(),
        root: root.clone(),
        page,
        class: Cell::new(WindowClass::of(window.Bounds()?.width as f64)),
        tab: Cell::new(Tab::Home),
        home: RefCell::new(None),
    });
    SHELL.with(|s| *s.borrow_mut() = Some(shell.clone()));

    for (tab, name) in TABS {
        for prefix in ["Rail", "Bar"] {
            on_click(&root, &format!("{prefix}{name}"), move || show_tab(tab))?;
        }
    }
    let revoker = window.SizeChanged(|_: Ref<IInspectable>, _: Ref<WindowSizeChangedEventArgs>| {
        let _ = window_resized();
    })?;
    std::mem::forget(revoker);

    shell.apply_class()?;
    window.SetContent(&root.cast::<UIElement>()?)?;
    window.Activate()?;
    shell.show_home()?;

    let folder = ApplicationData::Current()?.cast::<IApplicationData>()?.LocalFolder()?;
    let data_dir = folder.cast::<IStorageItem>()?.Path()?;
    std::thread::spawn(move || {
        let opened = Backend::open(&data_dir, session::load()).map(Arc::new);
        post(&dispatcher, move || core_opened(opened));
    });
    Ok(())
}

fn core_opened(opened: std::result::Result<Arc<Backend>, String>) {
    let Some(shell) = shell() else { return };
    match opened {
        Ok(backend) => {
            let _ = shell.backend.set(backend);
            if let Some(home) = shell.home.borrow().as_ref() {
                home.start();
            }
        }
        Err(problem) => {
            if let Some(home) = shell.home.borrow().as_ref() {
                home.say(&format!("打不开数据：{problem}"));
            }
        }
    }
}

fn window_resized() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let class = WindowClass::of(Window::Current()?.cast::<IWindow>()?.Bounds()?.width as f64);
    if class != shell.class.replace(class) {
        shell.apply_class()?;
    }
    Ok(())
}

/// Switches to `tab`; leaving a video page or the login stops what it was doing.
pub fn show_tab(tab: Tab) {
    let Some(shell) = shell() else { return };
    player::leave();
    me::leave();
    shell.tab.set(tab);
    let shown = match tab {
        Tab::Home => shell.show_home(),
        Tab::Me => me::show(&shell),
        Tab::Follow | Tab::Live => load(PLACEHOLDER).and_then(|p| shell.set_page(&p)),
    };
    let _ = shown.and_then(|()| shell.mark_tab());
}

impl Shell {
    pub fn class(&self) -> WindowClass {
        self.class.get()
    }

    pub fn set_page(&self, page: &IFrameworkElement) -> Result<()> {
        self.page.SetContent(&page.cast::<IInspectable>()?)
    }

    /// The page on show.
    pub fn current_page(&self) -> Result<IFrameworkElement> {
        self.page.Content()?.cast()
    }

    pub fn home_page(&self) -> Option<Rc<HomePage>> {
        self.home.borrow().clone()
    }

    /// Back from a page opened over the current tab.
    pub fn back(&self) {
        show_tab(self.tab.get());
    }

    fn show_home(&self) -> Result<()> {
        if self.home.borrow().is_none() {
            *self.home.borrow_mut() = Some(HomePage::new()?);
        }
        let home = self.home.borrow();
        let Some(home) = home.as_ref() else { return Ok(()) };
        self.set_page(home.root())?;
        self.mark_tab()
    }

    /// Lays the navigation out for the window's width.
    fn apply_class(&self) -> Result<()> {
        let class = self.class.get();
        let rail = named::<IFrameworkElement>(&self.root, "Rail")?;
        rail.SetWidth(class.rail_width())?;
        visible(&self.root, "Rail", class != WindowClass::Narrow)?;
        visible(&self.root, "Bottom", class == WindowClass::Narrow)?;
        for (_, name) in TABS {
            visible(&self.root, &format!("Label{name}"), class == WindowClass::Wide)?;
        }
        if let Some(home) = self.home.borrow().as_ref() {
            home.resized();
        }
        player::resized();
        self.mark_tab()
    }

    fn mark_tab(&self) -> Result<()> {
        for (tab, name) in TABS {
            let color = if tab == self.tab.get() { COBALT } else { DIM };
            let brush = SolidColorBrush::CreateInstanceWithColor(color)?;
            for prefix in ["Rail", "Bar"] {
                named::<IControl>(&self.root, &format!("{prefix}{name}"))?.SetForeground(&brush)?;
            }
        }
        Ok(())
    }
}

fn visible(root: &IFrameworkElement, name: &str, on: bool) -> Result<()> {
    named::<IUIElement>(root, name)?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })
}
