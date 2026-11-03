package dev.rubi.app.ui

import kotlinx.coroutines.flow.first
import androidx.compose.runtime.snapshotFlow
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
import dev.rubi.app.RubiApp
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
    app: RubiApp,
    nav: Nav,
    bvid: String,
    title: String,
    focus: FfiCommentRef? = null,
    fromPlaylist: Boolean = false,
    startMs: Long = 0L,
    episode: ULong? = null,
    episodeCid: ULong = 0uL,
) {
    val flights = LocalCardFlights.current
    /** Streams at `qn`: a 专区 episode's (if the account may watch it), else the video's. */
    suspend fun streamsAt(qn: UInt, part: ULong?) = when {
        episode != null -> app.core.pgcPlay(episode, episodeCid, qn)
        part != null -> app.core.playbackPart(bvid, part, qn)
        else -> app.core.playback(bvid, qn)
    }
    val c = LocalMetro.current
    val ctx = LocalContext.current
    val media = LocalMedia.current
    var video by remember { mutableStateOf<FfiVideoCard?>(null) }
    var danmaku by remember { mutableStateOf<FfiDanmakuSet?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var tab by remember { mutableStateOf(if (focus != null) 1 else 0) }
    var display by remember { mutableStateOf(DanmakuDisplay.load(app.core)) }
    var dmProblem by remember { mutableStateOf<String?>(null) }
    var dmSheet by remember { mutableStateOf(false) }
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
    val holdSpeed = remember { PlaybackPrefs.longPressSpeed(app.core) }
    val inPip by media.pip.active
    // Fullscreen and picture-in-picture both show the video alone.
    val compact = fullscreen || inPip
    val activity = remember(ctx) { ctx.activity() }
    PlayerPageLifecycle(app)
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()
    // Where each part was left, so going back to one carries on from there.
    val partPositions = remember { mutableStateMapOf<ULong, Long>() }
    var autoNext by remember { mutableStateOf(app.ui.autoNext) }
    // Playing an offline copy (离线缓存).
    var offline by remember { mutableStateOf(false) }
    // 继续观看: where the user stopped last time, shown for a few seconds; playback starts at 0.
    var resume by remember { mutableStateOf<FfiResumePoint?>(null) }
    LaunchedEffect(bvid) {
        // Opened at a set point (from 听视频) or from the play list: nothing to offer.
        if (startMs == 0L && !fromPlaylist) resume = app.core.resumePoint(bvid)
    }

    /** Switches to part `i` of the video, remembering where the current one was left. */
    fun pickPart(i: Int) {
        val part = detail?.parts?.getOrNull(i) ?: return
        if (part.cid == cid) return
        cid?.let { partPositions[it] = media.player.get().currentPosition }
        partCid = part.cid
    }

    LaunchedEffect(bvid) { detail = runCatching { app.core.videoDetail(bvid) }.getOrNull() }

    // Recording the click is bookkeeping; never let it delay playback.
    LaunchedEffect(bvid) { runCatching { app.core.record(bvid, FfiSignal.Click) } }

    ReportWatching(app, bvid, cid)
    // Interactive videos follow their own story instead of the list.
    ContinuePlaylistOnEnd(app, nav, bvid, enabled = fromPlaylist && stein == null)
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
            val qn = PlaybackPrefs.preferredQuality(app.core)
            val part = partCid
            // 离线缓存: a finished copy plays from its files, network or not.
            val local = if (part == null && episode == null) runCatching { app.core.downloadLocal(bvid) }.getOrNull() else null
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
                val v = app.core.video(bvid).also { video = it }
                // Interactive videos start their story here; ordinary videos answer null.
                stein = runCatching { app.core.steinOpen(bvid, pb.cid) }.getOrNull()
                steinNode = stein?.node()
                nodeCid = pb.cid
                maxOf(v.durationS.toULong() * 1000u, pb.durationMs).toUInt()
            } else {
                pb.durationMs.toUInt()
            }
            loadDanmakuNearFirst(app, pb.cid, durationMs, media.player.get().currentPosition) { set, problem ->
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

    Column(Modifier.fillMaxSize()) {
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
                    runCatching { DanmakuDisplay.save(app.core, display) }
                },
            )
            Text(
                "设置",
                style = MetroType.meta.copy(fontSize = 12.sp),
                color = c.fg,
                modifier = Modifier.size(44.dp, 44.dp).wrapContentSize(Alignment.CenterEnd).clickable { dmSheet = true },
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
                .background(Color.Black),
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
                },
                modifier = Modifier.fillMaxSize(),
            )
            val set = danmaku
            // No danmaku in the small window: too little room to read them.
            if (display.enabled && set != null && !inPip) DanmakuLayer(app.core, set, display, Modifier.fillMaxSize())
            // Only while the controls are hidden: their buttons must get their own taps.
            if (fullscreen && !inPip && !controlsVisible) FullscreenGestures(media.player.get(), playerView, holdSpeed, gestures)
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
                        runCatching { DanmakuDisplay.save(app.core, display) }
                    },
                    onSettings = { dmSheet = true },
                    quality = playback?.let(::qualityLabel).orEmpty(),
                    onQuality = { qualitySheet = true },
                    send = { text, mode, color ->
                        val id = cid ?: throw IllegalStateException("视频还没加载完")
                        val at = media.player.get().currentPosition.toUInt()
                        app.core.sendDanmaku(bvid, id, at, text, mode, color)
                    },
                    // Show it at once: the next segment fetch would not include it yet.
                    onSent = { dm ->
                        danmaku = danmaku?.let { it.copy(items = it.items + dm) } ?: FfiDanmakuSet(listOf(dm), emptyList(), emptyList())
                    },
                    modifier = Modifier.align(Alignment.TopStart),
                )
            }
        }
        if (compact) return@Column
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
        fun switchTab(picked: Int) {
            scope.launch {
                tab = picked
                if (listState.firstVisibleItemIndex >= TABS_INDEX) listState.scrollToItem(TABS_INDEX)
            }
        }
        // A sideways swipe anywhere on the page moves between 推荐 and 评论.
        LazyColumn(Modifier.weight(1f).swipeTabs(tab, 2, ::switchTab), state = listState) {
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
                        key(it.bvid) { VideoActions(app, nav, it, offline = episode == null) }
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
                                QueueBlock(app, bvid) { v -> nav.replaceTop(Screen.Player(v.bvid, v.title, fromPlaylist = true)) }
                            } else {
                                d.season?.let { s ->
                                    SeasonBlock(s, bvid, subscribe = { SeasonSubscribeButton(app, nav, s.id) }) { ep -> nav.replaceTop(Screen.Player(ep.bvid, ep.title)) }
                                }
                            }
                            // A list plays on by its own order.
                            if (!fromPlaylist) AutoNextRow(autoNext) {
                                autoNext = !autoNext
                                app.ui.autoNext = autoNext
                            }
                        }
                    }
                    error?.let { Text(it, color = c.fg2, fontSize = 13.sp) }
                    danmaku?.removed?.takeIf { it.isNotEmpty() }?.let { Meta("已过滤 · " + it.joinToString(" · ")) }
                }
            }
            // Pinned under the player while scrolling. Tapping the active tab returns to the top;
            // tapping the other one switches and keeps the bar where it is.
            stickyHeader(key = "tabs") {
                Box(Modifier.fillMaxWidth().background(c.bg)) {
                    Pivot(listOf("推荐", "评论"), tab, { picked ->
                        if (picked == tab) scope.launch { listState.animateScrollToItem(0) } else switchTab(picked)
                    }, MetroType.section)
                }
            }
            if (tab == 0) {
                item(key = "related") { RelatedList(app, nav, bvid) }
            } else {
                video?.let { v ->
                    item(key = "comments") {
                        // Opened from a notice: bring the comment list up under the player.
                        CommentSection(
                            app,
                            remember(v.aid) { FfiCommentArea(1u, v.aid) },
                            focus,
                            onOpenVideo = { link -> nav.openVideo(scope, app, link.video, link.title) },
                            onOpenArticle = { link -> nav.push(Screen.Article(link)) },
                        ) { scope.launch { listState.scrollToItem(TABS_INDEX) } }
                    }
                }
            }
        }
    }

    val current = playback
    if (qualitySheet && current != null) {
        QualitySheet(current.accept, current.quality, onDismiss = { qualitySheet = false }, side = fullscreen) { q ->
            qualitySheet = false
            PlaybackPrefs.setPreferredQuality(app.core, q.qn)
            scope.launch {
                // Same video, other streams: carry on from where it was.
                val at = media.player.get().currentPosition
                runCatching { if (episode != null) streamsAt(q.qn, null) else streamsAt(q.qn, cid) }
                    .onSuccess { p -> playback = p; media.player.play(p, video?.title ?: title, at) }
                    .onFailure { error = "切换清晰度失败：${it.message}" }
            }
        }
    }

    if (dmSheet) {
        DanmakuSettingsSheet(
            display = display,
            onChange = {
                display = it
                runCatching { DanmakuDisplay.save(app.core, it) }
            },
            onDismiss = { dmSheet = false },
            side = fullscreen,
        )
    }
}

/** The label of the quality being played ("1080P 高清"), or its qn when the list lacks it. */
private fun qualityLabel(p: FfiPlayback): String = p.accept.firstOrNull { it.qn == p.quality }?.label ?: "${p.quality}"

/** Why a video cannot be opened, in words: deleted, hidden or region-locked videos say so. */
private fun playbackError(e: Throwable): String {
    val msg = e.message.orEmpty()
    return when {
        listOf("62002", "62004", "62012", "-404").any { it in msg } -> "这个视频已失效或不可见"
        e is dev.rubi.core.RubiException.RiskControl -> "被 B 站风控拦截，稍等片刻再试"
        e is dev.rubi.core.RubiException.Network -> "网络错误：${msg.substringAfter("：")}"
        else -> "无法播放：$msg"
    }
}
