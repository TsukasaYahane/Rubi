//! The menu of a row in a list, by right click, long press or the menu key: a card's
//! 稍后再看 / 复制链接, or whatever choices a list offers.

use std::rc::Rc;

use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::settings::escape;
use crate::ui::shell::{shell, toast};
use crate::ui::why;
use crate::ui::{keep, load_object, post};

/// A cell of a grid menu: wide enough for a filter's longest names.
const GRID_CELL_WIDTH: u32 = 112;
const GRID_CELL_HEIGHT: u32 = 40;
use crate::xaml::{
    Clipboard, ContextRequestedEventArgs, DataPackage, FrameworkElement, IButtonBase, IContextRequestedEventArgs,
    IDataPackage, IFlyout, IFlyoutBase, IFrameworkElement, IMenuFlyout, IMenuFlyout2, IMenuFlyoutItem, IPanel,
    IPropertyValue, IRoutedEventArgs, IUIElement4, IWindow, Point, RoutedEventArgs, UIElement, Window,
};

/// What a card's menu acts on.
pub struct Target {
    /// The video, when 稍后再看 fits it.
    pub later: Option<String>,
    pub link: String,
    /// The video as the feed sees it, when 为什么推荐 fits it.
    pub recommended: Option<Recommended>,
    /// The video, when 加入播放列表 fits it.
    pub add_list: Option<String>,
}

/// A card of the feed: enough to explain its score and to mark it not interesting.
pub struct Recommended {
    pub bvid: String,
    pub title: String,
}

/// One line of a menu and what it does.
pub type Choice = (String, Rc<dyn Fn()>);

/// One line of a card's menu: something it does, or a submenu of further lines.
pub struct Item {
    pub label: String,
    pub act: Option<Rc<dyn Fn()>>,
    pub children: Vec<Item>,
}

impl Item {
    /// A line that runs `act`.
    pub fn act(label: impl Into<String>, act: impl Fn() + 'static) -> Self {
        Self { label: label.into(), act: Some(Rc::new(act)), children: Vec::new() }
    }

    /// A line that opens `children`.
    pub fn sub(label: impl Into<String>, children: Vec<Item>) -> Self {
        Self { label: label.into(), act: None, children }
    }
}

/// Offers a card's menu on `list`, whose rows are numbered by their data; `target` says what row is
/// what.
pub fn attach(list: &IFrameworkElement, target: impl Fn(usize) -> Option<Target> + 'static) -> Result<()> {
    attach_items(list, move |at| target(at).map(card_choices))
}

/// Offers on `list` the card menu `items` gives for a row; none for an empty one.
fn attach_items(list: &IFrameworkElement, choices: impl Fn(usize) -> Option<Vec<Item>> + 'static) -> Result<()> {
    keep(list.cast::<IUIElement4>()?.ContextRequested(
        move |sender: Ref<UIElement>, args: Ref<ContextRequestedEventArgs>| {
            let (Ok(sender), Ok(args)) = (sender.ok(), args.ok()) else { return };
            let _ = request(sender, &args, &choices);
        },
    )?);
    Ok(())
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

/// Shows `choices` under `button` as a grid of buttons, for a list too long for one column;
/// `chosen` is marked.
pub fn show_grid_under(button: &FrameworkElement, choices: Vec<Choice>, chosen: Option<usize>) -> Result<()> {
    // As many columns as the window has room for, the flyout's own padding aside.
    let room = Window::Current()?.cast::<IWindow>()?.Bounds()?.width as f64 - 48.0;
    let fits = ((room / f64::from(GRID_CELL_WIDTH)).floor() as usize).max(1);
    let columns = grid_columns(choices.len()).min(fits);
    let cells: String = choices
        .iter()
        .enumerate()
        .map(|(at, choice)| {
            let (label, _) = choice;
            let tint = if chosen == Some(at) { r##" Foreground="#0050EF" BorderBrush="#0050EF""## } else { "" };
            format!(
                r##"<Button AutomationProperties.AutomationId="Pick{at}" Content="{}" Margin="3" Padding="8,4" FontSize="13" BorderThickness="1"
                  HorizontalAlignment="Stretch" VerticalAlignment="Stretch" Background="Transparent"{tint} />"##,
                escape(label)
            )
        })
        .collect();
    let flyout = load_object(&format!(
        r##"<Flyout xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
            xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Placement="Bottom">
          <VariableSizedWrapGrid Orientation="Horizontal" MaximumRowsOrColumns="{columns}" ItemWidth="{GRID_CELL_WIDTH}"
            ItemHeight="{GRID_CELL_HEIGHT}">{cells}</VariableSizedWrapGrid></Flyout>"##
    ))?;
    // Names inside a flyout are not found from its content: the buttons are taken in order.
    let cells = flyout.cast::<IFlyout>()?.Content()?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
    let closing = flyout.cast::<IFlyoutBase>()?;
    for (at, (_, act)) in choices.into_iter().enumerate() {
        let closing = closing.clone();
        let button = cells.GetAt(at as u32)?.cast::<IButtonBase>()?;
        keep(button.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
            let _ = closing.Hide();
            act();
        })?);
    }
    closing.ShowAt(button)
}

