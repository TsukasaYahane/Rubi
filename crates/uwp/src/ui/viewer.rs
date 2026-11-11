//! 图片查看器: pictures over the whole window but its title bar, one at a time. Arrows and ←/→
//! go through them; the wheel or a double click enlarges one (up to 4×) and the mouse drags it; a
//! 长图 fills the width and scrolls. A rough copy shows first and the original replaces it. The
//! bar saves the picture to 图片 › Rubi, shares it, or copies its address. Esc, ✕ or a click
//! beside the picture closes it.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use windows_collections::IVector;
use windows_core::{EventRevoker, HSTRING, IInspectable, Interface, Ref, Result};

use crate::state::images::sized;
use crate::state::pictures::{DOUBLE_ZOOM, MAX_ZOOM, file_name, fit, original, zoom_offsets};
use crate::ui::card_menu::{self, Choice, copy_text};
use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load, named, on_click, post};
use crate::xaml::{
    BitmapImage, Clipboard, CreationCollisionOption, DataPackage, DataRequestedEventArgs, DataTransferManager,
    DoubleTappedRoutedEventArgs, FileIO, FileSavePicker, FocusState, HttpClient, IBitmapImage, IBitmapSource, IControl,
    IDataPackage, IDataPackage2, IDataPackagePropertySet, IDataRequest, IDataRequestedEventArgs, IDataTransferManager,
    IDoubleTappedRoutedEventArgs, IFileSavePicker, IFrameworkElement, IHttpClient, IImage, IKeyRoutedEventArgs,
    IPointer, IPointerRoutedEventArgs, IRoutedEventArgs, IScrollViewer, IScrollViewer2, IStorageFolder, IStorageItem,
    IUIElement, KeyRoutedEventArgs, KnownFolders, PickerLocationId, PointerDeviceType, PointerRoutedEventArgs,
    RandomAccessStreamReference, RoutedEventArgs, SizeChangedEventArgs, StorageFile, UIElement, Uri, VirtualKey,
    Visibility,
};

/// Width of the rough copy shown while the original comes.
const ROUGH_WIDTH: u32 = 240;
const ROUGH_QUALITY: u32 = 30;
/// A 长图 is decoded this wide at most: a very long one would take a great deal of memory.
const LONG_DECODE_WIDTH: i32 = 1440;
/// A wheel notch enlarges or shrinks by this much.
const WHEEL_STEP: f32 = 1.2;
/// Moves smaller than this are a click, not a drag.
const DRAG_SLOP: f64 = 4.0;

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#EB000000" RequestedTheme="Dark">
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <Grid Background="#8C000000" Padding="8,4">
    <Grid.Resources>
      <Style TargetType="Button">
        <Setter Property="Background" Value="Transparent" />
        <Setter Property="Foreground" Value="White" />
        <Setter Property="BorderThickness" Value="0" />
        <Setter Property="Padding" Value="10,6" />
        <Setter Property="Margin" Value="2,0" />
      </Style>
    </Grid.Resources>
    <Grid.ColumnDefinitions>
      <ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" />
      <ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" />
    </Grid.ColumnDefinitions>
    <TextBlock x:Name="ViewCount" FontSize="13" Margin="6,0" VerticalAlignment="Center" Foreground="White" />
    <Button x:Name="ViewSave" Grid.Column="1" Content="保存" FontSize="13" />
    <Button x:Name="ViewShare" Grid.Column="2" Content="分享" FontSize="13" />
    <Button x:Name="ViewCopy" Grid.Column="3" Content="复制链接" FontSize="13" />
    <Button x:Name="ViewClose" Grid.Column="4" AutomationProperties.Name="关闭"><FontIcon Glyph="&#xE711;" FontSize="14" /></Button>
  </Grid>
  <ScrollViewer x:Name="ViewScroll" Grid.Row="1" ZoomMode="Enabled" MinZoomFactor="1" MaxZoomFactor="4"
                HorizontalScrollMode="Auto" VerticalScrollMode="Auto" HorizontalScrollBarVisibility="Auto"
                VerticalScrollBarVisibility="Auto">
    <Grid x:Name="ViewArea" Background="Transparent">
      <Image x:Name="ViewImage" Stretch="Uniform" HorizontalAlignment="Center" VerticalAlignment="Center" />
    </Grid>
  </ScrollViewer>
  <Button x:Name="ViewPrev" Grid.Row="1" HorizontalAlignment="Left" VerticalAlignment="Center" Margin="8,0"
          Width="40" Height="64" Padding="0" Background="#8C000000" Foreground="White" BorderThickness="0"
          AutomationProperties.Name="上一张"><FontIcon Glyph="&#xE76B;" FontSize="16" /></Button>
  <Button x:Name="ViewNext" Grid.Row="1" HorizontalAlignment="Right" VerticalAlignment="Center" Margin="8,0"
          Width="40" Height="64" Padding="0" Background="#8C000000" Foreground="White" BorderThickness="0"
          AutomationProperties.Name="下一张"><FontIcon Glyph="&#xE76C;" FontSize="16" /></Button>
  <Border x:Name="ViewHint" Grid.Row="1" VerticalAlignment="Bottom" HorizontalAlignment="Center" Margin="0,0,0,12"
          Padding="10,4" Background="#8C000000" Visibility="Collapsed" IsHitTestVisible="False">
    <TextBlock Text="长图 · 上下滚动浏览" FontSize="12" Foreground="White" />
  </Border>
