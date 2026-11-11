//! 小窗: a live room's or a video's picture alone, in a small window of its own that stays on top
//! while the main window goes on as usual. The window is a second view with its own UI thread: the player
//! moves there and back, and each element lets go of it before another takes it.

use std::cell::RefCell;

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::ui::shell::{shell, toast};
use crate::ui::{keep, live_room, load, media, named, on_click, player, post};
use crate::xaml::{
    ApplicationView, ApplicationViewConsolidatedEventArgs, ApplicationViewMode, ApplicationViewSwitcher, Color,
    CoreApplication, CoreDispatcher, IApplicationView, IApplicationView3, IApplicationView4, ICoreApplicationView2,
    ICoreApplicationView3, IFrameworkElement, IMediaPlayerElement, IUIElement, IWindow, IWindow2, MediaPlayer,
    MediaSource, PointerRoutedEventArgs, UIElement, Window,
};

/// How long the menu stays once the mouse is still.
const MENU_STAYS: std::time::Duration = std::time::Duration::from_secs(3);
const CLEAR: Color = Color { a: 0, r: 0, g: 0, b: 0 };

/// The picture fills the window. The system title bar is gone: a strip along the top is what the
/// window is dragged by, and the system's own buttons there are drawn see-through. On the mouse a
/// dimmed menu comes up over the picture, as in a phone's 画中画.
const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="Black" RequestedTheme="Dark">
  <MediaPlayerElement x:Name="PipVideo" AreTransportControlsEnabled="False" Stretch="Uniform" />
  <Grid x:Name="PipMenu" Background="#80000000" Opacity="0">
    <StackPanel Orientation="Horizontal" HorizontalAlignment="Center" VerticalAlignment="Center">
      <StackPanel.Resources>
        <Style TargetType="Button">
          <Setter Property="Background" Value="Transparent" />
          <Setter Property="Foreground" Value="White" />
          <Setter Property="BorderThickness" Value="0" />
          <Setter Property="Padding" Value="14,8" />
          <Setter Property="Margin" Value="8,0" />
        </Style>
      </StackPanel.Resources>
      <Button x:Name="PipBack"><StackPanel>
        <FontIcon Glyph="&#xE73F;" FontSize="22" HorizontalAlignment="Center" />
        <TextBlock Text="还原窗口" FontSize="12" Margin="0,6,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
      <Button x:Name="PipClose"><StackPanel>
        <FontIcon Glyph="&#xE711;" FontSize="22" HorizontalAlignment="Center" />
        <TextBlock Text="关闭小窗" FontSize="12" Margin="0,6,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
    </StackPanel>
  </Grid>
  <Border x:Name="PipDrag" Height="32" VerticalAlignment="Top" Background="Transparent" />
</Grid>"##;

/// What plays in the small window, as the main thread holds it.
struct Pip {
    owner: Owner,
    player: MediaPlayer,
    source: MediaSource,
    /// The small window's thread, once it is up.
    view: Option<CoreDispatcher>,
}

/// What reopens the room.
#[derive(Clone)]
pub struct Room {
    pub id: u64,
    pub title: String,
    pub uploader: String,
    pub uid: u64,
}

/// What reopens the video page.
#[derive(Clone)]
pub struct Video {
    pub bvid: String,
    pub title: String,
    pub uploader: String,
}

/// Whose picture the small window shows.
#[derive(Clone)]
pub enum Owner {
    Live(Room),
    Video(Video),
}

thread_local! {
    /// On the main thread.
    static PIP: RefCell<Option<Pip>> = const { RefCell::new(None) };
    /// On the small window's thread: its picture element.
    static ELEMENT: RefCell<Option<IMediaPlayerElement>> = const { RefCell::new(None) };
    /// On the small window's thread: the menu over the picture.
    static MENU: RefCell<Option<IUIElement>> = const { RefCell::new(None) };
}

/// Whether room `id` plays in the small window.
pub fn holds(id: u64) -> bool {
    PIP.with_borrow(|p| p.as_ref().is_some_and(|p| matches!(&p.owner, Owner::Live(room) if room.id == id)))
}

/// Whether video `bvid` plays in the small window.
pub fn holds_video(bvid: &str) -> bool {
    PIP.with_borrow(|p| p.as_ref().is_some_and(|p| matches!(&p.owner, Owner::Video(video) if video.bvid == bvid)))
}

