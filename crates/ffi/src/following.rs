//! 关注: followed UPs' videos and posts.

use crate::{
    FfiDynamic, FfiDynamicPage, FfiFollowPage, FfiFollowedUp, FfiFollowing, FfiFollowings, FfiVideoCard, Res, RubiCore,
};

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// One page of followed UPs' videos; pass `next` back for the following page.
    pub async fn followed_feed(&self, offset: Option<String>) -> Res<FfiFollowPage> {
        let page = self.following.videos(offset.as_deref()).await?;
        Ok(FfiFollowPage {
            cards: page.cards.iter().map(|c| FfiVideoCard::of(c, None)).collect(),
            next: page.next,
            hidden: page.hidden as u32,
        })
    }

    /// Followed UPs with recent posts, for the bar above the 关注 feed.
    pub async fn followed_ups(&self) -> Res<Vec<FfiFollowedUp>> {
        Ok(self.following.ups().await?.into_iter().map(Into::into).collect())
    }

    /// One UP's posts of every kind; pass `next` back for more.
    pub async fn up_dynamics(&self, mid: u64, offset: Option<String>) -> Res<FfiDynamicPage> {
        let (items, next) = self.following.up_posts(mid, offset.as_deref()).await?;
        Ok(FfiDynamicPage { items: items.iter().map(FfiDynamic::of).collect(), next })
    }

    /// 空间内搜索: an UP's posts (videos, 专栏, posts) matching `keyword`, page from 1; `next` is
    /// the following page number, absent at the end.
    pub async fn up_search_posts(&self, mid: u64, keyword: String, page: u32) -> Res<FfiDynamicPage> {
        let (items, more) = self.following.search_up_posts(mid, &keyword, page).await?;
        Ok(FfiDynamicPage {
            items: items.iter().map(FfiDynamic::of).collect(),
            next: more.then(|| (page.max(1) + 1).to_string()),
        })
    }

    /// 全部关注: every followed UP, those who posted lately first (with when).
    pub async fn all_followings(&self) -> Res<FfiFollowings> {
        let all = self.following.all().await?;
        Ok(FfiFollowings {
            ups: all
                .ups
                .into_iter()
                .map(|u| FfiFollowing {
                    mid: u.mid,
                    name: u.name,
                    face: u.face,
                    last_post_at: u.last_post_at.unwrap_or(0),
                    has_update: u.has_update,
                })
                .collect(),
            checked_since: all.checked_since,
        })
    }

    /// Copies the following list into the recommender's 关注 UP factor. Returns how many.
    pub async fn sync_followings(&self) -> Res<u32> {
        Ok(self.following.sync_to_profile().await? as u32)
    }
}
