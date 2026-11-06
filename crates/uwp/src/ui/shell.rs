//! The window: navigation that follows its width, and the page on show.

use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::backend::Backend;
use crate::session;
use crate::state::layout::WindowClass;
use crate::state::panes::{self, Widths};
use crate::state::settings::Prefs;
use crate::ui::home::HomePage;
use crate::ui::keep;
use crate::ui::splitter::{self, BAR_TEMPLATE};
use crate::ui::{follow, live, live_room, load, me, named, on_click, player, post};
use crate::xaml::{
    ApplicationData, ApplicationView, Color, CoreApplication, CoreDispatcher, CoreWindow, CoreWindowActivationState,
    GridLength, GridUnitType, IApplicationData, IApplicationView3, IContentControl, IControl, ICoreApplicationView3,
    ICoreWindow, IFrameworkElement, IRowDefinition, IStorageItem, IUIElement, IUISettings3, IWindow, IWindow2,
    IWindowActivatedEventArgs, KeyEventArgs, SolidColorBrush, UIColorType, UIElement, UISettings, Visibility, Window,
    WindowActivatedEventArgs, WindowSizeChangedEventArgs,
};

const COBALT: Color = Color { a: 255, r: 0x00, g: 0x50, b: 0xEF };
/// Height of the strip that replaces the title bar.
const TITLE_HEIGHT: f64 = 32.0;
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
    <ColumnDefinition Width="Auto" />
    <ColumnDefinition Width="*" />
  </Grid.ColumnDefinitions>
  <Grid.RowDefinitions>
    <RowDefinition x:Name="TitleRow" Height="32" />
    <RowDefinition Height="*" />
    <RowDefinition Height="Auto" />
  </Grid.RowDefinitions>
  <Thumb x:Name="RailBar" Grid.Column="1" Grid.RowSpan="3" Width="5" Background="Transparent"><!--bar--></Thumb>
  <Grid x:Name="DragBar" Grid.Column="2" Background="Transparent">
    <TextBlock Text="Rubi" FontSize="12" Margin="12,0,0,0" VerticalAlignment="Center"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </Grid>
  <StackPanel x:Name="Rail" Grid.RowSpan="3" Padding="0,6,0,0"
              Background="{ThemeResource SystemControlBackgroundChromeMediumLowBrush}">
    <Button x:Name="RailHome" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon x:Name="RailIconHome" Glyph="&#xE80F;" FontSize="20" /><TextBlock x:Name="LabelHome" Margin="12,0,0,0" Text="首页" /></StackPanel></Button>
    <Button x:Name="RailFollow" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon x:Name="RailIconFollow" Glyph="&#xEB51;" FontSize="20" /><TextBlock x:Name="LabelFollow" Margin="12,0,0,0" Text="关注" /></StackPanel></Button>
    <Button x:Name="RailLive" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon x:Name="RailIconLive" Glyph="&#xE93E;" FontSize="20" /><TextBlock x:Name="LabelLive" Margin="12,0,0,0" Text="直播" /></StackPanel></Button>
    <Button x:Name="RailMe" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon x:Name="RailIconMe" Glyph="&#xE77B;" FontSize="20" /><TextBlock x:Name="LabelMe" Margin="12,0,0,0" Text="我的" /></StackPanel></Button>
  </StackPanel>
  <ContentControl x:Name="Page" Grid.Column="2" Grid.Row="1" HorizontalContentAlignment="Stretch"
                  VerticalContentAlignment="Stretch" IsTabStop="False" />
  <Grid x:Name="Bottom" Grid.Column="2" Grid.Row="2" BorderThickness="0,1,0,0"
        BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <Grid.ColumnDefinitions>
      <ColumnDefinition /><ColumnDefinition /><ColumnDefinition /><ColumnDefinition />
    </Grid.ColumnDefinitions>
    <Button x:Name="BarHome" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon x:Name="BarIconHome" Glyph="&#xE80F;" FontSize="20" /><TextBlock Text="首页" FontSize="11" /></StackPanel></Button>
    <Button x:Name="BarFollow" Grid.Column="1" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon x:Name="BarIconFollow" Glyph="&#xEB51;" FontSize="20" /><TextBlock Text="关注" FontSize="11" /></StackPanel></Button>
    <Button x:Name="BarLive" Grid.Column="2" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon x:Name="BarIconLive" Glyph="&#xE93E;" FontSize="20" /><TextBlock Text="直播" FontSize="11" /></StackPanel></Button>
    <Button x:Name="BarMe" Grid.Column="3" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon x:Name="BarIconMe" Glyph="&#xE77B;" FontSize="20" /><TextBlock Text="我的" FontSize="11" /></StackPanel></Button>
  </Grid>
