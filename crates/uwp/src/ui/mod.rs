//! The pages. XAML objects stay on the UI thread: work goes to the backend, and comes back
//! through [`post`] as plain data.

pub mod article;
pub mod captcha;
mod card_grid;
pub mod card_menu;
pub mod clip;
pub mod comment_list;
pub mod contacts;
pub mod control_bar;
pub mod danmaku;
pub mod download_sheet;
pub mod downloads;
mod extend;
pub mod emote_text;
pub mod feed;
pub mod filling;
pub mod filtered;
pub mod folder_edit;
pub mod follow;
mod follow_all;
pub mod frames;
pub mod home;
pub mod inbox;
pub mod library;
mod login_sheet;
pub mod listening;
pub mod live;
pub mod live_extras;
pub mod live_room;
pub mod match_rows;
pub mod me;
pub mod media;
pub mod mentions;
pub mod pgc_season;
pub mod pgc_zone;
pub mod pip;
pub mod play_settings;
pub mod player;
pub mod player_actions;
pub mod player_side;
pub mod playlists;
pub mod post_list;
pub mod post_page;
pub mod popup;
pub mod quality;
pub mod queue;
pub mod rows;
pub mod schedule;
pub mod search;
pub mod series;
pub mod settings;
pub mod settings_page;
pub mod share_card;
pub mod share_friend;
pub mod sharing;
mod playing;
mod update;
mod update_sheet;
pub mod shell;
pub mod smtc;
pub mod splitter;
pub mod twin_video;
pub mod up;
pub mod video_list;
pub mod viewer;
pub mod watch;
pub mod why;

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Mutex;

use windows_core::{EventRevoker, IInspectable, Interface, Ref, Result};

use crate::xaml::{
    CoreDispatcher, CoreDispatcherPriority, DispatchedHandler, IButtonBase, ICoreDispatcher, IFrameworkElement,
    IKeyRoutedEventArgs, ITextBlock, IUIElement, KeyRoutedEventArgs, RoutedEventArgs, VirtualKey, Visibility,
    XamlReader,
};

/// Runs `job` on the UI thread.
pub fn post(dispatcher: &CoreDispatcher, job: impl FnOnce() + Send + 'static) {
    let job = Mutex::new(Some(job));
    let handler = DispatchedHandler::new(move || {
        if let Some(job) = job.lock().unwrap_or_else(|e| e.into_inner()).take() {
            job();
        }
    });
    let _ = dispatcher.cast::<ICoreDispatcher>().and_then(|d| d.RunAsync(CoreDispatcherPriority::Normal, &handler));
}

/// Appends a line with a millisecond stamp to `probe.log` in the app's own folder: for judging
/// where a page spends its time. The clock starts at the first call of a run.
pub fn trace(what: &str) {
    use std::io::Write;
    thread_local! {
        static AT: std::cell::Cell<Option<std::time::Instant>> = const { std::cell::Cell::new(None) };
        static LOG: std::cell::RefCell<Option<std::path::PathBuf>> = const { std::cell::RefCell::new(None) };
    }
    let start = AT.with(|slot| {
        if slot.get().is_none() {
            slot.set(Some(std::time::Instant::now()));
        }
        slot.get().unwrap_or_else(std::time::Instant::now)
    });
    let path = LOG.with(|slot| {
        slot.borrow_mut()
            .get_or_insert_with(|| {
                let folder = crate::xaml::ApplicationData::Current()
                    .and_then(|data| data.LocalFolder())
                    .and_then(|folder| folder.cast::<crate::xaml::IStorageItem>())
                    .and_then(|item| item.Path())
                    .map(|path| std::path::PathBuf::from(path.to_string()))
                    .unwrap_or_else(|_| std::path::PathBuf::from("."));
                folder.join("probe.log")
            })
            .clone()
    });
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{:>7}ms {what}", start.elapsed().as_millis());
    }
}

/// Runs `job` once the UI thread has dealt with input and layout, however often this is called
/// before that: a stream of size changes costs one layout pass per frame, not one per change.
pub fn soon(pending: &Rc<Cell<bool>>, job: impl Fn() + 'static) {
    let Some(dispatcher) = shell::shell().map(|s| s.dispatcher.clone()) else { return };
    if pending.replace(true) {
        return;
    }
    let flag = pending.clone();
    let handler = DispatchedHandler::new(move || {
        flag.set(false);
        job();
    });
    let _ = dispatcher.cast::<ICoreDispatcher>().and_then(|d| d.RunAsync(CoreDispatcherPriority::Low, &handler));
}

/// Loads a XAML string.
pub fn load(xaml: &str) -> Result<IFrameworkElement> {
    XamlReader::Load(xaml)?.cast()
}

/// Loads a XAML string whose root is not an element, such as a flyout.
pub fn load_object(xaml: &str) -> Result<IInspectable> {
    XamlReader::Load(xaml)
}

/// The element named `name` under `root`.
pub fn named<T: Interface>(root: &IFrameworkElement, name: &str) -> Result<T> {
    root.FindName(name)?.cast()
}

pub fn set_text(root: &IFrameworkElement, name: &str, text: &str) -> Result<()> {
    named::<ITextBlock>(root, name)?.SetText(text)
}

/// A button to read a page's list again; goes in a layout's namespace.
pub const REFRESH_BUTTON: &str = r##"<Button x:Name="Refresh" Grid.Column="2" Background="Transparent" BorderThickness="0"
  Padding="8,2" VerticalAlignment="Center"><StackPanel Orientation="Horizontal">
  <FontIcon x:Name="RefreshIcon" Glyph="&#xE72C;" FontSize="14" Margin="0,0,6,0" /><TextBlock Text="刷新" FontSize="13" />
  </StackPanel></Button>"##;

/// Hides the refresh button's icon in the words style.
pub fn style_refresh(root: &IFrameworkElement) -> Result<()> {
    let words = shell::shell().is_some_and(|s| s.prefs().words);
    let shown = if words { Visibility::Collapsed } else { Visibility::Visible };
    named::<IUIElement>(root, "RefreshIcon")?.SetVisibility(shown)
}

/// Calls `action` when Enter goes down in the box named `name`.
pub fn on_enter(root: &IFrameworkElement, name: &str, action: impl Fn() + 'static) -> Result<()> {
    keep(named::<IUIElement>(root, name)?.KeyDown(move |_: Ref<IInspectable>, args: Ref<KeyRoutedEventArgs>| {
        if let Ok(args) = args.ok().and_then(|a| a.cast::<IKeyRoutedEventArgs>())
            && args.Key().is_ok_and(|k| k == VirtualKey::Enter)
        {
            let _ = args.SetHandled(true);
            action();
        }
    })?);
    Ok(())
}

/// Calls `action` when the button named `name` is clicked, for as long as the process runs:
/// pages are few and rebuilt rarely, and XAML drops a handler with its button.
/// Keeps an event wired for as long as its source lives. `EventRevoker::forget` would keep the
/// source too, and the page with it.
pub fn keep(revoker: EventRevoker) {
    let _ = revoker.into_token();
}

pub fn on_click(root: &IFrameworkElement, name: &str, action: impl Fn() + 'static) -> Result<()> {
    let revoker =
        named::<IButtonBase>(root, name)?.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| action())?;
    keep(revoker);
    Ok(())
}