/// Moves `player` into a small window of its own. The caller's element has let go of it.
pub fn open(owner: Owner, player: MediaPlayer, source: MediaSource) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    if let Some(old) = PIP.with_borrow_mut(Option::take) {
        retire(old);
    }
    let view = CoreApplication::CreateNewViewFromMainView()?;
    let dispatcher = view.cast::<ICoreApplicationView2>()?.Dispatcher()?;
    PIP.with_borrow_mut(|p| *p = Some(Pip { owner, player: player.clone(), source, view: Some(dispatcher.clone()) }));
    let main = shell.dispatcher.clone();
    post(&dispatcher, move || {
        let shown = show_window(&player, &main);
        post(&main, move || match shown {
            Ok(id) => ask_to_show(id),
            Err(problem) => {
                toast(&format!("小窗打不开：{}", problem.message()));
                back();
            }
        });
    });
    Ok(())
}

/// Asks the system to show view `id` as a small window on top; if it will not, the page takes
/// its picture back.
fn ask_to_show(id: i32) {
    let Some(main) = shell().map(|s| s.dispatcher.clone()) else { return };
    let asked = match ApplicationViewSwitcher::TryShowAsViewModeAsync(id, ApplicationViewMode::CompactOverlay) {
        Ok(asked) => asked,
        Err(problem) => {
            toast(&format!("小窗打不开：{}", problem.message()));
            back();
            return;
        }
    };
    // The answer is waited for off the UI thread.
    std::thread::spawn(move || {
        if !asked.join().unwrap_or(false) {
            post(&main, || {
                toast("系统没有让小窗打开");
                back();
            });
        }
    });
}

/// On the small window's thread: puts the picture up and says the view's id.
fn show_window(player: &MediaPlayer, main: &CoreDispatcher) -> Result<i32> {
    let root = load(LAYOUT)?;
    let element = named::<IMediaPlayerElement>(&root, "PipVideo")?;
    element.SetMediaPlayer(player)?;
    ELEMENT.with_borrow_mut(|e| *e = Some(element));
    wake_menu_on_mouse(&root)?;
    let asking = main.clone();
    on_click(&root, "PipBack", move || post(&asking, back))?;
    on_click(&root, "PipClose", close_window)?;
    let window = Window::Current()?;
    window.cast::<IWindow>()?.SetContent(&root.cast::<UIElement>()?)?;
    hide_title_bar(&root)?;
    window.cast::<IWindow>()?.Activate()?;
    let view = ApplicationView::GetForCurrentView()?;
    let closing = main.clone();
    keep(view.cast::<IApplicationView>()?.Consolidated(
        move |_: Ref<ApplicationView>, _: Ref<ApplicationViewConsolidatedEventArgs>| {
            let_go();
            post(&closing, closed);
            if let Ok(window) = Window::Current().and_then(|w| w.cast::<IWindow>()) {
                let _ = window.Close();
            }
        },
    )?);
    view.cast::<IApplicationView>()?.Id()
}

/// On the small window's thread: the view reaches up into the title bar, which keeps only a strip
/// to drag by; the system's buttons there are drawn see-through.
fn hide_title_bar(root: &IFrameworkElement) -> Result<()> {
    CoreApplication::GetCurrentView()?.cast::<ICoreApplicationView3>()?.TitleBar()?.SetExtendViewIntoTitleBar(true)?;
    Window::Current()?.cast::<IWindow2>()?.SetTitleBar(&named::<UIElement>(root, "PipDrag")?)?;
    let bar = ApplicationView::GetForCurrentView()?.cast::<IApplicationView3>()?.TitleBar()?;
    let clear = Some(CLEAR);
    bar.SetButtonBackgroundColor(clear)?;
    bar.SetButtonInactiveBackgroundColor(clear)?;
    bar.SetButtonHoverBackgroundColor(clear)?;
    bar.SetButtonPressedBackgroundColor(clear)?;
    bar.SetButtonForegroundColor(clear)?;
    bar.SetButtonInactiveForegroundColor(clear)?;
    bar.SetButtonHoverForegroundColor(clear)?;
    bar.SetButtonPressedForegroundColor(clear)?;
    Ok(())
}

