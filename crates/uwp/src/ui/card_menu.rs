//! The menu of a row in a list, by right click, long press or the menu key: a card's
//! 稍后再看 / 复制链接, or whatever choices a list offers.

use std::rc::Rc;

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::settings::escape;
use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load_object, post};
use crate::xaml::{
    Clipboard, ContextRequestedEventArgs, DataPackage, FrameworkElement, IContextRequestedEventArgs, IDataPackage,
    IFlyoutBase, IFrameworkElement, IMenuFlyout, IMenuFlyout2, IMenuFlyoutItem, IPropertyValue, IRoutedEventArgs,
    IUIElement4, Point, RoutedEventArgs, UIElement,
};

/// What a card's menu acts on.
pub struct Target {
    /// The video, when 稍后再看 fits it.
    pub later: Option<String>,
    pub link: String,
}

/// One line of a menu and what it does.
pub type Choice = (String, Rc<dyn Fn()>);

/// Offers a card's menu on `list`, whose rows are numbered by their data; `target` says what row is
/// what.
pub fn attach(list: &IFrameworkElement, target: impl Fn(usize) -> Option<Target> + 'static) -> Result<()> {
    attach_choices(list, move |at| target(at).map(card_choices))
}

/// Offers on `list` the menu `choices` gives for a row; none for an empty one.
pub fn attach_choices(
    list: &IFrameworkElement,
    choices: impl Fn(usize) -> Option<Vec<Choice>> + 'static,
) -> Result<()> {
    keep(list.cast::<IUIElement4>()?.ContextRequested(
        move |sender: Ref<UIElement>, args: Ref<ContextRequestedEventArgs>| {
            let (Ok(sender), Ok(args)) = (sender.ok(), args.ok()) else { return };
            let _ = requested(sender, args, &choices);
        },
    )?);
    Ok(())
}

/// Shows `choices` under `button`.
pub fn show_under(button: &FrameworkElement, choices: Vec<Choice>) -> Result<()> {
    menu(choices)?.cast::<IFlyoutBase>()?.ShowAt(button)
}

fn requested(
    list: &UIElement,
    args: &ContextRequestedEventArgs,
    choices: &impl Fn(usize) -> Option<Vec<Choice>>,
) -> Result<()> {
    let source = args.cast::<IRoutedEventArgs>()?.OriginalSource()?;
    let row = source.cast::<IFrameworkElement>()?.DataContext()?.cast::<IPropertyValue>()?.GetInt32()?;
    let Some(choices) = choices(row as usize).filter(|c| !c.is_empty()) else { return Ok(()) };
    let asked = args.cast::<IContextRequestedEventArgs>()?;
    asked.SetHandled(true)?;
    let mut point = Point::default();
    let by_pointer = asked.TryGetPosition(list, &mut point)?;
    let menu = menu(choices)?;
    if by_pointer {
        menu.cast::<IMenuFlyout2>()?.ShowAt(list, point)
    } else {
        menu.cast::<IFlyoutBase>()?.ShowAt(&source.cast::<FrameworkElement>()?)
    }
}

fn item(label: &str) -> String {
    format!(r##"<MenuFlyoutItem Text="{}" />"##, escape(label))
}

fn card_choices(target: Target) -> Vec<Choice> {
    let Target { later, link } = target;
    let mut choices: Vec<Choice> = Vec::new();
    if let Some(bvid) = later {
        choices.push(("稍后再看".to_owned(), Rc::new(move || save_for_later(&bvid))));
    }
    choices.push(("复制链接".to_owned(), Rc::new(move || copy(&link))));
    choices
}

fn menu(choices: Vec<Choice>) -> Result<IInspectable> {
    let items: String = choices.iter().map(|(label, _)| item(label)).collect();
    let menu = load_object(&format!(
        r##"<MenuFlyout xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">{items}</MenuFlyout>"##
    ))?;
    let listed = menu.cast::<IMenuFlyout>()?.Items()?;
    for (at, (_, act)) in choices.into_iter().enumerate() {
        keep(
            listed
                .GetAt(at as u32)?
                .cast::<IMenuFlyoutItem>()?
                .Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| act())?,
        );
    }
    Ok(menu)
}

fn save_for_later(bvid: &str) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (bvid, dispatcher, core) = (bvid.to_owned(), shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let said = match core.services.actions.watch_later(&bvid).await {
            Ok(()) => "已加入稍后再看".to_owned(),
            Err(problem) => format!("加入失败：{problem}"),
        };
        post(&dispatcher, move || toast(&said));
    });
}

fn copy(link: &str) {
    let copied = (|| {
        let package = DataPackage::new()?;
        package.cast::<IDataPackage>()?.SetText(link)?;
        Clipboard::SetContent(&package)
    })();
    toast(if copied.is_ok() { "已复制链接" } else { "复制失败" });
}
