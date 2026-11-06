//! 关注: followed UPs' videos and posts, and copying the following list into the profile.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use rubi_api::{DynamicApi, FeedApi, FollowingsApi};
use rubi_model::{Dynamic, FollowedUp, Following as FollowedAccount, FollowingUp, VideoCard, now_secs};

use crate::error::Result;
use crate::ports::{FeedbackSink, RuleSource, keep_cards};
use crate::videos::CardCache;

/// Following lists are fetched at most this many pages (50 per page).
const MAX_FOLLOWING_PAGES: u32 = 40;

/// Pages of the followed feed read back to find who posted when (20 posts a page).
const LOOKBACK_PAGES: usize = 5;
/// …or until posts this old are reached.
const LOOKBACK_SECS: i64 = 7 * 24 * 3600;

/// Every followed UP, those who posted lately first.
#[derive(Debug, Clone, Default)]
pub struct AllFollowings {
    pub ups: Vec<FollowingUp>,
    /// How far back posts were read (Unix seconds): an UP without `last_post_at` has not
    /// posted since then.
    pub checked_since: i64,
}

/// A page of followed UPs' videos.
#[derive(Debug, Clone, Default)]
pub struct FollowedPage {
    pub cards: Vec<VideoCard>,
    pub next: Option<String>,
    pub hidden: usize,
}

pub struct Following {
    feed: Arc<dyn FeedApi>,
    dynamics: Arc<dyn DynamicApi>,
    followings: Arc<dyn FollowingsApi>,
    rules: Arc<dyn RuleSource>,
    feedback: Arc<dyn FeedbackSink>,
    cards: Arc<CardCache>,
}

impl Following {
    pub(crate) fn new(
        feed: Arc<dyn FeedApi>,
        dynamics: Arc<dyn DynamicApi>,
        followings: Arc<dyn FollowingsApi>,
        rules: Arc<dyn RuleSource>,
        feedback: Arc<dyn FeedbackSink>,
        cards: Arc<CardCache>,
    ) -> Self {
        Self { feed, dynamics, followings, rules, feedback, cards }
    }

    /// One page of videos from followed UPs, filtered like every list.
    pub async fn videos(&self, offset: Option<&str>) -> Result<FollowedPage> {
        let (cards, next) = self.feed.followed_videos(offset).await?;
        let (cards, hidden) = keep_cards(self.rules.as_ref(), cards)?;
        self.cards.remember(&cards);
        Ok(FollowedPage { cards, next, hidden })
    }

    /// Followed UPs with recent posts, for the bar above the feed.
    pub async fn ups(&self) -> Result<Vec<FollowedUp>> {
        Ok(self.dynamics.followed_ups().await?)
    }

    /// One UP's posts of every kind. A post whose video the rules block is dropped; other
    /// kinds are kept. Also returns the next offset.
    /// An UP's posts matching `keyword` (空间内搜索), page from 1, the rule table applied as in
    /// [`up_posts`](Self::up_posts); also whether more follow.
    pub async fn search_up_posts(&self, mid: u64, keyword: &str, page: u32) -> Result<(Vec<Dynamic>, bool)> {
        let (items, more) = self.dynamics.search_up(mid, keyword.trim(), page.max(1)).await?;
        Ok((self.kept(items)?, more))
    }

    /// Every followed UP's posts, newest first, the rule table applied as in
    /// [`up_posts`](Self::up_posts); also the next offset.
    pub async fn posts(&self, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)> {
        let (items, next) = self.dynamics.followed_posts(offset).await?;
        Ok((self.kept(items)?, next))
    }

    pub async fn up_posts(&self, mid: u64, offset: Option<&str>) -> Result<(Vec<Dynamic>, Option<String>)> {
        let (items, next) = self.dynamics.up_dynamics(mid, offset).await?;
        Ok((self.kept(items)?, next))
    }

