//! The composition root: the only place that knows the concrete API clients and the store.
//! It wires every service once and hands them out; callers talk to one service at a time and
//! never reach through one to another.

use std::path::Path;
use std::sync::Arc;

use rubi_api::{
    ActionApi, ChatApi, FeedApi, HttpClient, LibraryApi, NoticeApi, Session, WebAccount, WebArticle, WebAuth,
    WebComment, WebDanmaku, WebDynamic, WebEmote, WebFeed, WebMedia, WebMessages, WebSearch, WebSpace, WebStein,
    WebVideo,
};
use rubi_recommend::Source;
use rubi_store::SqliteStore;

use crate::account::Account;
use crate::actions::Actions;
use crate::comments::Comments;
use crate::danmaku::Danmakus;
use crate::discover::Discover;
use crate::downloads::Downloads;
use crate::emotes::Emotes;
use crate::error::Result;
use crate::feed::{FeedService, Seeded};
use crate::following::Following;
use crate::home::Home;
use crate::library::Library;
use crate::messages::Messages;
use crate::playlist::{Playlist, Playlists};
use crate::ports::{FeedbackSink, RuleSource};
use crate::preferences::Preferences;
use crate::reading::Reading;
use crate::sources::{
    BiliRecommendSource, FollowedUpsSource, PopularSource, PreferredZoneSource, RankingSource, RelatedSource,
};
use crate::space::Space;
use crate::stein::Stein;
use crate::videos::{CardCache, Videos};

/// Every service, built once per process.
pub struct Services {
    pub account: Arc<Account>,
    pub home: Home,
    pub videos: Arc<Videos>,
    pub danmaku: Danmakus,
    pub stein: Stein,
    pub comments: Comments,
    pub discover: Discover,
    pub following: Following,
    pub space: Space,
    pub actions: Actions,
    pub library: Library,
    pub messages: Messages,
    pub playlists: Playlists,
    pub preferences: Preferences,
    pub emotes: Emotes,
    pub reading: Reading,
    pub downloads: Downloads,
}

impl Services {
    /// Opens (or creates) the database in `data_dir` and builds all services; offline copies
    /// go under `data_dir/downloads`.
    pub fn open(data_dir: impl AsRef<Path>, session: Session) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let http = Arc::new(HttpClient::new(session)?);
        let store = Arc::new(SqliteStore::open(data_dir.join("rubi.db"))?);
        let cards = Arc::new(CardCache::default());

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
        let feed =
            Arc::new(FeedService::new(store.clone(), sources, Seeded { related: Some(related), zones: Some(zones) })?);
        let rules: Arc<dyn RuleSource> = feed.clone();
        let feedback: Arc<dyn FeedbackSink> = feed.clone();

        let web_account = Arc::new(WebAccount::new(http.clone()));
        let actions_api: Arc<dyn ActionApi> = web_account.clone();
        let library_api: Arc<dyn LibraryApi> = web_account;
        let messages = Arc::new(WebMessages::new(http.clone()));
        let notices: Arc<dyn NoticeApi> = messages.clone();
        let chat: Arc<dyn ChatApi> = messages;

        let account = Arc::new(Account::new(http.clone(), Arc::new(WebAuth::new(http.clone()))));
        let videos = Arc::new(Videos::new(Arc::new(WebVideo::new(http.clone())), cards.clone()));
        let playlist = Arc::new(Playlist::new(store.clone()));
        let downloads =
            Downloads::new(videos.clone(), Arc::new(WebMedia::new(http.clone())), store, data_dir.join("downloads"));

        Ok(Self {
            home: Home::new(feed.clone(), videos.clone(), cards.clone()),
            danmaku: Danmakus::new(Arc::new(WebDanmaku::new(http.clone())), rules.clone()),
            stein: Stein::new(Arc::new(WebStein::new(http.clone()))),
            comments: Comments::new(Arc::new(WebComment::new(http.clone())), rules.clone()),
            discover: Discover::new(
                feed_api.clone(),
                Arc::new(WebSearch::new(http.clone())),
                rules.clone(),
                feedback.clone(),
                cards.clone(),
            ),
            following: Following::new(
                feed_api,
                Arc::new(WebDynamic::new(http.clone())),
                library_api.clone(),
                rules.clone(),
                feedback.clone(),
                cards.clone(),
            ),
            space: Space::new(Arc::new(WebSpace::new(http.clone())), cards.clone()),
            actions: Actions::new(actions_api.clone(), videos.clone(), feedback),
            library: Library::new(library_api, actions_api, account.clone(), videos.clone(), cards.clone()),
            messages: Messages::new(notices, chat, rules.clone()),
            playlists: Playlists::new(playlist.clone(), videos.clone(), cards),
            preferences: Preferences::new(feed, playlist),
            reading: Reading::new(Arc::new(WebArticle::new(http.clone())), rules.clone()),
            downloads,
            emotes: Emotes::new(Arc::new(WebEmote::new(http))),
            account,
            videos,
        })
    }
}
