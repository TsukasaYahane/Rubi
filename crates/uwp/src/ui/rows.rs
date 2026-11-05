//! Virtualized lists: a row's content is filled in as it comes on screen, which also stands in
//! for data binding. A row's template is one panel whose children are the parts to fill.

use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::xaml::{
    ContainerContentChangingEventArgs, IContentControl2, IFrameworkElement, IItemClickEventArgs, IItemsControl,
    IListViewBase, IListViewBase2, IPanel, IPropertyValue, ItemClickEventArgs, ListViewBase, PropertyValue, UIElement,
};

/// Calls `fill` with a row's index and parts when it comes on screen.
pub fn on_row(
    list: &IFrameworkElement,
    fill: impl Fn(usize, &IVector<UIElement>) -> Result<()> + 'static,
) -> Result<()> {
    let revoker = list.cast::<IListViewBase2>()?.ContainerContentChanging(
        move |_: Ref<ListViewBase>, args: Ref<ContainerContentChangingEventArgs>| {
            if let Ok(args) = args.ok() {
                let _ = fill_row(args, &fill);
            }
        },
    )?;
    std::mem::forget(revoker);
    Ok(())
}

fn fill_row(
    args: &ContainerContentChangingEventArgs,
    fill: &impl Fn(usize, &IVector<UIElement>) -> Result<()>,
) -> Result<()> {
    if args.InRecycleQueue()? {
        return Ok(());
    }
    let row = args.ItemContainer()?.cast::<IContentControl2>()?.ContentTemplateRoot()?;
    let parts = row.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
    fill(args.ItemIndex()? as usize, &parts)
}

/// Calls `open` with the index of a clicked row.
pub fn on_row_click(list: &IFrameworkElement, open: impl Fn(usize) + 'static) -> Result<()> {
    let revoker =
        list.cast::<IListViewBase>()?.ItemClick(move |_: Ref<IInspectable>, args: Ref<ItemClickEventArgs>| {
            let clicked = args
                .ok()
                .and_then(|a| a.cast::<IItemClickEventArgs>()?.ClickedItem()?.cast::<IPropertyValue>()?.GetInt32());
            if let Ok(at) = clicked {
                open(at as usize);
            }
        })?;
    std::mem::forget(revoker);
    Ok(())
}

/// Adds rows `from..to` to the list.
pub fn add_rows(list: &IFrameworkElement, from: usize, to: usize) -> Result<()> {
    let items = list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?;
    for at in from..to {
        items.Append(&PropertyValue::CreateInt32(at as i32)?)?;
    }
    Ok(())
}
