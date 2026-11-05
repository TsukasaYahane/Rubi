//! The app-facing API (Kotlin today, Swift later). [`RubiCore`] opens the core and hands out
//! one object per business (`core.history()`, `core.favorites()`…), so each page holds only
//! the calls of its own business. The modules below export those objects' calls and only
//! convert between the flat records in [`types`] and the core's types. No rules or state
//! live here.

mod account;
mod actions;
mod comments;
mod discover;
mod downloads;
mod error;
mod following;
mod home;
mod library;
mod live;
mod messages;
mod pgc;
mod player;
mod playlist;
mod reading;
mod seasons;
mod settings;
mod space;
mod types;

use std::path::PathBuf;
use std::sync::Arc;

use rubi_core::Session;
use rubi_core::{
    Account, Actions, Comments, Danmakus, Discover, Downloads, Emotes, Esports, Favorites, Following, History, Home,
    Listening, Live, Messages, Pgc, PlayQueue, Playlists, Preferences, Reading, Seasons, Services, Space, Stein,
    Videos, WatchLater,
};

pub use error::RubiError;
pub use live::FfiLiveChannel;
pub use player::{FfiDanmakuSession, FfiStein};
pub use types::*;

uniffi::setup_scaffolding!();

type Res<T> = Result<T, RubiError>;

/// 登录与账号。
#[derive(uniffi::Object)]
pub struct AccountService {
    account: Arc<Account>,
}

/// 首页推荐流。
#[derive(uniffi::Object)]
pub struct HomeService {
    home: Home,
}

/// 视频详情、播放地址与投屏。
#[derive(uniffi::Object)]
pub struct VideosService {
    videos: Arc<Videos>,
}

/// 弹幕读取、排布与发送。
#[derive(uniffi::Object)]
pub struct DanmakuService {
    danmaku: Danmakus,
}

/// 互动视频。
#[derive(uniffi::Object)]
pub struct SteinService {
    stein: Stein,
}

/// 评论区。
#[derive(uniffi::Object)]
pub struct CommentsService {
    comments: Comments,
}

/// 热门、排行、分区与搜索。
#[derive(uniffi::Object)]
pub struct DiscoverService {
    discover: Discover,
}

/// 表情。
#[derive(uniffi::Object)]
pub struct EmotesService {
    emotes: Emotes,
}

/// 关注动态与关注列表。
#[derive(uniffi::Object)]
pub struct FollowingService {
    following: Following,
}

/// UP 主页。
#[derive(uniffi::Object)]
pub struct SpaceService {
    space: Space,
}

/// 视频页的点赞、投币、收藏、稍后再看与关注。
#[derive(uniffi::Object)]
pub struct ActionsService {
    actions: Actions,
}

/// 历史记录与观看上报。
#[derive(uniffi::Object)]
pub struct HistoryService {
    history: History,
}

/// 稍后再看列表。
#[derive(uniffi::Object)]
pub struct WatchLaterService {
    watch_later: WatchLater,
}

/// 收藏夹。
#[derive(uniffi::Object)]
pub struct FavoritesService {
    favorites: Favorites,
}

/// 消息与私信。
#[derive(uniffi::Object)]
pub struct MessagesService {
    messages: Messages,
}

/// 播放列表编辑。
#[derive(uniffi::Object)]
pub struct PlaylistsService {
    playlists: Playlists,
}

/// 视频页连播。
#[derive(uniffi::Object)]
pub struct PlayQueueService {
    play_queue: PlayQueue,
}

/// 听视频。
#[derive(uniffi::Object)]
pub struct ListeningService {
    listening: Listening,
}

/// 设置与备份。
#[derive(uniffi::Object)]
pub struct PreferencesService {
    preferences: Preferences,
}

/// 专栏。
#[derive(uniffi::Object)]
pub struct ReadingService {
    reading: Reading,
}

/// 离线缓存。
#[derive(uniffi::Object)]
pub struct DownloadsService {
    downloads: Downloads,
}

/// 直播。
#[derive(uniffi::Object)]
pub struct LiveService {
    live: Live,
}

/// 赛事。
#[derive(uniffi::Object)]
pub struct EsportsService {
    esports: Esports,
}

/// 番剧与影视。
#[derive(uniffi::Object)]
pub struct PgcService {
    pgc: Pgc,
}

/// 合集订阅。
#[derive(uniffi::Object)]
pub struct SeasonsService {
    seasons: Seasons,
}

#[derive(uniffi::Object)]
pub struct RubiCore {
    account: Arc<AccountService>,
    home: Arc<HomeService>,
    videos: Arc<VideosService>,
    danmaku: Arc<DanmakuService>,
    stein: Arc<SteinService>,
    comments: Arc<CommentsService>,
    discover: Arc<DiscoverService>,
    emotes: Arc<EmotesService>,
    following: Arc<FollowingService>,
    space: Arc<SpaceService>,
    actions: Arc<ActionsService>,
    history: Arc<HistoryService>,
    watch_later: Arc<WatchLaterService>,
    favorites: Arc<FavoritesService>,
    messages: Arc<MessagesService>,
    playlists: Arc<PlaylistsService>,
    play_queue: Arc<PlayQueueService>,
    listening: Arc<ListeningService>,
    preferences: Arc<PreferencesService>,
    reading: Arc<ReadingService>,
    downloads: Arc<DownloadsService>,
    live: Arc<LiveService>,
    esports: Arc<EsportsService>,
    pgc: Arc<PgcService>,
    seasons: Arc<SeasonsService>,
}

