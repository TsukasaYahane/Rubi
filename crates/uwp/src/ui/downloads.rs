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
    /// The rows on show, with the video each stands for.
    static SHOWN: RefCell<Vec<Shown>> = const { RefCell::new(Vec::new()) };
    /// Bumped for each page built: the one before stops refreshing itself.
    static TURN: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// Where the copy being written out goes: a folder cannot cross to a worker, so it waits
    /// here for the thread that asked for it.
    static TARGET: RefCell<Option<crate::xaml::StorageFolder>> = const { RefCell::new(None) };
    /// Copies already moved out of the app's folder, so each is moved once a run.
    static MOVED: RefCell<std::collections::HashSet<String>> = RefCell::default();
}

/// A row on show, kept so it can be written to again instead of being built anew.
struct Shown {
    bvid: String,
    row: IFrameworkElement,
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
            // Finished copies go to the folder picked in 设置, and the app's own copy with them:
            // the core can write here, and a folder it was handed can write there.
            if let Some(cache) = crate::state::exports::download_folder() {
                for item in &list {
                    let bvid = &item.card.bvid;
                    if item.state != DownloadState::Done
                        || MOVED.with_borrow(|moved| moved.contains(bvid))
                    {
                        continue;
                    }
                    let _ = stash(bvid, &cache);
                    MOVED.with_borrow_mut(|moved| {
                        moved.insert(bvid.clone());
                    });
                }
            }
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
    // The same list as last time: only what moves is written again. Building a row afresh
    // would load its cover again, which is what made the page jump every half second.
    let same = SHOWN.with_borrow(|shown| {
        shown.len() == list.len() && shown.iter().zip(list).all(|(had, item)| had.bvid == item.card.bvid)
    });
    if same {
        SHOWN.with_borrow(|shown| {
            for (had, item) in shown.iter().zip(list) {
                let _ = fill_row(&had.row, item);
            }
        });
        return Ok(());
    }
    let panel = named::<IPanel>(root, "Rows")?;
    let children = panel.Children()?.cast::<IVector<UIElement>>()?;
    children.Clear()?;
    let _ = set_text(root, "Meta", &format!("{} 个", list.len()));
    let mut built = Vec::with_capacity(list.len());
    for (at, item) in list.iter().enumerate() {
        let row = load(ROW)?;
        fill_row(&row, item)?;
        if let Ok(image) = named::<IImage>(&row, "Cover") {
            let covers = Covers::default();
            let _ = covers.show(&image, &item.card.cover, 720);
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
        children.Append(&row.cast::<UIElement>()?)?;
        built.push(Shown { bvid: item.card.bvid.clone(), row });
    }
    SHOWN.with_borrow_mut(|shown| *shown = built);
    Ok(())
}

/// Writes what a row shows: its words, its bar, and which buttons it offers.
fn fill_row(row: &IFrameworkElement, item: &Download) -> Result<()> {
    let _ = set_text(row, "Title", &item.card.title);
    let _ = set_text(row, "Line", &line_of(item));
    if let Ok(button) = named::<IContentControl>(row, "Go") {
        let label = match item.state {
            DownloadState::Running => "暂停",
            DownloadState::Done => "已完成",
            _ => "继续",
        };
        if let Ok(value) = crate::xaml::PropertyValue::CreateString(label) {
            let _ = button.SetContent(&value);
        }
    }
    let done = item.state == DownloadState::Done;
    if let Ok(bar) = named::<IRangeBase>(row, "Bar") {
        let share = if item.total_bytes > 0 { item.done_bytes as f64 / item.total_bytes as f64 } else { 0.0 };
        let _ = bar.SetMaximum(1.0);
        let _ = bar.SetValue(share.clamp(0.0, 1.0));
    }
    let _ = named::<IUIElement>(row, "Bar").and_then(|bar| {
        bar.SetVisibility(if done { Visibility::Collapsed } else { Visibility::Visible })
    });
    let _ = named::<IUIElement>(row, "Go").and_then(|go| {
        go.SetVisibility(if done { Visibility::Collapsed } else { Visibility::Visible })
    });
    for name in ["OutVideo", "OutAudio"] {
        let _ = named::<IUIElement>(row, name).and_then(|out| {
            out.SetVisibility(if done { Visibility::Visible } else { Visibility::Collapsed })
        });
    }
    Ok(())
}

fn line_of(item: &Download) -> String {
    let state = match &item.state {
        DownloadState::Running => "正在缓存",
        DownloadState::Paused => "已暂停",
        DownloadState::Done => "已完成",
        // Said as it came, or a failure reads as "waiting" and the list looks like it lost one.
        DownloadState::Failed(why) => {
            // The one failure whose fix is not in this app.
            let hint = if why.contains("os error 5") {
                "（到「设置 → 隐私和安全性 → 文件系统」允许 Rubi）"
            } else {
                ""
            };
            return format!("失败：{why}{hint}");
        }
        DownloadState::Queued => "等待中",
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
            _ => core.services.downloads.start(&item.card.bvid, item.quality, rubi_core::Streams::Both).await,
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

/// Moves a finished copy out of the app's folder into `cache`, under its own bvid: the app may
/// write here, and a folder it was handed may write there, but never the other way round.
fn stash(bvid: &str, cache: &crate::xaml::StorageFolder) -> std::result::Result<(), String> {
    let Some(shell) = shell() else { return Ok(()) };
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    let Some(copy) = backend.services.downloads.local_here(bvid).map_err(|e| e.to_string())? else {
        // Moved already: the copy here is gone.
        return Ok(());
    };
    let home = copy.video.as_ref().or(copy.audio.as_ref()).and_then(|p| p.parent()).map(|p| p.to_path_buf());
    let folder = cache
        .CreateFolderAsync(bvid, crate::xaml::CreationCollisionOption::OpenIfExists)
        .map_err(|e| e.to_string())?
        .join()
        .map_err(|e| e.to_string())?;
    // Both files go over before either is dropped here: a copy half carried off would leave a
    // record whose streams disagree.
    for path in [copy.video.as_ref(), copy.audio.as_ref()].into_iter().flatten() {
        copy_into(path, &folder)?;
    }
    // The app's own copy stays. A folder Windows handed us can be written to, but the core can
    // only read what sits in the app's own folder — so the copy here is what everything else
    // keeps working off, and the folder is where the files are kept for the user to reach.
    // The core looks for the copy where it was carried to; say so now, so that an export this
    // run finds it even when the folder was picked after the core was opened.
    backend.services.downloads.set_mirror(
        folder.cast::<crate::xaml::IStorageItem>().ok().and_then(|item| item.Path().ok()).map(|path| std::path::PathBuf::from(path.to_string())),
    );
    if let Some(home) = home {
        let _ = std::fs::remove_dir(home);
    }
    Ok(())
}

/// Writes the copy at `at` out into 视频\Rubi or 音乐\Rubi.
fn export_at(at: usize, format: ExportFormat) {
    let Some(item) = ROWS.with_borrow(|rows| rows.get(at).cloned()) else { return };
    export_one(item.card.bvid, format);
}

/// Writes `bvid` out as `format`, and says where it went. The core stages the bytes inside the
/// app's own folder; a `StorageFolder` takes them from there, since a packaged app cannot write
/// to a path of its own choosing.
fn export_one(bvid: String, format: ExportFormat) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    // Both are asked for on the UI thread: the folders are handed out there, and a worker gets
    // 拒绝访问 for asking. The folder is kept aside because it cannot be sent to the worker.
    let staged = match staging_dir() {
        Ok(dir) => dir,
        Err(problem) => {
            toast(&format!("导出失败：{problem}"));
            return;
        }
    };
    TARGET.with_borrow_mut(|slot| *slot = target_folder(format));
    // A copy lives in one of two places: the app's own folder, while it is being written, and
    // the folder picked in 设置, once it finishes. Both are looked in here, so that an export
    // does not depend on the core having been told where the second one is.
    let carried = found_copy(&bvid);
    crate::ui::trace(&format!("export {bvid} from {carried:?}"));
    let looked = format!(
        "{} / {}",
        own_copy_dir(&bvid).map(|dir| dir.display().to_string()).unwrap_or_else(|| "—".to_owned()),
        crate::state::exports::download_path().unwrap_or_else(|| "未设置".to_owned()),
    );
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let written = match &carried {
            Some(dir) => core.services.downloads.export_from(&bvid, dir, &staged, format).await,
            None => Err(rubi_core::CoreError::Invalid(format!("找不到缓存的文件，查过：{looked}"))),
        }
        .map_err(|e| e.to_string());
        let files = match written {
            Ok(files) => files,
            Err(problem) => {
                post(&dispatcher, move || toast(&format!("导出失败：{problem}")));
                return;
            }
        };
        post(&dispatcher, move || {
            let target = TARGET.with_borrow(Clone::clone);
            let said = match target {
                Some(target) => deliver(&files, &target),
                None => {
                    let dir =
                        files.first().and_then(|p| p.parent()).map(|p| p.display().to_string()).unwrap_or_default();
                    format!("已放进应用目录：{dir}")
                }
            };
            toast(&said);
        });
    });
}

