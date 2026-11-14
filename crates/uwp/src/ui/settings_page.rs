//! 设置: a page of its own. It shares only the panel builder with 我的, which keeps the account
//! and the shelves.

use std::cell::Cell;
use std::rc::Rc;

use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::settings::Category;
use crate::ui::settings;
use crate::ui::shell::shell;
use crate::ui::{keep, load, named};
use crate::xaml::{Color, IContentControl, IControl, IFrameworkElement, IButtonBase, IPanel, RoutedEventArgs, SolidColorBrush, UIElement};

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };

const LAYOUT: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <StackPanel x:Name="Cats" Orientation="Horizontal" Margin="16,12,16,10" />
  <ContentControl x:Name="Body" Grid.Row="1" IsTabStop="False" HorizontalContentAlignment="Stretch"
                  VerticalContentAlignment="Stretch" />
</Grid>"##;

thread_local! {
    static PAGE: std::cell::RefCell<Option<Rc<Page>>> = const { std::cell::RefCell::new(None) };
    static CATEGORY: Cell<Category> = const { Cell::new(Category::Look) };
}

struct Page {
    root: IFrameworkElement,
    /// The category buttons, kept because elements added to a panel are not found by name.
    cats: std::cell::RefCell<Vec<(Category, IControl)>>,
}

/// Shows the settings page.
pub fn show() -> Result<()> {
    let Some(host) = shell() else { return Ok(()) };
    let root = load(LAYOUT)?;
    let page = Rc::new(Page { root: root.clone(), cats: std::cell::RefCell::new(Vec::new()) });
    PAGE.with_borrow_mut(|slot| *slot = Some(page.clone()));
    let row = named::<IPanel>(&root, "Cats")?.Children()?.cast::<IVector<UIElement>>()?;
    row.Clear()?;
    for category in Category::ALL {
        let button = load(&format!(
            r##"<Button xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                  xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Content="{}" Margin="0,0,18,0"
                  Background="Transparent" BorderThickness="0" Padding="0" FontSize="15" />"##,
            category.label()
        ))?;
        keep(button.cast::<IButtonBase>()?.Click(
            move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| pick(category),
        )?);
        row.Append(&button.cast::<UIElement>()?)?;
        if let Ok(control) = button.cast::<IControl>() {
            page.cats.borrow_mut().push((category, control));
        }
    }
    host.set_page(&root)?;
    paint()?;
    show_category(CATEGORY.get())
}

fn pick(category: Category) {
    CATEGORY.set(category);
    let _ = paint();
    let _ = show_category(category);
}

/// Paints which category is open.
fn paint() -> Result<()> {
    let Some(page) = PAGE.with_borrow(Clone::clone) else { return Ok(()) };
    let on = CATEGORY.get();
    for (category, control) in page.cats.borrow().iter() {
        let tint = if *category == on { COBALT } else { DIM };
        control.SetForeground(&SolidColorBrush::CreateInstanceWithColor(tint)?)?;
    }
    Ok(())
}

/// Fills the page in with `category`.
fn show_category(category: Category) -> Result<()> {
    let Some(page) = PAGE.with_borrow(Clone::clone) else { return Ok(()) };
    let Some(backend) = shell().and_then(|s| s.backend.get().cloned()) else { return Ok(()) };
    let body = settings::build(category, &backend)?;
    named::<IContentControl>(&page.root, "Body")?.SetContent(&body.cast::<UIElement>()?)
}