</Grid>"##;

/// A drag of the enlarged picture: where the pointer went down, and the offsets then.
#[derive(Clone, Copy)]
struct Drag {
    from: (f64, f64),
    offset: (f64, f64),
    moved: bool,
}

struct Viewer {
    root: IFrameworkElement,
    pictures: Vec<String>,
    at: Cell<usize>,
    /// The picture's height over width; 0 until a copy of it has come.
    ratio: Cell<f64>,
    /// Bumped per picture shown: a copy that comes late for another is dropped.
    turn: Cell<u64>,
    drag: Cell<Option<Drag>>,
    /// The loads of the picture on show.
    loads: RefCell<Vec<EventRevoker>>,
}

thread_local! {
    static VIEWER: RefCell<Option<Rc<Viewer>>> = const { RefCell::new(None) };
    /// What the system share sheet asks for; the handler is set once a run.
    static SHARING: RefCell<Option<String>> = const { RefCell::new(None) };
    static SHARE_HOOKED: Cell<bool> = const { Cell::new(false) };
}

fn viewer() -> Option<Rc<Viewer>> {
    VIEWER.with_borrow(Clone::clone)
}

/// Shows `pictures` from the one at `start`.
pub fn open(pictures: Vec<String>, start: usize) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    if pictures.is_empty() {
        return Ok(());
    }
    close();
    let root = load(LAYOUT)?;
    let host = shell.viewer_host()?;
    let children = host.Children()?.cast::<IVector<UIElement>>()?;
    children.Clear()?;
    children.Append(&root.cast::<UIElement>()?)?;
    host.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)?;
    let shown = Rc::new(Viewer {
        root: root.clone(),
        pictures,
        at: Cell::new(start),
        ratio: Cell::new(0.0),
        turn: Cell::new(0),
        drag: Cell::new(None),
        loads: RefCell::default(),
    });
    VIEWER.with_borrow_mut(|v| *v = Some(shown.clone()));
    wire(&root)?;
    shown.show(start)?;
    named::<IControl>(&root, "ViewClose")?.Focus(FocusState::Programmatic)?;
    Ok(())
}

