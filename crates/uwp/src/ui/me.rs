//! 我的: who is signed in, and the settings by category. Beside each other on a wide window, one
//! above the other on a narrow one. 账号 holds signing in with the QR code, and signing out.

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rubi_core::QrState;
use windows_core::{Interface, Result};

use crate::session;
use crate::state::layout::WindowClass;
use crate::state::library::Shelf;
use crate::state::settings::Category;
use crate::state::{qr, settings::escape};
use crate::ui::shell::shell;
use crate::ui::{follow, library, live, load, named, on_click, post, set_text, settings};
use crate::xaml::{
    BitmapImage, Color, IContentControl, IControl, IFrameworkElement, IImage, IUIElement, SolidColorBrush, UIElement,
    Uri, Visibility,
};

/// How often the login is asked about.
const POLL_EVERY: Duration = Duration::from_secs(2);
const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };
/// Bumped when the page or the category changes: a poll from an earlier showing stops.
static RUN: AtomicU64 = AtomicU64::new(0);

const HEAD: &str = r##"
<Grid Margin="16,14">
  <Grid.ColumnDefinitions><ColumnDefinition Width="56" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
  <Grid Width="56" Height="56" Background="#0050EF">
    <TextBlock Text="R" FontSize="22" Foreground="White" HorizontalAlignment="Center" VerticalAlignment="Center" />
    <Image x:Name="Face" Stretch="UniformToFill" />
  </Grid>
  <StackPanel Grid.Column="1" Margin="12,0,0,0" VerticalAlignment="Center">
    <TextBlock x:Name="Name" FontSize="18" FontWeight="Light" TextTrimming="CharacterEllipsis" />
    <TextBlock x:Name="Meta" FontSize="12" TextWrapping="Wrap"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </StackPanel>
</Grid>"##;

/// Side by side; `<!--head-->`, `<!--shelves-->` and `<!--cats-->` are filled in.
const WIDE: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.ColumnDefinitions><ColumnDefinition Width="<!--left-->" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
  <Border BorderThickness="0,0,1,0" BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <ScrollViewer><StackPanel>
      <!--head-->
      <TextBlock Text="内容" FontSize="12" Margin="16,8,16,2"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      <!--shelves-->
    </StackPanel></ScrollViewer>
  </Border>
  <ScrollViewer x:Name="SettingsHost" Grid.Column="1">
    <Grid>
      <Grid.ColumnDefinitions><ColumnDefinition MaxWidth="<!--max-->" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
      <ContentControl x:Name="Body" IsTabStop="False" HorizontalContentAlignment="Stretch" />
    </Grid>
  </ScrollViewer>
  <ContentControl x:Name="ShelfHost" Grid.Column="1" IsTabStop="False" Visibility="Collapsed"
                  HorizontalContentAlignment="Stretch" VerticalContentAlignment="Stretch" />
</Grid>"##;

/// One above the other, the categories in a row that scrolls; a shelf takes the whole page.
const NARROW: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <ScrollViewer x:Name="SettingsHost">
    <StackPanel>
      <!--head-->
      <!--shelves-->
      <ContentControl x:Name="Body" IsTabStop="False" HorizontalContentAlignment="Stretch" />
    </StackPanel>
  </ScrollViewer>
  <ContentControl x:Name="ShelfHost" IsTabStop="False" Visibility="Collapsed"
                  HorizontalContentAlignment="Stretch" VerticalContentAlignment="Stretch" />
</Grid>"##;

const ACCOUNT: &str = r##"
<StackPanel xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
            xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Padding="16,12">
  <TextBlock Text="账号" FontSize="20" FontWeight="Light" />
  <TextBlock x:Name="Status" Margin="0,8,0,12" FontSize="14" TextWrapping="Wrap"
             Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  <ContentControl x:Name="Code" IsTabStop="False" HorizontalAlignment="Left" />
  <Button x:Name="SignOut" Content="退出登录" Visibility="Collapsed" Background="Transparent" BorderThickness="0"
          Padding="0,4" Foreground="#0050EF" />
</StackPanel>"##;

fn cat_buttons() -> String {
    // 设置 is a page of its own: 我的 has no category row at all.
    String::new()
}
#[allow(dead_code)]

