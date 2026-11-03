//! 合集 subscriptions.

use crate::{FfiVideoCard, Res, RubiCore};

/// A subscribed 合集 under 我的 › 订阅的合集.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSeasonSub {
    pub id: u64,
    pub title: String,
    pub cover: String,
    pub count: u32,
    pub up_mid: u64,
    pub up_name: String,
    /// Videos added since it was last opened.
    pub new: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSeasonPage {
    pub cards: Vec<FfiVideoCard>,
    pub has_more: bool,
}

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    pub async fn season_subscriptions(&self) -> Res<Vec<FfiSeasonSub>> {
        Ok(self
            .seasons
            .subscriptions()
            .await?
            .into_iter()
            .map(|s| FfiSeasonSub {
                id: s.season.id,
                title: s.season.title,
                cover: s.season.cover,
                count: s.season.count,
                up_mid: s.season.up_mid,
                up_name: s.season.up_name,
                new: s.new,
            })
            .collect())
    }

    pub async fn season_subscribed(&self, season_id: u64) -> Res<bool> {
        Ok(self.seasons.is_subscribed(season_id).await?)
    }

    pub async fn subscribe_season(&self, season_id: u64, on: bool) -> Res<()> {
        Ok(self.seasons.subscribe(season_id, on).await?)
    }

    /// Page `pn` (from 1) of a season's videos; opening it marks them seen.
    pub async fn season_videos(&self, up_mid: u64, season_id: u64, pn: u32) -> Res<FfiSeasonPage> {
        let (cards, has_more) = self.seasons.videos(up_mid, season_id, pn).await?;
        Ok(FfiSeasonPage { cards: cards.iter().map(|c| FfiVideoCard::of(c, None)).collect(), has_more })
    }
}
