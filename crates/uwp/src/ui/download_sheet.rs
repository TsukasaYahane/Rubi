//! 缓存格式: how a copy is kept once it finishes reading — inside Rubi, as an MP4, or as the
//! sound alone — and at what quality it is fetched.

use std::cell::RefCell;

use windows_core::{IInspectable, Interface, Ref, Result};

use rubi_core::ExportFormat;

use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load, named, popup};
use crate::xaml::{IButtonBase, IUIElement, RoutedEventArgs};

/// Quality picked: audio needs none, and the sound is the same at any.
const QUALITIES: [(u32, &str); 3] = [(80, "1080P"), (64, "720P"), (32, "480P")];

const SHEET: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="440" Padding="20"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <StackPanel>
      <TextBlock Text="缓存格式" FontSize="18" FontWeight="Light" />
      <StackPanel Orientation="Horizontal" Margin="0,12,0,0">
        <Button x:Name="FmtRubi" Content="仅 Rubi" Margin="0,0,6,0" />
        <Button x:Name="FmtMp4" Content="MP4 视频" Margin="0,0,6,0" />
        <Button x:Name="FmtM4a" Content="音频 M4A" />
      </StackPanel>
      <TextBlock x:Name="Note" FontSize="12" Margin="0,10,0,0" TextWrapping="Wrap"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      <TextBlock x:Name="QualityHead" Text="缓存清晰度" FontSize="16" Margin="0,18,0,0" />
      <StackPanel Orientation="Horizontal" Margin="0,8,0,0">
        <Button x:Name="Q1080" Content="1080P" Margin="0,0,6,0" />
        <Button x:Name="Q720" Content="720P" Margin="0,0,6,0" />
        <Button x:Name="Q480" Content="480P" />
      </StackPanel>
      <StackPanel Orientation="Horizontal" HorizontalAlignment="Right" Margin="0,20,0,0">
        <Button x:Name="Cancel" Content="取消" Margin="0,0,8,0" />
        <Button x:Name="Go" Content="开始缓存" Background="#0050EF" Foreground="White" BorderThickness="0" />
      </StackPanel>
    </StackPanel>
  </Grid>
</Grid>"##;

thread_local! {
    /// What the sheet on show is set to.
    static CHOICE: RefCell<(ExportFormat, u32)> = const { RefCell::new((ExportFormat::Rubi, 80)) };
}

/// Opens the sheet for `bvid`.
pub fn open(bvid: &str) -> Result<()> {
    let root = load(SHEET)?;
    CHOICE.with_borrow_mut(|slot| *slot = (ExportFormat::Rubi, 80));
    for (name, format) in
        [("FmtRubi", ExportFormat::Rubi), ("FmtMp4", ExportFormat::Mp4), ("FmtM4a", ExportFormat::Audio)]
    {
        let picked = root.clone();
        keep(named::<IButtonBase>(&root, name)?.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
            CHOICE.with_borrow_mut(|slot| slot.0 = format);
            let _ = paint(&picked);
        })?);
    }
    for (at, (qn, _)) in QUALITIES.iter().enumerate() {
        let name = ["Q1080", "Q720", "Q480"][at];
        let picked = root.clone();
        let qn = *qn;
        keep(named::<IButtonBase>(&root, name)?.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
            CHOICE.with_borrow_mut(|slot| slot.1 = qn);
            let _ = paint(&picked);
        })?);
    }
    keep(named::<IButtonBase>(&root, "Cancel")?.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
        let _ = popup::hide();
    })?);
    let bvid = bvid.to_owned();
    keep(named::<IButtonBase>(&root, "Go")?.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
        let _ = popup::hide();
        start(&bvid);
    })?);
    paint(&root)?;
    popup::show(&root)
}

/// Shows what is picked, and hides the qualities when only the sound is wanted.
fn paint(root: &crate::xaml::IFrameworkElement) -> Result<()> {
    let (format, qn) = CHOICE.with_borrow(|slot| *slot);
    for (name, this) in
        [("FmtRubi", ExportFormat::Rubi), ("FmtMp4", ExportFormat::Mp4), ("FmtM4a", ExportFormat::Audio)]
    {
        mark(root, name, this == format)?;
    }
    for at in 0..QUALITIES.len() {
        mark(root, ["Q1080", "Q720", "Q480"][at], QUALITIES[at].0 == qn)?;
    }
    let audio = format == ExportFormat::Audio;
    let shown = if audio { crate::xaml::Visibility::Collapsed } else { crate::xaml::Visibility::Visible };
    for name in ["QualityHead", "Q1080", "Q720", "Q480"] {
        named::<IUIElement>(root, name)?.SetVisibility(shown)?;
    }
    let note = match format {
        ExportFormat::Rubi => "在 Rubi 的离线缓存里播放",
        ExportFormat::Mp4 => "缓存完成后另存一份到「视频」文件夹",
        ExportFormat::Audio => "缓存完成后另存音频（M4A）到「音乐」文件夹",
    };
    crate::ui::set_text(root, "Note", note)
}

fn mark(root: &crate::xaml::IFrameworkElement, name: &str, on: bool) -> Result<()> {
    let color = if on {
        crate::xaml::Color { a: 255, r: 0x00, g: 0x50, b: 0xEF }
    } else {
        crate::xaml::Color { a: 255, r: 0x76, g: 0x76, b: 0x76 }
    };
    let brush = crate::xaml::SolidColorBrush::CreateInstanceWithColor(color)?;
    named::<crate::xaml::IControl>(root, name)?.SetForeground(&brush)
}

/// Starts the copy, and keeps what to write out once it is done.
fn start(bvid: &str) {
    let (format, qn) = CHOICE.with_borrow(|slot| *slot);
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    let bvid = bvid.to_owned();
    backend.spawn(async move {
        let outcome = core.services.downloads.start(&bvid, qn).await;
        let said = match outcome {
            Ok(()) => format!("已加入缓存 · {}", format.label()),
            Err(problem) => format!("缓存没动：{problem}"),
        };
        crate::ui::post(&dispatcher, move || {
            if format != ExportFormat::Rubi {
                crate::state::exports::remember(&bvid, format);
            }
            toast(&said);
        });
    });
}
