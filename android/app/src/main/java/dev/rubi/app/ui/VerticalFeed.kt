package dev.rubi.app.ui

import dev.rubi.app.data.UiPrefs
import androidx.compose.foundation.border
import dev.rubi.core.PortraitService
import androidx.compose.foundation.layout.widthIn
import android.view.LayoutInflater
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.pager.VerticalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.ui.PlayerView
import androidx.compose.runtime.DisposableEffect
import androidx.media3.common.Player
import coil3.compose.AsyncImage
import dev.rubi.app.R
import dev.rubi.app.data.DanmakuDisplay
import dev.rubi.app.data.PlaybackPrefs
import dev.rubi.app.playback.LocalMedia
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.DiscoverService
import dev.rubi.core.VideosService
import dev.rubi.core.FfiCommentArea
import dev.rubi.core.FfiDanmaku
import dev.rubi.core.FfiDanmakuSet
import dev.rubi.core.FfiVideoCard
import dev.rubi.core.FfiVideoDetail
import dev.rubi.core.FfiVideoRelation
import dev.rubi.core.RubiException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** [dev.rubi.app.data.UiPrefs.portraitCover] value for 填充. */
const val PORTRAIT_FILL = 1

/** 竖屏视频封面 in 16:9 frames: true crops the picture to fill (填充), false fits it (适应). */
val LocalPortraitCover = staticCompositionLocalOf { false }

/**
 * A video's cover in a 16:9 frame. A portrait video's cover keeps the card's shape: 适应 shows
 * the whole picture over a blurred, dimmed copy of itself; 填充 crops it like any other.
 */
@Composable
fun VideoCover(url: String, portrait: Boolean, modifier: Modifier = Modifier) {
    if (!portrait || LocalPortraitCover.current) {
        AsyncImage(model = url, contentDescription = null, contentScale = ContentScale.Crop, modifier = modifier)
        return
    }
    Box(modifier.clipToBounds()) {
        // Blur needs Android 12; before that the dimmed copy alone stands behind.
        AsyncImage(model = url, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize().blur(16.dp))
        Box(Modifier.fillMaxSize().background(Color(0x66000000)))
        AsyncImage(model = url, contentDescription = null, contentScale = ContentScale.Fit, modifier = Modifier.fillMaxSize())
    }
}

/** A moving line along a cover's foot while its video is looked up before opening. */
@Composable
fun ProbeLine(bvid: String, modifier: Modifier = Modifier) {
    if (LocalProbing.current != bvid) return
    val c = LocalMetro.current
    val run = rememberInfiniteTransition(label = "probe")
    val at by run.animateFloat(0f, 1f, infiniteRepeatable(tween(700), RepeatMode.Restart), label = "probe-at")
    BoxWithConstraints(modifier.fillMaxWidth().height(3.dp)) {
        val w = maxWidth
        Box(Modifier.width(w / 3).height(3.dp).offset(x = w * at - w / 3).background(c.accent))
    }
}

/**
 * 竖屏推流: the portrait video opened, then portrait videos from its 相关推荐 one after another
 * (through the rule table, none twice), topped up as the end comes near. Entered from the 竖屏
 * entry instead (`pool` given, no video opened), it takes the recommendation pool's portrait
 * videos and plays the first as soon as there is one. Kept with the page in the back stack.
 */
class VerticalFeedState(private val videos: VideosService, private val discover: DiscoverService, private val startBvid: String, val startMs: Long, danmaku: Boolean = true, private val pool: PortraitService? = null) {
    /** Entered from the 竖屏 entry: the videos come from the recommendation pool. */
    val fromPool get() = pool != null
    /** The first portrait videos are being picked. */
    var picking by mutableStateOf(false)
        private set
    /** Risk control holds the sorting back. */
    var limited by mutableStateOf(false)
        private set
    private var poolStarted = false
    val items = mutableStateListOf<FfiVideoCard>()
    /** 弹幕 shown over the feed; the same setting as the video page's 弹 开 / 关. */
    var danmakuOn by mutableStateOf(danmaku)
    /** 清屏: only the picture and the danmaku. Kept while moving through the feed. */
    var clean by mutableStateOf(false)
    var problem by mutableStateOf<String?>(null)
        private set
    private val seen = mutableSetOf<String>()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private var loading = false