/// Closes the viewer, if it is open.
pub fn close() {
    let Some(shown) = VIEWER.with_borrow_mut(Option::take) else { return };
    shown.loads.borrow_mut().clear();
    if let Some(shell) = shell()
        && let Ok(host) = shell.viewer_host()
    {
        let _ = host.Children().and_then(|c| c.cast::<IVector<UIElement>>()).and_then(|c| c.Clear());
        let _ = host.cast::<IUIElement>().and_then(|h| h.SetVisibility(Visibility::Collapsed));
    }
}

fn step(by: isize) {
    if let Some(shown) = viewer() {
        let count = shown.pictures.len() as isize;
        let next = (shown.at.get() as isize + by).rem_euclid(count) as usize;
        let _ = shown.show(next);
    }
}

fn wire(root: &IFrameworkElement) -> Result<()> {
    on_click(root, "ViewClose", close)?;
    on_click(root, "ViewPrev", || step(-1))?;
    on_click(root, "ViewNext", || step(1))?;
    on_click(root, "ViewSave", || current(save))?;
    on_click(root, "ViewShare", || current(share))?;
    on_click(root, "ViewCopy", || current(|url| copy_text(original(url), "已复制图片链接")))?;
    card_menu::attach_menu(&named(root, "ViewArea")?, || {
        let mut choices: Vec<Choice> = Vec::new();
        current(|url| {
            let url = url.to_owned();
            let with = |act: fn(&str)| -> std::rc::Rc<dyn Fn()> {
                let url = url.clone();
                std::rc::Rc::new(move || act(&url))
            };
            choices = vec![
                ("复制图片".to_owned(), with(copy_picture)),
                ("另存为…".to_owned(), with(save_as)),
                ("保存到 图片 › Rubi".to_owned(), with(save)),
                ("分享…".to_owned(), with(share)),
                ("复制图片链接".to_owned(), with(|url| copy_text(original(url), "已复制图片链接"))),
            ];
        });
        choices
    })?;
    let keys = root.cast::<IUIElement>()?.KeyDown(|_: Ref<IInspectable>, args: Ref<KeyRoutedEventArgs>| {
        let Ok(args) = args.ok().and_then(|a| a.cast::<IKeyRoutedEventArgs>()) else { return };
        let act: Option<fn()> = match args.Key() {
            Ok(VirtualKey::Escape) => Some(close),
            Ok(VirtualKey::Left) => Some(|| step(-1)),
            Ok(VirtualKey::Right) => Some(|| step(1)),
            _ => None,
        };
        if let Some(act) = act {
            let _ = args.SetHandled(true);
            act();
        }
    })?;
    let scroll = named::<IUIElement>(root, "ViewScroll")?;
    let sized =
        scroll.cast::<IFrameworkElement>()?.SizeChanged(|_: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
            if let Some(shown) = viewer() {
                let _ = shown.lay_out();
            }
        })?;
    let area = named::<IUIElement>(root, "ViewArea")?;
    let wheel = area.PointerWheelChanged(|_: Ref<IInspectable>, args: Ref<PointerRoutedEventArgs>| {
        if let (Some(shown), Ok(args)) = (viewer(), args.ok()) {
            let _ = shown.wheel(args);
        }
    })?;
    let double = area.DoubleTapped(|_: Ref<IInspectable>, args: Ref<DoubleTappedRoutedEventArgs>| {
        if let (Some(shown), Ok(args)) = (viewer(), args.ok()) {
            let _ = shown.double_tapped(args);
        }
    })?;
    let pressed = area.PointerPressed(|_: Ref<IInspectable>, args: Ref<PointerRoutedEventArgs>| {
        if let (Some(shown), Ok(args)) = (viewer(), args.ok()) {
            let _ = shown.pressed(args);
        }
    })?;
    let moved = area.PointerMoved(|_: Ref<IInspectable>, args: Ref<PointerRoutedEventArgs>| {
        if let (Some(shown), Ok(args)) = (viewer(), args.ok()) {
            let _ = shown.moved(args);
        }
    })?;
    let released = area.PointerReleased(|_: Ref<IInspectable>, args: Ref<PointerRoutedEventArgs>| {
        if let (Some(shown), Ok(args)) = (viewer(), args.ok()) {
            let _ = shown.released(args);
        }
    })?;
    // The events live as long as the viewer's elements; they go with them.
    for revoker in [keys, sized, wheel, double, pressed, moved, released] {
        keep(revoker);
    }
    Ok(())
}

