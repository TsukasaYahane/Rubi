//! A bar the user drags to resize the panes on either side of it. Layout follows the drag as it
//! happens; a double click goes back to the default width.

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::ui::keep;
use crate::ui::named;
use crate::xaml::{
    Color, CoreCursor, CoreCursorType, CoreWindow, DoubleTappedRoutedEventArgs, DragCompletedEventArgs,
    DragDeltaEventArgs, IControl, ICoreWindow, IDragDeltaEventArgs, IFrameworkElement, IThumb, IUIElement,
    PointerRoutedEventArgs, SolidColorBrush,
};

/// The bar in a XAML layout.
pub const BAR_TEMPLATE: &str = r##"<Thumb.Template><ControlTemplate TargetType="Thumb">
  <Border Background="{TemplateBinding Background}" /></ControlTemplate></Thumb.Template>"##;

const COBALT: Color = Color { a: 255, r: 0x00, g: 0x50, b: 0xEF };
const CLEAR: Color = Color { a: 0, r: 0, g: 0, b: 0 };

fn set_cursor(kind: CoreCursorType) -> Result<()> {
    let cursor = CoreCursor::CreateCursor(kind, 0)?;
    CoreWindow::GetForCurrentThread()?.cast::<ICoreWindow>()?.SetPointerCursor(&cursor)
}

fn tint(bar: &IControl, color: Color) -> Result<()> {
    bar.SetBackground(&SolidColorBrush::CreateInstanceWithColor(color)?)
}

/// Wires the bar named `name`: `drag` gets each movement, `done` the end of a drag, `reset` a
/// double click.
pub fn attach(
    root: &IFrameworkElement,
    name: &str,
    drag: impl Fn(f64) + 'static,
    done: impl Fn() + 'static,
    reset: impl Fn() + 'static,
) -> Result<()> {
    let thumb = named::<IThumb>(root, name)?;
    let moved = thumb.DragDelta(move |_: Ref<IInspectable>, args: Ref<DragDeltaEventArgs>| {
        if let Ok(change) = args.ok().and_then(|a| a.cast::<IDragDeltaEventArgs>()?.HorizontalChange()) {
            drag(change);
        }
    })?;
    keep(moved);
    let finished = thumb.DragCompleted(move |_: Ref<IInspectable>, _: Ref<DragCompletedEventArgs>| done())?;
    keep(finished);

    let element = named::<IUIElement>(root, name)?;
    let entered = element.PointerEntered(move |sender: Ref<IInspectable>, _: Ref<PointerRoutedEventArgs>| {
        let _ = set_cursor(CoreCursorType::SizeWestEast);
        let _ = sender.ok().and_then(|bar| tint(&bar.cast()?, COBALT));
    })?;
    keep(entered);
    let exited = element.PointerExited(move |sender: Ref<IInspectable>, _: Ref<PointerRoutedEventArgs>| {
        let _ = set_cursor(CoreCursorType::Arrow);
        let _ = sender.ok().and_then(|bar| tint(&bar.cast()?, CLEAR));
    })?;
    keep(exited);
    let tapped = element.DoubleTapped(move |_: Ref<IInspectable>, _: Ref<DoubleTappedRoutedEventArgs>| reset())?;
    keep(tapped);
    Ok(())
}
