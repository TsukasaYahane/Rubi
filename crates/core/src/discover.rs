//! Lists outside the home feed: 热门, 排行, related videos, search and hot words. Every list goes
//! through the rule table.

use std::sync::Arc;

use rubi_api::{FeedApi, HotWord, SearchApi, SearchOrder, UserHits};
use rubi_model::{FeedbackEvent, Signal, VideoCard, now_secs};

use crate::error::Result;
use crate::ports::{FeedbackSink, RuleSource, keep_cards};
use crate::social::filter_hot_words;
use crate::videos::CardCache;

pub struct Discover {
    feed: Arc<dyn FeedApi>,
    search: Arc<dyn SearchApi>,
    rules: Arc<dyn RuleSource>,
    feedback: Arc<dyn FeedbackSink>,
    cards: Arc<CardCache>,
}

impl Discover {
    pub(crate) fn new(
        feed: Arc<dyn FeedApi>,
        search: Arc<dyn SearchApi>,
        rules: Arc<dyn RuleSource>,
        feedback: Arc<dyn FeedbackSink>,
        cards: Arc<CardCache>,
    ) -> Self {
        Self { feed, search, rules, feedback, cards }
    }

    pub async fn popular(&self, page: u32) -> Result<Vec<VideoCard>> {
        let cards = self.feed.popular(page.max(1), 20).await?;
        self.shown(cards).map(|(kept, _)| kept)
    }

    pub async fn ranking(&self) -> Result<Vec<VideoCard>> {
        let cards = self.feed.ranking(0).await?;
        self.shown(cards).map(|(kept, _)| kept)
    }

    /// A zone's 推荐 (v2 main `region`), page from 1, through the rule table.
    pub async fn zone_feed(&self, region: u32, page: u32) -> Result<Vec<VideoCard>> {
        let cards = self.feed.region_feed(region, page.max(1), 20).await?;
        self.shown(cards).map(|(kept, _)| kept)
    }

    /// A zone's 热门: its ranking (old parent tid `rid`), through the rule table.
    pub async fn zone_ranking(&self, rid: u32) -> Result<Vec<VideoCard>> {
        let cards = self.feed.ranking(rid).await?;
        self.shown(cards).map(|(kept, _)| kept)
    }

    pub async fn related(&self, bvid: &str) -> Result<Vec<VideoCard>> {
        let cards = self.feed.related(bvid).await?;
        let (kept, _) = self.shown(cards)?;
        Ok(kept.into_iter().filter(|c| c.bvid != bvid).collect())
    }

    /// One page of results and how many the rules hid; the term feeds the profile.
    pub async fn search(&self, keyword: &str, page: u32) -> Result<(Vec<VideoCard>, usize)> {
        let cards = self.search.videos(keyword, page.max(1), SearchOrder::Relevance).await?;
        let found = self.shown(cards)?;
        self.feedback.record(FeedbackEvent {
            signal: Signal::Search { term: keyword.to_owned() },
            item: None,
            at: now_secs(),
        })?;
        Ok(found)
    }

    /// UPs whose name or alias fits, without those the rule table blocks.
    pub async fn search_users(&self, keyword: &str, page: u32) -> Result<UserHits> {
        let mut hits = self.search.users(keyword, page.max(1)).await?;
        let blocked = self.rules.rules().up_blacklist.clone();
        hits.users.retain(|u| !blocked.contains(&u.mid));
        Ok(hits)
    }

    /// 热搜 without the promoted ones; also how many were dropped.
    pub async fn hot_words(&self, limit: u32) -> Result<(Vec<HotWord>, usize)> {
        let words = self.search.hot_words(limit).await?;
        Ok(filter_hot_words(&self.rules.rules(), words))
    }

    fn shown(&self, cards: Vec<VideoCard>) -> Result<(Vec<VideoCard>, usize)> {
        let (kept, hidden) = keep_cards(self.rules.as_ref(), cards)?;
        self.cards.remember(&kept);
        Ok((kept, hidden))
    }
}
