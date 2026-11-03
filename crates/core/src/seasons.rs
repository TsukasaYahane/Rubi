//! 合集 subscriptions: subscribing from the video page, the list under 我的 with how many
//! videos came since each was last opened, and a season's videos.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use rubi_api::{SEASON_PAGE, SeasonApi};
use rubi_model::{SeasonSub, VideoCard};
use rubi_store::{KvRepo, SqliteStore};

use crate::error::Result;
use crate::videos::CardCache;

/// Video count of each season when last opened (or first seen), by season id.
const SEEN_KEY: &str = "season_seen";
/// Pages of subscriptions read at most (20 each).
const MAX_PAGES: u32 = 10;

/// A subscribed 合集 and how many videos it gained since it was last opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subscription {
    pub season: SeasonSub,
    pub new: u32,
}

pub struct Seasons<S = SqliteStore> {
    api: Arc<dyn SeasonApi>,
    store: Arc<S>,
    cards: Arc<CardCache>,
    /// Ids subscribed to, once read; kept in step with (un)subscribing here.
    subscribed: Mutex<Option<HashSet<u64>>>,
}

impl<S: KvRepo + Send + Sync> Seasons<S> {
    pub(crate) fn new(api: Arc<dyn SeasonApi>, store: Arc<S>, cards: Arc<CardCache>) -> Self {
        Self { api, store, cards, subscribed: Mutex::default() }
    }

    /// Every 合集 subscribed to, newest subscription first, with its count of new videos. A
    /// season seen for the first time starts at none new.
    pub async fn subscriptions(&self) -> Result<Vec<Subscription>> {
        let list = self.all().await?;
        let mut out = Vec::with_capacity(list.len());
        // Read and written in one go, so opening a season meanwhile is not undone.
        self.store.update_json(SEEN_KEY, |seen: &mut HashMap<u64, u32>| {
            out = list
                .into_iter()
                .map(|season| {
                    let before = *seen.entry(season.id).or_insert(season.count);
                    Subscription { new: season.count.saturating_sub(before), season }
                })
                .collect();
        })?;
        Ok(out)
    }

    pub async fn is_subscribed(&self, season_id: u64) -> Result<bool> {
        if let Some(ids) = self.lock().as_ref() {
            return Ok(ids.contains(&season_id));
        }
        Ok(self.all().await?.iter().any(|s| s.id == season_id))
    }

    pub async fn subscribe(&self, season_id: u64, on: bool) -> Result<()> {
        self.api.subscribe(season_id, on).await?;
        if let Some(ids) = self.lock().as_mut() {
            if on {
                ids.insert(season_id)
            } else {
                ids.remove(&season_id)
            };
        }
        if !on {
            self.store.update_json(SEEN_KEY, |seen: &mut HashMap<u64, u32>| {
                seen.remove(&season_id);
            })?;
        }
        Ok(())
    }

    /// Page `pn` (from 1) of a season's videos and whether more follow. Opening it marks its
    /// videos as seen.
    pub async fn videos(&self, up_mid: u64, season_id: u64, pn: u32) -> Result<(Vec<VideoCard>, bool)> {
        let pn = pn.max(1);
        let (cards, total) = self.api.videos(up_mid, season_id, pn).await?;
        self.cards.remember(&cards);
        if pn == 1 && total > 0 {
            self.store.update_json(SEEN_KEY, |seen: &mut HashMap<u64, u32>| {
                seen.insert(season_id, total);
            })?;
        }
        let more = !cards.is_empty() && pn * SEASON_PAGE < total;
        Ok((cards, more))
    }

    async fn all(&self) -> Result<Vec<SeasonSub>> {
        let mut out = Vec::new();
        for pn in 1..=MAX_PAGES {
            let (page, more) = self.api.subscribed(pn).await?;
            out.extend(page);
            if !more {
                break;
            }
        }
        *self.lock() = Some(out.iter().map(|s| s.id).collect());
        Ok(out)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<HashSet<u64>>> {
        self.subscribed.lock().unwrap_or_else(|e| e.into_inner())
    }
}
