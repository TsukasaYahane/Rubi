//! 热门, 排行, related videos, search, hot words and emotes.

use rubi_model::VideoCard;

use crate::{DiscoverService, EmotesService, FfiEmote, FfiHotWords, FfiSearchResult, FfiVideoCard, Res};

/// 影视 works handed to the search page at most (it shows three and folds the rest).
const PGC_HITS: usize = 20;

/// A 番剧 / 电影 / 纪录片 / 电视剧 found by a search.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPgcHit {
    pub season_id: u64,
    pub title: String,
    pub cover: String,
    pub kind: String,
    pub badge: String,
    pub score: String,
    pub index_show: String,
    pub styles: String,
}

fn cards(list: &[VideoCard]) -> Vec<FfiVideoCard> {
    list.iter().map(|c| FfiVideoCard::of(c, None)).collect()
}

#[uniffi::export(async_runtime = "tokio")]
impl DiscoverService {
    pub async fn popular(&self, page: u32) -> Res<Vec<FfiVideoCard>> {
        Ok(cards(&self.discover.popular(page).await?))
    }

    pub async fn ranking(&self) -> Res<Vec<FfiVideoCard>> {
        Ok(cards(&self.discover.ranking().await?))
    }

    /// The zones on the home bar after 推荐 / 热门 / 排行.
    pub fn home_zones(&self) -> Vec<FfiZone> {
        rubi_core::HOME_ZONES
            .iter()
            .map(|(name, region, rid)| FfiZone { name: (*name).to_owned(), region: *region, rid: *rid })
            .collect()
    }

    /// A zone's 推荐, page from 1.
    pub async fn zone_feed(&self, region: u32, page: u32) -> Res<Vec<FfiVideoCard>> {
        Ok(cards(&self.discover.zone_feed(region, page).await?))
    }

    /// A zone's 热门 ranking.
    pub async fn zone_ranking(&self, rid: u32) -> Res<Vec<FfiVideoCard>> {
        Ok(cards(&self.discover.zone_ranking(rid).await?))
    }

    pub async fn related(&self, bvid: String) -> Res<Vec<FfiVideoCard>> {
        Ok(cards(&self.discover.related(&bvid).await?))
    }

    pub async fn search(&self, keyword: String, page: u32) -> Res<FfiSearchResult> {
        let (found, hidden) = self.discover.search(&keyword, page).await?;
        Ok(FfiSearchResult { cards: cards(&found), hidden: hidden as u32 })
    }

    /// 影视 works the term names (番剧 first), shown above the results with a way into them.
    pub async fn search_pgc(&self, keyword: String) -> Res<Vec<FfiPgcHit>> {
        Ok(self
            .discover
            .search_pgc(&keyword, PGC_HITS)
            .await?
            .into_iter()
            .map(|h| FfiPgcHit {
                season_id: h.season_id,
                title: h.title,
                cover: h.cover,
                kind: h.kind,
                badge: h.badge,
                score: h.score,
                index_show: h.index_show,
                styles: h.styles,
            })
            .collect())
    }

    /// UPs whose name or alias fits (「TES」 finds 滔搏电子竞技俱乐部), page from 1.
    pub async fn search_users(&self, keyword: String, page: u32) -> Res<FfiUserHits> {
        let hits = self.discover.search_users(&keyword, page).await?;
        Ok(FfiUserHits {
            users: hits
                .users
                .into_iter()
                .map(|u| FfiUserHit {
                    mid: u.mid,
                    name: u.name,
                    face: u.face,
                    sign: u.sign,
                    fans: u.fans,
                    videos: u.videos,
                    level: u.level,
                    live_room: u.live_room,
                })
                .collect(),
            has_more: hits.has_more,
        })
    }

    pub async fn hot_words(&self) -> Res<FfiHotWords> {
        let (words, hidden) = self.discover.hot_words(10).await?;
        Ok(FfiHotWords { words: words.into_iter().map(|w| w.keyword).collect(), hidden: hidden as u32 })
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl EmotesService {
    /// The default emote set, for rendering `[doge]`-style tokens as images.
    pub async fn emotes(&self) -> Res<Vec<FfiEmote>> {
        Ok(self.emotes.all().await?.iter().map(|e| FfiEmote { text: e.text.clone(), url: e.url.clone() }).collect())
    }
}

/// An UP found by search.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiUserHit {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub sign: String,
    pub fans: u64,
    pub videos: u64,
    pub level: u32,
    /// The live room when on air, else 0.
    pub live_room: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiUserHits {
    pub users: Vec<FfiUserHit>,
    pub has_more: bool,
}

/// A zone on the home bar.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiZone {
    pub name: String,
    /// v2 main region, for its 推荐.
    pub region: u32,
    /// Old parent tid, for its 热门.
    pub rid: u32,
}