fn shelf_buttons(narrow: bool) -> String {
    let (arrow, line) = if narrow {
        (
            r##"<FontIcon Grid.Column="1" Glyph="&#xE76C;" FontSize="12" />"##,
            r##"BorderThickness="0,0,0,1" BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}""##,
        )
    } else {
        ("", r##"BorderThickness="0""##)
    };
    Shelf::ALL
        .iter()
        .enumerate()
        .map(|(at, shelf)| {
            format!(
                r##"<Button x:Name="Shelf{at}" Background="Transparent" {line} Height="44" Padding="16,0"
                     HorizontalAlignment="Stretch" HorizontalContentAlignment="Stretch"><Grid>
                     <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
                     <TextBlock Text="{}" />{arrow}</Grid></Button>"##,
                shelf.label()
            )
        })
        .collect()
}

fn layout(class: WindowClass) -> String {
    let (left, max) = if class == WindowClass::Wide { ("260", "680") } else { ("220", "9999") };
    let frame = if class == WindowClass::Narrow { NARROW } else { WIDE };
    frame
        .replace("<!--left-->", left)
        .replace("<!--max-->", max)
        .replace("<!--head-->", HEAD)
        .replace("<!--shelves-->", &shelf_buttons(class == WindowClass::Narrow))
        
}

/// What the head of the page shows of the account.
#[derive(Clone, Default)]
struct Who {
    name: String,
    line: String,
    face_url: String,
    face: Option<BitmapImage>,
}

impl Who {
    fn paint(&self, root: &IFrameworkElement) -> Result<()> {
        set_text(root, "Name", &self.name)?;
        set_text(root, "Meta", &self.line)?;
        match &self.face {
            Some(face) => named::<IImage>(root, "Face")?.SetSource(face),
            None => Ok(()),
        }
    }
}

/// The page on show.
#[derive(Clone)]
struct Shown {
    root: IFrameworkElement,
    body: Option<IFrameworkElement>,
    class: WindowClass,
}

thread_local! {
    static SHOWN: RefCell<Option<Shown>> = const { RefCell::new(None) };
    static CATEGORY: Cell<Option<Category>> = const { Cell::new(None) };
    /// The shelf open instead of the category.
    static SHELF: Cell<Option<Shelf>> = const { Cell::new(None) };
    /// Who was last read, so the page opens on them and not on a blank that fills in.
    static WHO: RefCell<Option<Who>> = const { RefCell::new(None) };
}

fn shown() -> Option<Shown> {
    SHOWN.with(|s| s.borrow().clone())
}

/// Stops a login under way.
pub fn leave() {
    RUN.fetch_add(1, Ordering::Relaxed);
    if let Some(page) = SHOWN.with(|s| s.borrow_mut().take()) {
        // The pages of the shelves live on, and an element has one parent at a time.
        let _ = named::<IContentControl>(&page.root, "ShelfHost")
            .and_then(|h| h.SetContent(None::<&windows_core::IInspectable>));
    }
}

pub fn show() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let class = shell.class();
    leave();
    let root = load(&layout(class))?;
    shell.set_page(&root)?;
    SHOWN.with(|s| *s.borrow_mut() = Some(Shown { root: root.clone(), body: None, class }));
    for (at, category) in Category::ALL.into_iter().enumerate() {
        on_click(&root, &format!("Cat{at}"), move || {
            let _ = open(category);
        })?;
    }
    show_head();
    for (at, shelf) in Shelf::ALL.into_iter().enumerate() {
        on_click(&root, &format!("Shelf{at}"), move || {
            if let Err(problem) = open_shelf(shelf, true) {
                crate::ui::shell::toast(&format!("打不开：{problem}"));
            }
        })?;
    }
    if let Some(shelf) = SHELF.get() {
        return open_shelf(shelf, false);
    }
    let signed_in = shell.backend.get().is_some_and(|b| b.services.account.is_logged_in());
    let _ = signed_in;
    let category = Category::Account;
    open(category)
}

fn shelf_on(root: &IFrameworkElement, on: bool) -> Result<()> {
    let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
    named::<IUIElement>(root, "ShelfHost")?.SetVisibility(shown(on))?;
    named::<IUIElement>(root, "SettingsHost")?.SetVisibility(shown(!on))
}

/// Paints which of the left column's entries is open.
fn mark() -> Result<()> {
    let Some(page) = shown() else { return Ok(()) };
    let shelf = SHELF.get();
    let tint = |on: bool| SolidColorBrush::CreateInstanceWithColor(if on { COBALT } else { DIM });
    if page.class != WindowClass::Narrow {
        // Painting a label is not worth failing the page for: these calls report success as an
        // error on this platform.
        for (at, each) in Shelf::ALL.into_iter().enumerate() {
            if let Ok(label) = named::<IControl>(&page.root, &format!("Shelf{at}"))
                && let Ok(brush) = tint(shelf == Some(each))
            {
                let _ = label.SetForeground(&brush);
            }
        }
    }
    Ok(())
}

