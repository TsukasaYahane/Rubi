//! 影视专区: 番剧, 电影, 纪录片, 电视剧.

use crate::{FfiPlayback, PgcService, Res};

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcZone {
    pub name: String,
    pub season_type: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcRelease {
    pub season_id: u64,
    pub title: String,
    pub cover: String,
    pub index: String,
    pub time: String,
    pub published: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcDay {
    pub date: String,
    pub today: bool,
    pub releases: Vec<FfiPgcRelease>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcChoice {
    pub value: String,
    pub name: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcFilter {
    pub field: String,
    pub name: String,
    pub values: Vec<FfiPgcChoice>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcConditions {
    pub filters: Vec<FfiPgcFilter>,
    pub orders: Vec<FfiPgcChoice>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcSeason {
    pub season_id: u64,
    pub title: String,
    pub subtitle: String,
    pub cover: String,
    pub score: String,
    pub badge: String,
    pub index_show: String,
    pub order_text: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcPage {
    pub seasons: Vec<FfiPgcSeason>,
    pub has_more: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcEpisode {
    pub ep_id: u64,
    pub aid: u64,
    pub bvid: String,
    pub cid: u64,
    pub title: String,
    pub long_title: String,
    pub badge: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcDetail {
    pub season_id: u64,
    pub title: String,
    pub cover: String,
    pub evaluate: String,
    pub score: String,
    pub areas: Vec<String>,
    pub styles: Vec<String>,
    pub pub_time: String,
    pub episodes: Vec<FfiPgcEpisode>,
    pub allow_download: bool,
}

#[uniffi::export(async_runtime = "tokio")]
impl PgcService {
    /// The four 专区, in order.
    pub fn pgc_zones(&self) -> Vec<FfiPgcZone> {
        rubi_core::PGC_ZONES.iter().map(|(n, t)| FfiPgcZone { name: (*n).to_owned(), season_type: *t }).collect()
    }

    /// 近期上映 (only 番剧 has a timeline; empty for the others).
    pub async fn pgc_timeline(&self, season_type: u32) -> Res<Vec<FfiPgcDay>> {
        Ok(self
            .pgc
            .timeline(season_type)
            .await?
            .into_iter()
            .map(|d| FfiPgcDay {
                date: d.date,
                today: d.today,
                releases: d
                    .releases
                    .into_iter()
                    .map(|r| FfiPgcRelease {
                        season_id: r.season_id,
                        title: r.title,
                        cover: r.cover,
                        index: r.index,
                        time: r.time,
                        published: r.published,
                    })
                    .collect(),
            })
            .collect())
    }

    pub async fn pgc_conditions(&self, season_type: u32) -> Res<FfiPgcConditions> {
        let c = self.pgc.conditions(season_type).await?;
        let choices =
            |l: Vec<(String, String)>| l.into_iter().map(|(value, name)| FfiPgcChoice { value, name }).collect();
        Ok(FfiPgcConditions {
            filters: c
                .filters
                .into_iter()
                .map(|f| FfiPgcFilter { field: f.field, name: f.name, values: choices(f.values) })
                .collect(),
            orders: choices(c.orders),
        })
    }

    /// Page `page` (from 1) of a 专区's index; `filters` are the chosen (field, value) pairs.
    pub async fn pgc_index(
        &self,
        season_type: u32,
        filters: Vec<FfiPgcChoice>,
        order: String,
        page: u32,
    ) -> Res<FfiPgcPage> {
        let f: Vec<(String, String)> = filters.into_iter().map(|c| (c.name, c.value)).collect();
        let (list, has_more) = self.pgc.index(season_type, &f, &order, page).await?;
        Ok(FfiPgcPage {
            seasons: list
                .into_iter()
                .map(|s| FfiPgcSeason {
                    season_id: s.season_id,
                    title: s.title,
                    subtitle: s.subtitle,
                    cover: s.cover,
                    score: s.score,
                    badge: s.badge,
                    index_show: s.index_show,
                    order_text: s.order_text,
                })
                .collect(),
            has_more,
        })
    }

    pub async fn pgc_season(&self, season_id: u64) -> Res<FfiPgcDetail> {
        let d = self.pgc.season(season_id).await?;
        Ok(FfiPgcDetail {
            season_id: d.season_id,
            title: d.title,
            cover: d.cover,
            evaluate: d.evaluate,
            score: d.score,
            areas: d.areas,
            styles: d.styles,
            pub_time: d.pub_time,
            episodes: d
                .episodes
                .into_iter()
                .map(|e| FfiPgcEpisode {
                    ep_id: e.ep_id,
                    aid: e.aid,
                    bvid: e.bvid,
                    cid: e.cid,
                    title: e.title,
                    long_title: e.long_title,
                    badge: e.badge,
                    duration_ms: e.duration_ms,
                })
                .collect(),
            allow_download: d.allow_download,
        })
    }

    /// An episode's streams when the account may watch all of it; otherwise an error saying to
    /// open it on B站's site or app.
    pub async fn pgc_play(&self, ep_id: u64, cid: u64, qn: u32) -> Res<FfiPlayback> {
        Ok(self.pgc.play(ep_id, cid, qn).await?.into())
    }
}
