package dev.rubi.app.ui

import androidx.compose.runtime.derivedStateOf
import androidx.compose.ui.unit.Dp
import kotlinx.coroutines.flow.first
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.layout.layout
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.runtime.snapshotFlow
import dev.rubi.core.FfiDanmaku
import dev.rubi.core.FfiResumePoint
import dev.rubi.core.FfiCommentArea
import dev.rubi.app.playback.LocalMedia
import android.os.SystemClock
import android.util.Log
import android.content.Intent
import android.net.Uri
import android.view.LayoutInflater
import android.view.View
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.lazy.rememberLazyListState
import dev.rubi.core.FfiVideoDetail
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.layout.boundsInWindow
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.ui.PlayerView
import dev.rubi.app.R
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import dev.rubi.app.data.DanmakuDisplay
import dev.rubi.app.data.PlaybackPrefs
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiCommentRef
import dev.rubi.core.FfiCommentRow
import dev.rubi.core.FfiCommentThread
import dev.rubi.core.FfiDanmakuSet
import dev.rubi.core.FfiPlayback
import dev.rubi.core.FfiVideoLink
import dev.rubi.core.FfiStein
import dev.rubi.core.FfiSteinNode
import dev.rubi.core.FfiSignal
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private const val TAG = "RubiPlayer"

