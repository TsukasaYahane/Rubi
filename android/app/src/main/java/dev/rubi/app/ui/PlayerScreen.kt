package dev.rubi.app.ui

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
import dev.rubi.app.playback.AudioQueue
import dev.rubi.app.playback.Pip
import dev.rubi.app.playback.PlayerHolder
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

/** Lines a comment shows before 展开全文. */
private const val COMMENT_LINES = 6

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
) {
    val c = LocalMetro.current
    val ctx = LocalContext.current
    var video by remember { mutableStateOf<FfiVideoCard?>(null) }
    var danmaku by remember { mutableStateOf<FfiDanmakuSet?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var tab by remember { mutableStateOf(if (focus != null) 1 else 0) }
    var display by remember { mutableStateOf(DanmakuDisplay.load(app.core)) }
    var dmStatus by remember { mutableStateOf("弹幕加载中…") }
    var dmSheet by remember { mutableStateOf(false) }
    var detail by remember { mutableStateOf<FfiVideoDetail?>(null) }
    var detailOpen by remember { mutableStateOf(false) }
    var fullscreen by remember { mutableStateOf(false) }
    var controlsVisible by remember { mutableStateOf(true) }
    var cid by remember { mutableStateOf<ULong?>(null) }
    var playback by remember { mutableStateOf<FfiPlayback?>(null) }
    var qualitySheet by remember { mutableStateOf(false) }
    var playerView by remember { mutableStateOf<PlayerView?>(null) }
    // 互动视频: the part being played (null: the video's own first part) and the story state.
    var partCid by remember { mutableStateOf<ULong?>(null) }
    var stein by remember { mutableStateOf<FfiStein?>(null) }
    var steinNode by remember { mutableStateOf<FfiSteinNode?>(null) }
    /** The part the current story node plays; its question waits until that part is loaded. */
    var nodeCid by remember { mutableStateOf<ULong?>(null) }
    val holdSpeed = remember { PlaybackPrefs.longPressSpeed(app.core) }
    val inPip by Pip.active
    // Fullscreen and picture-in-picture both show the video alone.
    val compact = fullscreen || inPip
    val activity = remember(ctx) { ctx.activity() }
    DisposableEffect(Unit) {
        Pip.onPlayer = true
        Pip.autoEnter = PlaybackPrefs.autoPip(app.core)
        activity?.let(Pip::sync)
        onDispose {
            Pip.onPlayer = false
            activity?.let(Pip::sync)
        }
    }
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()

    LaunchedEffect(bvid) { detail = runCatching { app.core.videoDetail(bvid) }.getOrNull() }

    // Recording the click is bookkeeping; never let it delay playback.
    LaunchedEffect(bvid) { runCatching { app.core.record(bvid, FfiSignal.Click) } }

    // Leaving the page stops playback. Going to the background does not dispose the page;
    // there playback pauses unless 后台播放 is on (read each time, so the toggle applies at once).
    DisposableEffect(Unit) { onDispose { PlayerHolder.clear() } }
    // A video page takes the player over from 听视频.
    LaunchedEffect(Unit) { AudioQueue.stop() }
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    DisposableEffect(lifecycle) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_STOP && !PlaybackPrefs.backgroundPlay(app.core)) {
                PlayerHolder.get(ctx).pause()
            }
        }
        lifecycle.addObserver(observer)
        onDispose { lifecycle.removeObserver(observer) }
    }

    // 历史记录: a heartbeat when a part starts, every 15 s while it plays, and when it is left
    // (another branch or the page closing). Positions are sampled every second so the last
    // report carries this part's own progress.
    val watched = remember { longArrayOf(0L, 0L) } // played s, real s
    LaunchedEffect(cid) {
        val part = cid ?: return@LaunchedEffect
        val player = PlayerHolder.get(ctx)
        val started = SystemClock.elapsedRealtime()
        watched[0] = 0L
        watched[1] = 0L
        runCatching { app.core.reportWatch(bvid, part, 0u, 0u, 1u) }
        var tick = 0
        while (true) {
            delay(1_000)
            if (player.isPlaying) watched[0] = player.currentPosition / 1000
            watched[1] = (SystemClock.elapsedRealtime() - started) / 1000
            if (++tick % 15 == 0 && player.isPlaying) {
                runCatching { app.core.reportWatch(bvid, part, watched[0].toULong(), watched[1].toULong(), 0u) }
            }
        }
    }
    DisposableEffect(cid) {
        val part = cid
        onDispose {
            if (part != null) {
                val played = watched[0].toULong()
                val real = watched[1].toULong()
                app.scope.launch { runCatching { app.core.reportWatch(bvid, part, played, real, 2u) } }
            }
        }
    }

    // From the play list: when this video ends (interactive ones follow their own story),
    // move on by the list's mode.
    DisposableEffect(fromPlaylist, stein) {
        if (!fromPlaylist || stein != null) return@DisposableEffect onDispose {}
        val player = PlayerHolder.get(ctx)
        val listener = object : androidx.media3.common.Player.Listener {
            override fun onPlaybackStateChanged(state: Int) {
                if (state != androidx.media3.common.Player.STATE_ENDED) return
                scope.launch {
                    val next = runCatching { app.core.playlistNext(bvid) }.getOrNull() ?: return@launch
                    if (next.bvid == bvid) {
                        player.seekTo(0)
                        player.play()
                    } else {
                        nav.replaceTop(Screen.Player(next.bvid, next.title, fromPlaylist = true))
                    }
                }
            }
        }
        player.addListener(listener)
        onDispose { player.removeListener(listener) }
    }

    LaunchedEffect(bvid, partCid) {
        // Drop the previous video (or branch) before fetching the next one.
        PlayerHolder.clear()
        danmaku = null
        dmStatus = "弹幕加载中…"
        runCatching {
            val t0 = SystemClock.elapsedRealtime()
            val qn = PlaybackPrefs.preferredQuality(app.core)
            val part = partCid
            val pb = if (part == null) app.core.playback(bvid, qn) else app.core.playbackPart(bvid, part, qn)
            playback = pb
            Log.i(TAG, "playurl ready in ${SystemClock.elapsedRealtime() - t0} ms")
            PlayerHolder.play(ctx, pb, title)
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
            runCatching {
                val session = app.core.danmakuSession(pb.cid, durationMs)
                val total = session.segmentCount()
                // The segment being watched first, then the rest from nearest to farthest.
                val here = session.segmentAt(PlayerHolder.get(ctx).currentPosition.toUInt())
                val order = (1u..total).sortedBy { kotlin.math.abs(it.toInt() - here.toInt()) }
                var done = 0
                for (batch in listOf(order.take(1)) + order.drop(1).chunked(3)) {
                    val set = session.load(batch)
                    done += batch.size
                    danmaku = set
                    dmStatus = "弹幕 ${set.items.size}" +
                        (if (set.jumpMarks.isNotEmpty()) " · 空降点 ${set.jumpMarks.size}" else "") +
                        if (done < total.toInt()) " · 加载中 $done/$total" else ""
                }
                Log.i(TAG, "danmaku loaded: ${danmaku?.items?.size} items in $total segments")
            }.onFailure { e ->
                dmStatus = if (danmaku == null) "弹幕加载失败：${e.message}" else "部分弹幕加载失败：${e.message}"
                Log.w(TAG, "danmaku load failed", e)
            }
        }.onFailure {
            error = playbackError(it)
            dmStatus = ""
        }
    }

    FullscreenMode(fullscreen)
    BackHandler(enabled = fullscreen) { fullscreen = false }

    Column(Modifier.fillMaxSize()) {
        if (!compact) Row(Modifier.fillMaxWidth().padding(start = 4.dp, end = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
            Meta(dmStatus, Modifier.weight(1f))
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
            if (activity != null && Pip.supported(activity)) {
                Text(
                    "小窗",
                    style = MetroType.meta.copy(fontSize = 12.sp),
                    color = c.fg,
                    modifier = Modifier.size(44.dp, 44.dp).wrapContentSize(Alignment.CenterEnd).clickable { Pip.enter(activity) },
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
            (if (compact) Modifier.fillMaxSize() else Modifier.fillMaxWidth().aspectRatio(16f / 9f)).background(Color.Black),
        ) {
            AndroidView(
                factory = {
                    (LayoutInflater.from(it).inflate(R.layout.player_view, null) as PlayerView).apply {
                        player = PlayerHolder.get(it)
                        setFullscreenButtonClickListener { on -> fullscreen = on }
                        setControllerVisibilityListener(PlayerView.ControllerVisibilityListener { v -> controlsVisible = v == View.VISIBLE })
                        playerView = this
                    }
                },
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
            if (fullscreen && !inPip && !controlsVisible) FullscreenGestures(PlayerHolder.get(ctx), playerView, holdSpeed)
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
                        val at = PlayerHolder.get(ctx).currentPosition.toUInt()
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
                            PlayerHolder.get(ctx).seekTo(m.targetMs.toLong())
                        },
                    )
                }
            }
        }
        LazyColumn(Modifier.weight(1f), state = listState) {
            item(key = "head") {
                Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(video?.title ?: title, style = MetroType.title, color = c.fg)
                    video?.let { Meta("${formatCount(it.views)} 播放 · ${formatAge(it.pubdate)} · ${it.zone ?: ""}") }
                    video?.let { v ->
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
                        key(it.bvid) { VideoActions(app, nav, it) }
                        VideoDetailBlock(it, detailOpen) { detailOpen = !detailOpen }
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
                        scope.launch {
                            if (picked == tab) {
                                listState.animateScrollToItem(0)
                            } else {
                                tab = picked
                                if (listState.firstVisibleItemIndex >= TABS_INDEX) listState.scrollToItem(TABS_INDEX)
                            }
                        }
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
                            v.aid,
                            focus,
                            onOpenVideo = { link -> nav.openVideo(scope, app, link.video, link.title) },
                        ) { scope.launch { listState.scrollToItem(TABS_INDEX) } }
                    }
                }
            }
        }
    }

    val current = playback
    if (qualitySheet && current != null) {
        QualitySheet(current.accept, current.quality, onDismiss = { qualitySheet = false }) { q ->
            qualitySheet = false
            PlaybackPrefs.setPreferredQuality(app.core, q.qn)
            scope.launch {
                // Same video, other streams: carry on from where it was.
                val at = PlayerHolder.get(ctx).currentPosition
                runCatching { cid?.let { app.core.playbackPart(bvid, it, q.qn) } ?: app.core.playback(bvid, q.qn) }
                    .onSuccess { p -> playback = p; PlayerHolder.play(ctx, p, video?.title ?: title, at) }
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
        )
    }
}

