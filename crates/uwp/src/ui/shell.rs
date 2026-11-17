//! The window: navigation that follows its width, and the page on show.

use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::backend::Backend;
use crate::session;
use crate::state::layout::WindowClass;
use crate::state::panes::{self, Widths};
use crate::state::settings::{Prefs, Skin};
use crate::ui::home::HomePage;
use crate::ui::keep;
use crate::ui::splitter;
use crate::ui::{follow, live, live_room, load, me, named, on_click, player, post, search, set_text, up};
use crate::xaml::{
    ApplicationData, ApplicationView, Color, CoreApplication, CoreDispatcher, CoreWindow, CoreWindowActivationState,
    ElementTheme, GridLength, GridUnitType, IApplicationData, IApplicationView3, IContentControl, IControl, ICoreApplicationView3,
    ICoreWindow, IFrameworkElement, IFrameworkElement2, IRowDefinition, IStorageItem, IUIElement, IUISettings3, IWindow,
    IWindow2,
    IWindowActivatedEventArgs, KeyEventArgs, SolidColorBrush, UIColorType, UIElement, UISettings, Visibility, Window,
    WindowActivatedEventArgs, WindowSizeChangedEventArgs,
};

const COBALT: Color = Color { a: 255, r: 0x00, g: 0x50, b: 0xEF };
/// Height of the strip that replaces the title bar.
const TITLE_HEIGHT: f64 = 32.0;
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Search,
    Home,
    Follow,
    Live,
    Inbox,
    Me,
    Settings,
}

/// How long a toast stays.
const TOAST_FOR: Duration = Duration::from_secs(2);
/// Bumped by each toast: only the last one takes it away.
static TOASTS: AtomicU64 = AtomicU64::new(0);
/// Set once the update look has been taken, so it happens one time per run.
static CHECKED: AtomicBool = AtomicBool::new(false);
static PAGE_TURN: AtomicU64 = AtomicU64::new(0);