/** Index of the sticky 推荐 / 评论 bar in the player's list (after head and detail). */
private const val TABS_INDEX = 2

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun PlayerScreen(
    deps: VideoPageDeps,
    nav: Nav,
    bvid: String,
    title: String,
    focus: FfiCommentRef? = null,
    fromPlaylist: Boolean = false,
    startMs: Long = 0L,
    episode: ULong? = null,
    episodeCid: ULong = 0uL,
    normal: Boolean = false,
) {
    val home = deps.home
    val videosService = deps.videos
    val danmakuService = deps.danmaku
    val steinService = deps.stein
    val comments = deps.comments
    val discover = deps.discover
    val following = deps.following
    val historyService = deps.history
    val playQueue = deps.playQueue
    val downloads = deps.downloads
    val pgc = deps.pgc
    val seasons = deps.seasons
    val uiPrefs = deps.ui
    val settings = deps.settings
    val appScope = deps.appScope
    val flights = LocalCardFlights.current
    /** Streams at `qn`: a 专区 episode's (if the account may watch it), else the video's. */
    suspend fun streamsAt(qn: UInt, part: ULong?) = when {
        episode != null -> pgc.pgcPlay(episode, episodeCid, qn)
        part != null -> videosService.playbackPart(bvid, part, qn)
        else -> videosService.playback(bvid, qn)
    }
    val c = LocalMetro.current
    val ctx = LocalContext.current
    val media = LocalMedia.current
    var video by remember { mutableStateOf<FfiVideoCard?>(null) }
    var danmaku by remember { mutableStateOf<FfiDanmakuSet?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var tab by remember { mutableStateOf(if (focus != null) 1 else 0) }
    var commentsOpened by remember { mutableStateOf(tab == 1) }
    LaunchedEffect(tab) { if (tab == 1) commentsOpened = true }
    // Where each tab was scrolled to (item and offset), to return there: in the page, and in
    // the side pane on a wide screen.
    val tabAt = remember { arrayOfNulls<Pair<Int, Int>>(2) }
    val sideAt = remember { arrayOfNulls<Pair<Int, Int>>(2) }
    var display by remember { mutableStateOf(DanmakuDisplay.load(settings)) }
    // 画面: how the picture fills the frame (kept), and a pinch zoom on top (this video only).
    var videoScale by remember { mutableStateOf(VideoScale.entries.getOrElse(uiPrefs.videoScale) { VideoScale.Fit }) }
    val zoom = remember { VideoZoom() }
    var dmProblem by remember { mutableStateOf<String?>(null) }
    // 设置 open: both parts on the video page, one of them from fullscreen's two buttons.
    var settingsSheet by remember { mutableStateOf<SettingsPart?>(null) }
    var detail by remember { mutableStateOf<FfiVideoDetail?>(null) }
    var detailOpen by remember { mutableStateOf(false) }
    var fullscreen by remember { mutableStateOf(false) }
    var controlsVisible by remember { mutableStateOf(true) }
    var cid by remember { mutableStateOf<ULong?>(null) }
    var playback by remember { mutableStateOf<FfiPlayback?>(null) }
    var qualitySheet by remember { mutableStateOf(false) }
    var playerView by remember { mutableStateOf<PlayerView?>(null) }
    // Brightness and volume set by fullscreen drags stay until this page closes.
    val gestures = rememberGestureRestore()
    // 互动视频: the part being played (null: the video's own first part) and the story state.
    var partCid by remember { mutableStateOf<ULong?>(null) }
    var stein by remember { mutableStateOf<FfiStein?>(null) }
    var steinNode by remember { mutableStateOf<FfiSteinNode?>(null) }
    // The story's native state is freed with the page, not left to the garbage collector.
    DisposableEffect(stein) {
        val held = stein
        onDispose { held?.close() }
    }
    /** The part the current story node plays; its question waits until that part is loaded. */
    var nodeCid by remember { mutableStateOf<ULong?>(null) }
    val holdSpeed = remember { PlaybackPrefs.longPressSpeed(settings) }
    val inPip by media.pip.active
    // Fullscreen and picture-in-picture both show the video alone.
    val compact = fullscreen || inPip
    val activity = remember(ctx) { ctx.activity() }
    PlayerPageLifecycle(settings)
    val listState = rememberLazyListState()
    // 宽屏 held sideways: 推荐 / 评论 have a pane (and a scroll) of their own.
    val sideList = rememberLazyListState()
    val twoPane = LocalScreenLayout.current.twoPane
    val scope = rememberCoroutineScope()
    // Where each part was left, so going back to one carries on from there.
    val partPositions = remember { mutableStateMapOf<ULong, Long>() }
    var autoNext by remember { mutableStateOf(uiPrefs.autoNext) }
    // Playing an offline copy (离线缓存).
    var offline by remember { mutableStateOf(false) }
    // 继续观看: where the user stopped last time, shown for a few seconds; playback starts at 0.
    var resume by remember { mutableStateOf<FfiResumePoint?>(null) }
    LaunchedEffect(bvid) {
        // Opened at a set point (from 听视频) or from the play list: nothing to offer.
        if (startMs == 0L && !fromPlaylist) resume = videosService.resumePoint(bvid)
    }

    /** Switches to part `i` of the video, remembering where the current one was left. */
    fun pickPart(i: Int) {
        val part = detail?.parts?.getOrNull(i) ?: return
        if (part.cid == cid) return
        cid?.let { partPositions[it] = media.player.get().currentPosition }
        partCid = part.cid
    }

    LaunchedEffect(bvid) {
        detail = runCatching { videosService.videoDetail(bvid) }.getOrNull()
        // A portrait video the list could not tell about: on to the 竖屏 feed from here.
        val d = detail
        if (d != null && d.portrait && Screen.Player(bvid, title, focus, fromPlaylist, startMs, episode, episodeCid, normal).feedable) {
            nav.replaceTop(Screen.VerticalFeed(VerticalFeedState(videosService, discover, bvid, media.player.get().currentPosition, display.enabled)))
        }
    }

    // Recording the click is bookkeeping; never let it delay playback.
    LaunchedEffect(bvid) { runCatching { home.record(bvid, FfiSignal.Click) } }

    ReportWatching(historyService, appScope, bvid, cid)
    // Interactive videos follow their own story instead of the list.
    ContinuePlaylistOnEnd(playQueue, nav, bvid, enabled = fromPlaylist && stein == null)
    // 自动连播: the next part, then the next video of the 合集 (the play list has its own order).
    OnPlaybackEnded(enabled = autoNext && !fromPlaylist && stein == null) {
        val d = detail ?: return@OnPlaybackEnded
        cid?.let { partPositions[it] = 0L }
        val at = d.parts.indexOfFirst { it.cid == cid }
        if (at in 0 until d.parts.lastIndex) {
            pickPart(at + 1)
        } else {
            val all = d.season?.sections?.flatMap { it.episodes }.orEmpty()
            all.getOrNull(all.indexOfFirst { it.bvid == bvid } + 1)?.let { nav.replaceTop(Screen.Player(it.bvid, it.title)) }
        }
    }

    LaunchedEffect(bvid, partCid) {
        // Drop the previous video (or branch) before fetching the next one.
        media.player.clear()
        danmaku = null
        dmProblem = null
        runCatching {
            val t0 = SystemClock.elapsedRealtime()
            val qn = PlaybackPrefs.preferredQuality(settings)
            val part = partCid
            // 离线缓存: a finished copy plays from its files, network or not.
            val local = if (part == null && episode == null) offMain { runCatching { downloads.downloadLocal(bvid) }.getOrNull() } else null
            // 卡牌 / 书页: the video starts once the card has finished turning into this page.
            snapshotFlow { flights.flight }.first { it == null }
            if (local != null) {
                media.player.playLocal(local.videoPath, local.audioPath, title, startMs)
                offline = true
            }
            val pb = streamsAt(qn, part)
            playback = pb
            Log.i(TAG, "playurl ready in ${SystemClock.elapsedRealtime() - t0} ms")
            if (local == null) media.player.play(pb, title, if (part == null) startMs else partPositions[part] ?: 0L)
            cid = pb.cid
            val durationMs = if (part == null) {
                val v = videosService.video(bvid).also { video = it }
                // Interactive videos start their story here; ordinary videos answer null.
                stein = runCatching { steinService.steinOpen(bvid, pb.cid) }.getOrNull()
                steinNode = stein?.node()
                nodeCid = pb.cid
                maxOf(v.durationS.toULong() * 1000u, pb.durationMs).toUInt()
            } else {
                pb.durationMs.toUInt()
            }
            loadDanmakuNearFirst(danmakuService, pb.cid, durationMs, media.player.get().currentPosition) { set, problem ->
                if (set != null) danmaku = set
                dmProblem = problem
            }
        }.onFailure {
            // Offline with a local copy: the copy plays; only extras (danmaku, details) are missing.
            if (!offline) error = playbackError(it)
        }
    }

    FullscreenMode(fullscreen)
    BackHandler(enabled = fullscreen) { fullscreen = false }

    // 发弹幕, from the fullscreen bar or the one under the video: at the current position.
    val sendDanmaku: suspend (String, UByte, UInt) -> FfiDanmaku = { text, mode, color ->
        val id = cid ?: throw IllegalStateException("视频还没加载完")
        val at = media.player.get().currentPosition.toUInt()
        danmakuService.sendDanmaku(bvid, id, at, text, mode, color)
    }
    // Shown at once: the next segment fetch would not include it yet.
    val showSent: (FfiDanmaku) -> Unit = { dm ->
        danmaku = danmaku?.let { it.copy(items = it.items + dm) } ?: FfiDanmakuSet(listOf(dm), emptyList(), emptyList())
    }

    // 推荐 or 评论, in the page's list on a phone or in the side pane on a wide screen. `toTop`
    // brings the list back to where the tabs are.
    // Both stay composed once opened (one item, the other measured but not shown), so switching
    // shows what was loaded instead of loading it again. 评论 loads the first time it is opened.
    // `minHeight` keeps the tabs able to sit at the top of the page while the shown tab is still
    // short (评论 loading the first time), so switching never drops the page back to the details.
    fun LazyListScope.tabContent(minHeight: Dp, toTop: () -> Unit) {
        item(key = "tab-content") {
            Column(Modifier.heightIn(min = minHeight)) {
                Box(Modifier.shownIf(tab == 0)) { RelatedList(home, discover, nav, bvid) }
                if (commentsOpened) video?.let { v ->
                    // Opened from a notice: bring the comment list up under the player.
                    Box(Modifier.shownIf(tab == 1)) {
                        CommentSection(
                            comments, following,
                            remember(v.aid) { FfiCommentArea(1u, v.aid) },
                            focus,
                            onOpenVideo = { link -> nav.openVideo(scope, videosService, link.video, link.title) },
                            onOpenArticle = { link -> nav.push(Screen.Article(link)) },
                        ) { toTop() }
                    }
                }
            }
        }
    }

    // The video side keeps one slot in every mode (the left pane on a wide screen), so the
    // PlayerView survives switching to fullscreen.
    Row(Modifier.fillMaxSize()) {
    Column(Modifier.weight(if (twoPane && !compact) 0.62f else 1f).fillMaxHeight()) {
        if (!compact) Row(Modifier.fillMaxWidth().padding(start = 4.dp, end = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
            // Several videos deep: straight back to where the chain started (首页 when it began
            // on a tab).
            if (nav.chainLength > 1) {
                val glyph = if (nav.stack.size == nav.chainLength) Glyph.Home else Glyph.Exit
                Box(Modifier.size(44.dp).clickable { nav.leaveChain() }, contentAlignment = Alignment.Center) { GlyphIcon(glyph) }
            }
            // Only trouble is worth a line here; the danmaku count is not.
            Meta(dmProblem.orEmpty(), Modifier.weight(1f))
            Text(
                if (display.enabled) "弹 开" else "弹 关",
                style = MetroType.meta.copy(fontSize = 12.sp),
                color = if (display.enabled) c.fg else c.fg2,
                modifier = Modifier.size(52.dp, 44.dp).wrapContentSize(Alignment.CenterEnd).clickable {
                    display = display.copy(enabled = !display.enabled)
                    runCatching { DanmakuDisplay.save(settings, display) }
                },
            )
            Text(
                "设置",
                style = MetroType.meta.copy(fontSize = 12.sp),
                color = c.fg,
                modifier = Modifier.size(44.dp, 44.dp).wrapContentSize(Alignment.CenterEnd).clickable { settingsSheet = SettingsPart.Both },
            )
            playback?.let { p ->
                Text(
                    qualityLabel(p),
                    style = MetroType.meta.copy(fontSize = 12.sp),
                    color = c.fg,
                    modifier = Modifier.height(44.dp).padding(start = 12.dp).wrapContentSize(Alignment.CenterEnd).clickable { qualitySheet = true },
                )
            }
            if (activity != null && media.pip.supported(activity)) {
                Text(
                    "小窗",
                    style = MetroType.meta.copy(fontSize = 12.sp),
                    color = c.fg,
                    modifier = Modifier.size(44.dp, 44.dp).wrapContentSize(Alignment.CenterEnd).clickable { media.pip.enter(activity) },
                )
            }
            Text(
                "全屏",
                style = MetroType.meta.copy(fontSize = 12.sp),
                color = c.fg,
                modifier = Modifier.size(44.dp, 44.dp).wrapContentSize(Alignment.CenterEnd).clickable { fullscreen = true },
            )
        }
        // The same slot in both modes, so the PlayerView (and its frame) survives the switch.
        Box(
            (if (compact) Modifier.fillMaxSize() else Modifier.fillMaxWidth().aspectRatio(16f / 9f).flightSource { flights.videoFrame = it })
                // Where the video is, so 小窗 shrinks from it instead of showing the app icon.
                .onGloballyPositioned { coords ->
                    val a = activity
                    if (a != null && !inPip) {
                        val r = coords.boundsInWindow()
                        media.pip.videoAt(a, r.left.toInt(), r.top.toInt(), r.right.toInt(), r.bottom.toInt())
                    }
                }
                .background(Color.Black)
                .clipToBounds()
                .pinchToZoom(zoom),
        ) {
            AndroidView(
                factory = {
                    (LayoutInflater.from(it).inflate(R.layout.player_view, null) as PlayerView).apply {
                        player = media.player.get()
                        setFullscreenButtonClickListener { on -> fullscreen = on }
                        setControllerVisibilityListener(PlayerView.ControllerVisibilityListener { v -> controlsVisible = v == View.VISIBLE })
                        playerView = this
                    }
                },
                // Detached when the page goes: a view left attached keeps listening to the shared
                // player and holds its surface, and each video entered again piled up another.
                onRelease = { it.player = null },
                update = {
                    it.setFullscreenButtonState(fullscreen)
                    // The small window has no room for controls; the system draws its own.
                    it.useController = !inPip
                    it.resizeMode = videoScale.resizeMode()
                },
                modifier = Modifier.fillMaxSize().zoomedBy(zoom),
            )
            val set = danmaku
            // No danmaku in the small window: too little room to read them.
            if (display.enabled && set != null && !inPip) DanmakuLayer(danmakuService, set, display, Modifier.fillMaxSize())
            // Only while the controls are hidden: their buttons must get their own taps.
            // Swipes for progress, brightness and volume, and double-tap to pause, in either mode.
            if (!inPip && !controlsVisible) FullscreenGestures(media.player.get(), playerView, holdSpeed, gestures)
            if (!inPip) ZoomReset(zoom, Modifier.align(Alignment.TopEnd))
            val session = stein
            val node = steinNode
            if (session != null && node != null && !inPip) {
                SteinOverlay(
                    node = node,
                    playingCid = cid,
                    nodeCid = nodeCid,
                    onChosen = { next, nextCid -> steinNode = next; nodeCid = nextCid; partCid = nextCid },
                    onRestart = { partCid = null },
                    choose = { session.choose(it) },
                    onError = { error = it },
                    modifier = Modifier.align(Alignment.Center),
                )
            }
            val hint = resume
            val parts = detail?.parts.orEmpty()
            if (hint != null && cid != null && !inPip && stein == null) {
                val at = parts.indexOfFirst { it.cid == hint.cid }
                val elsewhere = at >= 0 && hint.cid != cid
                val partMs = parts.getOrNull(at)?.durationS?.toLong()?.times(1000)
                ResumeHint(
                    positionMs = hint.positionMs.toLong(),
                    durationMs = partMs ?: media.player.get().duration.coerceAtLeast(0L),
                    otherPart = if (elsewhere) "P${at + 1}" else null,
                    onResume = {
                        resume = null
                        if (elsewhere) {
                            partPositions[hint.cid] = hint.positionMs.toLong()
                            pickPart(at)
                        } else {
                            media.player.get().seekTo(hint.positionMs.toLong())
                        }
                    },
                    onGone = { resume = null },
                    modifier = Modifier.align(Alignment.BottomStart),
                )
            }
            if (fullscreen && !inPip) {
                FullscreenOverlay(
                    title = video?.title ?: title,
                    controlsVisible = controlsVisible,
                    danmakuOn = display.enabled,
                    onExit = { fullscreen = false },
                    onToggleDanmaku = {
                        display = display.copy(enabled = !display.enabled)
                        runCatching { DanmakuDisplay.save(settings, display) }
                    },
                    onSettings = { settingsSheet = SettingsPart.Danmaku },
                    onPlaySettings = { settingsSheet = SettingsPart.Play },
                    quality = playback?.let(::qualityLabel).orEmpty(),
                    onQuality = { qualitySheet = true },
                    send = sendDanmaku,
                    onSent = showSent,
                    modifier = Modifier.align(Alignment.TopStart),
                )
            }
        }
        if (compact) return@Column
        // 发弹幕 without going fullscreen.
        DanmakuSendBar(sendDanmaku, showSent)
        danmaku?.jumpMarks?.takeIf { it.isNotEmpty() }?.let { marks ->
            // One line that scrolls sideways, however many marks there are.
            Row(
                Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 4.dp),
                horizontalArrangement = Arrangement.spacedBy(16.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Meta("空降点")
                marks.forEach { m ->
                    Text(
                        m.label,
                        style = MetroType.meta,
                        color = c.fg,
                        maxLines = 1,
                        softWrap = false,
                        modifier = Modifier.heightIn(min = 32.dp).wrapContentSize(Alignment.Center).clickable {
                            media.player.get().seekTo(m.targetMs.toLong())
                        },
                    )
                }
            }
        }
        // The list's own height: the tab shown is never shorter (see tabContent). Read through
        // derivedStateOf so scrolling does not recompose the page.
        val viewportPx by remember { derivedStateOf { listState.layoutInfo.viewportSize.height } }
        val pageHeight = with(LocalDensity.current) { viewportPx.toDp() }
        // Scrolled into the tabs: each one comes back where it was left (or at its top). Above
        // them, the page stays where it is.
        fun switchTab(picked: Int) {
            scope.launch {
                val inTabs = listState.firstVisibleItemIndex >= TABS_INDEX
                if (inTabs) tabAt[tab] = listState.firstVisibleItemIndex to listState.firstVisibleItemScrollOffset
                tab = picked
                if (inTabs) {
                    // After the shown tab has been laid out: restored against the other, shorter,
                    // one the position would be cut short.
                    repeat(2) { withFrameNanos { } }
                    val back = tabAt[picked]
                    if (back != null) listState.scrollToItem(back.first, back.second) else listState.scrollToItem(TABS_INDEX)
                }
            }
        }
        // A sideways swipe anywhere on the page moves between 推荐 and 评论 (in their own pane on a
        // wide screen).
        LazyColumn(Modifier.weight(1f).then(if (twoPane) Modifier else Modifier.swipeTabs(tab, 2, ::switchTab)), state = listState) {
            item(key = "head") {
                Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(video?.title ?: title, style = MetroType.title, color = c.fg)
                    // Counts from the fresh details; a card from a list may have none (history).
                    val d = detail
                    if (d != null) Meta("${formatCount(d.views)} 播放 · ${formatAge(d.pubdate)} · ${d.zone ?: ""}")
                    else video?.let { Meta("${formatCount(it.views)} 播放 · ${formatAge(it.pubdate)} · ${it.zone ?: ""}") }
                    if (d == null) video?.let { v ->
                        Text(
                            "${v.upName}  ›",
                            fontSize = 15.sp,
                            fontWeight = FontWeight.SemiBold,
                            color = c.fg,
                            modifier = Modifier.clickable { nav.push(Screen.Up(v.upMid, v.upName)) }.padding(vertical = 6.dp),
                        )
                    }
                }
            }
            item(key = "detail") {
                Column(Modifier.padding(horizontal = 16.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    detail?.let {
                        // 专区 episodes are never cached offline here: only official clients may.
                        key(it.bvid) { VideoActions(deps.actionRow, nav, it, offline = episode == null) }
                        VideoDetailBlock(it, detailOpen) { detailOpen = !detailOpen }
                    }
                    // 分P and 合集 (interactive videos use parts as their story, so not there).
                    val d = detail
                    if (d != null && stein == null && (d.parts.size > 1 || d.season != null || fromPlaylist)) {
                        Column(Modifier.bleed(16.dp)) {
                            if (d.parts.size > 1) {
                                PartsRow(
                                    d.parts,
                                    current = d.parts.indexOfFirst { it.cid == cid },
                                    progressOf = { p ->
                                        val total = p.durationS.toFloat() * 1000f
                                        if (total > 0f) (partPositions[p.cid] ?: 0L) / total else 0f
                                    },
                                    onPick = ::pickPart,
                                )
                            }
                            // Played from a list: its 播放队列 stands in for the 合集.
                            if (fromPlaylist) {
                                QueueBlock(playQueue, bvid) { v -> nav.replaceTop(Screen.Player(v.bvid, v.title, fromPlaylist = true)) }
                            } else {
                                d.season?.let { s ->
                                    SeasonBlock(s, bvid, subscribe = { SeasonSubscribeButton(seasons, nav, s.id) }) { ep -> nav.replaceTop(Screen.Player(ep.bvid, ep.title)) }
                                }
                            }
                            // A list plays on by its own order.
                            if (!fromPlaylist) AutoNextRow(autoNext) {
                                autoNext = !autoNext
                                uiPrefs.autoNext = autoNext
                            }
                        }
                    }
                    error?.let { Text(it, color = c.fg2, fontSize = 13.sp) }
                    danmaku?.removed?.takeIf { it.isNotEmpty() }?.let { Meta("已过滤 · " + it.joinToString(" · ")) }
                }
            }
            if (!twoPane) {
                // Pinned under the player while scrolling. Tapping the active tab returns to the top;
                // tapping the other one switches and keeps the bar where it is.
                // 推荐 / 评论 switch at once, without the page motion.
                stickyHeader(key = "tabs") {
                    Box(Modifier.fillMaxWidth().background(c.bg)) {
                        Pivot(listOf("推荐", "评论"), tab, { picked ->
                            if (picked == tab) scope.launch { listState.animateScrollToItem(0) } else switchTab(picked)
                        }, MetroType.section)
                    }
                }
                tabContent(pageHeight) { scope.launch { listState.scrollToItem(TABS_INDEX) } }
            }
        }
    }
    if (twoPane && !compact) {
        Box(Modifier.fillMaxHeight().width(1.dp).background(c.line))
        // The side pane's own tabs, each back where it was left.
        fun switchSide(picked: Int) {
            scope.launch {
                sideAt[tab] = sideList.firstVisibleItemIndex to sideList.firstVisibleItemScrollOffset
                tab = picked
                repeat(2) { withFrameNanos { } }
                val back = sideAt[picked] ?: (0 to 0)
                sideList.scrollToItem(back.first, back.second)
            }
        }
        Column(Modifier.weight(0.38f).fillMaxHeight().swipeTabs(tab, 2, ::switchSide)) {
            Pivot(listOf("推荐", "评论"), tab, { picked ->
                if (picked == tab) scope.launch { sideList.animateScrollToItem(0) } else switchSide(picked)
            }, MetroType.section)
            LazyColumn(Modifier.weight(1f), state = sideList) {
                tabContent(0.dp) { scope.launch { sideList.scrollToItem(0) } }
            }
        }
    }
    }

    val current = playback
    if (qualitySheet && current != null) {
        QualitySheet(current.accept, current.quality, onDismiss = { qualitySheet = false }, side = fullscreen) { q ->
            qualitySheet = false
            PlaybackPrefs.setPreferredQuality(settings, q.qn)
            scope.launch {
                // Same video, other streams: carry on from where it was.
                val at = media.player.get().currentPosition
                runCatching { if (episode != null) streamsAt(q.qn, null) else streamsAt(q.qn, cid) }
                    .onSuccess { p -> playback = p; media.player.play(p, video?.title ?: title, at) }
                    .onFailure { error = "切换清晰度失败：${it.message}" }
            }
        }
    }

    settingsSheet?.let { part ->
        DanmakuSettingsSheet(
            display = display,
            onChange = {
                display = it
                runCatching { DanmakuDisplay.save(settings, it) }
            },
            onDismiss = { settingsSheet = null },
            side = fullscreen,
            play = if (part == SettingsPart.Danmaku) null else PlayControls(
                speed = media.player.get().playbackParameters.speed,
                onSpeed = { media.player.get().setPlaybackSpeed(it) },
                scale = videoScale,
                onScale = {
                    videoScale = it
                    uiPrefs.videoScale = it.ordinal
                },
            ),
            danmaku = part != SettingsPart.Play,
        )
    }
}

/** The label of the quality being played ("1080P 高清"), or its qn when the list lacks it. */
private fun qualityLabel(p: FfiPlayback): String = p.accept.firstOrNull { it.qn == p.quality }?.label ?: "${p.quality}"

/** Why a video cannot be opened, in words: deleted, hidden or region-locked videos say so. */
internal fun playbackError(e: Throwable): String {
    val msg = e.message.orEmpty()
    return when {
        listOf("62002", "62004", "62012", "-404").any { it in msg } -> "这个视频已失效或不可见"
        e is dev.rubi.core.RubiException.RiskControl -> "被 B 站风控拦截，稍等片刻再试"
        e is dev.rubi.core.RubiException.Network -> "网络错误：${msg.substringAfter("：")}"
        else -> "无法播放：$msg"
    }
}

/** Measured (so it keeps its state and size ready) but neither shown nor taking room when hidden. */
private fun Modifier.shownIf(shown: Boolean): Modifier = layout { measurable, constraints ->
    val placeable = measurable.measure(constraints)
    if (shown) layout(placeable.width, placeable.height) { placeable.place(0, 0) } else layout(placeable.width, 0) {}
}

/** Which part of the player's settings is open. */
private enum class SettingsPart { Both, Play, Danmaku }