#[uniffi::export]
impl RubiCore {
    /// `data_dir` is the app's private files directory; `session_json` is what a previous
    /// login returned (decrypted by the caller), or `None` for a guest.
    #[uniffi::constructor]
    pub fn new(data_dir: String, session_json: Option<String>) -> Res<Arc<Self>> {
        let session: Session = session_json.as_deref().map(serde_json::from_str).transpose()?.unwrap_or_default();
        let Services {
            account,
            home,
            videos,
            danmaku,
            stein,
            comments,
            discover,
            emotes,
            following,
            space,
            actions,
            history,
            watch_later,
            favorites,
            messages,
            playlists,
            play_queue,
            listening,
            preferences,
            reading,
            downloads,
            live,
            esports,
            pgc,
            seasons,
        } = Services::open(PathBuf::from(data_dir), session)?;
        Ok(Arc::new(Self {
            account: Arc::new(AccountService { account }),
            home: Arc::new(HomeService { home }),
            videos: Arc::new(VideosService { videos }),
            danmaku: Arc::new(DanmakuService { danmaku }),
            stein: Arc::new(SteinService { stein }),
            comments: Arc::new(CommentsService { comments }),
            discover: Arc::new(DiscoverService { discover }),
            emotes: Arc::new(EmotesService { emotes }),
            following: Arc::new(FollowingService { following }),
            space: Arc::new(SpaceService { space }),
            actions: Arc::new(ActionsService { actions }),
            history: Arc::new(HistoryService { history }),
            watch_later: Arc::new(WatchLaterService { watch_later }),
            favorites: Arc::new(FavoritesService { favorites }),
            messages: Arc::new(MessagesService { messages }),
            playlists: Arc::new(PlaylistsService { playlists }),
            play_queue: Arc::new(PlayQueueService { play_queue }),
            listening: Arc::new(ListeningService { listening }),
            preferences: Arc::new(PreferencesService { preferences }),
            reading: Arc::new(ReadingService { reading }),
            downloads: Arc::new(DownloadsService { downloads }),
            live: Arc::new(LiveService { live }),
            esports: Arc::new(EsportsService { esports }),
            pgc: Arc::new(PgcService { pgc }),
            seasons: Arc::new(SeasonsService { seasons }),
        }))
    }

    /// 登录与账号。
    pub fn account(&self) -> Arc<AccountService> {
        self.account.clone()
    }

    /// 首页推荐流。
    pub fn home(&self) -> Arc<HomeService> {
        self.home.clone()
    }

    /// 视频详情、播放地址与投屏。
    pub fn videos(&self) -> Arc<VideosService> {
        self.videos.clone()
    }

    /// 弹幕读取、排布与发送。
    pub fn danmaku(&self) -> Arc<DanmakuService> {
        self.danmaku.clone()
    }

    /// 互动视频。
    pub fn stein(&self) -> Arc<SteinService> {
        self.stein.clone()
    }

    /// 评论区。
    pub fn comments(&self) -> Arc<CommentsService> {
        self.comments.clone()
    }

    /// 热门、排行、分区与搜索。
    pub fn discover(&self) -> Arc<DiscoverService> {
        self.discover.clone()
    }

    /// 表情。
    pub fn emotes(&self) -> Arc<EmotesService> {
        self.emotes.clone()
    }

    /// 关注动态与关注列表。
    pub fn following(&self) -> Arc<FollowingService> {
        self.following.clone()
    }

    /// UP 主页。
    pub fn space(&self) -> Arc<SpaceService> {
        self.space.clone()
    }

    /// 视频页的点赞、投币、收藏、稍后再看与关注。
    pub fn actions(&self) -> Arc<ActionsService> {
        self.actions.clone()
    }

    /// 历史记录与观看上报。
    pub fn history(&self) -> Arc<HistoryService> {
        self.history.clone()
    }

    /// 稍后再看列表。
    pub fn watch_later(&self) -> Arc<WatchLaterService> {
        self.watch_later.clone()
    }

    /// 收藏夹。
    pub fn favorites(&self) -> Arc<FavoritesService> {
        self.favorites.clone()
    }

    /// 消息与私信。
    pub fn messages(&self) -> Arc<MessagesService> {
        self.messages.clone()
    }

    /// 播放列表编辑。
    pub fn playlists(&self) -> Arc<PlaylistsService> {
        self.playlists.clone()
    }

    /// 视频页连播。
    pub fn play_queue(&self) -> Arc<PlayQueueService> {
        self.play_queue.clone()
    }

    /// 听视频。
    pub fn listening(&self) -> Arc<ListeningService> {
        self.listening.clone()
    }

    /// 设置与备份。
    pub fn preferences(&self) -> Arc<PreferencesService> {
        self.preferences.clone()
    }

    /// 专栏。
    pub fn reading(&self) -> Arc<ReadingService> {
        self.reading.clone()
    }

    /// 离线缓存。
    pub fn downloads(&self) -> Arc<DownloadsService> {
        self.downloads.clone()
    }

    /// 直播。
    pub fn live(&self) -> Arc<LiveService> {
        self.live.clone()
    }

    /// 赛事。
    pub fn esports(&self) -> Arc<EsportsService> {
        self.esports.clone()
    }

    /// 番剧与影视。
    pub fn pgc(&self) -> Arc<PgcService> {
        self.pgc.clone()
    }

    /// 合集订阅。
    pub fn seasons(&self) -> Arc<SeasonsService> {
        self.seasons.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guest_core_opens_and_ranks_an_empty_pool() {
        let dir = std::env::temp_dir().join(format!("rubi-ffi-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let core = RubiCore::new(dir.to_string_lossy().into_owned(), None).unwrap();
        assert!(core.home().next_page(10).unwrap().is_empty());
        assert!(core.preferences().settings_json().unwrap().contains("rules"));
        drop(core);
        let _ = std::fs::remove_dir_all(dir);
    }
}
