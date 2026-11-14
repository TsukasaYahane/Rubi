//! Watching the clipboard for a video address: Windows raises Clipboard.ContentChanged on every
//! change, so only the newest item is looked at, never the history.

use std::sync::Mutex;

use windows_core::{IInspectable, Interface, Ref};

use crate::ui::keep;
use crate::xaml::{Clipboard, IDataPackageView};

static LAST: Mutex<String> = Mutex::new(String::new());

thread_local! {
    /// The question on show: XAML cannot cross threads, so the worker finds it here.
    static ASKING: std::cell::RefCell<Option<crate::xaml::IFrameworkElement>> = const { std::cell::RefCell::new(None) };
}

const ASK: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="420" Padding="18"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" /></Grid.RowDefinitions>
    <TextBlock Text="剪贴板里有一个视频" FontSize="18" FontWeight="Light" />
    <StackPanel Grid.Row="1" Margin="0,10,0,0" Orientation="Horizontal">
      <Border Width="120" Height="68" Background="#1c1c1c"><Image x:Name="Cover" Stretch="UniformToFill" /></Border>
      <StackPanel Margin="12,0,0,0" VerticalAlignment="Center" Width="240">
        <TextBlock x:Name="Title" FontSize="14" TextWrapping="Wrap" MaxLines="2" TextTrimming="CharacterEllipsis" />
        <TextBlock x:Name="What" FontSize="11" Margin="0,4,0,0" TextTrimming="CharacterEllipsis"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      </StackPanel>
    </StackPanel>
    <StackPanel Grid.Row="2" Orientation="Horizontal" HorizontalAlignment="Right" Margin="0,14,0,0">
      <Button x:Name="Ignore" Content="忽略" Margin="0,0,8,0" Background="Transparent" BorderThickness="0" />
      <Button x:Name="Open" Content="打开" Background="#0050EF" Foreground="White" BorderThickness="0" />
    </StackPanel>
  </Grid>
</Grid>"##;

/// Remembers what the clipboard holds now, without asking: starting up says nothing.
pub fn prime() {
    if let Some(text) = current_text() {
        *LAST.lock().unwrap_or_else(|e| e.into_inner()) = text;
    }
}

/// Looks at the clipboard's newest item; a video address there asks before opening it.
pub fn check() {
    let Some(text) = current_text() else { return };
    if text.trim().is_empty() {
        return;
    }
    {
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        if *last == text {
            return;
        }
        *last = text.clone();
    }
    if let Some(bvid) = crate::ui::search::video_in(&text) {
        let _ = ask(&bvid, &text);
    }
}

fn ask(bvid: &str, text: &str) -> windows_core::Result<()> {
    let root = crate::ui::load(ASK)?;
    let shown: String = text.chars().take(40).collect();
    let _ = crate::ui::set_text(&root, "What", &shown);
    let _ = crate::ui::set_text(&root, "Title", "正在读取视频信息…");
    ASKING.with_borrow_mut(|slot| *slot = Some(root.clone()));
    describe(bvid);
    let wanted = bvid.to_owned();
    crate::ui::on_click(&root, "Open", move || {
        let _ = crate::ui::popup::hide();
        let _ = crate::ui::search::open_video(&wanted);
    })?;
    crate::ui::on_click(&root, "Ignore", || {
        let _ = crate::ui::popup::hide();
    })?;
    crate::ui::popup::show(&root)
}

/// Fills the question in with the video's own cover and title.
fn describe(bvid: &str) {
    let Some(shell) = crate::ui::shell::shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (bvid, dispatcher, core) = (bvid.to_owned(), shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let found = core.services.videos.detail(&bvid).await.map_err(|e| e.to_string()).map(|info| info.card);
        let _ = crate::ui::post(&dispatcher, move || {
            let Ok(card) = found else { return };
            let Some(root) = ASKING.with_borrow(Clone::clone) else { return };
            let _ = crate::ui::set_text(&root, "Title", &card.title);
            let _ = crate::ui::set_text(&root, "What", &format!("{} · {} 播放", card.owner.name, card.stats.view));
            if let Ok(image) = crate::ui::named::<crate::xaml::IImage>(&root, "Cover") {
                let covers = crate::ui::rows::Covers::default();
                let _ = covers.show(&image, &card.cover, 240);
            }
        });
    });
}

/// The clipboard's text, if it has any.
fn current_text() -> Option<String> {
    let view = Clipboard::GetContent().ok()?;
    let view = view.cast::<IDataPackageView>().ok()?;
    let operation = view.GetTextAsync().ok()?;
    for _ in 0..100 {
        if let Ok(text) = operation.GetResults() {
            return Some(text.to_string_lossy());
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    None
}
