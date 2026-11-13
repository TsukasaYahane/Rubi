//! 为什么推荐: the eight factors behind one card's score, and the three 不感兴趣 reasons. Drawn over
//! the window like the other sheets.

use rubi_core::{FactorId, ScoreBreakdown};
use rubi_model::{DislikeReason, Signal};
use windows_core::{Interface, Result};

use crate::ui::popup;
use crate::ui::shell::{shell, toast};
use crate::ui::{load, named, on_click, post, set_text};
use crate::xaml::{Color, IBorder, IFrameworkElement, IPanel, SolidColorBrush, UIElement};

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const RED: Color = Color { a: 255, r: 0xC0, g: 0x4A, b: 0x4A };
/// The longest a factor's bar may reach, either side of the middle.
const BAR: f64 = 75.0;

const LAYOUT: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="520" MaxHeight="600" Padding="18"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
      <RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <TextBlock Text="为什么推荐" FontSize="20" FontWeight="Light" />
    <TextBlock x:Name="WhyTitle" Grid.Row="1" FontSize="13" TextWrapping="Wrap" Margin="0,4,0,10" />
    <ScrollViewer Grid.Row="2" MaxHeight="380">
      <StackPanel x:Name="WhyRows" />
    </ScrollViewer>
    <TextBlock x:Name="WhyNote" Grid.Row="3" FontSize="12" TextWrapping="Wrap" Margin="0,10,0,0"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    <StackPanel Grid.Row="4" Orientation="Horizontal" Margin="0,12,0,0" HorizontalAlignment="Right">
      <Button x:Name="WhyDislike" Content="不感兴趣…" Margin="0,0,8,0" />
      <Button x:Name="WhyClose" Content="知道了" Background="#0050EF" Foreground="White" BorderThickness="0" />
    </StackPanel>
  </Grid>
</Grid>"##;

const DISLIKE: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="420" Padding="18"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <TextBlock Text="不感兴趣" FontSize="20" FontWeight="Light" />
    <TextBlock Grid.Row="1" FontSize="12" TextWrapping="Wrap" Margin="0,6,0,4"
               Text="选一个原因。这条会从推荐里消失，并记进屏蔽名单。"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    <StackPanel Grid.Row="2" Margin="0,8,0,0">
      <Button x:Name="WhyUp" Content="这个 UP 主" HorizontalAlignment="Stretch" Margin="0,0,0,6" />
      <Button x:Name="WhyZone" Content="这个分区" HorizontalAlignment="Stretch" Margin="0,0,0,6" />
      <Button x:Name="WhyTitleReason" Content="这类标题" HorizontalAlignment="Stretch" />
    </StackPanel>
    <StackPanel Grid.Row="3" Orientation="Horizontal" Margin="0,14,0,0" HorizontalAlignment="Right">
      <Button x:Name="WhyCancel" Content="取消" />
    </StackPanel>
  </Grid>
</Grid>"##;

pub fn factor_name(factor: FactorId) -> &'static str {
    match factor {
        FactorId::Interest => "兴趣匹配",
        FactorId::Follow => "关注 UP",
        FactorId::Freshness => "新鲜度",
        FactorId::Topic => "话题热度",
        FactorId::Quality => "内容质量",
        FactorId::Explore => "探索奖励",
        FactorId::Dislike => "不感兴趣",
        FactorId::RepeatExposure => "重复曝光",
    }
}

/// Shows why `bvid` was recommended, titled `title`.
pub fn show(bvid: &str, title: &str) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (bvid, title) = (bvid.to_owned(), title.to_owned());
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        // Explaining a score reads the pool and the profile: local, no waiting.
        let found = core.services.home.explain(&bvid).map_err(|e| e.to_string());
        post(&dispatcher, move || match found {
            Ok(Some(breakdown)) => {
                if let Err(problem) = draw(&bvid, &title, &breakdown) {
                    toast(&format!("打不开：{problem}"));
                }
            }
            Ok(None) => toast("这一条已经不在推荐池里了"),
            Err(problem) => toast(&format!("读取失败：{problem}")),
        });
    });
}