</Grid>"##;

pub struct Shell {
    pub dispatcher: CoreDispatcher,
    pub backend: OnceCell<Arc<Backend>>,
    root: IFrameworkElement,
    page: IContentControl,
    class: Cell<WindowClass>,
    /// Where the user dragged the panes to; kept in the app's folder.
    widths: Cell<Widths>,
    prefs: Cell<Prefs>,
    folder: String,
    tab: Cell<Tab>,
    immersive: Cell<bool>,
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
    let root = load(&LAYOUT.replace("<!--bar-->", BAR_TEMPLATE))?;
    let page = named::<IContentControl>(&root, "Page")?;
    let dispatcher = window.Dispatcher()?;
    let folder = ApplicationData::Current()?.cast::<IApplicationData>()?.LocalFolder()?;
    let data_dir = folder.cast::<IStorageItem>()?.Path()?;
    let widths = std::fs::read_to_string(panes_file(&data_dir)).map(|t| Widths::parse(&t)).unwrap_or_default();
    let prefs = std::fs::read_to_string(prefs_file(&data_dir)).map(|t| Prefs::parse(&t)).unwrap_or_default();
    let shell = Rc::new(Shell {
        dispatcher: dispatcher.clone(),
        backend: OnceCell::new(),
        root: root.clone(),
        page,
        class: Cell::new(WindowClass::of(window.Bounds()?.width as f64)),
        widths: Cell::new(widths),
        prefs: Cell::new(prefs),
        folder: data_dir.clone(),
        tab: Cell::new(Tab::Home),
        immersive: Cell::new(false),
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
    keep(revoker);
    let revoker = CoreWindow::GetForCurrentThread()?.cast::<ICoreWindow>()?.KeyDown(
        |_: Ref<CoreWindow>, args: Ref<KeyEventArgs>| {
            if let Ok(args) = args.ok()
                && let Ok(key) = args.VirtualKey()
            {
                if !player::key(key) {
                    live_room::key(key);
                }
            }
        },
    )?;
    keep(revoker);
    let revoker = CoreWindow::GetForCurrentThread()?.cast::<ICoreWindow>()?.Activated(
        |_: Ref<CoreWindow>, args: Ref<WindowActivatedEventArgs>| {
            let state = args.ok().and_then(|a| a.cast::<IWindowActivatedEventArgs>()?.WindowActivationState());
            if let Ok(state) = state {
                player::focus_changed(state != CoreWindowActivationState::Deactivated);
            }
        },
    )?;
    keep(revoker);

    shell.apply_class()?;
    window.SetContent(&root.cast::<UIElement>()?)?;
    extend_into_title_bar(&window, &root)?;
    window.Activate()?;
    shell.show_home()?;

    splitter::attach(&root, "RailBar", drag_rail, save_widths, reset_rail)?;
    std::thread::spawn(move || {
        let opened = Backend::open(&data_dir, session::load()).map(Arc::new);
        post(&dispatcher, move || core_opened(opened));
    });
    Ok(())
}

/// No system bar: the content reaches the top of the window, the caption buttons sit on it and
/// `DragBar` moves the window.
fn extend_into_title_bar(window: &IWindow, root: &IFrameworkElement) -> Result<()> {
    CoreApplication::GetCurrentView()?.cast::<ICoreApplicationView3>()?.TitleBar()?.SetExtendViewIntoTitleBar(true)?;
    window.cast::<IWindow2>()?.SetTitleBar(&named::<UIElement>(root, "DragBar")?)?;
    let clear = Color { a: 0, r: 0, g: 0, b: 0 };
    let bar = ApplicationView::GetForCurrentView()?.cast::<IApplicationView3>()?.TitleBar()?;
    bar.SetButtonBackgroundColor(Some(clear))?;
    bar.SetButtonInactiveBackgroundColor(Some(clear))?;
    // Clear buttons take no theme of their own: their glyphs follow the system's text colour.
    let text = UISettings::new()?.cast::<IUISettings3>()?.GetColorValue(UIColorType::Foreground)?;
    bar.SetButtonForegroundColor(Some(text))
}

fn prefs_file(data_dir: &str) -> std::path::PathBuf {
    std::path::Path::new(data_dir).join("prefs.txt")
}

fn panes_file(data_dir: &str) -> std::path::PathBuf {
    std::path::Path::new(data_dir).join("panes.txt")
}

fn drag_rail(change: f64) {
    let Some(shell) = shell() else { return };
    let mut widths = shell.widths.get();
    widths.rail = Some(panes::rail_width(shell.rail_width() + change));
    shell.widths.set(widths);
    let _ = shell.apply_rail();
}

fn reset_rail() {
    let Some(shell) = shell() else { return };
    let mut widths = shell.widths.get();
    widths.rail = None;
    shell.widths.set(widths);
    let _ = shell.apply_rail();
    save_widths();
}

/// Keeps the widths for the next start.
pub fn save_widths() {
    if let Some(shell) = shell() {
        let _ = std::fs::write(panes_file(&shell.folder), shell.widths.get().to_text());
    }
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
    player::window_changed();
    live_room::window_changed();
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
    live_room::leave();
    me::leave();
    shell.tab.set(tab);
    let shown = match tab {
        Tab::Home => shell.show_home(),
        Tab::Me => me::show(),
        Tab::Live => live::show(),
        Tab::Follow => follow::show(),
    };
    let _ = shown.and_then(|()| shell.mark_tab());
}

impl Shell {
    pub fn class(&self) -> WindowClass {
        self.class.get()
    }

    /// The side pane's dragged width, if any.
    pub fn side_width(&self) -> Option<f64> {
        self.widths.get().side
    }

    pub fn set_side_width(&self, width: Option<f64>) {
        let mut widths = self.widths.get();
        widths.side = width;
        self.widths.set(widths);
    }

    /// Takes the window to fullscreen and folds the navigation away, or back.
    pub fn fill_screen(&self, on: bool) -> Result<()> {
        let view = ApplicationView::GetForCurrentView()?.cast::<IApplicationView3>()?;
        if on {
            view.TryEnterFullScreenMode()?;
        } else {
            view.ExitFullScreenMode()?;
        }
        self.immersive(on)
    }

    pub fn is_filling_screen(&self) -> bool {
        ApplicationView::GetForCurrentView()
            .and_then(|v| v.cast::<IApplicationView3>())
            .and_then(|v| v.IsFullScreenMode())
            .unwrap_or(false)
    }

    pub fn prefs(&self) -> Prefs {
        self.prefs.get()
    }

    /// Keeps `prefs` for the next start, and shows the style at once.
    pub fn set_prefs(&self, prefs: Prefs) -> Result<()> {
        self.prefs.set(prefs);
        let _ = std::fs::write(prefs_file(&self.folder), prefs.to_text());
        self.apply_rail()
    }

    fn rail_width(&self) -> f64 {
        self.widths.get().rail.unwrap_or_else(|| self.class.get().rail_width())
    }

    /// Sizes the rail and shows its labels when they fit.
    fn apply_rail(&self) -> Result<()> {
        let width = self.rail_width();
        named::<IFrameworkElement>(&self.root, "Rail")?.SetWidth(width)?;
        let wide_enough = panes::rail_has_labels(width);
        let words = self.prefs.get().words;
        for (_, name) in TABS {
            visible(&self.root, &format!("Label{name}"), wide_enough)?;
            // A narrow rail has no labels to stand in for its icons.
            visible(&self.root, &format!("RailIcon{name}"), !(words && wide_enough))?;
            visible(&self.root, &format!("BarIcon{name}"), !words)?;
        }
        Ok(())
    }

    /// Folds the navigation away while a page fills the screen.
    pub fn immersive(&self, on: bool) -> Result<()> {
        self.immersive.set(on);
        self.apply_class()
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
        self.apply_rail()?;
        let bare = self.immersive.get();
        let title = GridLength { value: if bare { 0.0 } else { TITLE_HEIGHT }, grid_unit_type: GridUnitType::Pixel };
        named::<IRowDefinition>(&self.root, "TitleRow")?.SetHeight(title)?;
        visible(&self.root, "Rail", class != WindowClass::Narrow && !bare)?;
        visible(&self.root, "RailBar", class != WindowClass::Narrow && !bare)?;
        visible(&self.root, "Bottom", class == WindowClass::Narrow && !bare)?;
        if let Some(home) = self.home.borrow().as_ref() {
            home.resized();
        }
        player::resized();
        me::resized();
        live::resized();
        follow::resized();
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
