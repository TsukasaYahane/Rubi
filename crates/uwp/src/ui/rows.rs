//! Virtualized lists: a row's content is filled in as it comes on screen, which also stands in
//! for data binding. A row's template is one panel whose children are the parts to fill.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use windows_collections::IVector;
use windows_core::{EventRevoker, IInspectable, Interface, Ref, Result, Weak as WeakRef};

use crate::state::images::{ROUGH_QUALITY, rough_width, sharp_width, sized};
use crate::ui::keep;
use crate::xaml::{
    BitmapImage, ContainerContentChangingEventArgs, IBitmapImage, IContentControl2, IFrameworkElement, IImage,
    IItemClickEventArgs, IItemsControl, IListViewBase, IListViewBase2, IPanel, IPropertyValue, ItemClickEventArgs,
    ListViewBase, PropertyValue, RoutedEventArgs, UIElement, Uri,
};

/// Most covers held.
const COVERS_HELD: usize = 240;

/// Covers already asked for. A row that comes back takes its picture from here at once, instead
/// of starting a new load and showing nothing for a moment. A picture on B站's servers shows a
/// tiny rough copy first, then a sharp one at the size shown once that has come.
#[derive(Default)]
pub struct Covers(RefCell<HashMap<String, Held>>);

struct Held {
    rough: Option<BitmapImage>,
    sharp: BitmapImage,
    /// The sharp copy has come.
    ready: Rc<Cell<bool>>,
    /// Images showing the rough copy, to be given the sharp one.
    waiting: Rc<RefCell<Vec<WeakRef<IImage>>>>,
    _opened: EventRevoker,
}

fn bitmap(url: &str, decode_width: i32) -> Result<BitmapImage> {
    let bitmap = BitmapImage::CreateInstanceWithUriSource(&Uri::CreateUri(url)?)?;
    bitmap.cast::<IBitmapImage>()?.SetDecodePixelWidth(decode_width)?;
    Ok(bitmap)
}

/// What an image is meant to show, so a reused row does not take an earlier row's picture.
fn wants(image: &IImage) -> Option<String> {
    image.cast::<IFrameworkElement>().ok()?.Tag().ok()?.cast::<IPropertyValue>().ok()?.GetString().ok()
}

impl Held {
    fn new(url: &str, decode_width: i32) -> Result<Self> {
        let sharp_url = sized(url, sharp_width(decode_width), None).unwrap_or_else(|| url.to_owned());
        let rough = match sized(url, rough_width(decode_width), Some(ROUGH_QUALITY)) {
            Some(rough_url) => Some(bitmap(&rough_url, decode_width)?),
            None => None,
        };
        let sharp = bitmap(&sharp_url, decode_width)?;
        let ready = Rc::new(Cell::new(false));
        let waiting: Rc<RefCell<Vec<WeakRef<IImage>>>> = Rc::default();
        let (came, given, url) = (ready.clone(), waiting.clone(), url.to_owned());
        let opened =
            sharp.cast::<IBitmapImage>()?.ImageOpened(move |sender: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                came.set(true);
                let Ok(sharp) = sender.ok().and_then(|s| s.cast::<BitmapImage>()) else { return };
                for image in given.take().into_iter().filter_map(|w| w.upgrade()) {
                    if wants(&image).as_deref() == Some(url.as_str()) {
                        let _ = image.SetSource(&sharp);
                    }
                }
            })?;
        Ok(Self { rough, sharp, ready, waiting, _opened: opened })
    }
}

impl Covers {
    /// Shows the picture at `url` in `image`, decoded `decode_width` px wide.
    pub fn show(&self, image: &IImage, url: &str, decode_width: i32) -> Result<()> {
        image.cast::<IFrameworkElement>()?.SetTag(&PropertyValue::CreateString(url)?)?;
        let mut held = self.0.borrow_mut();
        if !held.contains_key(url) {
            if held.len() >= COVERS_HELD {
                held.clear();
            }
            held.insert(url.to_owned(), Held::new(url, decode_width)?);
        }
        let Some(entry) = held.get(url) else { return Ok(()) };
        if entry.ready.get() {
            return image.SetSource(&entry.sharp);
        }
        match &entry.rough {
            Some(rough) => {
                image.SetSource(rough)?;
                entry.waiting.borrow_mut().push(image.downgrade()?);
                Ok(())
            }
            None => image.SetSource(&entry.sharp),
        }
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
