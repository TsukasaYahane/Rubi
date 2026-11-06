//! The pages. XAML objects stay on the UI thread: work goes to the backend, and comes back
//! through [`post`] as plain data.

pub mod card_grid;
pub mod card_menu;
pub mod danmaku;
pub mod feed;
pub mod follow;
pub mod home;
pub mod library;
pub mod live;
pub mod live_room;
pub mod me;
pub mod media;
pub mod player;
pub mod player_actions;
pub mod player_side;
pub mod rows;
pub mod schedule;
pub mod settings;
pub mod shell;
pub mod splitter;
pub mod video_list;
pub mod watch;

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Mutex;

use windows_core::{EventRevoker, IInspectable, Interface, Ref, Result};

use crate::xaml::{
    CoreDispatcher, CoreDispatcherPriority, DispatchedHandler, IButtonBase, ICoreDispatcher, IFrameworkElement,
    IKeyRoutedEventArgs, ITextBlock, IUIElement, KeyRoutedEventArgs, RoutedEventArgs, VirtualKey, XamlReader,
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
