use std::path::Path;
use std::sync::Arc;

use rubi_api::{
    ActionApi, AuthApi, ChatApi, CommentApi, CommentSort, DEFAULT_PACKAGES, DanmakuApi, Emote, EmoteApi, FeedApi,
    HotWord, HttpClient, LibraryApi, NoticeApi, SearchApi, SearchOrder, Session, SpaceApi, Triple, UpOrder, VideoApi,
    WebAccount, WebAuth, WebComment, WebDanmaku, WebEmote, WebFeed, WebMessages, WebSearch, WebSpace, WebVideo,
};
use rubi_model::{ChatSession, Comment, FeedbackEvent, ItemRef, Signal, UpProfile, VideoCard, now_secs};
use rubi_recommend::Source;
use rubi_store::SqliteStore;
use tokio::sync::Mutex as AsyncMutex;

use crate::error::Result;
use crate::feed::{FeedService, Seeded};
use crate::social::{self, FilteredComments};
use crate::sources::{
    BiliRecommendSource, FollowedUpsSource, PopularSource, PreferredZoneSource, RankingSource, RelatedSource,
};

/// Following lists are fetched at most this many pages (50 per page).
const MAX_FOLLOWING_PAGES: u32 = 40;

/// A page of [`App::up_videos`].
#[derive(Debug, Clone, Default)]
pub struct UpUploads {
    pub items: Vec<VideoCard>,
    pub total: u32,
    pub hidden_charging: usize,
    pub has_more: bool,
}

/// Every service, wired once. Hold one per process.
pub struct App {
    pub http: Arc<HttpClient>,
    pub auth: Arc<dyn AuthApi>,
    pub feed_api: Arc<dyn FeedApi>,
    pub video: Arc<dyn VideoApi>,
    pub danmaku: Arc<dyn DanmakuApi>,
    pub comments: Arc<dyn CommentApi>,
    pub search: Arc<dyn SearchApi>,
    pub notices: Arc<dyn NoticeApi>,
    pub chat: Arc<dyn ChatApi>,
    pub actions: Arc<dyn ActionApi>,
    pub library: Arc<dyn LibraryApi>,
    pub emote: Arc<dyn EmoteApi>,
    pub space: Arc<dyn SpaceApi>,
    pub feed: FeedService<SqliteStore>,
    emotes: AsyncMutex<Option<Arc<Vec<Emote>>>>,
}

impl App {
    /// Opens (or creates) the database at `db_path` and builds all services.
    pub fn open(db_path: impl AsRef<Path>, session: Session) -> Result<Self> {
        let http = Arc::new(HttpClient::new(session)?);
        let feed_api: Arc<dyn FeedApi> = Arc::new(WebFeed::new(http.clone()));
        let related = Arc::new(RelatedSource::new(feed_api.clone()));
        let zones = Arc::new(PreferredZoneSource::new(feed_api.clone()));
        let sources: Vec<Arc<dyn Source>> = vec![
            Arc::new(BiliRecommendSource::new(feed_api.clone())),
            Arc::new(FollowedUpsSource::new(feed_api.clone())),
            Arc::new(PopularSource::new(feed_api.clone())),
            Arc::new(RankingSource::new(feed_api.clone())),
            related.clone(),
            zones.clone(),
        ];
        let store = Arc::new(SqliteStore::open(db_path)?);
        let feed = FeedService::new(store, sources, Seeded { related: Some(related), zones: Some(zones) })?;
        let messages = Arc::new(WebMessages::new(http.clone()));
        let account = Arc::new(WebAccount::new(http.clone()));
        Ok(Self {
            auth: Arc::new(WebAuth::new(http.clone())),
            video: Arc::new(WebVideo::new(http.clone())),
            danmaku: Arc::new(WebDanmaku::new(http.clone())),
            comments: Arc::new(WebComment::new(http.clone())),
            search: Arc::new(WebSearch::new(http.clone())),
            notices: messages.clone(),
            chat: messages,
            actions: account.clone(),
            library: account,
            emote: Arc::new(WebEmote::new(http.clone())),
            space: Arc::new(WebSpace::new(http.clone())),
            feed_api,
            http,
            feed,
            emotes: AsyncMutex::new(None),
        })
    }

    /// The default emote set (`[doge]` → image url), fetched once per process.
    pub async fn emotes(&self) -> Result<Arc<Vec<Emote>>> {
        let mut cached = self.emotes.lock().await;
        if let Some(e) = cached.as_ref() {
            return Ok(e.clone());
        }
        let list = Arc::new(self.emote.packages(DEFAULT_PACKAGES).await?);
        *cached = Some(list.clone());
        Ok(list)
    }

    /// 热门 list with ads and blocked items removed.
    pub async fn popular(&self, pn: u32) -> Result<Vec<VideoCard>> {
        let cards = self.feed_api.popular(pn.max(1), 20).await?;
        self.filter_list(cards)
    }

    /// 排行 list with ads and blocked items removed.
    pub async fn ranking(&self) -> Result<Vec<VideoCard>> {
        let cards = self.feed_api.ranking(0).await?;
        self.filter_list(cards)
    }

    /// Videos related to `bvid`, filtered by the rule table (ads, blocked UPs, keywords…).
    pub async fn related(&self, bvid: &str) -> Result<Vec<VideoCard>> {
        let cards = self.feed_api.related(bvid).await?;
        self.filter_list(cards)
    }