/// Where a copy's streams are: the app's own folder first, then the one picked in 设置.
fn found_copy(bvid: &str) -> Option<std::path::PathBuf> {
    let streams = |dir: &std::path::Path| dir.join("video.m4s").exists() || dir.join("audio.m4s").exists();
    let here = own_copy_dir(bvid).filter(|dir| streams(dir));
    if here.is_some() {
        return here;
    }
    crate::state::exports::download_folder()
        .and_then(|cache| {
            cache.CreateFolderAsync(bvid, crate::xaml::CreationCollisionOption::OpenIfExists).ok()?.join().ok()
        })
        .and_then(|folder| {
            folder.cast::<crate::xaml::IStorageItem>().ok()?.Path().ok().map(|path| std::path::PathBuf::from(path.to_string()))
        })
        .filter(|dir| streams(dir))
}

/// `downloads/<bvid>` inside the app's own folder, where the core writes first.
fn own_copy_dir(bvid: &str) -> Option<std::path::PathBuf> {
    let data = crate::xaml::ApplicationData::Current().ok()?;
    let folder = data.LocalFolder().ok()?;
    let downloads = folder
        .CreateFolderAsync("downloads", crate::xaml::CreationCollisionOption::OpenIfExists)
        .ok()?
        .join()
        .ok()?;
    let path = downloads.cast::<crate::xaml::IStorageItem>().ok()?.Path().ok()?;
    Some(std::path::PathBuf::from(path.to_string()).join(bvid))
}

