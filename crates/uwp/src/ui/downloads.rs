//! 离线缓存: what has been copied to the device, with its progress and the ways to drive it.

use std::cell::RefCell;
use std::rc::Rc;

use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use rubi_core::ExportFormat;
use rubi_model::{Download, DownloadState};

use crate::ui::rows::Covers;
use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load, named, post, set_text};
use crate::xaml::{
    IButtonBase, IContentControl, IFrameworkElement, IImage, IPanel, IRangeBase, IUIElement, RoutedEventArgs, UIElement,
    Visibility,
};

const LAYOUT: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <Grid Margin="16,12,16,8">
    <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
    <TextBlock Text="离线缓存" FontSize="20" FontWeight="Light" VerticalAlignment="Center" />
    <TextBlock x:Name="Meta" Grid.Column="1" FontSize="12" Margin="12,0,0,0" VerticalAlignment="Center"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </Grid>
  <ScrollViewer Grid.Row="1">
    <StackPanel x:Name="Rows" Margin="16,0,16,16" />
  </ScrollViewer>
</Grid>"##;

const ROW: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Margin="0,0,0,10">
  <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" />
    <ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
  <Border Width="120" Height="68" Background="#1c1c1c"><Image x:Name="Cover" Stretch="UniformToFill" /></Border>
  <StackPanel Grid.Column="1" Margin="12,0,12,0" VerticalAlignment="Center">
    <TextBlock x:Name="Title" FontSize="14" TextWrapping="NoWrap" TextTrimming="CharacterEllipsis" />
    <TextBlock x:Name="Line" FontSize="11" Margin="0,5,0,0" TextTrimming="CharacterEllipsis"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    <ProgressBar x:Name="Bar" Height="3" Margin="0,6,0,0" Minimum="0" Maximum="1" Value="0" BorderThickness="0"
                 Foreground="#0050EF" Background="{ThemeResource SystemControlForegroundBaseLowBrush}" />
  </StackPanel>
  <StackPanel Grid.Column="2" Orientation="Horizontal" VerticalAlignment="Center">
    <Button x:Name="Go" Content="开始" Margin="0,0,8,0" Background="Transparent" BorderThickness="0"
            Foreground="#0050EF" FontSize="13" />
    <Button x:Name="OutVideo" Content="导出 MP4" Margin="0,0,8,0" Background="Transparent" BorderThickness="0"
            Foreground="#0050EF" FontSize="13" Visibility="Collapsed" />
    <Button x:Name="OutAudio" Content="导出音频" Margin="0,0,8,0" Background="Transparent" BorderThickness="0"
            Foreground="#0050EF" FontSize="13" Visibility="Collapsed" />
    <Button x:Name="Drop" Content="删除" Background="Transparent" BorderThickness="0" Foreground="#C04A4A"
            FontSize="13" />
  </StackPanel>
</Grid>"##;

thread_local! {
    static PAGE: RefCell<Option<IFrameworkElement>> = const { RefCell::new(None) };
    static ROWS: RefCell<Vec<Download>> = const { RefCell::new(Vec::new()) };
    /// Bumped for each page built: the one before stops refreshing itself.
    static TURN: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// The page of downloads.
pub fn build() -> Result<IFrameworkElement> {
    let root = load(LAYOUT)?;
    PAGE.with_borrow_mut(|slot| *slot = Some(root.clone()));
    let turn = TURN.get().wrapping_add(1);
    TURN.set(turn);
    reload()?;
    tick(turn);
    Ok(root)
}

/// Keeps the bars moving while this page is the one on show.
fn tick(turn: u64) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let dispatcher = shell.dispatcher.clone();
    backend.spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(700)).await;
            let (done, wait) = tokio::sync::oneshot::channel();
            post(&dispatcher, move || {
                let on_show = TURN.get() == turn;
                if on_show {
                    let _ = reload();
                }
                let _ = done.send(on_show);
            });
            if !wait.await.unwrap_or(false) {
                return;
            }
        }
    });
}

