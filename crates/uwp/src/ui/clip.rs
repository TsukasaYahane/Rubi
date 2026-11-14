//! Watching the clipboard for a video address: Windows raises Clipboard.ContentChanged on every
//! change, so only the newest item is looked at, never the history.

use std::sync::Mutex;

use windows_core::{IInspectable, Interface, Ref};

use crate::ui::keep;
use crate::xaml::{Clipboard, IDataPackageView};

static LAST: Mutex<String> = Mutex::new(String::new());

const ASK: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="420" Padding="18"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" /></Grid.RowDefinitions>
    <TextBlock Text="剪贴板里有一个视频" FontSize="18" FontWeight="Light" />
    <TextBlock x:Name="What" Grid.Row="1" Margin="0,8,0,0" FontSize="13" TextWrapping="Wrap"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
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
    let shown: String = text.chars().take(80).collect();
    let _ = crate::ui::set_text(&root, "What", &shown);
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
