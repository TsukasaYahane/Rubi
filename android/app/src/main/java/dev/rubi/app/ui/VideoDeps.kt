package dev.rubi.app.ui

import android.content.Context
import dev.rubi.app.Appearance
import dev.rubi.app.data.CoreSettings
import dev.rubi.app.data.UiPrefs
import dev.rubi.core.AccountService
import dev.rubi.core.ActionsService
import dev.rubi.core.CommentsService
import dev.rubi.core.DanmakuService
import dev.rubi.core.DiscoverService
import dev.rubi.core.DownloadsService
import dev.rubi.core.FollowingService
import dev.rubi.core.HistoryService
import dev.rubi.core.HomeService
import dev.rubi.core.MessagesService
import dev.rubi.core.PgcService
import dev.rubi.core.PlayQueueService
import dev.rubi.core.PlaylistsService
import dev.rubi.core.SeasonsService
import dev.rubi.core.SpaceService
import dev.rubi.core.SteinService
import dev.rubi.core.VideosService
import kotlinx.coroutines.CoroutineScope

/**
 * What the action row under a video ([VideoActions]) works with: 赞 / 币 / 藏, 稍后再看, 列表,
 * 缓存, 分享 (card, friend), 投屏, and the UP row.
 */
class VideoActionDeps(
    val account: AccountService,
    val videos: VideosService,
    val following: FollowingService,
    val space: SpaceService,
    val actions: ActionsService,
    val messages: MessagesService,
    val playlists: PlaylistsService,
    val downloads: DownloadsService,
    val ui: UiPrefs,
    val appearance: Appearance,
    val context: Context,
)

/** What the 评论 / 推荐 / 详情 panel over the 竖屏 feed ([FeedSheet]) works with. */
class VideoPanelDeps(
    val home: HomeService,
    val videos: VideosService,
    val comments: CommentsService,
    val discover: DiscoverService,
    val following: FollowingService,
    val seasons: SeasonsService,
    val ui: UiPrefs,
)

/**
 * What the video pages (the ordinary one and the 竖屏 feed) work with, built once by the root.
 * The parts they hand on go as their own objects: [actionRow] to the action row, [panel] to
 * the feed's panel, so neither sees more than it uses.
 */
class VideoPageDeps(
    val account: AccountService,
    val home: HomeService,
    val videos: VideosService,
    val danmaku: DanmakuService,
    val stein: SteinService,
    val comments: CommentsService,
    val discover: DiscoverService,
    val following: FollowingService,
    val history: HistoryService,
    val playQueue: PlayQueueService,
    val downloads: DownloadsService,
    val pgc: PgcService,
    val seasons: SeasonsService,
    val ui: UiPrefs,
    val settings: CoreSettings,
    /** For work that must outlive the page (the last history report). */
    val appScope: CoroutineScope,
    val actionRow: VideoActionDeps,
) {
    val panel = VideoPanelDeps(home, videos, comments, discover, following, seasons, ui)
}