    /** Loads the first video once. */
    suspend fun start() {
        if (items.isNotEmpty()) return
        pool?.let { pickFirst(it); return }
        runCatching { withContext(Dispatchers.IO) { videos.video(startBvid) } }
            .onSuccess { seen += it.bvid; items += it; more() }
            .onFailure { problem = errorText(it) }
    }

    /** Picks the first portrait videos from the pool; asks again on 继续等. */
    private suspend fun pickFirst(pool: PortraitService) {
        problem = null
        picking = true
        if (!poolStarted) {
            pool.startPortrait()
            poolStarted = true
        }
        runCatching { withContext(Dispatchers.IO) { pool.nextPortrait(1u) } }
            .onSuccess { picked ->
                limited = picked.limited
                picked.cards.filter { seen.add(it.bvid) }.forEach { items += it }
                if (items.isNotEmpty()) more()
            }
            .onFailure { problem = errorText(it) }
        picking = false
    }

    /** 继续等: picks again. */
    fun pickAgain() {
        val pool = pool ?: return
        scope.launch { pickFirst(pool) }
    }

    /**
     * Adds portrait videos: from the pool when the feed began there, else related to the last
     * ones shown, looking back through a few when the last one's related list has none to give.
     */
    fun more() {
        if (loading || items.isEmpty()) return
        loading = true
        pool?.let { pool ->
            scope.launch {
                val picked = runCatching { withContext(Dispatchers.IO) { pool.nextPortrait(BATCH.toUInt()) } }.getOrNull()
                if (picked != null) {
                    limited = picked.limited
                    items.addAll(picked.cards.filter { seen.add(it.bvid) })
                }
                loading = false
            }
            return
        }
        scope.launch {
            val added = mutableListOf<FfiVideoCard>()
            for (from in items.takeLast(LOOK_BACK).asReversed()) {
                val related = runCatching { withContext(Dispatchers.IO) { discover.related(from.bvid) } }.getOrDefault(emptyList())
                related.filter { it.portrait && seen.add(it.bvid) }.forEach { added += it }
                if (added.size >= BATCH) break
            }
            items.addAll(added)
            loading = false
        }
    }

    private companion object {
        /** Videos added at a time, at least. */
        const val BATCH = 6
        /** Videos looked back through for related ones. */
        const val LOOK_BACK = 3
    }
}

/** Fewer than this many left after the one shown: fetch more. */
private const val FEED_AHEAD = 3

@Composable
fun VerticalFeedScreen(deps: VideoPageDeps, nav: Nav, state: VerticalFeedState) {
    val settings = deps.settings
    PlayerPageLifecycle(settings)
    LaunchedEffect(state) { state.start() }
    val pager = rememberPagerState { state.items.size }
    LaunchedEffect(pager, state) {
        snapshotFlow { pager.currentPage to state.items.size }.collect { (at, size) -> if (at >= size - FEED_AHEAD) state.more() }
    }
    Box(Modifier.fillMaxSize().background(Color.Black)) {
        if (state.items.isEmpty()) {
            if (state.fromPool) {
                PoolWaiting(state, nav, Modifier.align(Alignment.Center))
            } else {
                Text(state.problem ?: "加载中…", color = Color.White, fontSize = 13.sp, modifier = Modifier.align(Alignment.Center))
            }
        }
        VerticalPager(pager, Modifier.fillMaxSize(), beyondViewportPageCount = 1, key = { state.items[it].bvid }) { page ->
            val card = state.items[page]
            FeedPage(deps, nav, state, card, index = page, active = page == pager.settledPage, startMs = if (page == 0) state.startMs else 0L)
        }
    }
}

