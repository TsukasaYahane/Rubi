//! 影视专区: 番剧, 电影, 纪录片 and 电视剧. Lists and details are read as B站 serves them; an
//! episode plays only when the server grants the whole episode to this account, never a trial
//! or DRM-protected stream, and quality stops at the free levels. Offline copies are not offered
//! for these (only official clients may keep them).

use std::sync::Arc;

use rubi_api::{PgcApi, PgcConditions, PgcDay, PgcSeason, PgcSeasonDetail};

use crate::error::{CoreError, Result};
use crate::videos::{MAX_FREE_QN, Playback};

/// The four 专区 as (name, season_type).
pub const PGC_ZONES: &[(&str, u32)] = &[("番剧", 1), ("电影", 2), ("纪录片", 3), ("电视剧", 5)];

/// What to tell someone whose account cannot watch an episode.
pub const NEED_RIGHTS: &str = "需要开通大会员或购买才能观看，请前往 B 站网站或官方 App 开通后再来";
pub const DRM_ONLY: &str = "这部影片受版权保护，只能在 B 站官方客户端观看";

pub struct Pgc {
    api: Arc<dyn PgcApi>,
}

impl Pgc {
    pub(crate) fn new(api: Arc<dyn PgcApi>) -> Self {
        Self { api }
    }

    /// 近期上映: the 番剧 timeline; other 专区 have none (their index sorted by date stands in).
    pub async fn timeline(&self, season_type: u32) -> Result<Vec<PgcDay>> {
        if season_type != 1 {
            return Ok(Vec::new());
        }
        Ok(self.api.timeline(1).await?)
    }

    pub async fn conditions(&self, season_type: u32) -> Result<PgcConditions> {
        Ok(self.api.conditions(season_type).await?)
    }

    pub async fn index(
        &self,
        season_type: u32,
        filters: &[(String, String)],
        order: &str,
        page: u32,
    ) -> Result<(Vec<PgcSeason>, bool)> {
        Ok(self.api.index(season_type, filters, order, page).await?)
    }

    pub async fn season(&self, season_id: u64) -> Result<PgcSeasonDetail> {
        Ok(self.api.season(season_id).await?)
    }

    /// An episode's streams, when the account may watch all of it; otherwise why not.
    pub async fn play(&self, ep_id: u64, cid: u64, qn: u32) -> Result<Playback> {
        let p = self.api.play(ep_id, qn.min(MAX_FREE_QN)).await?;
        if p.drm {
            return Err(CoreError::Invalid(DRM_ONLY.into()));
        }
        let Some(mut url) = p.url.filter(|_| !p.preview) else {
            return Err(CoreError::Invalid(NEED_RIGHTS.into()));
        };
        url.accept.retain(|(qn, _)| *qn <= MAX_FREE_QN);
        url.video.retain(|s| s.id <= MAX_FREE_QN);
        Ok(Playback { cid, url, headers: self.api.stream_headers() })
    }
}
