//! The home feed and the feedback that shapes it.

use rubi_model::{DislikeReason, Signal};

use crate::{FfiBreakdown, FfiFilterLogEntry, FfiRefreshReport, FfiSignal, FfiVideoCard, Res, RubiCore};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    pub async fn refresh_feed(&self, per_source: u32) -> Res<FfiRefreshReport> {
        let r = self.home.refresh(per_source as usize).await?;
        Ok(FfiRefreshReport { kept: r.kept as u32, blocked: r.blocked as u32, errors: r.errors })
    }

    /// Call before rebuilding the home list (pull to refresh) so earlier cards may come back.
    pub fn new_feed_session(&self) {
        self.home.new_session();
    }

    /// The next `n` cards not yet shown since `new_feed_session`; empty when the pool is used up.
    pub fn next_page(&self, n: u32) -> Res<Vec<FfiVideoCard>> {
        Ok(self.home.next_page(n as usize)?.iter().map(|r| FfiVideoCard::of(&r.card, Some(&r.score))).collect())
    }

    pub fn explain(&self, bvid: String) -> Res<Option<FfiBreakdown>> {
        Ok(self.home.explain(&bvid)?.map(Into::into))
    }

    pub async fn record(&self, bvid: String, signal: FfiSignal) -> Res<()> {
        let signal = match signal {
            FfiSignal::Click => Signal::Click,
            FfiSignal::QuickExit => Signal::QuickExit,
            FfiSignal::Completion { ratio } => Signal::Completion { ratio },
            FfiSignal::Rewatch => Signal::Rewatch,
            FfiSignal::DislikeUp => Signal::Dislike { reason: DislikeReason::Up },
            FfiSignal::DislikeZone => Signal::Dislike { reason: DislikeReason::Zone },
            FfiSignal::DislikeTag { tag } => Signal::Dislike { reason: DislikeReason::Tag(tag) },
            FfiSignal::DislikeTitle => Signal::Dislike { reason: DislikeReason::Title },
        };
        Ok(self.home.record(&bvid, signal).await?)
    }

    pub fn filter_log(&self, limit: u32) -> Res<Vec<FfiFilterLogEntry>> {
        Ok(self
            .home
            .filter_log(limit as usize)?
            .into_iter()
            .map(|e| FfiFilterLogEntry {
                at: e.at,
                rule_id: e.rule_id,
                category: e.category,
                title: e.title,
                source: e.source,
                reason: e.reason,
            })
            .collect())
    }
}
