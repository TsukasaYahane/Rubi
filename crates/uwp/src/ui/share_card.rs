//! 分享卡片: a video as a picture to pass on. Its cover, title, UP and plays, and a QR code that
//! opens it on B站, on white whatever the theme. The card is drawn over the window; 保存 puts it in
//! 图片 › Rubi as a PNG, 分享图片 hands it to the share sheet.

use std::cell::RefCell;

use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::images::sized;
use crate::state::qr;
use crate::state::settings::escape;
use crate::ui::sharing::{Shared, share};
use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load, named, post};
use crate::xaml::{
    BitmapAlphaMode, BitmapEncoder, BitmapPixelFormat, CreationCollisionOption, DataReader, FileAccessMode,
    IBitmapEncoder, IButtonBase, IDataReader, IRenderTargetBitmap, IStorageFile, IStorageFolder, IStorageItem,
    IUIElement, KnownFolders, RenderTargetBitmap, RoutedEventArgs, StorageFile, UIElement, Visibility,
};

/// What the card shows.
pub struct Card {
    pub bvid: String,
    pub title: String,
    pub uploader: String,
    pub plays: String,
    pub cover: String,
}

thread_local! {
    /// The card being drawn into pixels; on the UI thread.
    static DRAWING: RefCell<Option<RenderTargetBitmap>> = const { RefCell::new(None) };
}

/// Where the card goes once drawn.
#[derive(Clone, Copy, PartialEq)]
enum Goes {
    Saved,
    Shared,
}

fn link(bvid: &str) -> String {
    format!("https://www.bilibili.com/video/{bvid}")
}

/// Shows the card for `card` over the window.
pub fn open(card: Card) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let url = link(&card.bvid);
    let Some((side, data)) = qr::path_data(&url) else {
        toast("二维码生成失败");
        return Ok(());
    };
    let cover = sized(&card.cover, 720, None).unwrap_or_else(|| card.cover.clone());
    let layer = load(&format!(
        r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
              xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
          <StackPanel HorizontalAlignment="Center" VerticalAlignment="Center" Background="{{ThemeResource ApplicationPageBackgroundThemeBrush}}"
                      Padding="16" Width="392">
            <Border x:Name="ShareCard" Background="White" BorderBrush="#DDDDDD" BorderThickness="1" RequestedTheme="Light">
              <StackPanel>
                <Image Source="{}" Stretch="UniformToFill" Height="202" />
                <Grid Padding="12,10">
                  <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="84" /></Grid.ColumnDefinitions>
                  <StackPanel VerticalAlignment="Center" Margin="0,0,10,0">
                    <TextBlock Text="{}" FontSize="15" Foreground="#111111" TextWrapping="Wrap" MaxLines="3" />
                    <TextBlock Text="{} · {} 播放" FontSize="11" Foreground="#666666" Margin="0,6,0,0" />
                  </StackPanel>
                  <StackPanel Grid.Column="1">
                    <Viewbox Width="84" Height="84"><Canvas Width="{side}" Height="{side}"><Path Fill="Black" Data="{data}" /></Canvas></Viewbox>
                    <TextBlock Text="扫码在 B 站观看" FontSize="10" Foreground="#666666" HorizontalAlignment="Center" Margin="0,2,0,0" />
                  </StackPanel>
                </Grid>
                <TextBlock Text="Rubi" FontSize="11" Foreground="#0050EF" Margin="12,0,0,10" />
              </StackPanel>
            </Border>
            <StackPanel Orientation="Horizontal" HorizontalAlignment="Right" Margin="0,12,0,0">
              <Button x:Name="CardClose" Content="关闭" Margin="0,0,8,0" />
              <Button x:Name="CardShare" Content="分享图片" Margin="0,0,8,0" />
              <Button x:Name="CardSave" Content="保存到 图片 › Rubi" Background="#0050EF" Foreground="White" />
            </StackPanel>
          </StackPanel>
        </Grid>"##,
        escape(&cover),
        escape(&card.title),
        escape(&card.uploader),
        escape(&card.plays),
    ))?;
    let host = shell.viewer_host()?;
    let children = host.Children()?.cast::<IVector<UIElement>>()?;
    children.Clear()?;
    children.Append(&layer.cast::<UIElement>()?)?;
    host.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)?;
    let name = format!("rubi-{}.png", card.bvid);
    let title = card.title.clone();
    let wire = |button: &str, act: Box<dyn Fn()>| -> Result<()> {
        keep(named::<IButtonBase>(&layer, button)?.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| act())?);
        Ok(())
    };
    wire("CardClose", Box::new(close))?;
    let (saving, sharing) = ((layer.clone(), name.clone(), title.clone()), (layer.clone(), name, title));
    wire("CardSave", Box::new(move || draw(&saving.0, saving.1.clone(), saving.2.clone(), Goes::Saved)))?;
    wire("CardShare", Box::new(move || draw(&sharing.0, sharing.1.clone(), sharing.2.clone(), Goes::Shared)))?;
    Ok(())
}

