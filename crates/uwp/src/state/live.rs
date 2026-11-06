//! What the 直播 pages show, without touching XAML.

use rubi_model::{EsportsMatch, LiveRoom, MatchState};

use crate::state::library::{count, date};

const CST: i64 = 8 * 3_600;
const DAY: i64 = 86_400;

/// Most lines the chat keeps; past this the oldest half goes.
pub const CHAT_MOST: usize = 400;

pub fn room_line(room: &LiveRoom) -> String {
    if room.live && room.online > 0 {
        format!("{} · {} 人气", room.uname, count(room.online))
    } else {
        room.uname.clone()
    }
}

fn day_of(unix: i64) -> i64 {
    (unix + CST).div_euclid(DAY)
}

/// When a match starts: 今天 19:00, or a date further off.
pub fn when(start: i64, now: i64) -> String {
    let local = (start + CST).rem_euclid(DAY);
    let time = format!("{:02}:{:02}", local / 3_600, local % 3_600 / 60);
    match day_of(start) - day_of(now) {
        -1 => format!("昨天 {time}"),
        0 => format!("今天 {time}"),
        1 => format!("明天 {time}"),
        _ => format!("{} {time}", date(start)),
    }
}

pub fn state_text(state: MatchState) -> &'static str {
    match state {
        MatchState::Upcoming => "未开始",
        MatchState::Live => "进行中",
        MatchState::Done => "已结束",
    }
}

/// The score, once there is one.
pub fn score(game: &EsportsMatch) -> String {
    if game.state == MatchState::Upcoming {
        "-".to_owned()
    } else {
        format!("{} : {}", game.home_score, game.away_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-01-15 12:00 in China.
    const NOON: i64 = 1_768_449_600;

    #[test]
    fn near_days_have_names() {
        assert_eq!(when(NOON + 7 * 3_600, NOON), "今天 19:00");
        assert_eq!(when(NOON + 29 * 3_600, NOON), "明天 17:00");
        assert_eq!(when(NOON - 20 * 3_600, NOON), "昨天 16:00");
        assert_eq!(when(NOON + 3 * DAY, NOON), "2026-01-18 12:00");
    }

    #[test]
    fn an_upcoming_match_has_no_score() {
        let mut game = EsportsMatch { home_score: 2, away_score: 1, ..EsportsMatch::default() };
        assert_eq!(score(&game), "-");
        game.state = MatchState::Live;
        assert_eq!(score(&game), "2 : 1");
    }

    #[test]
    fn a_room_says_who_and_how_many() {
        let room = LiveRoom { uname: "猫".into(), online: 32_000, live: true, ..LiveRoom::default() };
        assert_eq!(room_line(&room), "猫 · 3.2 万 人气");
        assert_eq!(room_line(&LiveRoom { live: false, ..room }), "猫");
    }
}
