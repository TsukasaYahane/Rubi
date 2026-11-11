//! 赛事: matches from yesterday to a week ahead, by game. Only to read for now.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use rubi_model::EsportsMatch;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Result};

use crate::state::layout::WindowClass;
use crate::state::live::{score, state_text, when};
use crate::ui::live::{chips, held, mark_chip};
use crate::ui::rows::{Covers, add_rows, on_row};
use crate::ui::shell::shell;
use crate::ui::{load, on_click, post};
use crate::xaml::{IFrameworkElement, IImage, IItemsControl, ITextBlock, IUIElement, UIElement, Visibility};

/// Rows from the end at which more load.
const LOAD_AHEAD: usize = 6;
const LOGO_DECODE_WIDTH: i32 = 56;

const LIST: &str = r##"
<ListView xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" SelectionMode="None">
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
      </Grid.ColumnDefinitions>
      <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" /></Grid.RowDefinitions>
      <TextBlock FontSize="13" />
      <TextBlock Grid.Row="1" FontSize="11"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      <TextBlock Grid.Column="1" Grid.RowSpan="2" FontSize="14" VerticalAlignment="Center"
                 TextAlignment="Right" TextTrimming="CharacterEllipsis" />
      <TextBlock Grid.Column="3" Grid.RowSpan="2" FontSize="15" VerticalAlignment="Center" TextAlignment="Center" />
      <TextBlock Grid.Column="5" Grid.RowSpan="2" FontSize="14" VerticalAlignment="Center"
                 TextTrimming="CharacterEllipsis" />
      <Image Grid.Column="2" Grid.RowSpan="2" Width="28" Height="28" Margin="8,0,0,0" VerticalAlignment="Center" />
      <Image Grid.Column="4" Grid.RowSpan="2" Width="28" Height="28" Margin="0,0,8,0" VerticalAlignment="Center" />
    </Grid>
  </DataTemplate></ListView.ItemTemplate>
</ListView>"##;

pub struct Schedule {
    list: IFrameworkElement,
    matches: RefCell<Vec<EsportsMatch>>,
    game: Cell<u32>,
    page: Cell<u32>,
    ended: Cell<bool>,
    loading: Cell<bool>,
    started: Cell<bool>,
    turn: Cell<u64>,
    said: RefCell<String>,
    game_chips: RefCell<Option<IFrameworkElement>>,
    covers: Covers,
    /// Team logos show.
    logos: Cell<bool>,
}

fn wide() -> bool {
    shell().is_some_and(|s| s.class() == WindowClass::Wide)
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs() as i64)
}

