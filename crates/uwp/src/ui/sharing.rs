//! The system share sheet, for whatever the app offers: a picture on the web, a link, or a file it
//! made. The sheet asks for its contents through one handler per window, set here once.

use std::cell::{Cell, RefCell};

use windows_core::{Interface, Ref, Result};

use crate::ui::keep;
use crate::ui::shell::toast;
use crate::xaml::{
    DataRequestedEventArgs, DataTransferManager, IDataPackage, IDataPackage2, IDataPackagePropertySet, IDataRequest,
    IDataRequestedEventArgs, IDataTransferManager, RandomAccessStreamReference, StorageFile, Uri,
};

/// What goes to the share sheet.
#[derive(Clone)]
pub enum Shared {
    /// A picture on the web, with its address.
    Picture { url: String },
    /// A page: its title and address.
    Link { title: String, url: String },
    /// A picture the app made.
    Image { title: String, file: StorageFile },
}

thread_local! {
    static SHARING: RefCell<Option<Shared>> = const { RefCell::new(None) };
    static HOOKED: Cell<bool> = const { Cell::new(false) };
}

/// Opens the share sheet with `item`.
pub fn share(item: Shared) {
    SHARING.with_borrow_mut(|s| *s = Some(item));
    if !HOOKED.get() {
        let hooked = (|| -> Result<()> {
            let manager = DataTransferManager::GetForCurrentView()?;
            keep(manager.cast::<IDataTransferManager>()?.DataRequested(
                |_: Ref<DataTransferManager>, args: Ref<DataRequestedEventArgs>| {
                    let _ = args.ok().map(fill);
                },
            )?);
            Ok(())
        })();
        if hooked.is_err() {
            toast("分享不可用");
            return;
        }
        HOOKED.set(true);
    }
    if DataTransferManager::ShowShareUI().is_err() {
        toast("分享不可用");
    }
}

fn fill(args: &DataRequestedEventArgs) -> Result<()> {
    let Some(item) = SHARING.with_borrow(Clone::clone) else { return Ok(()) };
    let data = args.cast::<IDataRequestedEventArgs>()?.Request()?.cast::<IDataRequest>()?.Data()?;
    let package = data.cast::<IDataPackage>()?;
    let properties = package.Properties()?.cast::<IDataPackagePropertySet>()?;
    match item {
        Shared::Picture { url } => {
            properties.SetTitle("分享图片")?;
            let uri = Uri::CreateUri(&url)?;
            package.SetText(&url)?;
            data.cast::<IDataPackage2>()?.SetWebLink(&uri)?;
            package.SetBitmap(&RandomAccessStreamReference::CreateFromUri(&uri)?)
        }
        Shared::Link { title, url } => {
            properties.SetTitle(&title)?;
            package.SetText(&format!("【{title}】 {url}"))?;
            data.cast::<IDataPackage2>()?.SetWebLink(&Uri::CreateUri(&url)?)
        }
        Shared::Image { title, file } => {
            properties.SetTitle(&title)?;
            package.SetBitmap(&RandomAccessStreamReference::CreateFromFile(&file)?)
        }
    }
}