/// Calls `act` with the address of the picture on show.
fn current(act: impl FnOnce(&str)) {
    if let Some(shown) = viewer()
        && let Some(url) = shown.pictures.get(shown.at.get())
    {
        act(url);
    }
}

impl Viewer {
    fn scroll(&self) -> Result<IScrollViewer> {
        named(&self.root, "ViewScroll")
    }

    fn image(&self) -> Result<IImage> {
        named(&self.root, "ViewImage")
    }

    fn show(&self, at: usize) -> Result<()> {
        let Some(url) = self.pictures.get(at).cloned() else { return Ok(()) };
        self.at.set(at);
        let turn = self.turn.get() + 1;
        self.turn.set(turn);
        self.ratio.set(0.0);
        self.loads.borrow_mut().clear();
        let count = self.pictures.len();
        crate::ui::set_text(&self.root, "ViewCount", &format!("{} / {count}", at + 1))?;
        for name in ["ViewPrev", "ViewNext"] {
            named::<IUIElement>(&self.root, name)?.SetVisibility(if count > 1 {
                Visibility::Visible
            } else {
                Visibility::Collapsed
            })?;
        }
        self.image()?.SetSource(None::<&crate::xaml::ImageSource>)?;
        self.lay_out()?;
        match sized(&url, ROUGH_WIDTH, Some(ROUGH_QUALITY)) {
            Some(rough) => {
                let bitmap = BitmapImage::CreateInstanceWithUriSource(&Uri::CreateUri(&rough)?)?;
                let opened = bitmap.cast::<IBitmapImage>()?.ImageOpened(
                    move |sender: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                        if let Some(shown) = viewer().filter(|v| v.turn.get() == turn) {
                            let _ = shown.came(sender.ok().ok(), false);
                        }
                    },
                )?;
                let failed = bitmap.cast::<IBitmapImage>()?.ImageFailed(
                    move |_: Ref<IInspectable>, _: Ref<crate::xaml::ExceptionRoutedEventArgs>| {
                        if let Some(shown) = viewer().filter(|v| v.turn.get() == turn) {
                            let _ = shown.load_original();
                        }
                    },
                )?;
                self.loads.borrow_mut().extend([opened, failed]);
                self.image()?.SetSource(&bitmap)?;
            }
            None => self.load_original()?,
        }
        Ok(())
    }

    /// A copy came: the rough one gives the shape and the original is asked for; the original
    /// replaces it.
    fn came(&self, bitmap: Option<&IInspectable>, original: bool) -> Result<()> {
        let Some(bitmap) = bitmap else { return Ok(()) };
        let source = bitmap.cast::<IBitmapSource>()?;
        let (width, height) = (source.PixelWidth()?, source.PixelHeight()?);
        if self.ratio.get() <= 0.0 && width > 0 {
            self.ratio.set(f64::from(height) / f64::from(width));
            self.lay_out()?;
        }
        if original {
            self.image()?.SetSource(&bitmap.cast::<crate::xaml::ImageSource>()?)
        } else {
            self.load_original()
        }
    }

    fn load_original(&self) -> Result<()> {
        let Some(url) = self.pictures.get(self.at.get()) else { return Ok(()) };
        let turn = self.turn.get();
        let bitmap = BitmapImage::CreateInstanceWithUriSource(&Uri::CreateUri(original(url))?)?;
        if fit(self.ratio.get(), (1.0, 1.0)).long {
            bitmap.cast::<IBitmapImage>()?.SetDecodePixelWidth(LONG_DECODE_WIDTH)?;
        }
        let opened =
            bitmap.cast::<IBitmapImage>()?.ImageOpened(move |sender: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                if let Ok(sender) = sender.ok() {
                    crate::ui::shell::preloaded(sender);
                }
                if let Some(shown) = viewer().filter(|v| v.turn.get() == turn) {
                    let _ = shown.came(sender.ok().ok(), true);
                }
            })?;
        self.loads.borrow_mut().push(opened);
        // Before any copy has come, the original is what shows; otherwise it loads unseen.
        if self.ratio.get() <= 0.0 {
            self.image()?.SetSource(&bitmap)?;
        } else {
            crate::ui::shell::preload(&bitmap)?;
        }
        Ok(())
    }

    /// Fits the picture to the window, back at its whole.
    fn lay_out(&self) -> Result<()> {
        let scroll = self.scroll()?;
        let frame = scroll.cast::<IFrameworkElement>()?;
        let viewport = (frame.ActualWidth()?, frame.ActualHeight()?);
        let laid = fit(self.ratio.get(), viewport);
        let area = named::<IFrameworkElement>(&self.root, "ViewArea")?;
        area.SetWidth(laid.area.0)?;
        area.SetHeight(laid.area.1)?;
        let image = self.image()?.cast::<IFrameworkElement>()?;
        image.SetWidth(laid.picture.0)?;
        image.SetHeight(laid.picture.1)?;
        named::<IUIElement>(&self.root, "ViewHint")?.SetVisibility(if laid.long {
            Visibility::Visible
        } else {
            Visibility::Collapsed
        })?;
        scroll.cast::<IScrollViewer2>()?.ChangeViewWithOptionalAnimation(Some(0.0), Some(0.0), Some(1.0), true)?;
        Ok(())
    }

    fn long(&self) -> bool {
        fit(self.ratio.get(), (1.0, 1.0)).long
    }

    /// Where the pointer is in the viewport.
    fn point(&self, args: &PointerRoutedEventArgs) -> Result<(f64, f64)> {
        let at = args.GetCurrentPoint(&self.scroll()?.cast::<UIElement>()?)?.Position()?;
        Ok((f64::from(at.x), f64::from(at.y)))
    }

    fn offsets(&self) -> Result<(f64, f64)> {
        let scroll = self.scroll()?;
        Ok((scroll.HorizontalOffset()?, scroll.VerticalOffset()?))
    }

    fn zoom_to(&self, to: f32, at: (f64, f64)) -> Result<()> {
        let scroll = self.scroll()?;
        let from = scroll.ZoomFactor()?;
        let to = to.clamp(1.0, MAX_ZOOM);
        let (h, v) = zoom_offsets(at, self.offsets()?, from, to);
        scroll.cast::<IScrollViewer2>()?.ChangeViewWithOptionalAnimation(Some(h), Some(v), Some(to), true)?;
        Ok(())
    }

    /// The wheel enlarges an ordinary picture about the pointer; a 长图 scrolls with it.
    fn wheel(&self, args: &PointerRoutedEventArgs) -> Result<()> {
        if self.long() {
            return Ok(());
        }
        let delta = args.GetCurrentPoint(None::<&UIElement>)?.Properties()?.MouseWheelDelta()?;
        if delta == 0 {
            return Ok(());
        }
        args.cast::<IPointerRoutedEventArgs>()?.SetHandled(true)?;
        let zoom = self.scroll()?.ZoomFactor()?;
        let to = if delta > 0 { zoom * WHEEL_STEP } else { zoom / WHEEL_STEP };
        self.zoom_to(to, self.point(args)?)
    }

    fn double_tapped(&self, args: &DoubleTappedRoutedEventArgs) -> Result<()> {
        let at = args.cast::<IDoubleTappedRoutedEventArgs>()?.GetPosition(&self.scroll()?.cast::<UIElement>()?)?;
        let zoom = self.scroll()?.ZoomFactor()?;
        let to = if zoom > 1.01 { 1.0 } else { DOUBLE_ZOOM };
        self.zoom_to(to, (f64::from(at.x), f64::from(at.y)))
    }

    /// A mouse press starts a drag of the enlarged picture, or a click beside it.
    fn pressed(&self, args: &PointerRoutedEventArgs) -> Result<()> {
        let pointer = args.cast::<IPointerRoutedEventArgs>()?.Pointer()?;
        if pointer.cast::<IPointer>()?.PointerDeviceType()? != PointerDeviceType::Mouse {
            return Ok(());
        }
        if !args.GetCurrentPoint(None::<&UIElement>)?.Properties()?.IsLeftButtonPressed()? {
            return Ok(());
        }
        self.drag.set(Some(Drag { from: self.point(args)?, offset: self.offsets()?, moved: false }));
        named::<IUIElement>(&self.root, "ViewArea")?.CapturePointer(&pointer)?;
        Ok(())
    }

    fn moved(&self, args: &PointerRoutedEventArgs) -> Result<()> {
        let Some(mut drag) = self.drag.get() else { return Ok(()) };
        let at = self.point(args)?;
        let (dx, dy) = (at.0 - drag.from.0, at.1 - drag.from.1);
        if dx.abs() + dy.abs() > DRAG_SLOP {
            drag.moved = true;
            self.drag.set(Some(drag));
        }
        if drag.moved {
            let (h, v) = ((drag.offset.0 - dx).max(0.0), (drag.offset.1 - dy).max(0.0));
            self.scroll()?.cast::<IScrollViewer2>()?.ChangeViewWithOptionalAnimation(Some(h), Some(v), None, true)?;
        }
        Ok(())
    }

    /// The end of a drag; a click beside the picture closes the viewer.
    fn released(&self, args: &PointerRoutedEventArgs) -> Result<()> {
        let Some(drag) = self.drag.take() else { return Ok(()) };
        let pointer = args.cast::<IPointerRoutedEventArgs>()?.Pointer()?;
        named::<IUIElement>(&self.root, "ViewArea")?.ReleasePointerCapture(&pointer)?;
        let source = args.cast::<IRoutedEventArgs>()?.OriginalSource()?;
        let on_picture = source.cast::<IImage>().is_ok();
        if !drag.moved && !on_picture {
            close();
        }
        Ok(())
    }
}