    fn filter_list(&self, cards: Vec<VideoCard>) -> Result<Vec<VideoCard>> {
        let pipeline = self.feed.settings().rules.card_pipeline(None, now_secs)?;
        Ok(cards.into_iter().filter(|c| !pipeline.judge(c).is_blocked()).collect())
    }

    fn signal(&self, card: &VideoCard, signal: Signal) -> Result<()> {
        self.feed.record(FeedbackEvent { signal, item: Some(ItemRef::of(card)), at: now_secs() })
    }

    // ----- account actions: each also feeds the recommendation profile -----

    pub async fn like(&self, card: &VideoCard, on: bool) -> Result<()> {
        self.actions.like(&card.bvid, on).await?;
        if on { self.signal(card, Signal::Like) } else { Ok(()) }
    }

    pub async fn coin(&self, card: &VideoCard, count: u32, also_like: bool) -> Result<()> {
        self.actions.coin(&card.bvid, count, also_like).await?;
        self.signal(card, Signal::Coin)
    }

    /// 一键三连; each part that succeeded feeds the profile.
    pub async fn triple(&self, card: &VideoCard) -> Result<Triple> {
        let t = self.actions.triple(&card.bvid).await?;
        let done = [(t.liked, Signal::Like), (t.coined, Signal::Coin), (t.favorited, Signal::Favorite)];
        for (_, s) in done.into_iter().filter(|(ok, _)| *ok) {
            self.signal(card, s)?;
        }
        Ok(t)
    }

    pub async fn favorite(&self, card: &VideoCard, add: &[u64], remove: &[u64]) -> Result<()> {
        self.actions.set_fav(card.aid, add, remove).await?;
        if add.is_empty() { Ok(()) } else { self.signal(card, Signal::Favorite) }
    }

    pub async fn watch_later(&self, card: &VideoCard) -> Result<()> {
        self.actions.watch_later(&card.bvid).await.map_err(Into::into)
    }

    pub async fn follow(&self, card: &VideoCard, on: bool) -> Result<()> {
        self.actions.follow(card.owner.mid, on).await?;
        if on { self.signal(card, Signal::Follow) } else { Ok(()) }
    }

    /// Follow / unfollow from the UP's page, where there is no video to attach.
    pub async fn follow_mid(&self, mid: u64, name: &str, on: bool) -> Result<()> {
        self.actions.follow(mid, on).await?;
        if !on {
            return Ok(());
        }
        let item = ItemRef { up_mid: mid, title: name.to_owned(), ..ItemRef::of(&VideoCard::default()) };
        self.feed.record(FeedbackEvent { signal: Signal::Follow, item: Some(item), at: now_secs() })
    }

    pub async fn up_profile(&self, mid: u64) -> Result<UpProfile> {
        Ok(self.space.profile(mid).await?)
    }

    /// One page of an UP's uploads without 充电专属 videos; also returns the total count and
    /// how many charging-only videos this page hid.
    pub async fn up_videos(&self, mid: u64, pn: u32, order: UpOrder) -> Result<UpUploads> {
        let page = self.space.videos(mid, pn, order).await?;
        let fetched = page.items.len();
        let items: Vec<VideoCard> = page.items.into_iter().filter(|v| !v.charging).map(|v| v.card).collect();
        Ok(UpUploads { hidden_charging: fetched - items.len(), has_more: page.has_more, items, total: page.total })
    }

    /// Copies the account's following list into the follow factor. Returns how many.
    pub async fn sync_followings(&self) -> Result<usize> {
        let mut all = Vec::new();
        for pn in 1..=MAX_FOLLOWING_PAGES {
            let (mids, total) = self.library.followings(pn).await?;
            let done = mids.is_empty();
            all.extend(mids);
            if done || all.len() as u32 >= total {
                break;
            }
        }
        all.sort_unstable();
        all.dedup();
        self.feed.set_followed(&all)?;
        Ok(all.len())
    }

    // ----- search, comments, message center -----

    /// Searches videos, drops ads and blocked items, and records the search term.
    pub async fn search_videos(&self, keyword: &str, page: u32, order: SearchOrder) -> Result<(Vec<VideoCard>, usize)> {
        let cards = self.search.videos(keyword, page, order).await?;
        let pipeline = self.feed.settings().rules.card_pipeline(None, now_secs)?;
        let before = cards.len();
        let kept: Vec<VideoCard> = cards.into_iter().filter(|c| !pipeline.judge(c).is_blocked()).collect();
        let hidden = before - kept.len();
        self.feed.record(FeedbackEvent {
            signal: Signal::Search { term: keyword.to_owned() },
            item: None,
            at: now_secs(),
        })?;
        Ok((kept, hidden))
    }

    pub async fn hot_words(&self, limit: u32) -> Result<(Vec<HotWord>, usize)> {
        let words = self.search.hot_words(limit).await?;
        Ok(social::filter_hot_words(&self.feed.settings().rules, words))
    }

    pub async fn comments(&self, aid: u64, sort: CommentSort, offset: Option<&str>) -> Result<FilteredComments> {
        social::load_comments(self.comments.as_ref(), &self.feed.settings().rules, aid, sort, offset).await
    }

    pub async fn thread(&self, aid: u64, root: u64, max_pages: u32) -> Result<Vec<Comment>> {
        social::load_thread(self.comments.as_ref(), aid, root, max_pages).await
    }

    pub async fn chat_sessions(&self) -> Result<(Vec<ChatSession>, usize)> {
        let sessions = self.chat.sessions().await?;
        Ok(social::filter_sessions(&self.feed.settings().rules, sessions))
    }
}