/// Columns for a grid menu of `count` choices.
fn grid_columns(count: usize) -> usize {
    match count {
        0..=6 => 2,
        7..=12 => 3,
        _ => 4,
    }
}

/// Shows `choices` under `button`.
pub fn show_under(button: &FrameworkElement, choices: Vec<Choice>) -> Result<()> {
    menu(choices)?.cast::<IFlyoutBase>()?.ShowAt(button)
}

/// Offers the menu `choices` gives where `element` is right-clicked or pressed and held.
pub fn attach_menu(element: &IFrameworkElement, choices: impl Fn() -> Vec<Choice> + 'static) -> Result<()> {
    keep(element.cast::<IUIElement4>()?.ContextRequested(
        move |sender: Ref<UIElement>, args: Ref<ContextRequestedEventArgs>| {
            let (Ok(sender), Ok(args)) = (sender.ok(), args.ok()) else { return };
            let source = args.cast::<IRoutedEventArgs>().and_then(|a| a.OriginalSource());
            if let Ok(source) = source {
                let _ = show_menu(sender, &source, args, choices());
            }
        },
    )?);
    Ok(())
}

fn requested(
    list: &UIElement,
    args: &ContextRequestedEventArgs,
    choices: &impl Fn(usize) -> Option<Vec<Choice>>,
) -> Result<()> {
    let source = args.cast::<IRoutedEventArgs>()?.OriginalSource()?;
    let Some(row) = row_of(&source) else { return Ok(()) };
    let Some(choices) = choices(row) else { return Ok(()) };
    show_menu(list, &source, args, choices)
}

/// The same, for a card's tree menu.
fn request(
    list: &UIElement,
    args: &ContextRequestedEventArgs,
    choices: &impl Fn(usize) -> Option<Vec<Item>>,
) -> Result<()> {
    let source = args.cast::<IRoutedEventArgs>()?.OriginalSource()?;
    let Some(row) = row_of(&source) else { return Ok(()) };
    let Some(items) = choices(row) else { return Ok(()) };
    let asked = args.cast::<IContextRequestedEventArgs>()?;
    asked.SetHandled(true)?;
    let mut point = Point::default();
    let by_pointer = asked.TryGetPosition(list, &mut point)?;
    let menu = menu_items(&items)?;
    if by_pointer {
        menu.cast::<IMenuFlyout2>()?.ShowAt(list, point)
    } else {
        menu.cast::<IFlyoutBase>()?.ShowAt(&source.cast::<FrameworkElement>()?)
    }
}

/// The row index carried by `source` or its nearest ancestor that has one: a click lands on a
/// cover or a title inside the row, which carry no data of their own.
pub fn row_of(source: &IInspectable) -> Option<usize> {
    let mut element = source.cast::<IFrameworkElement>().ok();
    while let Some(current) = element {
        if let Ok(context) = current.DataContext() {
            if let Ok(index) = context.cast::<IPropertyValue>().and_then(|v| v.GetInt32()) {
                return Some(index as usize);
            }
        }
        element = current.Parent().ok().and_then(|parent| parent.cast::<IFrameworkElement>().ok());
    }
    None
}

