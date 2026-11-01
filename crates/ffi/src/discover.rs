//! 热门, 排行, related videos, search, hot words and emotes.

use rubi_model::VideoCard;

use crate::{FfiEmote, FfiHotWords, FfiSearchResult, FfiVideoCard, Res, RubiCore};

fn cards(list: &[VideoCard]) -> Vec<FfiVideoCard> {
    list.iter().map(|c| FfiVideoCard::of(c, None)).collect()
}

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    pub async fn popular(&self, page: u32) -> Res<Vec<FfiVideoCard>> {
        Ok(cards(&self.discover.popular(page).await?))
    }

    pub async fn ranking(&self) -> Res<Vec<FfiVideoCard>> {
        Ok(cards(&self.discover.ranking().await?))
    }

    pub async fn related(&self, bvid: String) -> Res<Vec<FfiVideoCard>> {
        Ok(cards(&self.discover.related(&bvid).await?))
    }

    pub async fn search(&self, keyword: String, page: u32) -> Res<FfiSearchResult> {
        let (found, hidden) = self.discover.search(&keyword, page).await?;
        Ok(FfiSearchResult { cards: cards(&found), hidden: hidden as u32 })
    }

    pub async fn hot_words(&self) -> Res<FfiHotWords> {
        let (words, hidden) = self.discover.hot_words(10).await?;
        Ok(FfiHotWords { words: words.into_iter().map(|w| w.keyword).collect(), hidden: hidden as u32 })
    }

    /// The default emote set, for rendering `[doge]`-style tokens as images.
    pub async fn emotes(&self) -> Res<Vec<FfiEmote>> {
        Ok(self.emotes.all().await?.iter().map(|e| FfiEmote { text: e.text.clone(), url: e.url.clone() }).collect())
    }
}
