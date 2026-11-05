package dev.rubi.app.ui

import dev.rubi.core.VideosService
import dev.rubi.core.PgcService
import dev.rubi.core.HomeService
import dev.rubi.core.DiscoverService
import dev.rubi.core.DanmakuService
import dev.rubi.app.data.CoreSettings
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.runtime.key
import android.util.Log
import androidx.compose.foundation.background
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.itemsIndexed
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.data.PreviewPrefs
import dev.rubi.app.data.UiPrefs
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiBreakdown
import dev.rubi.core.FfiSignal
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private const val PAGE = 20u

/** One list (推荐 / 热门 / 排行) that keeps its cards and scroll position. */
class FeedList {
    val cards = mutableStateListOf<FfiVideoCard>()
    val grid = LazyGridState()
    var page by mutableStateOf(1u)
    var loading by mutableStateOf(false)
    var error by mutableStateOf<String?>(null)
    /** The last attempt brought nothing new; stop loading at the end until asked again. */
    var ended by mutableStateOf(false)
}

/**
 * Home-tab state hoisted above the screen: it survives opening a video, switching tabs
 * and coming back, so the feed only changes when the user pulls to refresh.
 */
class HomeState(private val ui: UiPrefs) {
    /** Asks the page to fetch afresh, as when someone signs in. */
    var reloads by mutableStateOf(0)
    /** The last of [reloads] acted on: the page may only come on screen later, and only once. */
    var reloaded = 0

    val recommend = FeedList()
    val hot = FeedList()
    val rank = FeedList()
    var status by mutableStateOf("")
    /** Why sources failed on the last refresh; shown when the status line is tapped. */
    var sourceErrors by mutableStateOf(emptyList<String>())
    var showErrors by mutableStateOf(false)
    /** Two columns or one; remembered on this device. */
    var dual by mutableStateOf(ui.homeDual)
        private set

    fun toggleDual() {
        dual = !dual
        ui.homeDual = dual
    }
    var pivot by mutableStateOf(0)
    /** On a zone: 0 its 推荐, 1 its 热门. */
    var zoneSub by mutableStateOf(0)
    private val zoneLists = mutableMapOf<String, FeedList>()

    /** The list of zone `name`'s 推荐 or 热门, kept while the app runs. */
    fun zoneList(name: String, sub: Int): FeedList = zoneLists.getOrPut("$name/$sub") { FeedList() }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(homeService: HomeService, videos: VideosService, danmaku: DanmakuService, discover: DiscoverService, pgcService: PgcService, settings: CoreSettings, nav: Nav, home: HomeState) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val feed = home.recommend
    val cards = feed.cards
    val grid = feed.grid
    var why by remember { mutableStateOf<FfiVideoCard?>(null) }

    /** `reset` = user pulled to refresh; otherwise append the next page from the pool. */
    // `pulled`: 下拉刷新 — a new batch fetched now, and cards already seen kept out of it.
    suspend fun loadPage(reset: Boolean, pulled: Boolean = false) {
        // A rebuilt list may show earlier cards again; an appended page must not repeat them.
        // A pull keeps the session, so the new list starts with cards not yet shown.
        if (!pulled && (reset || cards.isEmpty())) homeService.newFeedSession()
        // Off the main thread: the core filters and stores hundreds of candidates while this
        // runs, and on the main thread taps would be dropped meanwhile.
        suspend fun refresh() = withContext(Dispatchers.IO) {
            runCatching { if (pulled) homeService.refreshFeedNow(20u) else homeService.refreshFeed(20u) }
        }
        var report = if (reset || cards.isEmpty()) refresh() else null
        suspend fun next() = withContext(Dispatchers.IO) { runCatching { homeService.nextPage(PAGE) }.getOrDefault(emptyList()) }
        var page = next()
        if (pulled && page.isEmpty()) {
            // Everything in the pool was seen already: start over rather than show nothing.
            homeService.newFeedSession()
            page = next()
        }
        if (page.isEmpty() && report == null) {
            // Pool used up while scrolling: fetch more candidates and append, never replace.
            report = refresh()
            page = next()
        }
        // Leaving the screen mid-refresh is not a failure worth reporting.
        (report?.exceptionOrNull() as? CancellationException)?.let { throw it }
        if (reset) cards.clear()
        val fresh = page.filter { p -> cards.none { it.bvid == p.bvid } }
        cards.addAll(fresh)
        feed.ended = fresh.isEmpty()
        report?.let {
            home.sourceErrors = it.getOrNull()?.errors.orEmpty()
            home.sourceErrors.forEach { e -> Log.w("RubiFeed", "source failed: $e") }
            home.status = it.fold(
                { r -> "↻ 本次保留 ${r.kept} · 过滤 ${r.blocked}" + if (r.errors.isNotEmpty()) " · ${r.errors.size} 源失败 ▾" else "" },
                { e -> "刷新失败：${e.message}" },
            )
        }
    }

