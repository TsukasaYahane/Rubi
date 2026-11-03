//! 赛事: the schedule around today and 预约.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rubi_api::EsportsApi;
use rubi_model::{EsportsGame, EsportsMatch, MatchState, now_secs};

use crate::error::Result;

/// The schedule covers yesterday (for replays) to a week ahead.
const DAYS_BEFORE: i64 = 1;
const DAYS_AFTER: i64 = 6;
const PAGE_SIZE: u32 = 20;
/// The schedule's days are China days.
const CHINA_OFFSET: i64 = 8 * 3600;
/// Search reads the whole schedule (a few pages) at most this often.
const SCHEDULE_TTL: Duration = Duration::from_secs(10 * 60);
/// Pages of the schedule search looks through.
const SEARCH_PAGES: u32 = 6;
/// Match cards shown above search results.
const SEARCH_CARDS: usize = 3;

pub struct Esports {
    api: Arc<dyn EsportsApi>,
    games: Mutex<Option<Vec<EsportsGame>>>,
    /// Every game's schedule, for search.
    all: Mutex<Option<(Instant, Vec<EsportsMatch>)>>,
}

impl Esports {
    pub(crate) fn new(api: Arc<dyn EsportsApi>) -> Self {
        Self { api, games: Mutex::new(None), all: Mutex::new(None) }
    }

    /// One page of matches of `game` (0 for every game), earliest first; also whether more follow.
    pub async fn schedule(&self, game: u32, page: u32) -> Result<(Vec<EsportsMatch>, bool)> {
        let today = (now_secs() + CHINA_OFFSET).div_euclid(86_400);
        let (from, to) = (date_of(today - DAYS_BEFORE), date_of(today + DAYS_AFTER));
        let page = page.max(1);
        let p = self.api.matches(game, &from, &to, page, PAGE_SIZE).await?;
        let more = !p.matches.is_empty() && p.total > page * PAGE_SIZE;
        Ok((p.matches, more))
    }

    /// The games the schedule can be narrowed to, read once per process.
    pub async fn games(&self) -> Result<Vec<EsportsGame>> {
        if let Some(g) = self.games.lock().unwrap_or_else(|e| e.into_inner()).clone() {
            return Ok(g);
        }
        let g = self.api.games().await?;
        *self.games.lock().unwrap_or_else(|e| e.into_inner()) = Some(g.clone());
        Ok(g)
    }

    /// The matches a search is about (a team, an event): on now first, then the next to come,
    /// then the latest results. Empty when the words name nothing on the schedule.
    pub async fn related(&self, keyword: &str) -> Result<Vec<EsportsMatch>> {
        if words(keyword).is_empty() {
            return Ok(Vec::new());
        }
        let cached = self.all.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let all = match cached {
            Some((at, list)) if at.elapsed() < SCHEDULE_TTL => list,
            _ => {
                let mut list = Vec::new();
                for page in 1..=SEARCH_PAGES {
                    let (mut m, more) = self.schedule(0, page).await?;
                    list.append(&mut m);
                    if !more {
                        break;
                    }
                }
                *self.all.lock().unwrap_or_else(|e| e.into_inner()) = Some((Instant::now(), list.clone()));
                list
            }
        };
        Ok(relevant(all, keyword))
    }

    pub async fn subscribe(&self, match_id: u64, on: bool) -> Result<()> {
        Ok(self.api.subscribe(match_id, on).await?)
    }
}

/// The words of a search worth matching: split at spaces and 「vs」, at least two letters.
fn words(keyword: &str) -> Vec<String> {
    keyword
        .to_lowercase()
        .replace("vs", " ")
        .split(|c: char| c.is_whitespace() || matches!(c, ':' | '：' | ',' | '，' | '/'))
        .filter(|w| w.chars().count() >= 2)
        .map(str::to_owned)
        .collect()
}

/// Matches whose team is one of the words, or whose event is named by one of them.
fn relevant(all: Vec<EsportsMatch>, keyword: &str) -> Vec<EsportsMatch> {
    let words = words(keyword);
    let team = |name: &str| name != "待定" && words.iter().any(|w| *w == name.to_lowercase());
    let event = |m: &EsportsMatch| {
        let names = format!("{} {}", m.season, m.season_aka).to_lowercase();
        words.iter().any(|w| names.contains(w.as_str()))
    };
    let mut hits: Vec<EsportsMatch> =
        all.into_iter().filter(|m| team(&m.home.name) || team(&m.away.name) || event(m)).collect();
    // On now, then soonest to come, then the latest over.
    hits.sort_by_key(|m| match m.state {
        MatchState::Live => (0, 0),
        MatchState::Upcoming => (1, m.start_at),
        MatchState::Done => (2, -m.start_at),
    });
    hits.truncate(SEARCH_CARDS);
    hits
}

/// `YYYY-MM-DD` of a day counted from 1970-01-01 (Howard Hinnant's civil-from-days).
fn date_of(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(id: u64, home: &str, away: &str, state: MatchState, start_at: i64) -> EsportsMatch {
        EsportsMatch {
            id,
            season: "2026全球总决赛".into(),
            season_aka: "S16，英雄联盟赛事，世界赛".into(),
            home: rubi_model::Team { name: home.into(), logo: String::new() },
            away: rubi_model::Team { name: away.into(), logo: String::new() },
            state,
            start_at,
            ..Default::default()
        }
    }

    #[test]
    fn finds_matches_a_search_is_about() {
        let all = vec![
            game(1, "HLE", "BLG", MatchState::Done, 100),
            game(2, "BLG", "T1", MatchState::Upcoming, 300),
            game(3, "TES", "IG", MatchState::Upcoming, 200),
            game(4, "待定", "待定", MatchState::Upcoming, 400),
            game(5, "KC", "MKOI", MatchState::Live, 50),
        ];
        let ids = |kw: &str| relevant(all.clone(), kw).iter().map(|m| m.id).collect::<Vec<_>>();
        assert_eq!(ids("blg"), vec![2, 1], "the next match before the last result");
        assert_eq!(ids("TES vs IG"), vec![3]);
        assert_eq!(ids("S16"), vec![5, 3, 2], "the event's other name; live first, three at most");
        assert_eq!(ids("总决赛"), vec![5, 3, 2]);
        assert!(ids("待定").is_empty(), "placeholders are no team");
        assert!(ids("原神 攻略").is_empty());
        assert!(ids("T").is_empty(), "single letters match nothing");
    }

    #[test]
    fn writes_dates() {
        assert_eq!(date_of(0), "1970-01-01");
        assert_eq!(date_of(59), "1970-03-01");
        // 1793577600 = 2026-11-02 00:00 China time.
        assert_eq!(date_of((1_793_577_600 + CHINA_OFFSET) / 86_400), "2026-11-02");
        assert_eq!(date_of(11_016), "2000-02-29");
    }
}