@Composable
private fun RelatedList(app: RubiApp, nav: Nav, current: String) {
    val c = LocalMetro.current
    val cards = remember { mutableStateListOf<FfiVideoCard>() }
    var note by remember { mutableStateOf("") }
    LaunchedEffect(current) {
        runCatching { app.core.related(current) }
            .onSuccess { list -> cards.addAll(list.filter { it.bvid != current }) }
            .onFailure { e ->
                // Fall back to the local pool so the tab is never empty.
                note = "相关推荐加载失败，显示本地推荐：${e.message}"
                val page = withContext(Dispatchers.IO) { runCatching { app.core.nextPage(6u) }.getOrDefault(emptyList()) }
                cards.addAll(page.filter { it.bvid != current })
            }
    }
    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(24.dp)) {
        if (note.isNotEmpty()) Text(note, color = c.fg2, fontSize = 12.sp)
        cards.forEach { card -> VideoCardView(card, big = false, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) }) }
    }
}

@Composable
private fun CommentSection(
    app: RubiApp,
    aid: ULong,
    focus: FfiCommentRef?,
    onOpenVideo: (FfiVideoLink) -> Unit,
    onFocusShown: () -> Unit,
) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var byTime by remember { mutableStateOf(false) }
    var pinned by remember { mutableStateOf(emptyList<FfiCommentRow>()) }
    var hiddenPinned by remember { mutableStateOf(0u) }
    val threads = remember { mutableStateListOf<FfiCommentThread>() }
    val collapsed = remember { mutableStateMapOf<ULong, Boolean>() }
    var error by remember { mutableStateOf<String?>(null) }
    // The thread holding the notice's comment, shown first and kept across sort changes.
    var focusThread by remember { mutableStateOf<FfiCommentThread?>(null) }
    var focusNote by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(aid, focus) {
        if (focus == null) return@LaunchedEffect
        runCatching { app.core.focusThread(aid, focus) }
            .onSuccess { focusThread = it; onFocusShown() }
            .onFailure { focusNote = "无法定位到消息里的评论（可能已被删除或折叠）" }
    }

    // Pages after the first follow the API's cursor; `round` restarts from the top (tapping
    // the active sort again).
    var next by remember { mutableStateOf<String?>(null) }
    var hasMore by remember { mutableStateOf(true) }
    var loadingMore by remember { mutableStateOf(false) }
    var round by remember { mutableStateOf(0) }

    fun loadMore() {
        if (loadingMore || !hasMore) return
        loadingMore = true
        scope.launch {
            runCatching { app.core.comments(aid, byTime, next) }
                .onSuccess { page ->
                    threads.addAll(page.threads.filter { t -> threads.none { it.rootRpid == t.rootRpid } })
                    next = page.nextOffset
                    hasMore = page.nextOffset != null && page.threads.isNotEmpty()
                    error = null
                }
                .onFailure { error = "评论加载失败：${it.message}" }
            loadingMore = false
        }
    }

    LaunchedEffect(aid, byTime, round) {
        loadingMore = true
        runCatching { app.core.comments(aid, byTime, null) }
            .onSuccess { page ->
                pinned = page.pinned
                hiddenPinned = page.hiddenPinned
                threads.clear()
                threads.addAll(page.threads)
                next = page.nextOffset
                hasMore = page.nextOffset != null && page.threads.isNotEmpty()
                error = null
            }
            .onFailure { error = it.message }
        loadingMore = false
    }

    Column(Modifier.padding(horizontal = 16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Meta("已隐藏置顶带货评论 $hiddenPinned 条", Modifier.weight(1f))
            listOf("按热度" to false, "按时间" to true).forEach { (label, t) ->
                Text(
                    label,
                    style = MetroType.meta.copy(fontSize = 12.sp),
                    color = if (byTime == t) c.fg else c.fg2,
                    fontWeight = if (byTime == t) FontWeight.SemiBold else FontWeight.Normal,
                    modifier = Modifier.padding(start = 16.dp).size(width = 56.dp, height = 44.dp).clickable {
                        if (byTime == t) round++ else byTime = t
                    },
                )
            }
        }
        error?.let { Text(it, color = c.fg2, fontSize = 13.sp) }
        focusNote?.let { Meta(it) }
        pinned.forEach { CommentRow(it, open = BooleanArray(0), pinnedLabel = true, onLink = onOpenVideo) }
        val focused = focusThread
        val shownThreads = listOfNotNull(focused) + threads.filter { it.rootRpid != focused?.rootRpid }
        fun replace(t: FfiCommentThread) {
            if (t.rootRpid == focused?.rootRpid) focusThread = t
            else threads.indexOfFirst { it.rootRpid == t.rootRpid }.takeIf { it >= 0 }?.let { threads[it] = t }
        }
        shownThreads.forEach { thread ->
            val isCollapsed = collapsed[thread.rootRpid] == true
            val rows = if (isCollapsed) thread.rows.take(1) else thread.rows
            Column(Modifier.padding(vertical = 10.dp)) {
                val open = BooleanArray(16)
                rows.forEach { row ->
                    // Snapshot: drawing happens later, after `open` has moved on.
                    CommentRow(row, open.copyOf(), highlight = thread === focused && row.rpid == focus?.rpid, onLink = onOpenVideo)
                    if (row.depth.toInt() in 1 until open.size) open[row.depth.toInt()] = row.hasLaterSibling
                }
                val shown = (thread.rows.size - 1).toUInt()
                val actions = buildList {
                    if (isCollapsed) add("展开 ${thread.totalReplies} 条回复" to { collapsed[thread.rootRpid] = false })
                    else {
                        if (thread.hidden > 0u) add("再看 5 条 · 剩 ${thread.hidden}" to {
                            scope.launch {
                                runCatching { app.core.commentThread(aid, thread.rootRpid, shown + 5u) }
                                    .onSuccess { replace(it) }
                            }
                            Unit
                        })
                        if (thread.rows.size > 1) add("折叠" to { collapsed[thread.rootRpid] = true })
                    }
                }
                if (actions.isNotEmpty()) {
                    Row(Modifier.padding(start = 22.dp), horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                        actions.forEach { (label, act) -> TextAction(label, onClick = act) }
                    }
                }
            }
        }
        // The whole section is one list item, so the end is noticed by where this row is:
        // once it scrolls into the window, the next page loads.
        val screenH = with(LocalDensity.current) { LocalConfiguration.current.screenHeightDp.dp.toPx() }
        Column(
            Modifier.fillMaxWidth().padding(vertical = 16.dp).onGloballyPositioned { coords ->
                if (coords.boundsInWindow().top < screenH && error == null) loadMore()
            },
        ) {
            when {
                loadingMore -> Meta("评论加载中…")
                error != null -> TextAction("加载更多评论", onClick = { error = null; loadMore() })
                !hasMore && threads.isNotEmpty() -> Meta("没有更多评论了")
                else -> {}
            }
        }
    }
}