    /// Drops posts whose video the rule table blocks, and remembers the videos left.
    fn kept(&self, items: Vec<Dynamic>) -> Result<Vec<Dynamic>> {
        let pipeline = self.rules.card_filter();
        let kept: Vec<Dynamic> =
            items.into_iter().filter(|d| d.video.as_ref().is_none_or(|v| !pipeline.judge(v).is_blocked())).collect();
        let videos: Vec<VideoCard> = kept
            .iter()
            .flat_map(|d| d.video.iter().chain(d.orig.iter().flat_map(|o| o.video.iter())))
            .cloned()
            .collect();
        self.cards.remember(&videos);
        Ok(kept)
    }

    /// Copies the account's following list into the follow factor. Returns how many.
    pub async fn sync_to_profile(&self) -> Result<usize> {
        let mut all: Vec<u64> = self.following_list().await?.into_iter().map(|f| f.mid).collect();
        all.sort_unstable();
        all.dedup();
        self.feedback.set_followed(&all)?;
        Ok(all.len())
    }

    /// 全部关注: every followed UP. Who posted when is read from the followed feed a few pages
    /// back (no request per UP); those seen there come first, latest post first, the rest in
    /// the order they were followed.
    pub async fn all(&self) -> Result<AllFollowings> {
        let list = self.following_list().await?;
        let now = now_secs();
        let mut latest: HashMap<u64, i64> = HashMap::new();
        let mut since = now;
        let mut offset: Option<String> = None;
        for _ in 0..LOOKBACK_PAGES {
            let (posts, next) = self.dynamics.followed_posts(offset.as_deref()).await?;
            for p in &posts {
                let at = latest.entry(p.author_mid).or_insert(p.published_at);
                *at = (*at).max(p.published_at);
                since = since.min(p.published_at);
            }
            match next {
                Some(n) if !posts.is_empty() && since > now - LOOKBACK_SECS => offset = Some(n),
                _ => break,
            }
        }
        // The 有更新 marks are a nicety; the list stands without them.
        let updated: HashSet<u64> = self
            .dynamics
            .followed_ups()
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|u| u.has_update)
            .map(|u| u.mid)
            .collect();
        Ok(AllFollowings { ups: order_followings(list, &latest, &updated), checked_since: since })
    }

    async fn following_list(&self) -> Result<Vec<FollowedAccount>> {
        let mut all = Vec::new();
        for pn in 1..=MAX_FOLLOWING_PAGES {
            let (page, total) = self.followings.followings(pn).await?;
            let done = page.is_empty();
            all.extend(page);
            if done || all.len() as u32 >= total {
                break;
            }
        }
        Ok(all)
    }
}

/// Those with a known latest post first (newest first), then the rest by follow time.
fn order_followings(
    list: Vec<FollowedAccount>,
    latest: &HashMap<u64, i64>,
    updated: &HashSet<u64>,
) -> Vec<FollowingUp> {
    let mut seen = HashSet::new();
    let mut rows: Vec<(FollowingUp, i64)> = list
        .into_iter()
        .filter(|f| seen.insert(f.mid))
        .map(|f| {
            let up = FollowingUp {
                mid: f.mid,
                last_post_at: latest.get(&f.mid).copied(),
                has_update: updated.contains(&f.mid),
                name: f.name,
                face: f.face,
            };
            (up, f.followed_at)
        })
        .collect();
    rows.sort_by(|(a, af), (b, bf)| match (a.last_post_at, b.last_post_at) {
        (Some(x), Some(y)) => y.cmp(&x),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => bf.cmp(af),
    });
    rows.into_iter().map(|(u, _)| u).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(mid: u64, followed_at: i64) -> FollowedAccount {
        FollowedAccount { mid, name: format!("u{mid}"), followed_at, ..Default::default() }
    }

    #[test]
    fn recent_posters_first_then_by_follow_time() {
        let latest = HashMap::from([(3, 500), (1, 900)]);
        let updated = HashSet::from([1]);
        let order = order_followings(vec![f(1, 10), f(2, 50), f(3, 20), f(4, 80), f(2, 50)], &latest, &updated);
        let mids: Vec<u64> = order.iter().map(|u| u.mid).collect();
        assert_eq!(mids, [1, 3, 4, 2], "posters newest first, then newest follow; duplicates dropped");
        assert!(order[0].has_update && !order[1].has_update);
        assert_eq!(order[2].last_post_at, None);
    }
}
