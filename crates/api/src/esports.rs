//! 赛事: the match schedule, the games it can be narrowed to, and 预约.

use std::sync::Arc;

use async_trait::async_trait;
use rubi_model::{EsportsGame, EsportsMatch, MatchState, Team, VideoRef, find_video_ref};
use serde_json::Value;

use crate::error::Result;
use crate::http::HttpClient;
use crate::params;
use crate::parse::{https_image, str_of, u64_of};

const MATCHES: &str = "https://api.bilibili.com/x/esports/matchs/list";
const FILTER: &str = "https://api.bilibili.com/x/esports/matchs/filter";
const SUBSCRIBE: &str = "https://api.bilibili.com/x/esports/fav/add";
const UNSUBSCRIBE: &str = "https://api.bilibili.com/x/esports/fav/del";
/// Team and season logos are written as paths on this host.
const IMAGE_HOST: &str = "https://i0.hdslb.com";

/// One page of the schedule and the total it is part of.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MatchPage {
    pub matches: Vec<EsportsMatch>,
    pub total: u32,
}

#[async_trait]
pub trait EsportsApi: Send + Sync {
    /// Matches from `from` to `to` (both `YYYY-MM-DD`, China time) of one game (0 for all).
    async fn matches(&self, game: u32, from: &str, to: &str, page: u32, size: u32) -> Result<MatchPage>;
    async fn games(&self) -> Result<Vec<EsportsGame>>;
    /// 预约 or cancel it (needs login).
    async fn subscribe(&self, match_id: u64, on: bool) -> Result<()>;
}

pub struct WebEsports {
    http: Arc<HttpClient>,
}

impl WebEsports {
    pub fn new(http: Arc<HttpClient>) -> Self {
        Self { http }
    }
}

#[async_trait]
impl EsportsApi for WebEsports {
    async fn matches(&self, game: u32, from: &str, to: &str, page: u32, size: u32) -> Result<MatchPage> {
        let d = self
            .http
            .get_json(
                MATCHES,
                params![
                    "mid" => 0, "gid" => game, "tid" => 0, "pn" => page, "ps" => size,
                    "contest_status" => "", "stime" => from, "etime" => to,
                ],
                false,
            )
            .await?;
        Ok(page_of(&d))
    }

    async fn games(&self) -> Result<Vec<EsportsGame>> {
        let d = self.http.get_json(FILTER, params!["mid" => 0, "gid" => 0, "tid" => 0], false).await?;
        Ok(d["games"]
            .as_array()
            .map(|l| {
                l.iter()
                    .map(|g| EsportsGame { id: u64_of(&g["id"]) as u32, title: str_of(&g["title"]) })
                    .filter(|g| g.id > 0)
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn subscribe(&self, match_id: u64, on: bool) -> Result<()> {
        self.http.post_form(if on { SUBSCRIBE } else { UNSUBSCRIBE }, params!["cid" => match_id]).await?;
        Ok(())
    }
}

pub(crate) fn page_of(d: &Value) -> MatchPage {
    MatchPage {
        matches: d["list"].as_array().map(|l| l.iter().map(match_of).collect()).unwrap_or_default(),
        total: u64_of(&d["page"]["total"]) as u32,
    }
}

pub(crate) fn match_of(v: &Value) -> EsportsMatch {
    let replay = ["playback", "play_back", "collection_url"]
        .iter()
        .find_map(|k| match find_video_ref(v[*k].as_str().unwrap_or_default()) {
            Some(VideoRef::Bvid(b)) => Some(b),
            _ => None,
        })
        .or_else(|| Some(str_of(&v["collection_bvid"])).filter(|b| !b.is_empty()))
        .unwrap_or_default();
    EsportsMatch {
        id: u64_of(&v["id"]),
        season: str_of(&v["season"]["title"]),
        season_aka: str_of(&v["season"]["sub_title"]),
        stage: Some(str_of(&v["game_stage"])).filter(|s| !s.is_empty()).unwrap_or_else(|| str_of(&v["title"])),
        start_at: u64_of(&v["stime"]) as i64,
        home: team_of(&v["home_team"]),
        away: team_of(&v["away_team"]),
        home_score: u64_of(&v["home_score"]) as u32,
        away_score: u64_of(&v["away_score"]) as u32,
        // contest_status: 1 not started, 2 on, 3 over.
        state: match u64_of(&v["contest_status"]) {
            2 => MatchState::Live,
            3 => MatchState::Done,
            _ => MatchState::Upcoming,
        },
        live_room: u64_of(&v["live_room"]),
        replay_bvid: replay,
        subscribed: u64_of(&v["is_sub"]) == 1,
    }
}

fn team_of(t: &Value) -> Team {
    let logo = str_of(&t["logo"]);
    let logo = if logo.starts_with('/') && !logo.starts_with("//") {
        format!("{IMAGE_HOST}{logo}")
    } else {
        https_image(&logo)
    };
    Team { name: str_of(&t["title"]), logo }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// 2026-11-02: a finished S16 match from `matchs/list`.
    #[test]
    fn reads_a_match() {
        let d = json!({"page": {"num": 1, "size": 5, "total": 33}, "list": [{
            "id": 37747, "game_stage": "瑞士轮", "stime": 1793466000, "home_score": 0, "away_score": 2,
            "live_room": 7734200, "collection_bvid": "", "contest_status": 3, "is_sub": 0,
            "playback": "https://www.bilibili.com/video/BV1jpxJ6JEtF",
            "season": {"title": "2026全球总决赛"},
            "home_team": {"title": "MKOI", "logo": "/bfs/legacy/c80c.png"},
            "away_team": {"title": "KC", "logo": "//i0.hdslb.com/bfs/legacy/3784.png"}}]});
        let p = page_of(&d);
        assert_eq!(p.total, 33);
        let m = &p.matches[0];
        assert_eq!((m.id, m.stage.as_str(), m.season.as_str()), (37747, "瑞士轮", "2026全球总决赛"));
        assert_eq!((m.state, m.home_score, m.away_score, m.live_room), (MatchState::Done, 0, 2, 7734200));
        assert_eq!(m.replay_bvid, "BV1jpxJ6JEtF");
        assert_eq!(m.home.logo, "https://i0.hdslb.com/bfs/legacy/c80c.png");
        assert_eq!(m.away.logo, "https://i0.hdslb.com/bfs/legacy/3784.png");
    }

    #[test]
    fn upcoming_and_live_states() {
        let up = match_of(&json!({"contest_status": 1, "is_sub": 1, "playback": ""}));
        assert_eq!((up.state, up.subscribed, up.replay_bvid.as_str()), (MatchState::Upcoming, true, ""));
        assert_eq!(match_of(&json!({"contest_status": 2})).state, MatchState::Live);
    }
}
