//! 小窗: a live room's picture alone, in a small window of its own that stays on top while the
//! main window goes on as usual. The window is a second view with its own UI thread: the player
//! moves there and back, and each element lets go of it before another takes it.

use std::cell::RefCell;

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::ui::shell::{shell, toast};
use crate::ui::{keep, live_room, load, media, named, on_click, post};
use crate::xaml::{
    ApplicationView, ApplicationViewConsolidatedEventArgs, ApplicationViewMode, ApplicationViewSwitcher,
    CoreApplication, CoreDispatcher, IApplicationView, IApplicationView4, ICoreApplicationView2, IFontIcon,
    IFrameworkElement, IMediaPlayer, IMediaPlayerElement, IUIElement, IWindow, MediaPlayer, MediaSource,
    PointerRoutedEventArgs, UIElement, Window,
};

const MUTED: &str = "\u{E74F}";
const SOUND: &str = "\u{E767}";

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="Black" RequestedTheme="Dark">
  <MediaPlayerElement x:Name="PipVideo" AreTransportControlsEnabled="False" Stretch="Uniform" />
  <StackPanel x:Name="PipBar" Orientation="Horizontal" HorizontalAlignment="Right" VerticalAlignment="Top"
              Background="#99000000" Opacity="0">
    <Button x:Name="PipMute" Background="Transparent" BorderThickness="0" Padding="10,6" Foreground="White">
      <FontIcon x:Name="PipMuteIcon" Glyph="&#xE767;" FontSize="14" /></Button>
    <Button x:Name="PipBack" Background="Transparent" BorderThickness="0" Padding="10,6" Foreground="White">
      <FontIcon Glyph="&#xE73F;" FontSize="14" /></Button>
  </StackPanel>
</Grid>"##;

/// The room playing in the small window, as the main thread holds it.
struct Pip {
    room: Room,
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

thread_local! {
    /// On the main thread.
    static PIP: RefCell<Option<Pip>> = const { RefCell::new(None) };
    /// On the small window's thread: its picture element.
    static ELEMENT: RefCell<Option<IMediaPlayerElement>> = const { RefCell::new(None) };
}

/// Whether room `id` plays in the small window.
pub fn holds(id: u64) -> bool {
    PIP.with_borrow(|p| p.as_ref().is_some_and(|p| p.room.id == id))
}

/// Moves `player` into a small window of its own. The caller's element has let go of it.
pub fn open(room: Room, player: MediaPlayer, source: MediaSource) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    if let Some(old) = PIP.with_borrow_mut(Option::take) {
        retire(old);
    }
    let view = CoreApplication::CreateNewViewFromMainView()?;
    let dispatcher = view.cast::<ICoreApplicationView2>()?.Dispatcher()?;
    PIP.with_borrow_mut(|p| *p = Some(Pip { room, player: player.clone(), source, view: Some(dispatcher.clone()) }));
    let main = shell.dispatcher.clone();
    post(&dispatcher, move || {
        let shown = show_window(&player, &main);
        post(&main, move || match shown {
            Ok(id) => ask_to_show(id),
            Err(problem) => {
                toast(&format!("小窗打不开：{}", problem.message()));
                back_to_room();
            }
        });
    });
    Ok(())
}

/// Asks the system to show view `id` as a small window on top; if it will not, the room takes
/// its picture back.
fn ask_to_show(id: i32) {
    let Some(main) = shell().map(|s| s.dispatcher.clone()) else { return };
    let asked = match ApplicationViewSwitcher::TryShowAsViewModeAsync(id, ApplicationViewMode::CompactOverlay) {
        Ok(asked) => asked,
        Err(problem) => {
            toast(&format!("小窗打不开：{}", problem.message()));
            back_to_room();
            return;
        }
    };
    // The answer is waited for off the UI thread.
    std::thread::spawn(move || {
        if !asked.join().unwrap_or(false) {
            post(&main, || {
                toast("系统没有让小窗打开");
                back_to_room();
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
    let bar = named::<IUIElement>(&root, "PipBar")?;
    let showing = bar.clone();
    keep(root.cast::<IUIElement>()?.PointerEntered(move |_: Ref<IInspectable>, _: Ref<PointerRoutedEventArgs>| {
        let _ = showing.SetOpacity(1.0);
    })?);
    let hiding = bar.clone();
    keep(root.cast::<IUIElement>()?.PointerExited(move |_: Ref<IInspectable>, _: Ref<PointerRoutedEventArgs>| {
        let _ = hiding.SetOpacity(0.0);
    })?);
    let (muting, icon) = (player.clone(), root.clone());
    on_click(&root, "PipMute", move || {
        let _ = toggle_mute(&muting, &icon);
    })?;
    show_sound(player, &root)?;
    let asking = main.clone();
    on_click(&root, "PipBack", move || post(&asking, back_to_room))?;
    let window = Window::Current()?;
    window.cast::<IWindow>()?.SetContent(&root.cast::<UIElement>()?)?;
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

fn toggle_mute(player: &MediaPlayer, root: &IFrameworkElement) -> Result<()> {
    let control = player.cast::<IMediaPlayer>()?;
    control.SetIsMuted(!control.IsMuted()?)?;
    show_sound(player, root)
}

fn show_sound(player: &MediaPlayer, root: &IFrameworkElement) -> Result<()> {
    let muted = player.cast::<IMediaPlayer>()?.IsMuted()?;
    named::<IFontIcon>(root, "PipMuteIcon")?.SetGlyph(if muted { MUTED } else { SOUND })
}

/// On the small window's thread: the element lets go of the player.
fn let_go() {
    if let Some(element) = ELEMENT.with_borrow_mut(Option::take) {
        let _ = element.SetMediaPlayer(None::<&MediaPlayer>);
    }
}

/// The small window was closed by hand, and its element has let go: the room takes its picture
/// back if it is on show, and otherwise the player goes.
fn closed() {
    let Some(pip) = PIP.with_borrow_mut(Option::take) else { return };
    if !live_room::take_back(pip.room.id, &pip.player, &pip.source) {
        media::close(Some(pip.player), Some(pip.source));
    }
}

/// Ends the small window and goes back to the room, opening it again if it was left.
pub fn back_to_room() {
    let Some(pip) = PIP.with_borrow_mut(Option::take) else { return };
    let room = pip.room.clone();
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
            if !live_room::take_back(room.id, &player, &source) {
                media::close(Some(player), Some(source));
                let _ = live_room::open(room.id, &room.title, &room.uploader, room.uid);
            }
        });
    });
}

/// Ends a small window another room replaces: its element lets go, then its player closes.
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