impl Schedule {
    pub fn new() -> Result<Rc<Self>> {
        let list = load(LIST)?;
        let shown = Rc::new(Self {
            list: list.clone(),
            matches: RefCell::default(),
            game: Cell::new(0),
            page: Cell::new(1),
            ended: Cell::new(false),
            loading: Cell::new(false),
            started: Cell::new(false),
            turn: Cell::new(0),
            said: RefCell::default(),
            game_chips: RefCell::default(),
            covers: Covers::default(),
            logos: Cell::new(wide()),
        });
        let filling = Rc::downgrade(&shown);
        on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |s| s.fill(at, parts)))?;
        Ok(shown)
    }

    pub fn root(&self) -> &IFrameworkElement {
        &self.list
    }

    pub fn said(&self) -> String {
        self.said.borrow().clone()
    }

    pub fn chips(&self) -> Option<IFrameworkElement> {
        self.game_chips.borrow().clone()
    }

    fn say(&self, text: &str) {
        *self.said.borrow_mut() = text.to_owned();
        if let Some(page) = held() {
            page.refresh();
        }
    }

    /// Loads the first time it shows.
    pub fn start(self: &Rc<Self>) -> Result<()> {
        if self.started.replace(true) {
            return Ok(());
        }
        self.load_games();
        self.open(0)
    }

    /// Reads the game on show again.
    pub fn refresh(&self) -> Result<()> {
        self.open(self.game.get())?;
        self.say("正在读取…");
        Ok(())
    }

    fn open(&self, game: u32) -> Result<()> {
        self.game.set(game);
        self.page.set(1);
        self.ended.set(false);
        self.loading.set(false);
        self.turn.set(self.turn.get() + 1);
        self.matches.borrow_mut().clear();
        self.list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        *self.said.borrow_mut() = "正在读取…".to_owned();
        self.load();
        Ok(())
    }

    fn load(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.ended.get() || self.loading.replace(true) {
            return;
        }
        let (game, page, turn) = (self.game.get(), self.page.get(), self.turn.get());
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let answer = core.services.esports.schedule(game, page).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(page) = held() {
                    let _ = page.schedule_answered(turn, answer);
                }
            });
        });
    }

    pub fn took(&self, turn: u64, answer: std::result::Result<(Vec<EsportsMatch>, bool), String>) -> Result<()> {
        if turn != self.turn.get() {
            return Ok(());
        }
        self.loading.set(false);
        match answer {
            Ok((matches, more)) => {
                self.ended.set(!more);
                self.page.set(self.page.get() + 1);
                let from = self.matches.borrow().len();
                self.matches.borrow_mut().extend(matches);
                let to = self.matches.borrow().len();
                add_rows(&self.list, from, to)?;
                self.say(if to == 0 { "这段时间没有比赛" } else { "" });
            }
            Err(problem) => {
                self.ended.set(true);
                self.say(&format!("读取失败：{problem}"));
            }
        }
        Ok(())
    }

    fn load_games(self: &Rc<Self>) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let Ok(games) = core.services.esports.games().await else { return };
            post(&dispatcher, move || {
                if let Some(page) = held() {
                    let _ = page.schedule_games(games);
                }
            });
        });
    }

    pub fn took_games(self: &Rc<Self>, games: Vec<rubi_model::EsportsGame>) -> Result<()> {
        let mut labels = vec!["全部".to_owned()];
        labels.extend(games.iter().map(|g| g.title.clone()));
        let row = chips(&labels)?;
        mark_chip(&row, labels.len(), 0)?;
        for at in 0..labels.len() {
            let (weak, shown, count) = (Rc::downgrade(self), row.clone(), labels.len());
            let game = at.checked_sub(1).and_then(|g| games.get(g)).map_or(0, |g| g.id);
            on_click(&row, &format!("Chip{at}"), move || {
                if let Some(schedule) = weak.upgrade() {
                    let _ = mark_chip(&shown, count, at);
                    let _ = schedule.open(game);
                    schedule.say("正在读取…");
                }
            })?;
        }
        *self.game_chips.borrow_mut() = Some(row);
        if let Some(page) = held() {
            page.refresh();
        }
        Ok(())
    }

    /// Follows the window: rows on screen are filled again when logos come or go.
    pub fn resized(&self) {
        let logos = wide();
        if self.logos.replace(logos) != logos {
            let _ = self.refill();
        }
    }

    fn refill(&self) -> Result<()> {
        self.list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        add_rows(&self.list, 0, self.matches.borrow().len())
    }

    fn fill(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let matches = self.matches.borrow();
        let Some(game) = matches.get(at) else { return Ok(()) };
        let texts = [
            when(game.start_at, now()),
            state_text(game.state).to_owned(),
            game.home.name.clone(),
            score(game),
            game.away.name.clone(),
        ];
        for (part, text) in texts.iter().enumerate() {
            parts.GetAt(part as u32)?.cast::<ITextBlock>()?.SetText(text)?;
        }
        let logos = self.logos.get();
        for (part, team) in [(5, &game.home), (6, &game.away)] {
            let on = logos && !team.logo.is_empty();
            let logo = parts.GetAt(part)?;
            logo.cast::<IUIElement>()?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })?;
            if on {
                self.covers.show(&logo.cast::<IImage>()?, &team.logo, LOGO_DECODE_WIDTH)?;
            }
        }
        let near_end = at + LOAD_AHEAD >= matches.len();
        drop(matches);
        if near_end {
            self.load();
        }
        Ok(())
    }
}
