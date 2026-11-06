//! The menu of a card in a list, by right click, long press or the menu key.

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

/// Offers the menu on `list`, whose rows are numbered by their data; `target` says what row is what.
pub fn attach(list: &IFrameworkElement, target: impl Fn(usize) -> Option<Target> + 'static) -> Result<()> {
    keep(list.cast::<IUIElement4>()?.ContextRequested(
        move |sender: Ref<UIElement>, args: Ref<ContextRequestedEventArgs>| {
            let (Ok(sender), Ok(args)) = (sender.ok(), args.ok()) else { return };
            let _ = requested(sender, args, &target);
        },
    )?);
    Ok(())
}

fn requested(
    list: &UIElement,
    args: &ContextRequestedEventArgs,
    target: &impl Fn(usize) -> Option<Target>,
) -> Result<()> {
    let source = args.cast::<IRoutedEventArgs>()?.OriginalSource()?;
    let row = source.cast::<IFrameworkElement>()?.DataContext()?.cast::<IPropertyValue>()?.GetInt32()?;
    let Some(target) = target(row as usize) else { return Ok(()) };
    let asked = args.cast::<IContextRequestedEventArgs>()?;
    asked.SetHandled(true)?;
    let mut point = Point::default();
    let by_pointer = asked.TryGetPosition(list, &mut point)?;
    let menu = build(target)?;
    if by_pointer {
        menu.cast::<IMenuFlyout2>()?.ShowAt(list, point)
    } else {
        menu.cast::<IFlyoutBase>()?.ShowAt(&source.cast::<FrameworkElement>()?)
    }
}

fn item(label: &str) -> String {
    format!(r##"<MenuFlyoutItem Text="{}" />"##, escape(label))
}

fn build(target: Target) -> Result<IInspectable> {
    let Target { later, link } = target;
    let mut items = String::new();
    if later.is_some() {
        items.push_str(&item("稍后再看"));
    }
    items.push_str(&item("复制链接"));
    let menu = load_object(&format!(
        r##"<MenuFlyout xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">{items}</MenuFlyout>"##
    ))?;
    let listed = menu.cast::<IMenuFlyout>()?.Items()?;
    let mut at = 0;
    if let Some(bvid) = later {
        let bvid = Rc::new(bvid);
        keep(
            listed
                .GetAt(at)?
                .cast::<IMenuFlyoutItem>()?
                .Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| save_for_later(&bvid))?,
        );
        at += 1;
    }
    keep(
        listed
            .GetAt(at)?
            .cast::<IMenuFlyoutItem>()?
            .Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| copy(&link))?,
    );
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
