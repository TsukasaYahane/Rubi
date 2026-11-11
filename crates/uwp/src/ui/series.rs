//! What a video belongs with, under its actions: its parts (分P), each with how far it was
//! watched this run, and the 合集 it is in, with the next one and the whole list. Clicks go to the
//! video page; this only draws.

use std::collections::HashMap;

use rubi_core::{Season, SeasonEpisode, VideoPart};
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::settings::escape;
use crate::ui::{keep, load, named, player, set_text};
use crate::xaml::{
    IButtonBase, IContentControl, IFrameworkElement, IUIElement, PropertyValue, RoutedEventArgs, Visibility,
};

/// A part's tile.
const PART_WIDTH: f64 = 156.0;

/// The rows of the 合集's videos, in order.
pub fn episodes(season: &Season) -> Vec<&SeasonEpisode> {
    season.sections.iter().flat_map(|s| s.episodes.iter()).collect()
}

fn mmss(seconds: u32) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

fn label(root: &IFrameworkElement, name: &str, text: &str) -> Result<()> {
    named::<IContentControl>(root, name)?.SetContent(&PropertyValue::CreateString(text)?)
}

fn shown(root: &IFrameworkElement, name: &str, on: bool) -> Result<()> {
    named::<IUIElement>(root, name)?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })
}

/// The parts, `current` marked; `progress` holds how far each was watched (ms). Folded they are one
/// scrolling row; `expanded`, they wrap.
pub fn show_parts(
    root: &IFrameworkElement,
    parts: &[VideoPart],
    current: u64,
    progress: &HashMap<u64, u64>,
    expanded: bool,
    auto_next: bool,
) -> Result<()> {
    let many = parts.len() > 1;
    shown(root, "PartsBlock", many)?;
    if !many {
        return Ok(());
    }
    let at = parts.iter().position(|p| p.cid == current).unwrap_or(0);
    set_text(root, "PartCount", &format!("第 {} / {} 个", at + 1, parts.len()))?;
    label(root, "AutoNext", &format!("播完自动连播下一个  {}", if auto_next { "开" } else { "关" }))?;
    label(root, "PartsExpand", if expanded { "收起 ▴" } else { "展开 ▾" })?;
    let width = named::<IFrameworkElement>(root, "PartsBlock")?.ActualWidth()?;
    let tiles: String = parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            let done = progress.get(&part.cid).map_or(0.0, |ms| {
                (*ms as f64 / (f64::from(part.duration_s.max(1)) * 1000.0)).clamp(0.0, 1.0)
            });
            let on = part.cid == current;
            let tint = if on { r##"BorderBrush="#0050EF" Foreground="#0050EF""## } else { "" };
            format!(
                r##"<Button Width="150" Height="54" Padding="8,4" HorizontalContentAlignment="Stretch" VerticalContentAlignment="Top"
                     BorderThickness="1" Background="{{ThemeResource SystemControlBackgroundBaseLowBrush}}" {tint}>
                  <StackPanel>
                    <TextBlock Text="P{} · {}" FontSize="12" FontWeight="SemiBold" />
                    <TextBlock Text="{}" FontSize="12" TextTrimming="CharacterEllipsis" />
                    <Grid Height="2" Margin="0,4,0,0" Background="{{ThemeResource SystemControlForegroundBaseLowBrush}}">
                      <Rectangle HorizontalAlignment="Left" Width="{}" Fill="#0050EF" />
                    </Grid>
                  </StackPanel></Button>"##,
                i + 1,
                mmss(part.duration_s),
                escape(&part.part),
                (done * 132.0).round()
            )
        })
        .collect();
    // Folded: one row that scrolls sideways. Open: every part, wrapped to the width.
    let panel = if expanded {
        let most = ((width / PART_WIDTH).floor() as i32).max(1);
        format!(
            r##"<VariableSizedWrapGrid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                 Orientation="Horizontal" MaximumRowsOrColumns="{most}" ItemWidth="{PART_WIDTH}" ItemHeight="60">{tiles}</VariableSizedWrapGrid>"##
        )
    } else {
        format!(
            r##"<StackPanel xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" Orientation="Horizontal"
                 Spacing="6">{tiles}</StackPanel>"##
        )
    };
    let panel = load(&panel)?;
    let children = panel.cast::<crate::xaml::IPanel>()?.Children()?;
    for (i, part) in parts.iter().enumerate() {
        let cid = part.cid;
        keep(
            children
                .GetAt(i as u32)?
                .cast::<IButtonBase>()?
                .Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| player::play_part(cid, None))?,
        );
    }
    let (into, other) = if expanded { ("PartsAll", "PartsScroll") } else { ("PartsScroll", "PartsAll") };
    named::<IContentControl>(root, other)?.SetContent(None::<&IInspectable>)?;
    shown(root, other, false)?;
    shown(root, into, true)?;
    named::<IContentControl>(root, into)?.SetContent(&panel.cast::<IInspectable>()?)
}

/// The 合集 `bvid` is in: where in it, the next one, subscribing, and the whole list when
/// `expanded`. `subscribed` is `None` while not known.
pub fn show_season(
    root: &IFrameworkElement,
    season: Option<&Season>,
    bvid: &str,
    expanded: bool,
    subscribed: Option<bool>,
) -> Result<()> {
    shown(root, "SeasonBlock", season.is_some())?;
    let Some(season) = season else { return Ok(()) };
    let all = episodes(season);
    let at = all.iter().position(|e| e.bvid == bvid);
    set_text(root, "SeasonTitle", &format!("合集 · {}", season.title))?;
    set_text(root, "SeasonCount", &at.map_or_else(String::new, |i| format!("第 {} / {} 个", i + 1, all.len())))?;
    label(root, "SeasonSub", if subscribed == Some(true) { "已订阅" } else { "订阅" })?;
    shown(root, "SeasonSub", subscribed.is_some())?;
    label(root, "SeasonExpand", if expanded { "收起 ▴" } else { "展开 ▾" })?;
    let next = at.and_then(|i| all.get(i + 1));
    match next {
        Some(next) => label(root, "SeasonNext", &format!("下一个：{} ›", next.title))?,
        None => label(root, "SeasonNext", "已是最后一个")?,
    }
    shown(root, "SeasonList", expanded)?;
    if !expanded {
        return Ok(());
    }
    let rows: String = all
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let tint = if Some(i) == at { r##" Foreground="#0050EF" FontWeight="SemiBold""## } else { "" };
            format!(
                r##"<Button HorizontalAlignment="Stretch" HorizontalContentAlignment="Left" Background="Transparent"
                     BorderThickness="0" Padding="6,5"{tint}>
                  <TextBlock TextTrimming="CharacterEllipsis" FontSize="13"><Run Text="{:>3}　" /><Run Text="{}" /></TextBlock></Button>"##,
                i + 1,
                escape(&e.title)
            )
        })
        .collect();
    let list = load(&format!(
        r##"<StackPanel xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">{rows}</StackPanel>"##
    ))?;
    let children = list.cast::<crate::xaml::IPanel>()?.Children()?;
    for (i, e) in all.iter().enumerate() {
        let (bvid, title) = (e.bvid.clone(), e.title.clone());
        keep(children.GetAt(i as u32)?.cast::<IButtonBase>()?.Click(
            move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| player::open_episode_of_season(&bvid, &title),
        )?);
    }
    named::<IContentControl>(root, "SeasonList")?.SetContent(&list.cast::<IInspectable>()?)
}