/// Reads the list again.
pub fn reload() -> Result<()> {
    if PAGE.with_borrow(Option::is_none) {
        return Ok(());
    }
    let Some(shell) = shell() else { return Ok(()) };
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let list = core.services.downloads.list().unwrap_or_default();
        post(&dispatcher, move || {
            let Some(root) = PAGE.with_borrow(Clone::clone) else { return };
            ROWS.with_borrow_mut(|rows| *rows = list.clone());
            let _ = paint(&root, &list);
            // Copies asked to be written out once they finished: now some of them have.
            for item in &list {
                if item.state == DownloadState::Done
                    && let Some(format) = crate::state::exports::take(&item.card.bvid)
                {
                    export_one(item.card.bvid.clone(), format);
                }
            }
        });
    });
    Ok(())
}

fn paint(root: &IFrameworkElement, list: &[Download]) -> Result<()> {
    let panel = named::<IPanel>(root, "Rows")?;
    let children = panel.Children()?.cast::<IVector<UIElement>>()?;
    children.Clear()?;
    let _ = set_text(root, "Meta", &format!("{} 个", list.len()));
    for (at, item) in list.iter().enumerate() {
        let row = load(ROW)?;
        let _ = set_text(&row, "Title", &item.card.title);
        let _ = set_text(&row, "Line", &line_of(item));
        if let Ok(image) = named::<IImage>(&row, "Cover") {
            let covers = Covers::default();
            let _ = covers.show(&image, &item.card.cover, 240);
        }
        if let Ok(button) = named::<IContentControl>(&row, "Go") {
            let label = match item.state {
                DownloadState::Running => "暂停",
                DownloadState::Done => "已完成",
                _ => "继续",
            };
            if let Ok(value) = crate::xaml::PropertyValue::CreateString(label) {
                let _ = button.SetContent(&value);
            }
        }
        if let Ok(button) = named::<IButtonBase>(&row, "Go") {
            keep(button.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| toggle(at))?);
        }
        if let Ok(button) = named::<IButtonBase>(&row, "Drop") {
            keep(button.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| drop_at(at))?);
        }
        for (name, format) in [("OutVideo", ExportFormat::Mp4), ("OutAudio", ExportFormat::Audio)] {
            if let Ok(button) = named::<IButtonBase>(&row, name) {
                keep(button.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                    export_at(at, format);
                })?);
            }
        }
        let done = item.state == DownloadState::Done;
        if let Ok(bar) = named::<IRangeBase>(&row, "Bar") {
            let share = if item.total_bytes > 0 {
                item.done_bytes as f64 / item.total_bytes as f64
            } else {
                0.0
            };
            let _ = bar.SetMaximum(1.0);
            let _ = bar.SetValue(share.clamp(0.0, 1.0));
        }
        let _ = named::<IUIElement>(&row, "Bar").and_then(|bar| {
            bar.SetVisibility(if done { Visibility::Collapsed } else { Visibility::Visible })
        });
        let _ = named::<IUIElement>(&row, "Go").and_then(|go| {
            go.SetVisibility(if done { Visibility::Collapsed } else { Visibility::Visible })
        });
        for name in ["OutVideo", "OutAudio"] {
            let _ = named::<IUIElement>(&row, name).and_then(|out| {
                out.SetVisibility(if done { Visibility::Visible } else { Visibility::Collapsed })
            });
        }
        children.Append(&row.cast::<UIElement>()?)?;
    }
    Ok(())
}

fn line_of(item: &Download) -> String {
    let state = match item.state {
        DownloadState::Running => "正在缓存",
        DownloadState::Paused => "已暂停",
        DownloadState::Done => "已完成",
        _ => "等待中",
    };
    if item.total_bytes > 0 {
        let share = item.done_bytes as f64 / item.total_bytes as f64 * 100.0;
        format!("{state} · {share:.0}% · {}", megabytes(item.done_bytes))
    } else {
        format!("{state} · {}", megabytes(item.done_bytes))
    }
}

fn megabytes(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_048_576.0)
}