/** The 竖屏 entry before its first video: picking, nothing found yet, or held back by risk control. */
@Composable
private fun PoolWaiting(state: VerticalFeedState, nav: Nav, modifier: Modifier) {
    val (title, line) = when {
        state.picking -> "正在挑选竖屏视频…" to "从推荐池里挑，找到第一个就开始播放"
        state.limited -> "B 站暂时限制了查询" to "过一会儿再挑选；可以先返回首页"
        state.problem != null -> "挑选失败" to (state.problem ?: "")
        else -> "推荐池里暂时没有竖屏视频" to "正在拉取新一批推荐并挑选，也会沿相关推荐找"
    }
    Column(modifier.padding(horizontal = 32.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        GlyphIcon(Glyph.Portrait, color = Color.White, size = 44.dp)
        Text(title, color = Color.White, fontSize = 16.sp, modifier = Modifier.padding(top = 14.dp))
        Text(line, color = Color(0xB3FFFFFF), fontSize = 12.sp, modifier = Modifier.padding(top = 6.dp))
        if (state.picking) {
            val run = rememberInfiniteTransition(label = "picking")
            val at by run.animateFloat(0f, 1f, infiniteRepeatable(tween(900), RepeatMode.Restart), label = "picking-at")
            BoxWithConstraints(Modifier.padding(top = 16.dp).width(180.dp).height(3.dp).background(Color(0x33FFFFFF)).clipToBounds()) {
                val w = maxWidth
                Box(Modifier.width(w / 3).height(3.dp).offset(x = w * at - w / 3).background(LocalMetro.current.accent))
            }
        } else {
            Row(Modifier.padding(top = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                if (!state.limited) PoolChoice("继续等") { state.pickAgain() }
                PoolChoice("返回首页") { nav.pop() }
            }
        }
    }
}

@Composable
private fun PoolChoice(label: String, onClick: () -> Unit) {
    Text(
        label,
        color = Color.White,
        fontSize = 13.sp,
        modifier = Modifier.border(1.dp, Color.White).clickable(onClick = onClick).padding(horizontal = 14.dp, vertical = 6.dp),
    )
}

/** One video of the feed: the picture (playing only while it is the one shown) and what lies over it. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun FeedPage(deps: VideoPageDeps, nav: Nav, state: VerticalFeedState, card: FfiVideoCard, index: Int, active: Boolean, startMs: Long) {
    val accountService = deps.account
    val videos = deps.videos
    val danmakuService = deps.danmaku
    val actionsService = deps.actionRow.actions
    val settings = deps.settings
    val media = LocalMedia.current
    val scope = rememberCoroutineScope()
    var detail by remember { mutableStateOf<FfiVideoDetail?>(null) }
    var rel by remember { mutableStateOf<FfiVideoRelation?>(null) }
    // Liked on this page, so the count shown includes it (the details were read before).
    // How far this page moved the counts from the ones read (given +1, taken back -1; coins
    // by how many), here or in the action panel.
    var likeDelta by remember { mutableStateOf(0) }
    var coinDelta by remember { mutableStateOf(0) }
    var favDelta by remember { mutableStateOf(0) }
    // The action panel changed the relation: the side column follows, counts included.
    fun relationChanged(n: FfiVideoRelation) {
        val old = rel ?: run { rel = n; return }
        if (old == n) return
        if (old.liked != n.liked) likeDelta += if (n.liked) 1 else -1
        coinDelta += n.coins.toInt() - old.coins.toInt()
        if (old.favorited != n.favorited) favDelta += if (n.favorited) 1 else -1
        rel = n
    }
    var cid by remember { mutableStateOf<ULong?>(null) }
    var danmaku by remember { mutableStateOf<FfiDanmakuSet?>(null) }
    val display = remember { DanmakuDisplay.load(settings) }
    var playing by remember { mutableStateOf(true) }
    var started by remember { mutableStateOf(false) }
    var note by remember { mutableStateOf("") }
    var panel by remember { mutableStateOf<FeedPanel?>(null) }
    val zoom = remember { VideoZoom() }
    // The like burst: where it shows and how far it has gone.
    var burstAt by remember { mutableStateOf<Offset?>(null) }
    val burst = remember { Animatable(0f) }

    LaunchedEffect(card.bvid) {
        detail = runCatching { videos.videoDetail(card.bvid) }.getOrNull()
        rel = runCatching { if (accountService.account().loggedIn) actionsService.relation(card.bvid) else null }.getOrNull()
    }
    // The cover stays over the picture until this video's own first frame is drawn. A frame
    // drawn before this video is loaded is the last one's: the new view, joining the player
    // while it still holds the video before, has it drawn again.
    var framed by remember { mutableStateOf(false) }
    var loaded by remember { mutableStateOf(false) }
    DisposableEffect(active) {
        if (!active) {
            framed = false
            loaded = false
            return@DisposableEffect onDispose {}
        }
        val player = media.player.get()
        val listener = object : Player.Listener {
            override fun onRenderedFirstFrame() {
                if (loaded) framed = true
            }
        }
        player.addListener(listener)
        onDispose { player.removeListener(listener) }
    }
    // Only the video shown plays; the next one waits with its cover.
    LaunchedEffect(active) {
        if (!active) {
            started = false
            return@LaunchedEffect
        }
        framed = false
        loaded = false
        media.player.clear()
        runCatching {
            val pb = videos.playback(card.bvid, PlaybackPrefs.preferredQuality(settings))
            loaded = true
            media.player.play(pb, card.title, startMs)
            started = true
            playing = true
            cid = pb.cid
            val durationMs = maxOf(card.durationS.toULong() * 1000u, pb.durationMs).toUInt()
            loadDanmakuNearFirst(danmakuService, pb.cid, durationMs, media.player.get().currentPosition) { set, _ -> if (set != null) danmaku = set }
        }.onFailure { note = playbackError(it) }
    }

    fun setLiked(r: FfiVideoRelation, on: Boolean) {
        val delta = if (on) 1 else -1
        rel = r.copy(liked = on)
        likeDelta += delta
        scope.launch {
            runCatching { actionsService.like(card.bvid, on) }.onFailure {
                rel = r
                likeDelta -= delta
                note = if (it is RubiException.NotLoggedIn) "需要登录" else (if (on) "点赞失败：" else "取消点赞失败：") + it.message
            }
        }
    }

    fun like() {
        val r = rel
        if (r == null) {
            nav.push(Screen.Login)
            return
        }
        if (r.liked) return
        setLiked(r, true)
    }

    // The 赞 button: likes, or takes the like back. (A double tap only ever likes.)
    fun toggleLike() {
        val r = rel
        if (r == null) {
            nav.push(Screen.Login)
            return
        }
        setLiked(r, !r.liked)
    }


    Box(
        Modifier.fillMaxSize().clipToBounds()
            .pinchToZoom(zoom)
            .pointerInput(card.bvid) {
                detectTapGestures(
                    // 单击: 暂停 / 继续.
                    onTap = {
                        if (!active) return@detectTapGestures
                        // By intent, not by state: a tap while buffering still pauses.
                        val p = media.player.get()
                        p.playWhenReady = !p.playWhenReady
                        playing = p.playWhenReady
                    },
                    // 双击: 点赞, with a burst where the finger was.
                    onDoubleTap = { at ->
                        like()
                        burstAt = at
                        scope.launch {
                            burst.snapTo(0f)
                            burst.animateTo(1f, tween(700))
                            burstAt = null
                        }
                    },
                )
            },
    ) {
        if (active) {
            AndroidView(
                factory = {
                    (LayoutInflater.from(it).inflate(R.layout.player_view, null) as PlayerView).apply {
                        player = media.player.get()
                        useController = false
                    }
                },
                onRelease = { it.player = null },
                update = { it.resizeMode = VideoScale.Fit.resizeMode() },
                modifier = Modifier.fillMaxSize().zoomedBy(zoom),
            )
        }
        // The cover until this video's first frame (and for the pages beside the one shown).
        if (!framed) VideoCover(card.cover, portrait = true, Modifier.fillMaxSize())
        if (active) {
            danmaku?.let { if (state.danmakuOn) DanmakuLayer(danmakuService, it, display, Modifier.fillMaxSize().padding(top = 56.dp)) }
        }
        if (active && !playing) {
            Box(Modifier.align(Alignment.Center).size(64.dp).background(Color(0x73000000)), contentAlignment = Alignment.Center) {
                Text("▶", color = Color.White, fontSize = 26.sp)
            }
        }
        burstAt?.let { at ->
            val density = LocalDensity.current
            val half = with(density) { 36.dp.toPx() }
            Box(
                Modifier.offset { IntOffset((at.x - half).toInt(), (at.y - half - burst.value * 60f).toInt()) }
                    .graphicsLayer {
                        val v = burst.value
                        alpha = if (v < 0.3f) v / 0.3f else 1f - (v - 0.3f) / 0.7f
                        val s = 0.6f + 0.5f * minOf(v / 0.3f, 1f)
                        scaleX = s
                        scaleY = s
                    },
            ) { LineIconView(LineIcon.Like, Color(0xFF5B8BFF), 72.dp) }
        }
        ZoomReset(zoom, Modifier.align(Alignment.TopEnd).padding(top = 92.dp))

        // Top: back, where in the feed, 清屏 and the ordinary video page. While the screen is
        // clear only 清屏 stays, where it was, to turn it off.
        Row(
            Modifier.fillMaxWidth().align(Alignment.TopStart).padding(start = 4.dp, end = 12.dp, top = 4.dp).height(44.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (!state.clean) {
                Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back, Color.White) }
                Text("竖屏推流 · 第 ${index + 1} 条", fontSize = 12.sp, color = Color(0xD9FFFFFF), modifier = Modifier.weight(1f))
            } else {
                Box(Modifier.weight(1f))
            }
            Row(
                Modifier.background(if (state.clean) LocalMetro.current.accent else Color(0x59000000))
                    .clickable { state.clean = !state.clean }
                    .padding(horizontal = 8.dp, vertical = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                if (!LocalWordsUi.current) LineIconView(LineIcon.Eye, Color.White, 16.dp)
                Text(if (state.clean) "清屏 开" else "清屏", fontSize = 11.sp, color = Color.White)
            }
            // Hidden while the screen is clear, but keeping its room so 清屏 does not move.
            Text(
                "普通页",
                fontSize = 12.sp,
                color = if (state.clean) Color.Transparent else Color.White,
                modifier = Modifier.clickable(enabled = !state.clean) {
                    val at = if (active) media.player.get().currentPosition else 0L
                    nav.replaceTop(Screen.Player(card.bvid, card.title, startMs = at, normal = true))
                }.padding(8.dp),
            )
        }
        if (state.clean) return@Box

        // Right: the actions, in the straight-line icons.
        Column(
            // Seven actions: kept clear of the 清屏 switch above them.
            Modifier.align(Alignment.BottomEnd).padding(end = 4.dp, bottom = 112.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(SIDE_GAP),
        ) {
            // Counts instead of words; a count turns Cobalt once this account has given its own.
            val r = rel
            val d = detail
            val liked = r?.liked == true
            fun count(n: ULong?, word: String) = n?.let(::formatCount) ?: word
            SideAction(LineIcon.Like, count(d?.likes?.let { (it.toLong() + likeDelta).coerceAtLeast(0L).toULong() }, "赞"), liked, "赞") { toggleLike() }
            SideAction(LineIcon.Coin, count(d?.coins?.let { (it.toLong() + coinDelta).coerceAtLeast(0L).toULong() }, "投币"), (r?.coins ?: 0u) > 0u, "币") { panel = FeedPanel.Actions(ActionOpen.Coin) }
            SideAction(LineIcon.Star, count(d?.favorites?.let { (it.toLong() + favDelta).coerceAtLeast(0L).toULong() }, "收藏"), r?.favorited == true, "藏") { panel = FeedPanel.Actions(ActionOpen.Favorite) }
            SideAction(LineIcon.Comment, count(d?.replies, "评论"), false, "评") { panel = FeedPanel.Sheet(0) }
            SideAction(LineIcon.Share, "分享", false) { panel = FeedPanel.Actions(ActionOpen.Share) }
            // 弹幕 on / off for the whole feed, kept as the video page's setting too.
            // 文字: always 「弹幕」, Cobalt when on and dimmed when off. 图形: the icon tells.
            val words = LocalWordsUi.current
            SideAction(
                if (state.danmakuOn) LineIcon.Danmaku else LineIcon.DanmakuOff,
                if (words || state.danmakuOn) "弹幕" else "弹幕关",
                words && state.danmakuOn,
                off = words && !state.danmakuOn,
            ) {
                state.danmakuOn = !state.danmakuOn
                runCatching { DanmakuDisplay.save(settings, DanmakuDisplay.load(settings).copy(enabled = state.danmakuOn)) }
            }
            SideAction(LineIcon.More, "更多", false) { panel = FeedPanel.Actions(null) }
        }

        // Bottom: who and what, the progress, and 发弹幕.
        Column(Modifier.align(Alignment.BottomStart).fillMaxWidth()) {
            Column(Modifier.padding(start = 12.dp, end = 72.dp, bottom = 8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(
                    "@${card.upName}",
                    fontSize = 14.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = Color.White,
                    modifier = Modifier.clickable { nav.push(Screen.Up(card.upMid, card.upName)) },
                )
                Text(card.title, fontSize = 13.sp, color = Color.White, maxLines = 2, overflow = TextOverflow.Ellipsis, modifier = Modifier.clickable { panel = FeedPanel.Sheet(2) })
                if (note.isNotEmpty()) Text(note, fontSize = 12.sp, color = Color(0xFFFF8A8A))
            }
            if (active) FeedProgress()
            Box(Modifier.fillMaxWidth().background(Brush.verticalGradient(listOf(Color.Transparent, Color(0xA6000000))))) {
                DanmakuSendBar(
                    send = { text, mode, color ->
                        val id = cid ?: throw IllegalStateException("视频还没加载完")
                        danmakuService.sendDanmaku(card.bvid, id, media.player.get().currentPosition.toUInt(), text, mode, color)
                    },
                    onSent = { dm: FfiDanmaku -> danmaku = danmaku?.let { it.copy(items = it.items + dm) } ?: FfiDanmakuSet(listOf(dm), emptyList(), emptyList()) },
                    strip = false,
                )
            }
        }
    }

    when (val p = panel) {
        is FeedPanel.Actions -> ModalBottomSheet(onDismissRequest = { panel = null }, containerColor = LocalMetro.current.bg2, shape = RectangleShape, dragHandle = null) {
            Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                val d = detail
                if (d == null) Meta("读取中…") else VideoActions(deps.actionRow, nav, d, open = p.open, onRelation = ::relationChanged)
            }
        }
        is FeedPanel.Sheet -> FeedSheet(deps.panel, nav, card, detail, p.tab) { panel = null }
        null -> {}
    }
}

/** What is pulled up over the feed: the action row (with one of its sheets open), or 评论 / 推荐 / 详情. */
private sealed interface FeedPanel {
    data class Actions(val open: ActionOpen?) : FeedPanel
    data class Sheet(val tab: Int) : FeedPanel
}

/**
 * One of the feed's side actions. 图形: the icon over `label` (its count, or a word). 文字: `name`
 * (a single character) large over `label`, or `label` alone when there is no name.
 */
/** Each side action takes the same cell, so the column lines up whatever it shows. */
private val SIDE_CELL_W = 56.dp
private val SIDE_CELL_H = 52.dp
private val SIDE_GAP = 6.dp

@Composable
private fun SideAction(icon: LineIcon, label: String, on: Boolean, name: String? = null, off: Boolean = false, onClick: () -> Unit) {
    val tint = when {
        on -> Color(0xFF5B8BFF)
        off -> Color(0x80FFFFFF)
        else -> Color.White
    }
    Column(
        Modifier.size(SIDE_CELL_W, SIDE_CELL_H).clickable(onClick = onClick),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        if (LocalWordsUi.current) {
            // 文字: one character over its count, or the word alone, at one size.
            if (name != null && name != label) {
                Text(name, fontSize = 19.sp, lineHeight = 22.sp, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Light, color = tint)
                Text(label, fontSize = 11.sp, lineHeight = 14.sp, color = tint, maxLines = 1)
            } else {
                Text(label, fontSize = 15.sp, lineHeight = 18.sp, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal, color = tint, maxLines = 1)
            }
        } else {
            LineIconView(icon, tint, 26.dp)
            Text(label, fontSize = 11.sp, lineHeight = 14.sp, color = tint, maxLines = 1, modifier = Modifier.padding(top = 3.dp))
        }
    }
}

/** A thin progress line under the text; a tap on it seeks there. */
@Composable
private fun FeedProgress() {
    val media = LocalMedia.current
    val scope = rememberCoroutineScope()
    var fraction by remember { mutableStateOf(0f) }
    LaunchedEffect(Unit) {
        while (true) {
            val p = media.player.get()
            val len = p.duration
            fraction = if (len > 0) (p.currentPosition.toFloat() / len).coerceIn(0f, 1f) else 0f
            delay(250)
        }
    }
    var width by remember { mutableStateOf(1) }
    Box(
        Modifier.fillMaxWidth().height(18.dp).padding(horizontal = 12.dp)
            .onSizeChanged { width = it.width.coerceAtLeast(1) }
            .pointerInput(Unit) {
                detectTapGestures { o ->
                    val p = media.player.get()
                    if (p.duration > 0) scope.launch { p.seekTo((o.x / width * p.duration).toLong()) }
                }
            },
        contentAlignment = Alignment.CenterStart,
    ) {
        Box(Modifier.fillMaxWidth().height(2.dp).background(Color(0x4DFFFFFF)))
        Box(Modifier.fillMaxWidth(fraction).height(2.dp).background(Color.White))
    }
}

/** 评论 / 推荐 / 详情 over the lower part of the screen; the video plays on above it. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun FeedSheet(deps: VideoPanelDeps, nav: Nav, card: FfiVideoCard, detail: FfiVideoDetail?, start: Int, onDismiss: () -> Unit) {
    val home = deps.home
    val videos = deps.videos
    val commentsService = deps.comments
    val discover = deps.discover
    val following = deps.following
    val seasons = deps.seasons
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var tab by remember { mutableStateOf(start) }
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg, shape = RectangleShape) {
        Pivot(listOf("评论", "推荐", "详情"), tab, { tab = it }, dev.rubi.app.ui.theme.MetroType.section)
        LazyColumn(Modifier.fillMaxWidth().height(460.dp)) {
            when (tab) {
                0 -> item(key = "comments") {
                    CommentSection(
                        commentsService, following,
                        remember(card.aid) { FfiCommentArea(1u, card.aid) },
                        null,
                        onOpenVideo = { link -> onDismiss(); nav.openVideo(scope, videos, link.video, link.title) },
                        onOpenArticle = { link -> onDismiss(); nav.push(Screen.Article(link)) },
                    ) {}
                }
                1 -> item(key = "related") { RelatedList(home, discover, nav, card.bvid) }
                else -> item(key = "detail") {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(card.title, style = dev.rubi.app.ui.theme.MetroType.title, color = c.fg)
                        val d = detail
                        if (d != null) {
                            Meta("${formatCount(d.views)} 播放 · ${formatAge(d.pubdate)} · ${d.zone ?: ""}")
                            VideoDetailBlock(d, true) {}
                            d.season?.let { s ->
                                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                                    Text("合集 · ${s.title}", fontSize = 15.sp, color = c.fg, modifier = Modifier.weight(1f))
                                    SeasonSubscribeButton(seasons, nav, s.id)
                                }
                            }
                        } else Meta("读取中…")
                    }
                }
            }
        }
    }
}
