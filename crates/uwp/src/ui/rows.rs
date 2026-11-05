//! Virtualized lists: a row's content is filled in as it comes on screen, which also stands in
//! for data binding. A row's template is one panel whose children are the parts to fill.

use std::cell::RefCell;
use std::collections::HashMap;

use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::ui::keep;
use crate::xaml::{
    BitmapImage, ContainerContentChangingEventArgs, IBitmapImage, IContentControl2, IFrameworkElement,
    IItemClickEventArgs, IItemsControl, IListViewBase, IListViewBase2, IPanel, IPropertyValue, ItemClickEventArgs,
    ListViewBase, PropertyValue, UIElement, Uri,
};

/// Most covers held.
const COVERS_HELD: usize = 240;

/// Covers already asked for. A row that comes back takes its picture from here at once, instead
/// of starting a new load and showing nothing for a moment.
#[derive(Default)]
pub struct Covers(RefCell<HashMap<String, BitmapImage>>);

impl Covers {
    pub fn get(&self, url: &str, decode_width: i32) -> Result<BitmapImage> {
        let mut held = self.0.borrow_mut();
        if let Some(bitmap) = held.get(url) {
            return Ok(bitmap.clone());
        }
        if held.len() >= COVERS_HELD {
            held.clear();
        }
        let bitmap = BitmapImage::CreateInstanceWithUriSource(&Uri::CreateUri(url)?)?;
        bitmap.cast::<IBitmapImage>()?.SetDecodePixelWidth(decode_width)?;
        held.insert(url.to_owned(), bitmap.clone());
        Ok(bitmap)
    }
}

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
    keep(revoker);
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
    keep(revoker);
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