    suspend fun load(reset: Boolean, pulled: Boolean = false) {
        if (feed.loading) return
        feed.loading = true
        try { loadPage(reset, pulled) } finally { feed.loading = false }
    }

    // First visit only: an existing list is kept as is when returning to this screen.
    LaunchedEffect(Unit) { if (cards.isEmpty()) load(reset = false) }
    LaunchedEffect(home.reloads) {
        if (home.reloads > home.reloaded) {
            home.reloaded = home.reloads
            load(reset = true)
        }
    }
    val nearEnd by remember { derivedStateOf { (grid.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0) >= cards.size - 4 } }
    // One long-lived collector: it fires again whenever a page lands and the end is still in
    // view, and nothing restarts (and so cancels) it while scrolling.
    LaunchedEffect(feed) {
        snapshotFlow { nearEnd && cards.isNotEmpty() && !feed.loading && !feed.ended }.collect { if (it) load(reset = false) }
    }
    LaunchedEffect(Unit) {
        nav.reselected.collect { t ->
            if (t == Tab.Home) {
                val target = when (home.pivot) { 1 -> home.hot.grid; 2 -> home.rank.grid; else -> grid }
                target.animateScrollToItem(0)
            }
        }
    }
    val refreshing = feed.loading
    val dual = home.dual
    val columns = LocalScreenLayout.current.videoColumns(dual)
    val ctx = LocalContext.current
    // Re-read when coming back from 设置.
    val preview = remember(nav.stack.size) { PreviewPrefs.load(settings) }
    val previewing = rememberPreviewTarget(grid, enabled = columns == 1 && home.pivot == 0 && preview.allowed(ctx))
    val pivot = home.pivot
    val zones = remember { discover.homeZones() }
    val pgcZones = remember { pgcService.pgcZones() }
    var zoneMenu by remember { mutableStateOf(false) }
    // How far the 分区 menu is shown: it and the dimming behind it fade in and out with the
    // page switch instead of vanishing at once.
    val motion = LocalMotion.current
    val menuShown = remember { Animatable(0f) }
    LaunchedEffect(zoneMenu) {
        val to = if (zoneMenu) 1f else 0f
        if (!motion.on) menuShown.snapTo(to) else runFixed { menuShown.animateTo(to, tween(motion.ms(380), easing = EnterEasing)) }
    }
    val turns = LocalPageTurns.current

