//! A match as a row, for the 赛事 page and the search's 赛事 block: when and which stage, the two
//! teams with their logos and the score, and what a click does. A match on now opens its room,
//! one over its replay, one to come is 预约 or not.

use std::time::{SystemTime, UNIX_EPOCH};

use rubi_model::{EsportsMatch, MatchState};
use windows_collections::IVector;
use windows_core::{Interface, Result};

use crate::state::live::{score, state_text, when};
use crate::ui::rows::Covers;
use crate::ui::shell::{shell, toast};
use crate::ui::{live_room, player, post};
use crate::xaml::{
    Color, IDependencyObject, IImage, ITextBlock, IUIElement, SolidColorBrush, TextBlock, UIElement, Visibility,
};

const LOGO_DECODE_WIDTH: i32 = 56;
const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const RED: Color = Color { a: 255, r: 0xE3, g: 0x33, b: 0x33 };

/// A list of matches named `name`, as tall as its rows when `fixed` (inside a page that scrolls).
pub fn list(name: &str, fixed: bool) -> String {
    let scroll = if fixed { r#" ScrollViewer.VerticalScrollMode="Disabled""# } else { "" };
    format!(
        r##"<ListView xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
          xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" x:Name="{name}"
          SelectionMode="None" IsItemClickEnabled="True"{scroll}>
  <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
  <ListView.ItemContainerStyle>
    <Style TargetType="ListViewItem">
      <Setter Property="Padding" Value="16,8" />
      <Setter Property="HorizontalContentAlignment" Value="Stretch" />
    </Style>
  </ListView.ItemContainerStyle>
  <ListView.ItemTemplate><DataTemplate>
    <Grid>
      <Grid.ColumnDefinitions>
        <ColumnDefinition Width="104" /><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" />
        <ColumnDefinition Width="64" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" />
        <ColumnDefinition Width="76" />
      </Grid.ColumnDefinitions>
      <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" /></Grid.RowDefinitions>
      <TextBlock FontSize="13" />
      <TextBlock Grid.Row="1" FontSize="11"
                 Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
      <TextBlock Grid.Column="1" Grid.RowSpan="2" FontSize="14" VerticalAlignment="Center"
                 TextAlignment="Right" TextTrimming="CharacterEllipsis" />
      <TextBlock Grid.Column="3" Grid.RowSpan="2" FontSize="15" VerticalAlignment="Center" TextAlignment="Center" />
      <TextBlock Grid.Column="5" Grid.RowSpan="2" FontSize="14" VerticalAlignment="Center"
                 TextTrimming="CharacterEllipsis" />
      <Image Grid.Column="2" Grid.RowSpan="2" Width="28" Height="28" Margin="8,0,0,0" VerticalAlignment="Center" />
      <Image Grid.Column="4" Grid.RowSpan="2" Width="28" Height="28" Margin="0,0,8,0" VerticalAlignment="Center" />
      <TextBlock Grid.Column="6" Grid.RowSpan="2" FontSize="12" VerticalAlignment="Center" TextAlignment="Right" />
    </Grid>
  </DataTemplate></ListView.ItemTemplate>
</ListView>"##
    )
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

/// What the last column says, and in which colour (none: the theme's quiet one).
fn action(game: &EsportsMatch) -> (&'static str, Option<Color>) {
    match game.state {
        MatchState::Live => ("直播中", Some(RED)),
        MatchState::Done if !game.replay_bvid.is_empty() => ("回放 ›", None),
        MatchState::Done => ("已结束", None),
        MatchState::Upcoming if game.subscribed => ("已预约", None),
        MatchState::Upcoming => ("预约", Some(COBALT)),
    }
}

/// Fills a row's `parts` with `game`, with team logos when `logos`.
pub fn fill(parts: &IVector<UIElement>, game: &EsportsMatch, logos: bool, covers: &Covers) -> Result<()> {
    let texts = [
        when(game.start_at, now()),
        if game.stage.is_empty() { state_text(game.state).to_owned() } else { game.stage.clone() },
        game.home.name.clone(),
        score(game),
        game.away.name.clone(),
    ];
    for (part, text) in texts.iter().enumerate() {
        parts.GetAt(part as u32)?.cast::<ITextBlock>()?.SetText(text)?;
    }
    for (part, team) in [(5, &game.home), (6, &game.away)] {
        let on = logos && !team.logo.is_empty();
        let logo = parts.GetAt(part)?;
        logo.cast::<IUIElement>()?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })?;
        if on {
            covers.show(&logo.cast::<IImage>()?, &team.logo, LOGO_DECODE_WIDTH)?;
        }
    }
    let (label, color) = action(game);
    let last = parts.GetAt(7)?;
    last.cast::<ITextBlock>()?.SetText(label)?;
    // Rows are reused: a colour comes off again.
    match color {
        Some(color) => last.cast::<ITextBlock>()?.SetForeground(&SolidColorBrush::CreateInstanceWithColor(color)?),
        None => last.cast::<IDependencyObject>()?.ClearValue(&TextBlock::ForegroundProperty()?),
    }
}

/// What clicking `game` does; `changed` gets it back once its 预约 has changed.
pub fn open(game: &EsportsMatch, changed: impl FnOnce(EsportsMatch) + Send + 'static) {
    let title = format!("{} vs {}", game.home.name, game.away.name);
    match game.state {
        MatchState::Live if game.live_room > 0 => {
            let _ = live_room::open(game.live_room, &title, &game.season, 0);
        }
        MatchState::Live => toast("这场没有直播间"),
        MatchState::Done if !game.replay_bvid.is_empty() => {
            let _ = player::show(&game.replay_bvid, &format!("{} {} {title}", game.season, game.stage), "");
        }
        MatchState::Done => toast("这场还没有回放"),
        MatchState::Upcoming => subscribe(game.clone(), changed),
    }
}

fn subscribe(mut game: EsportsMatch, changed: impl FnOnce(EsportsMatch) + Send + 'static) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let on = !game.subscribed;
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let done = core.services.esports.subscribe(game.id, on).await.map_err(|e| e.to_string());
        post(&dispatcher, move || match done {
            Ok(()) => {
                toast(if on { "已预约，开赛时 B 站会提醒" } else { "已取消预约" });
                game.subscribed = on;
                changed(game);
            }
            Err(problem) => toast(&format!("预约失败：{problem}")),
        });
    });
}
