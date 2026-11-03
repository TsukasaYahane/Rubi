//! 赛事: the match schedule.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MatchState {
    #[default]
    Upcoming,
    Live,
    Done,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Team {
    pub name: String,
    pub logo: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EsportsMatch {
    pub id: u64,
    /// The event: 「2026全球总决赛」.
    pub season: String,
    /// What else the event is called: 「S16，英雄联盟赛事，世界赛」.
    pub season_aka: String,
    /// The stage: 「瑞士轮」.
    pub stage: String,
    /// Unix seconds.
    pub start_at: i64,
    pub home: Team,
    pub away: Team,
    pub home_score: u32,
    pub away_score: u32,
    pub state: MatchState,
    /// The room it is broadcast in (0 when none).
    pub live_room: u64,
    /// The replay video (empty until there is one).
    pub replay_bvid: String,
    /// 已预约.
    pub subscribed: bool,
}

/// A game the schedule can be narrowed to.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EsportsGame {
    pub id: u32,
    pub title: String,
}