    Column(Modifier.fillMaxSize()) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            // 推荐 / 热门 / 排行, then one 分区 button that unfolds the zones and takes the name of
            // the one picked.
            val zoneLabel = (zones.getOrNull(pivot - 3)?.name ?: "分区") + if (zoneMenu) " ▴" else " ▾"
            // Fixed in place, sized so the four fit beside 双列 even with a four-character zone.
            Box(Modifier.weight(1f)) { Pivot(listOf("推荐", "热门", "排行", zoneLabel), pivot.coerceAtMost(3), scroll = false, gap = 14.dp, style = MetroType.pivot.copy(fontSize = 22.sp, lineHeight = 32.sp), onSelect = { i ->
                if (i == 3) zoneMenu = !zoneMenu
                else {
                    zoneMenu = false
                    turns.switch(i > pivot) { home.pivot = i }
                }
            }) }
            // A wide screen sets its own columns: no 单列 / 双列 choice there.
            if (!LocalScreenLayout.current.wide) Text(
                if (dual) "▦ 双列" else "▭ 单列",
                style = MetroType.meta.copy(fontSize = 12.sp),
                color = c.fg,
                modifier = Modifier.padding(end = 16.dp).clickable { home.toggleDual() },
            )
        }
        // The unfolded 分区 menu floats over the feed instead of pushing it down.
        Box(Modifier.weight(1f)) {
        // The chosen page slides in from the side it was picked on.
        Column(Modifier.fillMaxSize().pivotSlide(pivot)) {
        if (pivot >= 3) {
            val zone = zones[pivot - 3]
            Row(Modifier.padding(horizontal = 16.dp, vertical = 4.dp), horizontalArrangement = Arrangement.spacedBy(18.dp)) {
                listOf("推荐", "热门").forEachIndexed { i, label ->
                    val on = i == home.zoneSub
                    Text(
                        label,
                        fontSize = 15.sp,
                        fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
                        color = if (on) c.fg else c.fg2,
                        modifier = Modifier.clickable { home.zoneSub = i }.padding(vertical = 6.dp),
                    )
                }
            }
            val sub = home.zoneSub
            key(zone.name, sub) {
                HotRankList(videos, danmaku, settings, nav, home.zoneList(zone.name, sub), ranking = sub == 1, dual = dual) { page ->
                    if (sub == 1) discover.zoneRanking(zone.rid) else discover.zoneFeed(zone.region, page)
                }
            }
            return@Column
        }
        if (pivot != 0) {
            HotRankList(videos, danmaku, settings, nav, if (pivot == 2) home.rank else home.hot, ranking = pivot == 2, dual = dual) { page ->
                if (pivot == 2) discover.ranking() else discover.popular(page)
            }
            return@Column
        }
        Row(Modifier.fillMaxWidth().height(44.dp).padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Meta(home.status, Modifier.weight(1f).clickable(enabled = home.sourceErrors.isNotEmpty()) { home.showErrors = !home.showErrors })
            Text("被过滤内容 →", style = MetroType.meta, color = c.fg, modifier = Modifier.clickable { nav.push(Screen.Filtered) })
        }
        if (home.showErrors) {
            Column(Modifier.padding(horizontal = 16.dp)) { home.sourceErrors.forEach { Meta(it) } }
        }
        PullToRefreshBox(isRefreshing = refreshing, onRefresh = { scope.launch { load(reset = true, pulled = true) } }) {
            LazyVerticalGrid(
                columns = GridCells.Fixed(columns),
                state = grid,
                contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 28.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalArrangement = Arrangement.spacedBy(if (dual) 22.dp else 30.dp),
                modifier = Modifier.fillMaxSize(),
            ) {
                items(cards, key = { it.bvid }) { card ->
                    VideoCardView(
                        card,
                        big = columns == 1,
                        onOpen = { nav.push(Screen.Player(card.bvid, card.title)) },
                        onMore = { why = card },
                        preview = if (card.bvid == previewing) { { FeedPreview(videos, danmaku, settings, card, preview) } } else null,
                    )
                }
                if (cards.isEmpty() && !refreshing) {
                    item(span = { GridItemSpan(maxLineSpan) }) { Text("候选池为空，下拉刷新", color = c.fg2) }
                } else if (feed.ended && !refreshing) {
                    item(span = { GridItemSpan(maxLineSpan) }) {
                        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                            Meta("暂时没有新的推荐（推荐源 30 秒内不重复请求）· 下拉刷新换一批")
                            TextAction("再试一次", onClick = { feed.ended = false })
                        }
                    }
                }
            }
        }
        }
        if (zoneMenu || menuShown.value > 0f) {
            val shown = menuShown.value
            // A tap beside the menu folds it.
            Box(
                Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.3f * shown))
                    .then(if (zoneMenu) Modifier.pointerInput(Unit) { detectTapGestures { zoneMenu = false } } else Modifier),
            )
            ZoneMenu(
                Modifier.graphicsLayer {
                    alpha = shown
                    translationY = -(1f - shown) * 16.dp.toPx()
                },
                zones.map { it.name }, pgcZones.map { it.name }, selected = pivot - 3,
                onZone = { i ->
                    zoneMenu = false
                    if (pivot != 3 + i) turns.switch(3 + i > pivot) { home.pivot = 3 + i }
                },
                onPgc = { i ->
                    zoneMenu = false
                    val pgc = pgcZones[i]
                    nav.push(Screen.PgcZone(pgc.seasonType, pgc.name, PgcZoneState(pgcService, pgc.seasonType)))
                },
            )
        }
        }
    }

    why?.let { card ->
        WhySheet(homeService, card, onDismiss = { why = null }, onDisliked = {
            cards.removeAll { it.bvid == card.bvid }
            why = null
        })
    }
}