/// Shows `choices` at the pointer over `target`, or by `source` when asked from the keyboard.
fn show_menu(
    target: &UIElement,
    source: &IInspectable,
    args: &ContextRequestedEventArgs,
    choices: Vec<Choice>,
) -> Result<()> {
    if choices.is_empty() {
        return Ok(());
    }
    let list = target;
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

fn item_xaml(item: &Item) -> String {
    if item.children.is_empty() {
        return format!(r##"<MenuFlyoutItem Text="{}" />"##, escape(&item.label));
    }
    let inner: String = item.children.iter().map(item_xaml).collect();
    format!(r##"<MenuFlyoutSubItem Text="{}">{}</MenuFlyoutSubItem>"##, escape(&item.label), inner)
}

/// A flat choice as a leaf item.
fn leaf(choice: &Choice) -> Item {
    let (label, act) = choice;
    Item { label: label.clone(), act: Some(act.clone()), children: Vec::new() }
}

/// The actions of `items` in the order their lines appear, submenus flattened.
fn actions(items: &[Item], out: &mut Vec<Rc<dyn Fn()>>) {
    for item in items {
        if item.children.is_empty() {
            if let Some(act) = &item.act {
                out.push(act.clone());
            }
        } else {
            actions(&item.children, out);
        }
    }
}

fn card_choices(target: Target) -> Vec<Item> {
    let Target { later, link, recommended, add_list } = target;
    let mut choices: Vec<Item> = Vec::new();
    if let Some(rec) = recommended {
        let (bvid, title) = (rec.bvid.clone(), rec.title.clone());
        choices.push(Item::act("为什么推荐", move || why::show(&bvid, &title)));
        let dislike = rec.bvid.clone();
        choices.push(Item::act("不感兴趣", move || why::ask_dislike(&dislike)));
    }
    if let Some(bvid) = later {
        choices.push(Item::act("稍后再看", move || save_for_later(&bvid)));
    }
    if let Some(bvid) = add_list {
        let children = list_choices(&bvid);
        if !children.is_empty() {
            choices.push(Item::sub("加入播放列表", children));
        }
    }
    choices.push(Item::act("复制链接", move || copy(&link)));
    choices
}

fn menu(choices: Vec<Choice>) -> Result<IInspectable> {
    let items: Vec<Item> = choices.iter().map(leaf).collect();
    menu_items(&items)
}

/// Builds the flyout for `items` and binds each leaf to its action.
fn menu_items(items: &[Item]) -> Result<IInspectable> {
    let lines: String = items.iter().map(item_xaml).collect();
    let menu = load_object(&format!(
        r##"<MenuFlyout xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">{lines}</MenuFlyout>"##
    ))?;
    let listed = menu.cast::<IMenuFlyout>()?.Items()?;
    let mut leaves: Vec<Rc<dyn Fn()>> = Vec::new();
    actions(items, &mut leaves);
    let mut at = 0usize;
    for slot in 0..listed.Size()? {
        let element = listed.GetAt(slot)?;
        match element.cast::<crate::xaml::IMenuFlyoutSubItem>() {
            Ok(sub) => {
                let inner = sub.Items()?;
                for row in 0..inner.Size()? {
                    bind(&inner.GetAt(row)?, &leaves, &mut at);
                }
            }
            Err(_) => bind(&element, &leaves, &mut at),
        }
    }
    Ok(menu)
}

/// Binds the next leaf action to `element`.
fn bind(element: &crate::xaml::MenuFlyoutItemBase, leaves: &[Rc<dyn Fn()>], at: &mut usize) {
    let Some(act) = leaves.get(*at).cloned() else { return };
    *at += 1;
    if let Ok(line) = element.cast::<IMenuFlyoutItem>() {
        if let Ok(revoker) = line.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| act()) {
            keep(revoker);
        }
    }
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

/// The play lists to choose from, as menu lines.
fn list_choices(bvid: &str) -> Vec<Item> {
    let Some(shell) = shell() else { return Vec::new() };
    let Some(backend) = shell.backend.get() else { return Vec::new() };
    let lists = backend.services.playlists.lists().unwrap_or_default();
    lists
        .into_iter()
        .map(|list| {
            let label = format!("{} · {}", list.name, list.count);
            let bvid = bvid.to_owned();
            let id = list.id;
            Item::act(label, move || add_to(id, &bvid))
        })
        .collect()
}

/// Puts `bvid` in list `list`.
fn add_to(list: i64, bvid: &str) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (bvid, dispatcher, core) = (bvid.to_owned(), shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let said = match core.services.playlists.add(list, &bvid).await {
            Ok(true) => "已加入播放列表".to_owned(),
            Ok(false) => "已经在播放列表里".to_owned(),
            Err(problem) => format!("加入失败：{problem}"),
        };
        post(&dispatcher, move || toast(&said));
    });
}

/// Puts `bvid` in the default play list, which 播放全部 plays.
pub fn add_to_default(bvid: &str) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (bvid, dispatcher, core) = (bvid.to_owned(), shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let said = match core.services.playlists.add(rubi_core::DEFAULT_PLAYLIST, &bvid).await {
            Ok(true) => "已加入播放列表".to_owned(),
            Ok(false) => "已经在播放列表里".to_owned(),
            Err(problem) => format!("加入失败：{problem}"),
        };
        post(&dispatcher, move || toast(&said));
    });
}

fn copy(link: &str) {
    copy_text(link, "已复制链接");
}

/// Puts `text` on the clipboard and says `done`, or that it failed.
pub fn copy_text(text: &str, done: &str) {
    let copied = (|| {
        let package = DataPackage::new()?;
        package.cast::<IDataPackage>()?.SetText(text)?;
        Clipboard::SetContent(&package)
    })();
    toast(if copied.is_ok() { done } else { "复制失败" });
}