fn draw(bvid: &str, title: &str, breakdown: &ScoreBreakdown) -> Result<()> {
    let root = load(LAYOUT)?;
    set_text(&root, "WhyTitle", title)?;
    let note = format!(
        "总分 {:+.2}，其中来源调整 {:+.2}、惩罚调整 {:+.2}。因子权重在「我的 › 设置 › 推荐」里调。",
        breakdown.total, breakdown.source_adjust, breakdown.penalty_adjust
    );
    set_text(&root, "WhyNote", &note)?;
    let rows = named::<IPanel>(&root, "WhyRows")?.Children()?.cast::<windows_collections::IVector<UIElement>>()?;
    let widest = breakdown.parts.iter().map(|p| p.contribution.abs()).fold(0.0_f32, f32::max).max(0.01);
    for part in &breakdown.parts {
        let name = factor_name(part.factor);
        let row = load(&format!(
            r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                  xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Margin="0,3,0,3">
              <Grid.ColumnDefinitions>
                <ColumnDefinition Width="112" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="60" />
              </Grid.ColumnDefinitions>
              <TextBlock Text="{name}" FontSize="12" VerticalAlignment="Center"
                         Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
              <Grid Grid.Column="1" Width="152" Height="6" Background="#2A2A2A" VerticalAlignment="Center">
                <Border x:Name="Pos" Height="6" Width="0" HorizontalAlignment="Left" />
                <Border x:Name="Neg" Height="6" Width="0" HorizontalAlignment="Right" />
              </Grid>
              <TextBlock x:Name="Val" Grid.Column="2" FontSize="11" TextAlignment="Right" VerticalAlignment="Center" />
            </Grid>"##
        ))?;
        let width = f64::from(part.contribution.abs() / widest) * BAR;
        let bar = if part.contribution >= 0.0 { "Pos" } else { "Neg" };
        named::<IFrameworkElement>(&row, bar)?.SetWidth(width)?;
        let brush = SolidColorBrush::CreateInstanceWithColor(if part.contribution >= 0.0 { COBALT } else { RED })?;
        named::<IBorder>(&row, bar)?.SetBackground(&brush)?;
        set_text(&row, "Val", &format!("{:+.2}", part.contribution))?;
        rows.Append(&row.cast::<UIElement>()?)?;
    }
    let bvid = bvid.to_owned();
    on_click(&root, "WhyClose", || {
        let _ = popup::hide();
    })?;
    let asking = bvid.clone();
    on_click(&root, "WhyDislike", move || ask_dislike(&asking))?;
    popup::show(&root)
}

/// Asks why the card `bvid` is not wanted, then marks it.
pub fn ask_dislike(bvid: &str) {
    let root = match load(DISLIKE) {
        Ok(root) => root,
        Err(problem) => {
            toast(&format!("打不开：{problem}"));
            return;
        }
    };
    for (name, reason) in [
        ("WhyUp", DislikeReason::Up),
        ("WhyZone", DislikeReason::Zone),
        ("WhyTitleReason", DislikeReason::Title),
    ] {
        let bvid = bvid.to_owned();
        let reason = reason.clone();
        let _ = on_click(&root, name, move || mark(&bvid, reason.clone()));
    }
    let _ = on_click(&root, "WhyCancel", || {
        let _ = popup::hide();
    });
    if let Err(problem) = popup::show(&root) {
        toast(&format!("打不开：{problem}"));
    }
}

/// Puts `bvid` on the dislike list, which also takes it out of the feed.
fn mark(bvid: &str, reason: DislikeReason) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (bvid, dispatcher, core) = (bvid.to_owned(), shell.dispatcher.clone(), backend.clone());
    let _ = popup::hide();
    backend.spawn(async move {
        let said = match core.services.home.record(&bvid, Signal::Dislike { reason }).await {
            Ok(()) => "已不感兴趣 · 这个视频不会再出现在推荐里".to_owned(),
            Err(problem) => format!("记下失败：{problem}"),
        };
        post(&dispatcher, move || toast(&said));
    });
}