/// Hands the staged files to `target`, and says where that was.
fn deliver(files: &[std::path::PathBuf], target: &crate::xaml::StorageFolder) -> String {
    for path in files {
        if let Err(problem) = copy_into(path, target) {
            return format!("导出失败：{problem}");
        }
        let _ = std::fs::remove_file(path);
    }
    let landed = target
        .cast::<crate::xaml::IStorageItem>()
        .and_then(|item| item.Path())
        .map(|path| path.to_string())
        .unwrap_or_default();
    format!("已导出到 {landed}")
}

/// The folder the files belong in: the one picked in 设置, else 视频 or 音频 beside the offline
/// copies, else the same under the library.
fn target_folder(format: ExportFormat) -> Option<crate::xaml::StorageFolder> {
    // Sound goes under `audio`, pictures under `Video`; each is made where it is not there yet.
    let name = match format {
        ExportFormat::Audio => "audio",
        _ => "Video",
    };
    let parent = crate::state::exports::folder()
        .or_else(crate::state::exports::download_folder)
        .or_else(|| {
            match format {
                ExportFormat::Audio => crate::xaml::KnownFolders::MusicLibrary(),
                _ => crate::xaml::KnownFolders::VideosLibrary(),
            }
            .ok()
        })?;
    parent
        .CreateFolderAsync(name, crate::xaml::CreationCollisionOption::OpenIfExists)
        .ok()?
        .join()
        .ok()
}

/// Copies a staged file into `target`, a megabyte at a time: there can be hundreds of them.
fn copy_into(path: &std::path::Path, target: &crate::xaml::StorageFolder) -> std::result::Result<(), String> {
    let name =
        path.file_name().map(|name| name.to_string_lossy().to_string()).ok_or_else(|| "没有文件名".to_owned())?;
    let file = target
        .CreateFileAsync(&name, crate::xaml::CreationCollisionOption::ReplaceExisting)
        .map_err(|e| e.to_string())?
        .join()
        .map_err(|e| e.to_string())?;
    let stream = file
        .OpenAsync(crate::xaml::FileAccessMode::ReadWrite)
        .map_err(|e| e.to_string())?
        .join()
        .map_err(|e| e.to_string())?;
    let out = stream.cast::<crate::xaml::IOutputStream>().map_err(|e| e.to_string())?;
    let writer = crate::xaml::DataWriter::CreateDataWriter(&out)
        .map_err(|e| e.to_string())?
        .cast::<crate::xaml::IDataWriter>()
        .map_err(|e| e.to_string())?;
    let mut source = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = vec![0_u8; 1 << 20];
    loop {
        let read = std::io::Read::read(&mut source, &mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        writer.WriteBytes(&buffer[..read]).map_err(|e| e.to_string())?;
        writer.StoreAsync().map_err(|e| e.to_string())?.join().map_err(|e| e.to_string())?;
    }
    writer.DetachStream().map_err(|e| e.to_string())?;
    Ok(())
}

/// Where the core writes first: the app's own folder, which it may always write to.
fn staging_dir() -> std::result::Result<std::path::PathBuf, String> {
    let data = crate::xaml::ApplicationData::Current()
        .map_err(|e| e.to_string())?
        .LocalFolder()
        .map_err(|e| e.to_string())?;
    let folder = data
        .CreateFolderAsync("导出", crate::xaml::CreationCollisionOption::OpenIfExists)
        .map_err(|e| e.to_string())?
        .join()
        .map_err(|e| e.to_string())?;
    let path = folder
        .cast::<crate::xaml::IStorageItem>()
        .map_err(|e| e.to_string())?
        .Path()
        .map_err(|e| e.to_string())?;
    Ok(std::path::PathBuf::from(path.to_string()))
}

fn drop_at(at: usize) {
    let Some(item) = ROWS.with_borrow(|rows| rows.get(at).cloned()) else { return };
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    // The copy itself may have been carried to the folder picked in 设置: it goes with the record.
    if let Some(folder) = crate::state::exports::download_folder().and_then(|cache| {
        cache
            .CreateFolderAsync(&item.card.bvid, crate::xaml::CreationCollisionOption::OpenIfExists)
            .ok()?
            .join()
            .ok()
    }) {
        if let Ok(item) = folder.cast::<crate::xaml::IStorageItem>() {
            let _ = item
                .DeleteAsync(crate::xaml::StorageDeleteOption::PermanentDelete)
                .and_then(|action| action.cast::<windows_future::IAsyncAction>())
                .and_then(|action| action.join());
        }
    }
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