/// Opens `shelf`, read again when `fresh`: coming back from a video it stays as it was.
fn open_shelf(shelf: Shelf, fresh: bool) -> Result<()> {
    let Some(page) = shown() else { return Ok(()) };
    RUN.fetch_add(1, Ordering::Relaxed);
    SHELF.set(Some(shelf));
    let host = named::<IContentControl>(&page.root, "ShelfHost")?;
    host.SetContent(None::<&windows_core::IInspectable>)?;
    let content = library::page(shelf, fresh, page.class)?;
    host.SetContent(&content.cast::<UIElement>()?)?;
    shelf_on(&page.root, true)?;
    mark()
}

/// From a shelf back to the page it was opened from, on a narrow window.
/// Shows the settings page itself, as its own tab does.
pub fn show_settings() -> Result<()> {
    SHELF.set(None);
    CATEGORY.set(Some(Category::Look));
    show()
}

pub fn back_from_shelf() {
    SHELF.set(None);
    if let Some(page) = shown() {
        let _ = named::<IContentControl>(&page.root, "ShelfHost")
            .and_then(|h| h.SetContent(None::<&windows_core::IInspectable>));
        let _ = shelf_on(&page.root, false);
        let _ = mark();
        refresh();
    }
}

/// Follows a new window class, if the page is on show.
pub fn resized() {
    let Some(page) = shown() else { return };
    if shell().is_some_and(|s| s.class() != page.class) {
        RUN.fetch_add(1, Ordering::Relaxed);
        let _ = show();
    }
}

/// Shows the category again, as saved now.
pub fn refresh() {
    if let Some(category) = CATEGORY.get() {
        let _ = open(category);
    }
}

/// A line on the settings page, about what just happened.
pub fn note(text: &str) {
    if let Some(body) = shown().and_then(|p| p.body) {
        let _ = set_text(&body, "Note", text);
    }
}

fn open(category: Category) -> Result<()> {
    let (Some(page), Some(shell)) = (shown(), shell()) else { return Ok(()) };
    RUN.fetch_add(1, Ordering::Relaxed);
    CATEGORY.set(Some(category));
    SHELF.set(None);
    // These calls report success as an error here; none of them is worth refusing the page.
    let _ = shelf_on(&page.root, false);
    let _ = mark();
    let body = match (category, shell.backend.get()) {
        (Category::Account, _) | (_, None) => load(ACCOUNT)?,
        (_, Some(backend)) => settings::build(category, backend)?,
    };
    if let Ok(host) = named::<IContentControl>(&page.root, "Body")
        && let Ok(element) = body.cast::<UIElement>()
    {
        let _ = host.SetContent(&element);
    }
    SHOWN.with(|s| {
        if let Some(shown) = s.borrow_mut().as_mut() {
            shown.body = Some(body.clone());
        }
    });
    if category == Category::Account {
        on_click(&body, "SignOut", sign_out)?;
        run_account(&body)?;
    }
    Ok(())
}

/// Who is signed in, for the head of the page: what is known at once, then what is current.
fn show_head() {
    let (Some(shell), Some(page)) = (shell(), shown()) else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    if !backend.services.account.is_logged_in() {
        WHO.with(|w| *w.borrow_mut() = None);
        let guest = Who { name: "游客".into(), line: "未登录".into(), ..Who::default() };
        let _ = guest.paint(&page.root);
        return;
    }
    if let Some(known) = WHO.with(|w| w.borrow().clone()) {
        let _ = known.paint(&page.root);
    }
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let Ok(account) = core.services.account.whoami().await else { return };
        let profile = core.services.space.profile(account.mid).await.ok();
        post(&dispatcher, move || {
            let mut who = WHO.with(|w| w.borrow().clone()).unwrap_or_default();
            who.name = account.uname;
            if let Some(p) = profile {
                who.line = format!("Lv{} · 关注 {} · 粉丝 {}", p.level, p.followings, p.fans);
                if p.face != who.face_url {
                    who.face = Uri::CreateUri(&p.face).and_then(|u| BitmapImage::CreateInstanceWithUriSource(&u)).ok();
                    who.face_url = p.face;
                }
            }
            WHO.with(|w| *w.borrow_mut() = Some(who.clone()));
            if let Some(page) = shown() {
                let _ = who.paint(&page.root);
            }
        });
    });
}

