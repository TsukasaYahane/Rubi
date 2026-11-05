//! The application object and its first window.

use std::cell::RefCell;

use rubi_core::{Services, Session};
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result, implement};

use crate::xaml::{
    Application, ApplicationData, ApplicationInitializationCallback, ApplicationInitializationCallbackParams,
    CachedFileUpdaterActivatedEventArgs, ContainerContentChangingEventArgs, CoreDispatcher, CoreDispatcherPriority,
    DispatchedHandler, FileActivatedEventArgs, FileOpenPickerActivatedEventArgs, FileSavePickerActivatedEventArgs,
    IActivatedEventArgs, IApplicationData, IApplicationOverrides, IApplicationOverrides_Impl, IContentControl2,
    ICoreDispatcher, IFrameworkElement, IItemsControl, IListViewBase2, IStorageItem, ITextBlock, IWindow,
    LaunchActivatedEventArgs, ListViewBase, PropertyValue, SearchActivatedEventArgs, ShareTargetActivatedEventArgs,
    UIElement, Window, WindowCreatedEventArgs, XamlReader,
};

windows_link::link!("api-ms-win-core-winrt-l1-1-0.dll" "system" fn RoInitialize(init_type: i32) -> i32);
const RO_INIT_MULTITHREADED: i32 = 1;

thread_local! {
    /// The line the check below reports on; touched on the UI thread only.
    static STATUS: RefCell<Option<ITextBlock>> = const { RefCell::new(None) };
}

/// Starts XAML; returns when the app closes.
pub fn run() -> Result<()> {
    windows_core::HRESULT(unsafe { RoInitialize(RO_INIT_MULTITHREADED) }).ok()?;
    Application::Start(&ApplicationInitializationCallback::new(|_: Ref<ApplicationInitializationCallbackParams>| {
        // XAML holds the application for the life of the process.
        std::mem::forget(Application::compose(App));
    }))
}

#[implement(IApplicationOverrides)]
struct App;

impl IApplicationOverrides_Impl for App_Impl {
    fn OnLaunched(&self, _: Ref<LaunchActivatedEventArgs>) -> Result<()> {
        show_window()
    }

    fn OnActivated(&self, _: Ref<IActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnFileActivated(&self, _: Ref<FileActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnSearchActivated(&self, _: Ref<SearchActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnShareTargetActivated(&self, _: Ref<ShareTargetActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnFileOpenPickerActivated(&self, _: Ref<FileOpenPickerActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnFileSavePickerActivated(&self, _: Ref<FileSavePickerActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnCachedFileUpdaterActivated(&self, _: Ref<CachedFileUpdaterActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnWindowCreated(&self, _: Ref<WindowCreatedEventArgs>) -> Result<()> {
        Ok(())
    }
}

/// The window's layout. Laid out in XAML because layout and templates are data; behaviour is
/// attached in code, and no custom type is named here, so no metadata provider is needed.
const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
      Background="{ThemeResource ApplicationPageBackgroundThemeBrush}" Padding="24">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" />
    <RowDefinition Height="Auto" />
    <RowDefinition Height="*" />
  </Grid.RowDefinitions>
  <TextBlock Text="Rubi" FontSize="42" FontWeight="Light" Foreground="#0050EF" />
  <TextBlock x:Name="Status" Grid.Row="1" Text="正在检查核心…" FontSize="18" TextWrapping="Wrap" />
  <ListView x:Name="List" Grid.Row="2" Margin="0,12,0,0">
    <ListView.ItemTemplate>
      <DataTemplate><TextBlock FontSize="16" /></DataTemplate>
    </ListView.ItemTemplate>
  </ListView>
</Grid>"##;

/// Rows in the list check: far more than a screen shows.
const LIST_ROWS: i32 = 5_000;

/// Fills a row as it comes on screen: what stands in for data binding, and what a virtualized
/// list needs anyway.
fn fill_row(args: &ContainerContentChangingEventArgs) -> Result<()> {
    if args.InRecycleQueue()? {
        return Ok(());
    }
    let row = args.ItemContainer()?.cast::<IContentControl2>()?.ContentTemplateRoot()?.cast::<ITextBlock>()?;
    row.SetText(&format!("第 {} 条", args.ItemIndex()? + 1))
}

fn show_window() -> Result<()> {
    let root = XamlReader::Load(LAYOUT)?.cast::<IFrameworkElement>()?;
    let status = root.FindName("Status")?.cast::<ITextBlock>()?;
    STATUS.with(|s| *s.borrow_mut() = Some(status));

    let list = root.FindName("List")?;
    let items = list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?;
    for row in 0..LIST_ROWS {
        items.Append(&PropertyValue::CreateInt32(row)?)?;
    }
    let revoker = list.cast::<IListViewBase2>()?.ContainerContentChanging(
        |_: Ref<ListViewBase>, args: Ref<ContainerContentChangingEventArgs>| {
            if let Ok(args) = args.ok() {
                let _ = fill_row(args);
            }
        },
    )?;
    // The window lives as long as the process, and so does this handler.
    std::mem::forget(revoker);

    let window = Window::Current()?.cast::<IWindow>()?;
    window.SetContent(&root.cast::<UIElement>()?)?;
    let dispatcher = window.Dispatcher()?;
    window.Activate()?;

    let folder = ApplicationData::Current()?.cast::<IApplicationData>()?.LocalFolder()?;
    let data_dir = folder.cast::<IStorageItem>()?.Path()?;
    std::thread::spawn(move || {
        let report = check_core(&data_dir);
        let _ = std::fs::write(std::path::Path::new(&data_dir).join("check.txt"), &report);
        let _ = report_on(dispatcher, report);
    });
    Ok(())
}

/// Opens the core in the sandbox and makes one guest request: the database, TLS and the
/// network all have to work inside the app container.
fn check_core(data_dir: &str) -> String {
    let opened = match Services::open(data_dir, Session::default()) {
        Ok(app) => app,
        Err(e) => return format!("核心打开失败：{e}"),
    };
    let lists = opened.playlists.lists().map(|l| l.len()).unwrap_or(0);
    let runtime = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(r) => r,
        Err(e) => return format!("运行时启动失败：{e}"),
    };
    match runtime.block_on(opened.discover.popular(1)) {
        Ok(cards) => format!("核心已打开（{lists} 个列表）；热门 {} 条", cards.len()),
        Err(e) => format!("核心已打开（{lists} 个列表）；联网失败：{e}"),
    }
}

fn report_on(dispatcher: CoreDispatcher, report: String) -> Result<()> {
    let show = DispatchedHandler::new(move || {
        STATUS.with(|s| {
            if let Some(block) = s.borrow().as_ref() {
                let _ = block.SetText(&report);
            }
        });
    });
    dispatcher.cast::<ICoreDispatcher>()?.RunAsync(CoreDispatcherPriority::Normal, &show)?;
    Ok(())
}