/// Downloads the picture at `url` into `file`; blocks, so never on the UI thread.
fn write_picture(url: &str, file: &StorageFile) -> Result<()> {
    let client = HttpClient::new()?;
    let bytes = client.cast::<IHttpClient>()?.GetBufferAsync(&Uri::CreateUri(url)?)?.join()?;
    FileIO::WriteBufferAsync(file, &bytes)?.cast::<windows_future::IAsyncAction>()?.join()
}

/// Saves the original of `url` in 图片 › Rubi.
fn save(url: &str) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (url, name, dispatcher) = (original(url).to_owned(), file_name(url), shell.dispatcher.clone());
    backend.spawn(async move {
        // Blocking WinRT calls, off the UI thread.
        let saved = (|| -> Result<String> {
            let pictures = KnownFolders::PicturesLibrary()?;
            let folder = pictures
                .cast::<IStorageFolder>()?
                .CreateFolderAsync("Rubi", CreationCollisionOption::OpenIfExists)?
                .join()?;
            let file = folder
                .cast::<IStorageFolder>()?
                .CreateFileAsync(&name, CreationCollisionOption::GenerateUniqueName)?
                .join()?;
            write_picture(&url, &file)?;
            file.cast::<IStorageItem>()?.Name()
        })();
        let said = match saved {
            Ok(name) => format!("已保存到 图片 › Rubi › {name}"),
            Err(problem) => format!("保存失败：{}", problem.message()),
        };
        post(&dispatcher, move || toast(&said));
    });
}

