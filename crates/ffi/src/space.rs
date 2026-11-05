//! An UP's page.

use rubi_core::UpOrder;

use crate::{FfiUpOrder, FfiUpProfile, FfiUpUploads, FfiVideoCard, Res, SpaceService};

/// One 专栏 article in an UP's list.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiUpArticle {
    pub cvid: u64,
    pub title: String,
    pub summary: String,
    pub cover: String,
    pub published_at: i64,
    pub views: u64,
    pub likes: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiUpArticles {
    pub items: Vec<FfiUpArticle>,
    pub has_more: bool,
}

#[uniffi::export(async_runtime = "tokio")]
impl SpaceService {
    pub async fn up_profile(&self, mid: u64) -> Res<FfiUpProfile> {
        Ok(self.space.profile(mid).await?.into())
    }

    /// Page `pn` (from 1) of an UP's 专栏, newest first.
    pub async fn up_articles(&self, mid: u64, pn: u32) -> Res<FfiUpArticles> {
        let page = self.space.articles(mid, pn).await?;
        Ok(FfiUpArticles {
            items: page
                .items
                .into_iter()
                .map(|a| FfiUpArticle {
                    cvid: a.cvid,
                    title: a.title,
                    summary: a.summary,
                    cover: a.cover,
                    published_at: a.published_at,
                    views: a.views,
                    likes: a.likes,
                })
                .collect(),
            has_more: page.has_more,
        })
    }

    /// 空间内搜索: page `pn` (from 1) of an UP's uploads matching `keyword`.
    pub async fn up_search_videos(&self, mid: u64, keyword: String, pn: u32) -> Res<FfiUpUploads> {
        let page = self.space.search_uploads(mid, pn, UpOrder::Pubdate, &keyword).await?;
        Ok(FfiUpUploads {
            items: page.items.iter().map(|c| FfiVideoCard::of(c, None)).collect(),
            total: page.total,
            hidden_charging: page.hidden_charging as u32,
            has_more: page.has_more,
        })
    }

    /// Page `pn` (from 1) of an UP's uploads, 充电专属 videos removed.
    pub async fn up_videos(&self, mid: u64, pn: u32, order: FfiUpOrder) -> Res<FfiUpUploads> {
        let order = match order {
            FfiUpOrder::Newest => UpOrder::Pubdate,
            FfiUpOrder::MostViewed => UpOrder::Click,
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
