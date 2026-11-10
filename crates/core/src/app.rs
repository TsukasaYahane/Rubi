//! The composition root: the only place that knows the concrete API clients and the store.
//! It wires every service once and hands them out; callers talk to one service at a time and
//! never reach through one to another.

use std::path::Path;
use std::sync::Arc;

use rubi_api::{
    ChatApi, FavoritesApi, FeedApi, FollowingsApi, HistoryApi, HttpClient, NoticeApi, Session, VideoActionApi,
    WatchLaterApi, WebAccount, WebArticle, WebAuth, WebChannels, WebComment, WebDanmaku, WebDynamic, WebEmote,
    WebEsports, WebFeed, WebLive, WebMedia, WebMessages, WebPgc, WebSearch, WebSeasons, WebSpace, WebStein, WebVideo,
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
use crate::esports::Esports;
use crate::favorites::Favorites;
use crate::feed::{FeedService, Seedable};
use crate::following::Following;
use crate::history::History;
use crate::home::Home;
use crate::lineup::Lineup;
use crate::listening::Listening;
use crate::live::Live;
use crate::messages::Messages;
use crate::pgc::Pgc;
use crate::play_queue::PlayQueue;
use crate::playlist::{Playlist, Playlists};
use crate::portrait::PortraitFeed;
use crate::ports::{
    FeedConfig, FeedbackSink, HomeFeed, PlaylistBackup, PortraitPool, ProfileBackup, RuleSource, SettingsPort,
    ShapeLookup,
};
use crate::preferences::Preferences;
use crate::reading::Reading;
use crate::seasons::Seasons;
use crate::settings::SettingsService;
use crate::sources::{
    BiliRecommendSource, FollowedUpsSource, PopularSource, PreferredZoneSource, RankingSource, RelatedSource,
};
use crate::space::Space;
use crate::stein::Stein;
use crate::videos::{CardCache, Videos};
use crate::watch_later::WatchLater;

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
    pub history: History,
    pub watch_later: WatchLater,
    pub favorites: Favorites,
    pub messages: Messages,
    pub playlists: Playlists,
    pub play_queue: PlayQueue,
    pub listening: Listening,
    pub preferences: Preferences,
    pub emotes: Emotes,
    pub reading: Reading,
    pub downloads: Downloads,
    pub live: Live,
    pub esports: Esports,
    pub pgc: Pgc,
    pub portrait: PortraitFeed,
    pub seasons: Seasons,
}

impl Services {
    /// Opens (or creates) the database in `data_dir` and builds all services; offline copies
    /// go under `data_dir/downloads`.
    pub fn open(data_dir: impl AsRef<Path>, session: Session) -> Result<Self> {
        let data_dir = data_dir.as_ref();
        let http = Arc::new(HttpClient::new(session)?);
        let store = Arc::new(SqliteStore::open(data_dir.join("rubi.db"))?);
        let cards = Arc::new(CardCache::default());
        let settings = Arc::new(SettingsService::open(store.clone())?);
        let rules: Arc<dyn RuleSource> = settings.clone();
        let feed_config: Arc<dyn FeedConfig> = settings.clone();
        let settings_port: Arc<dyn SettingsPort> = settings;

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
        let seeded: Vec<Arc<dyn Seedable>> = vec![related, zones];
        let feed = Arc::new(FeedService::new(store.clone(), feed_config, sources, seeded)?);
        let home_feed: Arc<dyn HomeFeed> = feed.clone();
        let feedback: Arc<dyn FeedbackSink> = feed.clone();
        let portrait_pool: Arc<dyn PortraitPool> = feed.clone();
        let profile: Arc<dyn ProfileBackup> = feed;

        let web_account = Arc::new(WebAccount::new(http.clone()));
        // One client, handed to each service as only the interface it uses.
        let actions_api: Arc<dyn VideoActionApi> = web_account.clone();
        let favorites_api: Arc<dyn FavoritesApi> = web_account.clone();
        let watch_later_api: Arc<dyn WatchLaterApi> = web_account.clone();
        let history_api: Arc<dyn HistoryApi> = web_account.clone();
        let followings_api: Arc<dyn FollowingsApi> = web_account;
        let messages = Arc::new(WebMessages::new(http.clone()));
        let notices: Arc<dyn NoticeApi> = messages.clone();
        let chat: Arc<dyn ChatApi> = messages;

        let account = Arc::new(Account::new(http.clone(), Arc::new(WebAuth::new(http.clone()))));
        let videos = Arc::new(Videos::new(Arc::new(WebVideo::new(http.clone())), cards.clone()));
        let playlist = Arc::new(Playlist::new(store.clone()));
        let playlist_backup: Arc<dyn PlaylistBackup> = playlist.clone();
        let lineup = Arc::new(Lineup::new(playlist.clone(), videos.clone(), cards.clone()));
        let seasons = Seasons::new(Arc::new(WebSeasons::new(http.clone())), store.clone(), cards.clone());
        let shapes: Arc<dyn ShapeLookup> = videos.clone();
        let portrait = PortraitFeed::new(portrait_pool, shapes, feed_api.clone(), rules.clone(), store.clone());
        let downloads =
            Downloads::new(videos.clone(), Arc::new(WebMedia::new(http.clone())), store, data_dir.join("downloads"));

        Ok(Self {
            home: Home::new(home_feed, feedback.clone(), videos.clone(), cards.clone()),
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
                followings_api,
                rules.clone(),
                feedback.clone(),
                cards.clone(),
            ),
            space: Space::new(Arc::new(WebSpace::new(http.clone())), cards.clone()),
            actions: Actions::new(actions_api, videos.clone(), feedback),
            history: History::new(history_api, account.clone(), videos.clone()),
            watch_later: WatchLater::new(watch_later_api, cards.clone()),
            favorites: Favorites::new(favorites_api, cards.clone()),
            messages: Messages::new(notices, chat, rules.clone(), videos.clone()),
            playlists: Playlists::new(playlist, lineup.clone(), videos.clone()),
            play_queue: PlayQueue::new(lineup.clone()),
            listening: Listening::new(lineup),
            preferences: Preferences::new(settings_port, profile, playlist_backup),
            reading: Reading::new(Arc::new(WebArticle::new(http.clone())), rules.clone()),
            downloads,
            live: Live::new(Arc::new(WebLive::new(http.clone())), Arc::new(WebChannels), rules.clone()),
            esports: Esports::new(Arc::new(WebEsports::new(http.clone()))),
            pgc: Pgc::new(Arc::new(WebPgc::new(http.clone()))),
            portrait,
            seasons,
            emotes: Emotes::new(Arc::new(WebEmote::new(http))),
            account,
            videos,
        })
    }
}
