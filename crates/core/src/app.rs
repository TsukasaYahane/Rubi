use std::path::Path;
use std::sync::Arc;

use rubi_api::{AuthApi, DanmakuApi, FeedApi, HttpClient, Session, VideoApi, WebAuth, WebDanmaku, WebFeed, WebVideo};
use rubi_recommend::Source;
use rubi_store::SqliteStore;

use crate::error::Result;
use crate::feed::{FeedService, Seeded};
use crate::sources::{
    BiliRecommendSource, FollowedUpsSource, PopularSource, PreferredZoneSource, RankingSource, RelatedSource,
};

/// Every service, wired once. Hold one per process.
pub struct App {
    pub http: Arc<HttpClient>,
    pub auth: Arc<dyn AuthApi>,
    pub feed_api: Arc<dyn FeedApi>,
    pub video: Arc<dyn VideoApi>,
    pub danmaku: Arc<dyn DanmakuApi>,
    pub feed: FeedService<SqliteStore>,
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
        Ok(Self {
            auth: Arc::new(WebAuth::new(http.clone())),
            video: Arc::new(WebVideo::new(http.clone())),
            danmaku: Arc::new(WebDanmaku::new(http.clone())),
            feed_api,
            http,
            feed,
        })
    }
}