/// On the small window's thread: the menu comes up while the mouse moves over the window, and
/// goes when it leaves or has been still a while.
fn wake_menu_on_mouse(root: &IFrameworkElement) -> Result<()> {
    let menu = named::<IUIElement>(root, "PipMenu")?;
    let moves = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let dispatcher = Window::Current()?.cast::<IWindow>()?.Dispatcher()?;
    let (waking, counting) = (menu.clone(), moves.clone());
    keep(root.cast::<IUIElement>()?.PointerMoved(move |_: Ref<IInspectable>, _: Ref<PointerRoutedEventArgs>| {
        let _ = waking.SetOpacity(1.0);
        let this = counting.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        let (still, dispatcher) = (counting.clone(), dispatcher.clone());
        std::thread::spawn(move || {
            std::thread::sleep(MENU_STAYS);
            if still.load(std::sync::atomic::Ordering::Relaxed) == this {
                post(&dispatcher, hide_menu);
            }
        });
    })?);
    keep(root.cast::<IUIElement>()?.PointerExited(move |_: Ref<IInspectable>, _: Ref<PointerRoutedEventArgs>| {
        let _ = menu.SetOpacity(0.0);
    })?);
    let held = named::<IUIElement>(root, "PipMenu")?;
    MENU.with_borrow_mut(|m| *m = Some(held));
    Ok(())
}

fn hide_menu() {
    if let Some(menu) = MENU.with_borrow(Clone::clone) {
        let _ = menu.SetOpacity(0.0);
    }
}

/// On the small window's thread: 关闭小窗. The window goes as if closed by hand, so the room takes
/// its picture back if it is still on show.
fn close_window() {
    if let Ok(view) = ApplicationView::GetForCurrentView() {
        let _ = view.cast::<IApplicationView4>().and_then(|v| v.TryConsolidateAsync()).map(drop);
    }
}

/// On the small window's thread: the element lets go of the player.
fn let_go() {
    if let Some(element) = ELEMENT.with_borrow_mut(Option::take) {
        let _ = element.SetMediaPlayer(None::<&MediaPlayer>);
    }
}

/// Gives the player back to the page of `owner` if that page is on show without a picture.
fn take_back(owner: &Owner, player: &MediaPlayer, source: &MediaSource) -> bool {
    match owner {
        Owner::Live(room) => live_room::take_back(room.id, player, source),
        Owner::Video(video) => player::take_back(&video.bvid, player, source),
    }
}

/// The small window was closed by hand, and its element has let go: the page takes its picture
/// back if it is on show, and otherwise the player goes.
fn closed() {
    let Some(pip) = PIP.with_borrow_mut(Option::take) else { return };
    if !take_back(&pip.owner, &pip.player, &pip.source) {
        media::close(Some(pip.player), Some(pip.source));
    }
}

/// Ends the small window and goes back to its page, opening it again if it was left: a video
/// goes on from where the small window was.
pub fn back() {
    let Some(pip) = PIP.with_borrow_mut(Option::take) else { return };
    let owner = pip.owner.clone();
    let (view, main) = (pip.view.clone(), shell().map(|s| s.dispatcher.clone()));
    let (Some(view), Some(main)) = (view, main) else {
        media::close(Some(pip.player), Some(pip.source));
        return;
    };
    // The small window's element lets go first; then the room takes the player.
    let held = std::sync::Mutex::new(Some((pip.player, pip.source)));
    post(&view, move || {
        let_go();
        if let Ok(view) = ApplicationView::GetForCurrentView() {
            let _ = view.cast::<IApplicationView4>().and_then(|v| v.TryConsolidateAsync()).map(drop);
        }
        let held = held.lock().unwrap_or_else(|e| e.into_inner()).take();
        post(&main, move || {
            let Some((player, source)) = held else { return };
            if take_back(&owner, &player, &source) {
                return;
            }
            let at = media::session_of(&player).and_then(|s| s.Position()).map_or(0, |p| p.duration);
            media::close(Some(player), Some(source));
            let _ = match &owner {
                Owner::Live(room) => live_room::open(room.id, &room.title, &room.uploader, room.uid),
                Owner::Video(video) => player::open_at(&video.bvid, &video.title, &video.uploader, at),
            };
        });
    });
}

/// Ends a small window another one replaces: its element lets go, then its player closes.
fn retire(pip: Pip) {
    let Some(view) = pip.view else {
        media::close(Some(pip.player), Some(pip.source));
        return;
    };
    let held = std::sync::Mutex::new(Some((pip.player, pip.source)));
    post(&view, move || {
        let_go();
        if let Some((player, source)) = held.lock().unwrap_or_else(|e| e.into_inner()).take() {
            media::close(Some(player), Some(source));
        }
        if let Ok(view) = ApplicationView::GetForCurrentView() {
            let _ = view.cast::<IApplicationView4>().and_then(|v| v.TryConsolidateAsync()).map(drop);
        }
    });
}
