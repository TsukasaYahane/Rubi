package dev.rubi.app.ui

import android.util.Log
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import dev.rubi.app.RubiApp
import dev.rubi.app.data.PreviewPrefs
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
class HomeState {
    val recommend = FeedList()
    val hot = FeedList()
    val rank = FeedList()
    var status by mutableStateOf("")
    /** Why sources failed on the last refresh; shown when the status line is tapped. */
    var sourceErrors by mutableStateOf(emptyList<String>())
    var showErrors by mutableStateOf(false)
    var dual by mutableStateOf(true)
    var pivot by mutableStateOf(0)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(app: RubiApp, nav: Nav, home: HomeState) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val feed = home.recommend
    val cards = feed.cards
    val grid = feed.grid
    var why by remember { mutableStateOf<FfiVideoCard?>(null) }

    /** `reset` = user pulled to refresh; otherwise append the next page from the pool. */
    suspend fun loadPage(reset: Boolean) {
        // A rebuilt list may show earlier cards again; an appended page must not repeat them.
        if (reset || cards.isEmpty()) app.core.newFeedSession()
        var report = if (reset || cards.isEmpty()) runCatching { app.core.refreshFeed(20u) } else null
        suspend fun next() = withContext(Dispatchers.IO) { runCatching { app.core.nextPage(PAGE) }.getOrDefault(emptyList()) }
        var page = next()
        if (page.isEmpty() && report == null) {
            // Pool used up while scrolling: fetch more candidates and append, never replace.
            report = runCatching { app.core.refreshFeed(20u) }
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

    suspend fun load(reset: Boolean) {
        if (feed.loading) return
        feed.loading = true
        try { loadPage(reset) } finally { feed.loading = false }
    }

    // First visit only: an existing list is kept as is when returning to this screen.
    LaunchedEffect(Unit) { if (cards.isEmpty()) load(reset = false) }
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
    val ctx = LocalContext.current
    // Re-read when coming back from 设置.
    val preview = remember(nav.stack.size) { PreviewPrefs.load(app.core) }
    val previewing = rememberPreviewTarget(grid, enabled = !dual && home.pivot == 0 && preview.allowed(ctx))
    val pivot = home.pivot

    Column(Modifier.fillMaxSize()) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.weight(1f)) { Pivot(listOf("推荐", "热门", "排行"), pivot, { home.pivot = it }) }
            Text(
                if (dual) "▦ 双列" else "▭ 单列",
                style = MetroType.meta.copy(fontSize = 12.sp),
                color = c.fg,
                modifier = Modifier.padding(end = 16.dp).clickable { home.dual = !home.dual },
            )
        }
        if (pivot != 0) {
            HotRankList(app, nav, if (pivot == 2) home.rank else home.hot, ranking = pivot == 2, dual = dual)
            return@Column
        }
        Row(Modifier.fillMaxWidth().height(44.dp).padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Meta(home.status, Modifier.weight(1f).clickable(enabled = home.sourceErrors.isNotEmpty()) { home.showErrors = !home.showErrors })
            Text("被过滤内容 →", style = MetroType.meta, color = c.fg, modifier = Modifier.clickable { nav.push(Screen.Filtered) })
        }
        if (home.showErrors) {
            Column(Modifier.padding(horizontal = 16.dp)) { home.sourceErrors.forEach { Meta(it) } }
        }
        PullToRefreshBox(isRefreshing = refreshing, onRefresh = { scope.launch { load(reset = true) } }) {
            LazyVerticalGrid(
                columns = GridCells.Fixed(if (dual) 2 else 1),
                state = grid,
                contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 28.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalArrangement = Arrangement.spacedBy(if (dual) 22.dp else 30.dp),
                modifier = Modifier.fillMaxSize(),
            ) {
                items(cards, key = { it.bvid }) { card ->
                    VideoCardView(
                        card,
                        big = !dual,
                        onOpen = { nav.push(Screen.Player(card.bvid, card.title)) },
                        onMore = { why = card },
                        preview = if (card.bvid == previewing) { { FeedPreview(app, card, preview) } } else null,
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

    why?.let { card ->
        WhySheet(app, card, onDismiss = { why = null }, onDisliked = {
            cards.removeAll { it.bvid == card.bvid }
            why = null
        })
    }
}

/** 热门 (paged grid) and 排行 (numbered list); both already filtered by the rule table. */
@Composable
private fun HotRankList(app: RubiApp, nav: Nav, list: FeedList, ranking: Boolean, dual: Boolean) {
    val ctx = LocalContext.current
    val preview = remember(nav.stack.size) { PreviewPrefs.load(app.core) }
    val previewing = rememberPreviewTarget(list.grid, enabled = !dual && !ranking && preview.allowed(ctx))
    val c = LocalMetro.current
    val cards = list.cards
    val grid = list.grid
    val error = list.error

    suspend fun more() {
        if (list.loading) return
        list.loading = true
        try {
            val got = if (ranking) app.core.ranking() else app.core.popular(list.page)
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
        columns = GridCells.Fixed(if (dual && !ranking) 2 else 1),
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
                        Meta("${card.upName} · ${formatCount(card.views)}")
                    }
                }
            }
        } else {
            items(cards, key = { it.bvid }) { card ->
                VideoCardView(
                    card,
                    big = !dual,
                    onOpen = { nav.push(Screen.Player(card.bvid, card.title)) },
                    preview = if (card.bvid == previewing) { { FeedPreview(app, card, preview) } } else null,
                )
            }
        }
    }
}

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
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Column(Modifier.clickable(onClick = onOpen), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f).background(c.bg2)) {
                AsyncImage(model = card.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
                preview?.invoke()
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
                Text(card.upName, fontSize = 12.sp, color = c.fg2, maxLines = 1)
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
private fun WhySheet(app: RubiApp, card: FfiVideoCard, onDismiss: () -> Unit, onDisliked: () -> Unit) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var breakdown by remember { mutableStateOf<FfiBreakdown?>(null) }
    LaunchedEffect(card.bvid) { breakdown = withContext(Dispatchers.IO) { runCatching { app.core.explain(card.bvid) }.getOrNull() } }

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
                            runCatching { app.core.record(card.bvid, signal) }
                            onDisliked()
                        }
                    })
                }
            }
        }
    }
}