/// Takes the card off the window.
pub fn close() {
    if let Some(shell) = shell()
        && let Ok(host) = shell.viewer_host()
    {
        let _ = host.Children().and_then(|c| c.cast::<IVector<UIElement>>()).and_then(|c| c.Clear());
        let _ = host.cast::<IUIElement>().and_then(|h| h.SetVisibility(Visibility::Collapsed));
    }
}

/// Draws the card into pixels: the drawing is asked for here, waited for off the UI thread.
fn draw(layer: &crate::xaml::IFrameworkElement, name: String, title: String, goes: Goes) {
    let Some(dispatcher) = shell().map(|s| s.dispatcher.clone()) else { return };
    let asked = (|| -> Result<windows_future::IAsyncAction> {
        let card = named::<UIElement>(layer, "ShareCard")?;
        let bitmap = RenderTargetBitmap::new()?;
        let drawing =
            bitmap.cast::<IRenderTargetBitmap>()?.RenderAsync(&card)?.cast::<windows_future::IAsyncAction>()?;
        DRAWING.with_borrow_mut(|d| *d = Some(bitmap));
        Ok(drawing)
    })();
    let Ok(drawing) = asked else {
        toast("卡片生成失败");
        return;
    };
    toast("正在生成…");
    std::thread::spawn(move || {
        let drawn = drawing.join();
        post(&dispatcher, move || {
            if drawn.is_err() {
                toast("卡片生成失败");
                return;
            }
            pixels(name, title, goes);
        });
    });
}

/// The drawn card's pixels, written to a PNG off the UI thread.
fn pixels(name: String, title: String, goes: Goes) {
    let Some(dispatcher) = shell().map(|s| s.dispatcher.clone()) else { return };
    let Some(bitmap) = DRAWING.with_borrow_mut(Option::take) else { return };
    let asked = (|| -> Result<_> {
        let drawn = bitmap.cast::<IRenderTargetBitmap>()?;
        Ok((drawn.GetPixelsAsync()?, drawn.PixelWidth()? as u32, drawn.PixelHeight()? as u32))
    })();
    let Ok((pixels, width, height)) = asked else {
        toast("卡片生成失败");
        return;
    };
    std::thread::spawn(move || {
        let written = write_png(&pixels, width, height, &name, goes);
        let path = written.as_ref().ok().and_then(|f| f.cast::<IStorageItem>().ok()?.Name().ok());
        // A file goes back to the UI thread through an agile reference.
        let file = written.and_then(|f| windows_core::AgileReference::new(&f)).ok();
        post(&dispatcher, move || {
            let Some(file) = file.and_then(|f| f.resolve().ok()) else {
                toast("卡片生成失败");
                return;
            };
            match goes {
                Goes::Saved => toast(&format!("分享卡片已保存到 图片 › Rubi › {}", path.unwrap_or_default())),
                Goes::Shared => share(Shared::Image { title, file }),
            }
        });
    });
}

/// Encodes the pixels as a PNG: saved in 图片 › Rubi, or in the app's temporary folder to share.
fn write_png(
    pixels: &windows_future::IAsyncOperation<crate::xaml::IBuffer>,
    width: u32,
    height: u32,
    name: &str,
    goes: Goes,
) -> Result<StorageFile> {
    let buffer = pixels.join()?;
    let mut bytes = vec![0u8; buffer.Length()? as usize];
    DataReader::FromBuffer(&buffer)?.cast::<IDataReader>()?.ReadBytes(&mut bytes)?;
    let folder = match goes {
        Goes::Saved => KnownFolders::PicturesLibrary()?
            .cast::<IStorageFolder>()?
            .CreateFolderAsync("Rubi", CreationCollisionOption::OpenIfExists)?
            .join()?,
        Goes::Shared => {
            crate::xaml::ApplicationData::Current()?.cast::<crate::xaml::IApplicationData>()?.TemporaryFolder()?
        }
    };
    let collide = if goes == Goes::Saved {
        CreationCollisionOption::GenerateUniqueName
    } else {
        CreationCollisionOption::ReplaceExisting
    };
    let file = folder.cast::<IStorageFolder>()?.CreateFileAsync(name, collide)?.join()?;
    let stream = file.cast::<IStorageFile>()?.OpenAsync(FileAccessMode::ReadWrite)?.join()?;
    let encoder = BitmapEncoder::CreateAsync(BitmapEncoder::PngEncoderId()?, &stream)?.join()?;
    let encoding = encoder.cast::<IBitmapEncoder>()?;
    encoding.SetPixelData(
        BitmapPixelFormat::Bgra8,
        BitmapAlphaMode::Premultiplied,
        width,
        height,
        96.0,
        96.0,
        &bytes,
    )?;
    encoding.FlushAsync()?.cast::<windows_future::IAsyncAction>()?.join()?;
    Ok(file)
}