/// Starts or pauses the download at `at`.
fn toggle(at: usize) {
    let Some(item) = ROWS.with_borrow(|rows| rows.get(at).cloned()) else { return };
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let outcome = match item.state {
            DownloadState::Running => core.services.downloads.pause(&item.card.bvid).await,
            _ => core.services.downloads.start(&item.card.bvid, item.quality).await,
        };
        let said = match outcome {
            Ok(()) => String::new(),
            Err(problem) => format!("缓存没动：{problem}"),
        };
        post(&dispatcher, move || {
            if !said.is_empty() {
                toast(&said);
            }
            let _ = reload();
        });
    });
}

/// Writes the copy at `at` out into 视频\Rubi or 音乐\Rubi.
fn export_at(at: usize, format: ExportFormat) {
    let Some(item) = ROWS.with_borrow(|rows| rows.get(at).cloned()) else { return };
    export_one(item.card.bvid, format);
}

/// Writes `bvid` out as `format`, and says where it went.
fn export_one(bvid: String, format: ExportFormat) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    // Where it goes is asked for here, on the UI thread: the libraries are handed out there,
    // and a worker gets 拒绝访问 for asking.
    let dir = match library_dir(format) {
        Ok(dir) => dir,
        Err(problem) => match fallback_dir() {
            Ok(dir) => {
                toast(&format!("库拿不到（{problem}），先放进应用目录"));
                dir
            }
            Err(_) => {
                toast(&format!("导出失败：{problem}"));
                return;
            }
        },
    };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let said = match write_out(&core, &bvid, &dir, format).await {
            Ok(landed) => format!("已导出到 {landed}"),
            Err(problem) => format!("导出失败：{problem}"),
        };
        post(&dispatcher, move || toast(&said));
    });
}

/// Copies `bvid` out of Rubi into `dir`, and says what it wrote.
async fn write_out(
    core: &crate::backend::Backend,
    bvid: &str,
    dir: &std::path::Path,
    format: ExportFormat,
) -> std::result::Result<String, String> {
    let written = core.services.downloads.export(bvid, dir, format).await.map_err(|e| e.to_string())?;
    Ok(written.first().map(|p| p.display().to_string()).unwrap_or_else(|| dir.display().to_string()))
}

/// The folder a copy is written into: 视频\Rubi or 音乐\Rubi.
fn library_dir(format: ExportFormat) -> std::result::Result<std::path::PathBuf, String> {
    let library = match format {
        ExportFormat::Audio => crate::xaml::KnownFolders::MusicLibrary(),
        _ => crate::xaml::KnownFolders::VideosLibrary(),
    }
    .map_err(|e| e.to_string())?;
    let rubi = library
        .CreateFolderAsync("Rubi", crate::xaml::CreationCollisionOption::OpenIfExists)
        .map_err(|e| e.to_string())?
        .join()
        .map_err(|e| e.to_string())?;
    folder_path(&rubi)
}

/// Where a copy goes when a library will not have it: the app's own folder, which the user can
/// be pointed at.
fn fallback_dir() -> std::result::Result<std::path::PathBuf, String> {
    let cache = crate::xaml::ApplicationData::Current()
        .map_err(|e| e.to_string())?
        .LocalFolder()
        .map_err(|e| e.to_string())?;
    let folder = cache
        .CreateFolderAsync("导出", crate::xaml::CreationCollisionOption::OpenIfExists)
        .map_err(|e| e.to_string())?
        .join()
        .map_err(|e| e.to_string())?;
    folder_path(&folder)
}

fn folder_path(folder: &crate::xaml::StorageFolder) -> std::result::Result<std::path::PathBuf, String> {
    let path = folder.cast::<crate::xaml::IStorageItem>().map_err(|e| e.to_string())?.Path().map_err(|e| e.to_string())?;
    Ok(std::path::PathBuf::from(path.to_string()))
}

fn drop_at(at: usize) {
    let Some(item) = ROWS.with_borrow(|rows| rows.get(at).cloned()) else { return };
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let outcome = core.services.downloads.remove(&item.card.bvid).await;
        post(&dispatcher, move || {
            if let Err(problem) = outcome {
                toast(&format!("删不掉：{problem}"));
            }
            let _ = reload();
        });
    });
}