/** 热门 (paged grid) and 排行 (numbered list); both already filtered by the rule table. */
@Composable
private fun HotRankList(videos: VideosService, danmaku: DanmakuService, settings: CoreSettings, nav: Nav, list: FeedList, ranking: Boolean, dual: Boolean, load: suspend (page: UInt) -> List<FfiVideoCard>) {
    val layout = LocalScreenLayout.current
    val ctx = LocalContext.current
    val preview = remember(nav.stack.size) { PreviewPrefs.load(settings) }
    val previewing = rememberPreviewTarget(list.grid, enabled = layout.videoColumns(dual) == 1 && !ranking && preview.allowed(ctx))
    val c = LocalMetro.current
    val cards = list.cards
    val grid = list.grid
    val error = list.error

    suspend fun more() {
        if (list.loading) return
        list.loading = true
        try {
            val got = withContext(Dispatchers.IO) { load(list.page) }
            cards.addAll(got.filter { n -> cards.none { it.bvid == n.bvid } })
            list.page++
            list.error = null
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            list.error = e.message
        } finally {
            list.loading = false
        }
    }

    LaunchedEffect(list) { if (cards.isEmpty()) more() }
    if (!ranking) {
        val nearEnd by remember(list) { derivedStateOf { (grid.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0) >= cards.size - 4 } }
        LaunchedEffect(list) {
            snapshotFlow { nearEnd && cards.isNotEmpty() && !list.loading && list.error == null && list.page <= 10u }
                .collect { if (it) more() }
        }
    }

    LazyVerticalGrid(
        columns = GridCells.Fixed(if (ranking) layout.gridColumns(1) else layout.videoColumns(dual)),
        state = grid,
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 28.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(if (ranking) 4.dp else 22.dp),
        modifier = Modifier.fillMaxSize(),
    ) {
        error?.let { e -> item(span = { GridItemSpan(maxLineSpan) }) { Text("加载失败：$e", color = c.fg2, fontSize = 13.sp) } }
        if (ranking) {
            itemsIndexed(cards, key = { _, it -> it.bvid }) { i, card ->
                Row(
                    Modifier.fillMaxWidth().clickable { nav.push(Screen.Player(card.bvid, card.title)) }.padding(vertical = 10.dp),
                    horizontalArrangement = Arrangement.spacedBy(14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        "%02d".format(i + 1),
                        style = MetroType.meta.copy(fontSize = 28.sp),
                        color = if (i < 3) c.fg else c.fg2,
                        fontWeight = if (i < 3) androidx.compose.ui.text.font.FontWeight.SemiBold else androidx.compose.ui.text.font.FontWeight.Light,
                        modifier = Modifier.width(48.dp),
                    )
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text(card.title, fontSize = 15.sp, color = c.fg, maxLines = 2, overflow = TextOverflow.Ellipsis)
                        Row {
                            UpName(card.upName, card.upMid, MetroType.meta.fontSize, c.fg2)
                            Meta(" · ${formatCount(card.views)}")
                        }
                    }
                }
            }
        } else {
            items(cards, key = { it.bvid }) { card ->
                VideoCardView(
                    card,
                    big = layout.videoColumns(dual) == 1,
                    onOpen = { nav.push(Screen.Player(card.bvid, card.title)) },
                    preview = if (card.bvid == previewing) { { FeedPreview(videos, danmaku, settings, card, preview) } } else null,
                )
            }
        }
    }
}

