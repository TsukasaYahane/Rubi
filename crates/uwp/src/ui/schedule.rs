//! 赛事: matches from yesterday to a week ahead, by game. A click opens the room or the replay, or
//! changes the 预约 (see `match_rows`).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rubi_model::EsportsMatch;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Result};

use crate::state::layout::WindowClass;
use crate::ui::live::{chips, held, mark_chip};
use crate::ui::match_rows;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{load, on_click, post};
use crate::xaml::{IFrameworkElement, IItemsControl, UIElement};

/// Rows from the end at which more load.
const LOAD_AHEAD: usize = 6;


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


impl Schedule {
    pub fn new() -> Result<Rc<Self>> {
        let list = load(&match_rows::list("Schedule", false))?;
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
        on_row_click(&list, |at| {
            let Some(game) = held().and_then(|p| p.schedule_match(at)) else { return };
            match_rows::open(&game, move |changed| {
                if let Some(page) = held() {
                    page.schedule_changed(at, changed);
                }
            });
        })?;
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

    /// Match `at`, to act on.
    pub fn at(&self, at: usize) -> Option<EsportsMatch> {
        self.matches.borrow().get(at).cloned()
    }

    /// Match `at` came back changed: its row shows it again.
    pub fn changed(&self, at: usize, game: EsportsMatch) -> Result<()> {
        match self.matches.borrow_mut().get_mut(at) {
            Some(slot) => *slot = game,
            None => return Ok(()),
        }
        let items = self.list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?;
        items.SetAt(at as u32, &crate::xaml::PropertyValue::CreateInt32(at as i32)?)
    }

    fn fill(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let matches = self.matches.borrow();
        let Some(game) = matches.get(at) else { return Ok(()) };
        match_rows::fill(parts, game, self.logos.get(), &self.covers)?;
        let near_end = at + LOAD_AHEAD >= matches.len();
        drop(matches);
        if near_end {
            self.load();
        }
        Ok(())
    }
}