const TABS: [(Tab, &str); 6] =
    [
    (Tab::Home, "Home"),
    (Tab::Follow, "Follow"),
    (Tab::Live, "Live"),
    (Tab::Inbox, "Inbox"),
    (Tab::Me, "Me"),
    (Tab::Settings, "Settings"),
];
/// The unread count on the 消息 icon.
const BADGE: &str = r##"<Border x:Name="{}" Visibility="Collapsed" Background="#E33333" MinWidth="16" Height="16" Padding="4,0"
            HorizontalAlignment="Left" VerticalAlignment="Top" Margin="{}" IsHitTestVisible="False">
      <TextBlock FontSize="10" Foreground="White" HorizontalAlignment="Center" VerticalAlignment="Center" /></Border>"##;

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
  <Thumb x:Name="RailBar" Grid.Column="1" Grid.RowSpan="3" <!--bar-attrs-->><!--bar--></Thumb>
  <Grid x:Name="DragBar" Grid.Column="2" Background="Transparent">
    <TextBlock Text="Rubi" FontSize="12" Margin="12,0,0,0" VerticalAlignment="Center"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </Grid>
  <StackPanel x:Name="Rail" Grid.RowSpan="3" Padding="0,6,0,0"
              Background="{ThemeResource SystemControlBackgroundChromeMediumLowBrush}">
    <Button x:Name="RailSearch" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon x:Name="RailIconSearch" Glyph="&#xE721;" FontSize="20" /><TextBlock x:Name="LabelSearch" Margin="12,0,0,0" Text="搜索" /></StackPanel></Button>
    <Button x:Name="RailHome" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon x:Name="RailIconHome" Glyph="&#xE80F;" FontSize="20" /><TextBlock x:Name="LabelHome" Margin="12,0,0,0" Text="首页" /></StackPanel></Button>
    <Button x:Name="RailFollow" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon x:Name="RailIconFollow" Glyph="&#xEB51;" FontSize="20" /><TextBlock x:Name="LabelFollow" Margin="12,0,0,0" Text="关注" /></StackPanel></Button>
    <Button x:Name="RailLive" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon x:Name="RailIconLive" Glyph="&#xE93E;" FontSize="20" /><TextBlock x:Name="LabelLive" Margin="12,0,0,0" Text="直播" /></StackPanel></Button>
    <Button x:Name="RailInbox" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <Grid><FontIcon x:Name="RailIconInbox" Glyph="&#xE715;" FontSize="20" /><!--rail-badge--></Grid>
      <TextBlock x:Name="LabelInbox" Margin="12,0,0,0" Text="消息" /></StackPanel></Button>
    <Button x:Name="RailMe" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon x:Name="RailIconMe" Glyph="&#xE77B;" FontSize="20" /><TextBlock x:Name="LabelMe" Margin="12,0,0,0" Text="我的" /></StackPanel></Button>
    <Button x:Name="RailSettings" Style="{StaticResource Nav}"><StackPanel Orientation="Horizontal">
      <FontIcon x:Name="RailIconSettings" Glyph="&#xE713;" FontSize="20" /><TextBlock x:Name="LabelSettings" Margin="12,0,0,0" Text="设置" /></StackPanel></Button>
  </StackPanel>
  <ContentControl x:Name="Page" Grid.Column="2" Grid.Row="1" HorizontalContentAlignment="Stretch"
                  VerticalContentAlignment="Stretch" IsTabStop="False" />
  <!-- What is still playing while another page is on screen. -->
  <Border x:Name="NowPlaying" Grid.Column="2" Grid.Row="1" VerticalAlignment="Top" Height="32"
          Background="#E60050EF" Padding="12,0" Visibility="Collapsed">
    <Grid>
      <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
      <TextBlock x:Name="NowPlayingText" Foreground="White" FontSize="12" VerticalAlignment="Center"
                 TextTrimming="CharacterEllipsis" />
      <Button x:Name="NowStop" Grid.Column="1" Content="停止" Background="Transparent" BorderThickness="0"
              Foreground="White" FontSize="12" Padding="8,0" Height="32" />
    </Grid>
  </Border>
  <Grid x:Name="Viewer" Grid.Row="1" Grid.RowSpan="2" Grid.ColumnSpan="3" Visibility="Collapsed" />
  <Canvas x:Name="Preload" Width="1" Height="1" Opacity="0" IsHitTestVisible="False" />
  <Border x:Name="Toast" Grid.Column="2" Grid.Row="1" HorizontalAlignment="Center" VerticalAlignment="Bottom"
          Margin="0,0,0,24" Padding="14,7" Background="#E6000000" Visibility="Collapsed" IsHitTestVisible="False">
    <TextBlock x:Name="ToastText" FontSize="13" Foreground="White" TextWrapping="Wrap" />
  </Border>
  <Grid x:Name="Bottom" Grid.Column="2" Grid.Row="2" BorderThickness="0,1,0,0"
        BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <Grid.ColumnDefinitions>
      <ColumnDefinition /><ColumnDefinition /><ColumnDefinition /><ColumnDefinition /><ColumnDefinition />
      <ColumnDefinition />
    </Grid.ColumnDefinitions>
    <Button x:Name="BarHome" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon x:Name="BarIconHome" Glyph="&#xE80F;" FontSize="20" /><TextBlock Text="首页" FontSize="11" /></StackPanel></Button>
    <Button x:Name="BarFollow" Grid.Column="1" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon x:Name="BarIconFollow" Glyph="&#xEB51;" FontSize="20" /><TextBlock Text="关注" FontSize="11" /></StackPanel></Button>
    <Button x:Name="BarLive" Grid.Column="2" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon x:Name="BarIconLive" Glyph="&#xE93E;" FontSize="20" /><TextBlock Text="直播" FontSize="11" /></StackPanel></Button>
    <Button x:Name="BarInbox" Grid.Column="3" Style="{StaticResource Bar}"><Grid><StackPanel>
      <FontIcon x:Name="BarIconInbox" Glyph="&#xE715;" FontSize="20" /><TextBlock Text="消息" FontSize="11" /></StackPanel>
      <!--bar-badge--></Grid></Button>
    <Button x:Name="BarMe" Grid.Column="4" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon x:Name="BarIconMe" Glyph="&#xE77B;" FontSize="20" /><TextBlock Text="我的" FontSize="11" /></StackPanel></Button>
    <Button x:Name="BarSettings" Grid.Column="5" Style="{StaticResource Bar}"><StackPanel>
      <FontIcon x:Name="BarIconSettings" Glyph="&#xE713;" FontSize="20" /><TextBlock Text="设置" FontSize="11" /></StackPanel></Button>
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
    /// How to go back from pages that were opened from others, latest last.
    returns: RefCell<Vec<Box<dyn Fn()>>>,
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
    // The media panel brings the app back by launching it again. Building the shell a second time
    // would put 首页 over the page the user left, with the sound still playing behind it.
    if shell().is_some() {
        if let Ok(window) = Window::Current().and_then(|window| window.cast::<IWindow>()) {
            let _ = window.Activate();
        }
        return Ok(());
    }
    let window = Window::Current()?.cast::<IWindow>()?;
    let layout = LAYOUT
        .replace("<!--rail-badge-->", &badge("RailBadge", "12,-6,-12,0"))
        .replace("<!--bar-badge-->", &badge("BarBadge", "14,-4,-14,0"));
    let root = load(&splitter::fill(&layout))?;
    let page = named::<IContentControl>(&root, "Page")?;
    let dispatcher = window.Dispatcher()?;
    let folder = ApplicationData::Current()?.cast::<IApplicationData>()?.LocalFolder()?;
    let data_dir = folder.cast::<IStorageItem>()?.Path()?;
    let widths = std::fs::read_to_string(panes_file(&data_dir)).map(|t| Widths::parse(&t)).unwrap_or_default();
    let prefs = std::fs::read_to_string(prefs_file(&data_dir)).map(|t| Prefs::parse(&t)).unwrap_or_default();
    // The skin is asked for before the window is on screen, which is when it still counts.
    if let Ok(element) = root.cast::<IFrameworkElement2>() {
        let _ = element.SetRequestedTheme(skin_of(prefs.theme));
    }
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
        returns: RefCell::default(),
    });
    SHELL.with(|s| *s.borrow_mut() = Some(shell.clone()));

    let for_stop = shell.clone();
    on_click(&root, "NowStop", move || {
        crate::ui::player::stop_playing();
        let _ = for_stop.refresh_now_playing();
    })?;
    on_click(&root, "RailSearch", || show_tab(Tab::Search))?;
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
                let active = state != CoreWindowActivationState::Deactivated;
                player::focus_changed(active);
                if active {
                    crate::ui::inbox::read_unread();
                    // Clipboard.ContentChanged never fires in this app, so the moment the window
                    // comes back is when its newest item is looked at.
                    crate::ui::clip::check();
                }
            }
        },
    )?;
    keep(revoker);

    // The system brings a window back as it left: a page that went fullscreen is not here now.
    if shell.is_filling_screen() {
        ApplicationView::GetForCurrentView()?.cast::<IApplicationView3>()?.ExitFullScreenMode()?;
    }
    shell.apply_class()?;
    window.SetContent(&root.cast::<UIElement>()?)?;
    extend_into_title_bar(&window, &root)?;
    window.Activate()?;
    shell.show_home()?;

    splitter::attach(&root, "RailBar", drag_rail, save_widths, reset_rail)?;
    // Offline copies are always written inside the app's own folder: a packaged app can only
    // write elsewhere through a folder it was handed, and the core writes with plain file calls.
    // They are moved to the folder picked in 设置 as soon as they finish.
    std::thread::spawn(move || {
        let opened = Backend::open(&data_dir, session::load()).map(Arc::new);
        post(&dispatcher, move || core_opened(opened));
    });
    Ok(())
}