@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
@Composable
fun VideoCardView(
    card: FfiVideoCard,
    big: Boolean,
    onOpen: () -> Unit,
    onMore: (() -> Unit)? = null,
    /** Drawn over the cover (大图预览). */
    preview: (@Composable () -> Unit)? = null,
) {
    val c = LocalMetro.current
    val addToList = LocalAddToPlaylist.current
    val flights = LocalCardFlights.current
    val coverAt = remember { arrayOf<androidx.compose.ui.geometry.Rect?>(null) }
    Column(Modifier.pressTilt(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        // Long-press: put it on the play list.
        Column(
            Modifier.combinedClickable(onLongClick = { addToList(card) }, onClick = {
                // 卡牌 / 书页 grow the page out of this cover.
                coverAt[0]?.let { flights.pending = CardOrigin(it, card.cover, card.bvid) }
                onOpen()
            }),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            // While a flight back lands here the cover stays empty, so it does not show twice.
            Box(
                Modifier.fillMaxWidth().aspectRatio(16f / 9f).flightSource { coverAt[0] = it }
                    .graphicsLayer { alpha = if (flights.landingOn(card.bvid)) 0f else 1f }
                    .background(c.bg2),
            ) {
                VideoCover(card.cover, card.portrait, Modifier.fillMaxSize())
                preview?.invoke()
                ProbeLine(card.bvid, Modifier.align(Alignment.BottomStart))
                if (card.reason.isNotEmpty()) Tag(card.reason, Modifier.align(Alignment.TopStart))
                Text(
                    formatDuration(card.durationS),
                    style = MetroType.meta,
                    color = Color.White,
                    modifier = Modifier.align(Alignment.BottomEnd).background(Color(0xBF000000)).padding(horizontal = 6.dp),
                )
            }
            Text(
                card.title,
                style = if (big) MetroType.body.copy(fontSize = 18.sp, lineHeight = 26.sp) else MetroType.body,
                color = c.fg,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Row(verticalAlignment = Alignment.Top) {
            Column(Modifier.weight(1f)) {
                UpName(card.upName, card.upMid, 12.sp, c.fg2)
                Meta("${formatCount(card.views)} · ${formatAge(card.pubdate)}")
            }
            if (onMore != null) {
                Box(Modifier.size(44.dp).clickable(onClick = onMore), contentAlignment = Alignment.Center) { GlyphIcon(Glyph.More, size = 18.dp) }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class)
@Composable
private fun WhySheet(home: HomeService, card: FfiVideoCard, onDismiss: () -> Unit, onDisliked: () -> Unit) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var breakdown by remember { mutableStateOf<FfiBreakdown?>(null) }
    LaunchedEffect(card.bvid) { breakdown = withContext(Dispatchers.IO) { runCatching { home.explain(card.bvid) }.getOrNull() } }

    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg2, shape = androidx.compose.ui.graphics.RectangleShape, dragHandle = null) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(verticalAlignment = Alignment.Bottom) {
                Text("为什么推荐", style = MetroType.section, color = c.fg, modifier = Modifier.weight(1f))
                Text(breakdown?.let { "%.2f".format(it.total) } ?: "", style = MetroType.meta.copy(fontSize = 28.sp), color = c.fg)
            }
            Text(card.title, color = c.fg2, fontSize = 14.sp, maxLines = 2)
            breakdown?.parts?.forEach { p ->
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text(p.factor, fontSize = 13.sp, color = c.fg, modifier = Modifier.width(72.dp))
                    Box(Modifier.weight(1f).height(6.dp).background(c.line)) {
                        val frac = (kotlin.math.abs(p.contribution) / 0.4f).coerceIn(0f, 1f)
                        Box(Modifier.fillMaxWidth(frac).height(6.dp).background(if (p.contribution >= 0) c.accent else c.fg2))
                    }
                    Meta("%+.2f".format(p.contribution), Modifier.width(52.dp), c.fg)
                }
            }
            Meta("得分 = Σ 权重 × 归一化因子 · 权重可在设置中调整")
            Text("不感兴趣", fontSize = 16.sp, color = c.fg, modifier = Modifier.padding(top = 6.dp))
            FlowRow(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                listOf(
                    "这个 UP 主" to FfiSignal.DislikeUp,
                    "这个分区" to FfiSignal.DislikeZone,
                    "这个标签" to FfiSignal.DislikeTag(""),
                    "这类标题" to FfiSignal.DislikeTitle,
                ).forEach { (label, signal) ->
                    TextAction(label, onClick = {
                        scope.launch {
                            runCatching { home.record(card.bvid, signal) }
                            onDisliked()
                        }
                    })
                }
            }
        }
    }
}

/**
 * The unfolded 分区 button: the zones as tiles, four to a row, the one showing in Cobalt; below,
 * the 影视专区, which open pages of their own.
 */
@Composable
private fun ZoneMenu(modifier: Modifier, zones: List<String>, pgcZones: List<String>, selected: Int, onZone: (Int) -> Unit, onPgc: (Int) -> Unit) {
    val c = LocalMetro.current
    Column(modifier.fillMaxWidth().background(c.bg2).padding(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 14.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Meta("分区")
        zones.indices.chunked(4).forEach { row ->
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                row.forEach { i -> ZoneTile(zones[i], on = i == selected, Modifier.weight(1f)) { onZone(i) } }
                repeat(4 - row.size) { Spacer(Modifier.weight(1f)) }
            }
        }
        Meta("影视专区 · 打开专区页", Modifier.padding(top = 6.dp))
        pgcZones.indices.chunked(4).forEach { row ->
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                row.forEach { i -> ZoneTile(pgcZones[i] + " →", on = true, Modifier.weight(1f)) { onPgc(i) } }
                repeat(4 - row.size) { Spacer(Modifier.weight(1f)) }
            }
        }
    }
}

@Composable
private fun ZoneTile(label: String, on: Boolean, modifier: Modifier, onClick: () -> Unit) {
    val c = LocalMetro.current
    Box(
        modifier.height(40.dp).then(if (on) Modifier.background(c.accent) else Modifier.border(1.dp, c.line)).clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) { Text(label, fontSize = 13.sp, color = if (on) Color.White else c.fg, maxLines = 1) }
}