/// Asks where to save the original of `url`, and saves it there.
fn save_as(url: &str) {
    let Some(shell) = shell() else { return };
    let (url, name, dispatcher) = (original(url).to_owned(), file_name(url), shell.dispatcher.clone());
    let (stem, ext) = match name.rsplit_once('.') {
        Some((stem, ext)) => (stem.to_owned(), format!(".{ext}")),
        None => (name.clone(), ".jpg".to_owned()),
    };
    let asked = (|| -> Result<_> {
        let picker = FileSavePicker::new()?;
        let picking = picker.cast::<IFileSavePicker>()?;
        picking.SetSuggestedStartLocation(PickerLocationId::PicturesLibrary)?;
        picking.SetSuggestedFileName(&stem)?;
        let kinds = windows_collections::IVector::<HSTRING>::from(vec![HSTRING::from(ext.as_str())]);
        picking.FileTypeChoices()?.Insert(&HSTRING::from("图片"), &kinds.cast::<crate::xaml::IVector<HSTRING>>()?)?;
        picking.PickSaveFileAsync()
    })();
    let Ok(asked) = asked else {
        toast("另存为不可用");
        return;
    };
    // The answer and the download are waited for off the UI thread.
    std::thread::spawn(move || {
        let Ok(file) = asked.join() else { return };
        let said = match write_picture(&url, &file).and_then(|()| file.cast::<IStorageItem>()?.Name()) {
            Ok(name) => format!("已保存为 {name}"),
            Err(problem) => format!("保存失败：{}", problem.message()),
        };
        post(&dispatcher, move || toast(&said));
    });
}