/**
 * One fishbone row. `open[l]` tells whether the ancestor branch at depth `l` continues
 * below this row, so its vertical line is drawn through.
 */
@Composable
private fun CommentRow(
    row: FfiCommentRow,
    open: BooleanArray,
    pinnedLabel: Boolean = false,
    highlight: Boolean = false,
    onLink: (FfiVideoLink) -> Unit = {},
) {
    val c = LocalMetro.current
    val depth = row.depth.toInt()
    val indent = if (depth == 0) 0.dp else ((depth - 1) * 20 + 22).dp
    Column(
        Modifier.fillMaxWidth()
            .drawBehind {
                if (depth == 0) return@drawBehind
                val stroke = 1.dp.toPx()
                val tick = 16.dp.toPx()
                fun x(l: Int) = ((l - 1) * 20 + 6).dp.toPx()
                for (l in 1 until depth) {
                    if (l < open.size && open[l]) drawLine(c.spine, Offset(x(l), 0f), Offset(x(l), size.height), stroke)
                }
                val xd = x(depth)
                drawLine(c.spine, Offset(xd, 0f), Offset(xd, if (row.hasLaterSibling) size.height else tick), stroke)
                drawLine(c.spine, Offset(xd, tick), Offset(xd + 10.dp.toPx(), tick), stroke)
            }
            .padding(start = indent, top = 6.dp, bottom = 6.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            if (pinnedLabel) Tag("置顶")
            if (highlight) Tag("消息")
            Text(
                row.name,
                fontSize = if (depth == 0) 14.sp else 13.sp,
                fontWeight = FontWeight.SemiBold,
                color = if (highlight) c.accent else c.fg,
            )
            if (row.isUp) Tag("UP")
            if (row.noteCvid > 0u) Tag("笔记")
            Meta(formatAge(row.ctime))
        }
        // Long comments fold to six lines.
        var expanded by remember(row.rpid) { mutableStateOf(false) }
        var overflows by remember(row.rpid) { mutableStateOf(false) }
        EmoteText(
            (row.replyTo?.let { "回复 @$it：" } ?: "") + row.text,
            color = c.fg,
            fontSize = 14.sp,
            lineHeight = 21.sp,
            maxLines = if (expanded) Int.MAX_VALUE else COMMENT_LINES,
            extra = row.emotes,
            links = row.videoLinks,
            onLink = onLink,
            onTextLayout = { if (!expanded) overflows = it.hasVisualOverflow },
        )
        if (overflows || expanded) {
            Text(
                if (expanded) "收起" else "展开全文",
                fontSize = 13.sp,
                color = c.accent,
                modifier = Modifier.clickable { expanded = !expanded }.padding(vertical = 2.dp),
            )
        }
        if (row.pictures.isNotEmpty()) CommentPictures(row.pictures)
        if (row.noteCvid > 0u) {
            val ctx = LocalContext.current
            // Rubi has no article reader yet (文字专栏 is phase 2): the full note opens in the browser.
            TextAction("查看完整笔记", onClick = {
                ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse("https://www.bilibili.com/read/cv${row.noteCvid}")))
            })
        }
        Meta("赞 ${row.likes}")
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