thread_local! {
    /// A link that came in before the shell could open it: `rubi://` arrives before the core does.
    static PENDING_LINK: RefCell<Option<String>> = RefCell::new(None);
}

/// Opens a `rubi://` link — `rubi://video/BV1...` or any address naming a video — now if the
/// core is up, or as soon as it is.
pub fn open_link(link: &str) {
    crate::ui::trace(&format!("link {link}"));
    let Some(bvid) = bvid_in(link) else { return };
    if let Some(shell) = shell()
        && shell.backend.get().is_some()
    {
        let _ = crate::ui::player::show(&bvid, &bvid, "");
        return;
    }
    PENDING_LINK.with_borrow_mut(|slot| *slot = Some(bvid));
    // When a link is what started the app, nothing is up yet.
    if shell().is_none() {
        let _ = start();
    }
}

/// The video a link names: the BV id it carries.
fn bvid_in(link: &str) -> Option<String> {
    let at = link.find("BV")?;
    let id: String = link[at..].chars().take(12).collect();
    (id.len() == 12 && id.chars().all(|c| c.is_ascii_alphanumeric())).then_some(id)
}

/// The link that came in before the core was ready, once it is.
pub fn take_pending_link() -> Option<String> {
    PENDING_LINK.with_borrow_mut(Option::take)
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

/// The theme to ask XAML for.
fn skin_of(skin: Skin) -> ElementTheme {
    match skin {
        Skin::Light => ElementTheme::Light,
        Skin::Dark => ElementTheme::Dark,
        Skin::System => ElementTheme::Default,
    }
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
            // Finished copies are carried to the folder picked in 设置, and found there after.
            backend.services.downloads.set_mirror(
                crate::state::exports::download_path().map(std::path::PathBuf::from),
            );
            let _ = shell.backend.set(backend);
            crate::ui::emote_text::read_common();
            crate::ui::inbox::watch_unread();
            // Asked once, at the first start: the answer changes where caches can be written.
            crate::state::access::ask_once();
            if let Some(bvid) = take_pending_link() {
                let _ = crate::ui::player::show(&bvid, &bvid, "");
            }
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

/// Shows a short word at the foot of the window for a moment.
pub fn toast(text: &str) {
    let Some(shell) = shell() else { return };
    let _ = set_text(&shell.root, "ToastText", text);
    let _ = visible(&shell.root, "Toast", true);
    let turn = TOASTS.fetch_add(1, Ordering::Relaxed) + 1;
    let Some(backend) = shell.backend.get().cloned() else { return };
    let dispatcher = shell.dispatcher.clone();
    backend.spawn(async move {
        tokio::time::sleep(TOAST_FOR).await;
        post(&dispatcher, move || {
            if TOASTS.load(Ordering::Relaxed) == turn
                && let Some(shell) = self::shell()
            {
                let _ = visible(&shell.root, "Toast", false);
            }
        });
    });
}

/// Stops what the pages on show were doing.
pub fn leave_pages() {
    player::leave();
    live_room::leave();
    me::leave();
    PAGE_TURN.fetch_add(1, Ordering::Relaxed);
}

/// Bumped when pages change: answers for a page no longer on show are dropped.
pub fn page_turn() -> u64 {
    PAGE_TURN.load(Ordering::Relaxed)
}

/// Switches to `tab`; leaving a video page or the login stops what it was doing.
pub fn show_tab(tab: Tab) {
    let Some(shell) = shell() else { return };
    shell.returns.borrow_mut().clear();
    leave_pages();
    shell.tab.set(tab);
    let shown = match tab {
        Tab::Search => search::show(),
        Tab::Home => shell.show_home(),
        Tab::Me => me::show(),
        Tab::Live => live::show(),
        Tab::Follow => follow::show(),
        Tab::Inbox => crate::ui::inbox::show(),
        Tab::Settings => crate::ui::settings_page::show(),
    };
    if let Err(problem) = shown.and_then(|()| shell.mark_tab()) {
        toast(&format!("打不开：{}", problem.message()));
    }
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

    /// The layer over the page and the navigation that the picture viewer fills.
    pub fn viewer_host(&self) -> Result<crate::xaml::IPanel> {
        named(&self.root, "Viewer")
    }

    pub fn prefs(&self) -> Prefs {
        self.prefs.get()
    }

    /// Keeps `prefs` for the next start, and shows the style at once.
    pub fn set_prefs(&self, prefs: Prefs) -> Result<()> {
        let skinned = prefs.theme != self.prefs.get().theme;
        self.prefs.set(prefs);
        let _ = std::fs::write(prefs_file(&self.folder), prefs.to_text());
        let _ = self.apply_skin();
        let shown = self.apply_rail();
        // A page keeps the skin it was loaded with, so the one on show is built again.
        if skinned {
            show_tab(self.tab.get());
        }
        shown
    }

    /// The light or dark skin to ask for.
    fn theme(&self) -> ElementTheme {
        skin_of(self.prefs.get().theme)
    }

    /// Light, dark, or what Windows is set to: the window's navigation goes by it.
    fn apply_skin(&self) -> Result<()> {
        self.root.cast::<IFrameworkElement2>()?.SetRequestedTheme(self.theme())
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
        visible(&self.root, "LabelSearch", wide_enough)?;
        visible(&self.root, "RailIconSearch", !(words && wide_enough))?;
        for (_, name) in TABS {
            visible(&self.root, &format!("Label{name}"), wide_enough)?;
            // A narrow rail has no labels to stand in for its icons.
            visible(&self.root, &format!("RailIcon{name}"), !(words && wide_enough))?;
            visible(&self.root, &format!("BarIcon{name}"), !words)?;
        }
        Ok(())
    }

    /// Shows `count` unread messages on the 消息 icons; none hides them.
    pub fn set_unread(&self, count: u32) -> Result<()> {
        let label = if count > 99 { "99+".to_owned() } else { count.to_string() };
        for name in ["RailBadge", "BarBadge"] {
            let badge = named::<crate::xaml::IBorder>(&self.root, name)?;
            badge.Child()?.cast::<crate::xaml::ITextBlock>()?.SetText(&label)?;
            visible(&self.root, name, count > 0)?;
        }
        Ok(())
    }

    /// Folds the navigation away while a page fills the screen.
    pub fn immersive(&self, on: bool) -> Result<()> {
        self.immersive.set(on);
        self.apply_class()
    }

    pub fn set_page(&self, page: &IFrameworkElement) -> Result<()> {
        // A page keeps the skin it is loaded with: this has to come before it goes in.
        if let Ok(element) = page.cast::<IFrameworkElement2>() {
            let _ = element.SetRequestedTheme(self.theme());
        }
        self.page.SetContent(&page.cast::<IInspectable>()?)?;
        self.refresh_now_playing()
    }

    /// Shows what still plays while this page is on screen, and hides the line when nothing does.
    fn refresh_now_playing(&self) -> Result<()> {
        let bar = named::<crate::xaml::IUIElement>(&self.root, "NowPlaying")?;
        let sound = crate::ui::playing::note().filter(|_| !crate::ui::playing::on_page());
        if let Some(title) = &sound {
            let _ = set_text(&self.root, "NowPlayingText", title);
        }
        bar.SetVisibility(if sound.is_some() { Visibility::Visible } else { Visibility::Collapsed })
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
        let job = self.returns.borrow_mut().pop();
        match job {
            Some(job) => {
                leave_pages();
                job();
            }
            None => show_tab(self.tab.get()),
        }
    }

    /// Says how to come back to the page on show from the next one opened.
    pub fn remember(&self, back: impl Fn() + 'static) {
        self.returns.borrow_mut().push(Box::new(back));
    }

    fn show_home(&self) -> Result<()> {
        if self.home.borrow().is_none() {
            *self.home.borrow_mut() = Some(HomePage::new()?);
        }
        let home = self.home.borrow();
        let Some(home) = home.as_ref() else { return Ok(()) };
        home.style()?;
        self.set_page(home.root())?;
        self.mark_tab()?;
        // One look for a newer release per run, after the window is up.
        if !CHECKED.swap(true, Ordering::Relaxed) {
            crate::ui::update::check(true);
            // A folder that was moved in from elsewhere brings its files with it: what is on disk
            // and has no record is written down, so 离线缓存 shows it again.
            if let Some(dir) = crate::state::exports::download_path()
                && let Some(backend) = self.backend.get().cloned()
            {
                let inner = backend.clone();
                backend.spawn(async move {
                    let _ = inner.services.downloads.adopt(std::path::Path::new(&dir));
                });
            }
        }
        Ok(())
    }

    /// Lays the navigation out for the window's width.
    fn apply_class(&self) -> Result<()> {
        let class = self.class.get();
        let _ = self.apply_skin();
        self.apply_rail()?;
        self.apply_words_icons()?;
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
        crate::ui::inbox::resized();
        up::resized();
        crate::ui::pgc_season::resized();
        if let Some(home) = self.home.borrow().as_ref() {
            home.style()?;
        }
        self.mark_tab()
    }

    /// In the words style the rail and the bottom bar keep to type: no icons.
    fn apply_words_icons(&self) -> Result<()> {
        let words = self.prefs().words;
        let shown = if words { Visibility::Collapsed } else { Visibility::Visible };
        let mut names: Vec<String> = vec!["RailIconSearch".to_owned()];
        for (_, name) in TABS {
            names.push(format!("RailIcon{name}"));
            names.push(format!("BarIcon{name}"));
        }
        for name in names {
            if let Ok(icon) = named::<IUIElement>(&self.root, &name) {
                icon.SetVisibility(shown)?;
            }
        }
        Ok(())
    }

    fn mark_tab(&self) -> Result<()> {
        let color = if self.tab.get() == Tab::Search { COBALT } else { DIM };
        named::<IControl>(&self.root, "RailSearch")?
            .SetForeground(&SolidColorBrush::CreateInstanceWithColor(color)?)?;
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

pub(crate) fn visible(root: &IFrameworkElement, name: &str, on: bool) -> Result<()> {
    named::<IUIElement>(root, name)?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })
}

/// Pictures being loaded off screen. XAML loads a picture only once an element in the tree
/// shows it: a sharp copy waits here, unseen, until it has come and takes the rough one's place.
const PRELOADS_HELD: u32 = 80;

/// Starts loading `bitmap`, unseen; [`preloaded`] lets it go once it has come.
pub fn preload(bitmap: &crate::xaml::BitmapImage) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let canvas = named::<crate::xaml::IPanel>(&shell.root, "Preload")?;
    let children = canvas.Children()?.cast::<windows_collections::IVector<UIElement>>()?;
    if children.Size()? >= PRELOADS_HELD {
        children.RemoveAt(0)?;
    }
    let image =
        load(r#"<Image xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" Width="1" Height="1" />"#)?;
    image.cast::<crate::xaml::IImage>()?.SetSource(bitmap)?;
    children.Append(&image.cast::<UIElement>()?)
}

/// Lets go of a preloaded picture: it has come, or will not.
pub fn preloaded(bitmap: &windows_core::IInspectable) {
    let Some(shell) = shell() else { return };
    let Ok(canvas) = named::<crate::xaml::IPanel>(&shell.root, "Preload") else { return };
    let Ok(children) = canvas.Children().and_then(|c| c.cast::<windows_collections::IVector<UIElement>>()) else {
        return;
    };
    let size = children.Size().unwrap_or(0);
    for at in (0..size).rev() {
        let shows = children
            .GetAt(at)
            .and_then(|c| c.cast::<crate::xaml::IImage>())
            .and_then(|i| i.Source())
            .is_ok_and(|s| s.cast::<windows_core::IInspectable>().is_ok_and(|s| &s == bitmap));
        if shows {
            let _ = children.RemoveAt(at);
        }
    }
}

fn badge(name: &str, margin: &str) -> String {
    BADGE.replacen("{}", name, 1).replacen("{}", margin, 1)
}