fn run_account(body: &IFrameworkElement) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let Some(backend) = shell.backend.get().cloned() else {
        return set_text(body, "Status", "正在打开…");
    };
    let run = RUN.load(Ordering::Relaxed);
    let dispatcher = shell.dispatcher.clone();
    let core = backend.clone();
    let tell = move |update: Update| {
        post(&dispatcher, move || {
            if RUN.load(Ordering::Relaxed) == run {
                let _ = update.apply();
            }
        });
    };
    backend.spawn(async move {
        let account = &core.services.account;
        if account.is_logged_in() {
            match account.whoami().await {
                Ok(who) => tell(Update::SignedIn(who.uname, who.mid)),
                Err(e) => tell(Update::Status(format!("读取账号失败：{e}"))),
            }
            return;
        }
        let ticket = match account.qr_generate().await {
            Ok(t) => t,
            Err(e) => return tell(Update::Status(format!("获取二维码失败：{e}"))),
        };
        tell(Update::Code(ticket.url.clone()));
        while RUN.load(Ordering::Relaxed) == run {
            tokio::time::sleep(POLL_EVERY).await;
            match account.qr_poll(&ticket.key).await {
                Ok(QrState::Waiting) => {}
                Ok(QrState::Scanned) => tell(Update::Status("已扫码，请在手机上确认".into())),
                Ok(QrState::Expired) => return tell(Update::Status("二维码已过期，重新打开这一页".into())),
                Ok(QrState::Success(session)) => {
                    let who = account.whoami().await.ok();
                    let (name, mid) = who.map(|w| (w.uname, w.mid)).unwrap_or_default();
                    return tell(Update::Saved(session, name, mid));
                }
                Err(e) => return tell(Update::Status(format!("登录失败：{e}"))),
            }
        }
    });
    Ok(())
}

/// What the background login has to say; applied on the UI thread.
enum Update {
    Status(String),
    Code(String),
    SignedIn(String, u64),
    Saved(rubi_core::Session, String, u64),
}

impl Update {
    fn apply(self) -> Result<()> {
        let Some(body) = shown().and_then(|p| p.body) else { return Ok(()) };
        match self {
            Self::Status(text) => set_text(&body, "Status", &text),
            Self::Code(url) => show_code(&body, &url),
            Self::SignedIn(name, mid) => signed_in(&body, &name, mid),
            Self::Saved(session, name, mid) => {
                let stored = session::save(&session);
                library::forget();
                reload_home();
                signed_in(&body, &name, mid)?;
                show_head();
                if stored.is_err() {
                    set_text(&body, "Status", "已登录，但保存失败：下次启动要重新登录")?;
                }
                Ok(())
            }
        }
    }
}

fn reload_home() {
    follow::forget();
    live::forget();
    if let Some(home) = shell().and_then(|s| s.home_page()) {
        home.reload();
    }
}

fn signed_in(body: &IFrameworkElement, name: &str, mid: u64) -> Result<()> {
    set_text(body, "Status", &format!("{name} · mid {mid} · 凭证保存在系统凭据库"))?;
    named::<IContentControl>(body, "Code")?.SetContent(None::<&windows_core::IInspectable>)?;
    named::<IUIElement>(body, "SignOut")?.SetVisibility(Visibility::Visible)
}

fn sign_out() {
    let Some(shell) = shell() else { return };
    if let Some(backend) = shell.backend.get() {
        backend.services.account.logout();
    }
    let _ = session::clear();
    WHO.with(|w| *w.borrow_mut() = None);
    library::forget();
    SHELF.set(None);
    reload_home();
    let _ = show();
}

/// The QR code, on white with its quiet zone, whatever the theme.
fn show_code(body: &IFrameworkElement, url: &str) -> Result<()> {
    let Some((side, data)) = qr::path_data(url) else { return set_text(body, "Status", "二维码生成失败") };
    let code = load(&format!(
        r##"<Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" Background="White" Padding="14">
          <Viewbox Width="220" Height="220"><Canvas Width="{side}" Height="{side}">
            <Path Fill="Black" Data="{data}" />
          </Canvas></Viewbox></Border>"##
    ))?;
    named::<IContentControl>(body, "Code")?.SetContent(&code.cast::<UIElement>()?)?;
    set_text(body, "Status", "用 B 站手机客户端扫码登录")
}
