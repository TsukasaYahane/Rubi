//! An UP's page.

use rubi_api::UpOrder;

use crate::{FfiUpOrder, FfiUpProfile, FfiUpUploads, FfiVideoCard, Res, RubiCore};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    pub async fn up_profile(&self, mid: u64) -> Res<FfiUpProfile> {
        Ok(self.space.profile(mid).await?.into())
    }

    /// Page `pn` (from 1) of an UP's uploads, 充电专属 videos removed.
    pub async fn up_videos(&self, mid: u64, pn: u32, order: FfiUpOrder) -> Res<FfiUpUploads> {
        let order = match order {
            FfiUpOrder::Newest => UpOrder::Pubdate,
            FfiUpOrder::MostViewed => UpOrder::Click,
            FfiUpOrder::MostFavorited => UpOrder::Stow,
        };
        let page = self.space.uploads(mid, pn, order).await?;
        Ok(FfiUpUploads {
            items: page.items.iter().map(|c| FfiVideoCard::of(c, None)).collect(),
            total: page.total,
            hidden_charging: page.hidden_charging as u32,
            has_more: page.has_more,
        })
    }
}