/// Puts the picture at `url` on the clipboard, with its address.
fn copy_picture(url: &str) {
    let copied = (|| -> Result<()> {
        let uri = Uri::CreateUri(original(url))?;
        let package = DataPackage::new()?;
        let data = package.cast::<IDataPackage>()?;
        data.SetBitmap(&RandomAccessStreamReference::CreateFromUri(&uri)?)?;
        data.SetText(original(url))?;
        Clipboard::SetContent(&package)
    })();
    toast(if copied.is_ok() { "已复制图片" } else { "复制失败" });
}

/// Offers the picture at `url` to the system share sheet.
fn share(url: &str) {
    SHARING.with_borrow_mut(|s| *s = Some(original(url).to_owned()));
    if !SHARE_HOOKED.get() {
        let hooked = (|| -> Result<()> {
            let manager = DataTransferManager::GetForCurrentView()?;
            let revoker = manager.cast::<IDataTransferManager>()?.DataRequested(
                |_: Ref<DataTransferManager>, args: Ref<DataRequestedEventArgs>| {
                    let _ = args.ok().map(fill_share);
                },
            )?;
            keep(revoker);
            Ok(())
        })();
        if hooked.is_err() {
            toast("分享不可用");
            return;
        }
        SHARE_HOOKED.set(true);
    }
    if DataTransferManager::ShowShareUI().is_err() {
        toast("分享不可用");
    }
}

fn fill_share(args: &DataRequestedEventArgs) -> Result<()> {
    let Some(url) = SHARING.with_borrow(Clone::clone) else { return Ok(()) };
    let data = args.cast::<IDataRequestedEventArgs>()?.Request()?.cast::<IDataRequest>()?.Data()?;
    let package = data.cast::<IDataPackage>()?;
    package.Properties()?.cast::<IDataPackagePropertySet>()?.SetTitle("分享图片")?;
    let uri = Uri::CreateUri(&url)?;
    package.SetText(&url)?;
    data.cast::<IDataPackage2>()?.SetWebLink(&uri)?;
    package.SetBitmap(&RandomAccessStreamReference::CreateFromUri(&uri)?)
}
