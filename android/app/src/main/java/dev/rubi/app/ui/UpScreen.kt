package dev.rubi.app.ui

import dev.rubi.core.FfiDynamicKind
import androidx.compose.ui.graphics.Color
import androidx.compose.foundation.border
import dev.rubi.core.FfiUpArticle
import dev.rubi.core.FfiDynamic
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.foundation.layout.height
import androidx.compose.ui.platform.LocalContext
import android.net.Uri
import android.content.Intent
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiUpOrder
import dev.rubi.core.FfiUpProfile
import dev.rubi.core.FfiVideoCard
import dev.rubi.core.RubiException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

private val ORDERS = listOf("最新" to FfiUpOrder.NEWEST, "最多播放" to FfiUpOrder.MOST_VIEWED)

private val TABS = listOf("投稿", "动态", "专栏")

/** What a search inside the space shows. */
private val FOUND = listOf("全部", "视频", "专栏", "动态")

/** Least time between two upload pages. */
private const val UP_PAGE_GAP_MS = 1_000L

/**
 * An UP's page: profile, follow, then 投稿 (two columns, 充电专属 videos left out), 动态 and 专栏.
 * Each tab loads when first opened.
 */
@Composable
fun UpScreen(app: RubiApp, nav: Nav, mid: ULong, name: String) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var profile by remember { mutableStateOf<FfiUpProfile?>(null) }
    var order by remember { mutableStateOf(FfiUpOrder.NEWEST) }
    var tab by remember { mutableStateOf(0) }
    // 空间内搜索: the box, the term searched, and which kind of result shows.
    var query by remember { mutableStateOf("") }
    var searched by remember { mutableStateOf("") }
    var found by remember { mutableStateOf(0) }
    val ctx = LocalContext.current
    var note by remember { mutableStateOf("") }
    var confirmUnfollow by remember { mutableStateOf(false) }
    val grid = rememberLazyGridState()

    // Uploads by page number; the space API is touchy about bursts, so pages are spaced.
    val uploads = remember(mid) {
        Paged<FfiVideoCard, UInt>(scope, key = { it.bvid }, minGapMs = UP_PAGE_GAP_MS) { pn ->
            val at = pn ?: 1u
            val got = app.core.upVideos(mid, at, order)
            Page(got.items, if (got.hasMore) at + 1u else null, got.hiddenCharging)
        }
    }

    val posts = remember(mid) {
        Paged<FfiDynamic, String>(scope, key = { it.id }, minGapMs = UP_PAGE_GAP_MS) { at ->
            app.core.upDynamics(mid, at).let { Page(it.items, it.next) }
        }
    }
    val articles = remember(mid) {
        Paged<FfiUpArticle, UInt>(scope, key = { it.cvid }, minGapMs = UP_PAGE_GAP_MS) { pn ->
            val at = pn ?: 1u
            val got = app.core.upArticles(mid, at)
            Page(got.items, if (got.hasMore) at + 1u else null)
        }
    }
    val foundPosts = remember(mid) {
        Paged<FfiDynamic, String>(scope, key = { it.id }, minGapMs = UP_PAGE_GAP_MS) { at ->
            val page = at?.toUIntOrNull() ?: 1u
            app.core.upSearchPosts(mid, searched, page).let { Page(it.items, it.next) }
        }
    }
    val foundVideos = remember(mid) {
        Paged<FfiVideoCard, UInt>(scope, key = { it.bvid }, minGapMs = UP_PAGE_GAP_MS) { pn ->
            val at = pn ?: 1u
            val got = app.core.upSearchVideos(mid, searched, at)
            Page(got.items, if (got.hasMore) at + 1u else null, got.hiddenCharging)
        }
    }
    fun search(term: String) {
        searched = term.trim()
        found = 0
        if (searched.isEmpty()) return
        foundPosts.reset()
        foundVideos.reset()
    }

    LaunchedEffect(tab) {
        when (tab) {
            1 -> if (posts.items.isEmpty() && !posts.loading) posts.more()
            2 -> if (articles.items.isEmpty() && !articles.loading) articles.more()
        }
    }

    LaunchedEffect(mid, nav.stack.size) { profile = runCatching { app.core.upProfile(mid) }.getOrNull() ?: profile }
    LaunchedEffect(uploads, order) { uploads.reset() }
    when {
        searched.isNotEmpty() -> if (found == 1) LoadMoreNearEnd(grid, foundVideos) else LoadMoreNearEnd(grid, foundPosts)
        tab == 0 -> LoadMoreNearEnd(grid, uploads)
        tab == 1 -> LoadMoreNearEnd(grid, posts)
        else -> LoadMoreNearEnd(grid, articles)
    }

    fun toggleFollow(p: FfiUpProfile) {
        if (p.followed && !confirmUnfollow) {
            confirmUnfollow = true
            note = "再点一次取消关注"
            return
        }
        confirmUnfollow = false
        val on = !p.followed
        profile = p.copy(followed = on, fans = if (on) p.fans + 1u else p.fans - minOf(p.fans, 1uL))
        scope.launch {
            runCatching { app.core.followMid(mid, p.name, on) }
                .onSuccess { note = if (on) "已关注" else "已取消关注" }
                .onFailure {
                    profile = p
                    note = if (it is RubiException.NotLoggedIn) "登录后才能关注" else "操作失败：${it.message}"
                }
        }
    }

    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().padding(start = 4.dp, end = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
            MetroField(query, { query = it }, "搜索该 UP 的投稿、专栏、动态", Modifier.weight(1f)) { search(query) }
            if (searched.isNotEmpty()) {
                Text("清除", fontSize = 13.sp, color = c.fg2, modifier = Modifier.clickable { query = ""; search("") }.padding(8.dp))
            }
        }
        LazyVerticalGrid(
            columns = GridCells.Fixed(2),
            state = grid,
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = 28.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalArrangement = Arrangement.spacedBy(22.dp),
            modifier = Modifier.fillMaxSize(),
        ) {
            item(span = { GridItemSpan(maxLineSpan) }) {
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    val p = profile
                    Row(horizontalArrangement = Arrangement.spacedBy(14.dp), verticalAlignment = Alignment.CenterVertically) {
                        AsyncImage(
                            model = p?.face,
                            contentDescription = null,
                            contentScale = ContentScale.Crop,
                            modifier = Modifier.size(64.dp).background(c.bg2),
                        )
                        Column(Modifier.weight(1f)) {
                            Text(p?.name ?: name, style = MetroType.pivot.copy(fontSize = 28.sp, lineHeight = 36.sp), color = c.fg, maxLines = 1)
                            if (p != null) Meta(listOf("Lv${p.level}", p.official).filter { it.isNotBlank() }.joinToString(" · "))
                        }
                    }
                    if (p != null) {
                        if (p.sign.isNotBlank()) Text(p.sign, fontSize = 14.sp, lineHeight = 21.sp, color = c.fg2)
                        Row(horizontalArrangement = Arrangement.spacedBy(28.dp)) {
                            Stat("粉丝", formatCount(p.fans))
                            Stat("关注", formatCount(p.followings))
                            Stat("投稿", formatCount(p.archives))
                        }
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                            if (p.followed) TextAction(if (confirmUnfollow) "确认取消关注" else "已关注", onClick = { toggleFollow(p) }, dim = !confirmUnfollow)
                            else AccentButton("关注") { toggleFollow(p) }
                            if (note.isNotEmpty()) Meta(note)
                        }
                    }
                    if (searched.isNotEmpty()) {
                        Meta("在 ${p?.name ?: name} 的空间里搜索「$searched」", Modifier.padding(top = 10.dp))
                        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                            FOUND.forEachIndexed { i, label ->
                                val on = i == found
                                Text(
                                    label,
                                    fontSize = 13.sp,
                                    color = if (on) Color.White else c.fg,
                                    modifier = Modifier.then(if (on) Modifier.background(c.accent) else Modifier.border(1.dp, c.line))
                                        .clickable { found = i }
                                        .padding(horizontal = 10.dp, vertical = 5.dp),
                                )
                            }
                        }
                    } else Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(22.dp)) {
                        TABS.forEachIndexed { i, label ->
                            Text(
                                label,
                                style = MetroType.pivot.copy(fontSize = 26.sp, lineHeight = 34.sp),
                                fontWeight = if (i == tab) FontWeight.Normal else FontWeight.Light,
                                color = if (i == tab) c.fg else c.fg2,
                                modifier = Modifier.clickable { tab = i },
                            )
                        }
                    }
                    if (tab == 0 && searched.isEmpty()) Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                        ORDERS.forEach { (label, o) ->
                            val on = o == order
                            Text(
                                label,
                                fontSize = 16.sp,
                                fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
                                color = if (on) c.fg else c.fg2,
                                modifier = Modifier.heightIn(min = 44.dp).clickable { order = o }.padding(top = 10.dp),
                            )
                        }
                    }
                    if (tab == 0 && uploads.hidden > 0u) Meta("已隐藏充电专属视频 ${uploads.hidden} 个")
                    val failed = when (tab) {
                        0 -> uploads.error
                        1 -> posts.error
                        else -> articles.error
                    }
                    failed?.let { e ->
                        Text(errorText(e), color = c.fg2, fontSize = 13.sp)
                        TextAction("重试", onClick = { if (tab == 0) uploads.retry() else if (tab == 1) posts.retry() else articles.retry() })
                    }
                }
            }
            if (searched.isNotEmpty()) {
                if (found == 1) {
                    items(foundVideos.items, key = { "s" + it.bvid }) { card ->
                        VideoCardView(card, big = false, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) })
                    }
                } else {
                    // 专栏 are article posts; 动态 the rest that are neither videos nor articles.
                    val shown = foundPosts.items.filter {
                        when (found) {
                            2 -> it.body.kind == FfiDynamicKind.ARTICLE
                            3 -> it.body.kind != FfiDynamicKind.VIDEO && it.body.kind != FfiDynamicKind.ARTICLE
                            else -> true
                        }
                    }
                    items(shown, key = { "f" + it.id }, span = { GridItemSpan(maxLineSpan) }) { post ->
                        Column {
                            DynamicCard(
                                post,
                                onOpenVideo = { nav.push(Screen.Player(it.bvid, it.title)) },
                                onOpenArticle = { nav.push(Screen.Article(it)) },
                                onComments = { nav.push(Screen.DynamicComments(post)) },
                                onOpenLink = { url -> runCatching { ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url))) } },
                            )
                        }
                    }
                }
                val list = if (found == 1) foundVideos.ended to foundVideos.items.isEmpty() else foundPosts.ended to foundPosts.items.isEmpty()
                item(span = { GridItemSpan(maxLineSpan) }) {
                    Meta(
                        when {
                            (if (found == 1) foundVideos.loading else foundPosts.loading) -> "搜索中…"
                            list.first && list.second -> "这个 UP 没有相关内容"
                            list.first -> "没有更多了"
                            else -> ""
                        },
                    )
                }
            } else when (tab) {
                0 -> items(uploads.items, key = { it.bvid }) { card ->
                    VideoCardView(card, big = false, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) })
                }
                1 -> items(posts.items, key = { "d" + it.id }, span = { GridItemSpan(maxLineSpan) }) { post ->
                    Column {
                        DynamicCard(
                            post,
                            onOpenVideo = { nav.push(Screen.Player(it.bvid, it.title)) },
                            onOpenArticle = { nav.push(Screen.Article(it)) },
                            onComments = { nav.push(Screen.DynamicComments(post)) },
                            onOpenLink = { url -> runCatching { ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url))) } },
                        )
                    }
                }
                else -> items(articles.items, key = { "a" + it.cvid }, span = { GridItemSpan(maxLineSpan) }) { a ->
                    ArticleRow(a) { nav.push(Screen.Article("cv${a.cvid}")) }
                }
            }
            val shown = when (tab) {
                0 -> uploads.ended to uploads.items.isEmpty()
                1 -> posts.ended to posts.items.isEmpty()
                else -> articles.ended to articles.items.isEmpty()
            }
            if (searched.isEmpty() && shown.first) {
                item(span = { GridItemSpan(maxLineSpan) }) {
                    Meta(if (shown.second) listOf("还没有投稿", "还没有动态", "还没有专栏")[tab] else "没有更多了")
                }
            }
        }
    }
}

@Composable
private fun Stat(label: String, value: String) {
    val c = LocalMetro.current
    Column {
        Text(value, fontSize = 20.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
        Meta(label)
    }
}

/** A 专栏 in the UP's list: title, summary, when and how many read it, cover on the right. */
@Composable
private fun ArticleRow(a: FfiUpArticle, onClick: () -> Unit) {
    val c = LocalMetro.current
    Column(Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
        Row(Modifier.padding(vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(a.title, fontSize = 15.sp, color = c.fg, maxLines = 2, overflow = TextOverflow.Ellipsis)
                if (a.summary.isNotBlank()) Text(a.summary, fontSize = 13.sp, color = c.fg2, maxLines = 2, overflow = TextOverflow.Ellipsis)
                Meta("${formatAge(a.publishedAt)} · ${formatCount(a.views)} 阅读")
            }
            if (a.cover.isNotEmpty()) {
                AsyncImage(model = a.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.size(width = 96.dp, height = 64.dp).background(c.bg2))
            }
        }
    }
}
