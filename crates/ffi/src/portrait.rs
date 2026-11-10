//! 竖屏推流 straight from the recommendation pool.

use crate::{FfiPortraitPick, FfiVideoCard, PortraitService, Res};

#[uniffi::export(async_runtime = "tokio")]
impl PortraitService {
    /// Starts a new run of the feed: what was played before may come again.
    pub fn start_portrait(&self) {
        self.portrait.start();
    }

    /// Up to `want` portrait videos to play next, and whether risk control held the sorting back.
    pub async fn next_portrait(&self, want: u32) -> Res<FfiPortraitPick> {
        let picked = self.portrait.next(want as usize).await?;
        Ok(FfiPortraitPick {
            cards: picked.cards.iter().map(|c| FfiVideoCard::of(c, None)).collect(),
            limited: picked.limited,
        })
    }

    /// Sorts up to `budget` pooled videos into portrait and landscape, after the pool fills.
    /// Returns how many were portrait.
    pub async fn sort_pool(&self, budget: u32) -> Res<u32> {
        Ok(self.portrait.sort(budget as usize).await?.portrait as u32)
    }
}
