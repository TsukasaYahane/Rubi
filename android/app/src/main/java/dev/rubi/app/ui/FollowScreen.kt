package dev.rubi.app.ui

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ExperimentalMaterial3Api
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
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiDynamic
import dev.rubi.core.FfiDynamicBody
import dev.rubi.core.FfiDynamicKind
import dev.rubi.core.FfiFollowedUp
import dev.rubi.core.FfiPicture
import dev.rubi.core.FfiVideoCard
import dev.rubi.core.RubiException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch

/**
 * 关注 tab state, kept above the screen so switching tabs does not reload it. `selected` null
 * is 全部 (every followed UP's videos); otherwise one UP's posts of every kind.
 */
class FollowState {
    val cards = mutableStateListOf<FfiVideoCard>()
    val posts = mutableStateListOf<FfiDynamic>()
    val ups = mutableStateListOf<FfiFollowedUp>()
    val grid = LazyGridState()
    var selected by mutableStateOf<FfiFollowedUp?>(null)
    var next by mutableStateOf<String?>(null)
    var ended by mutableStateOf(false)
    var loading by mutableStateOf(false)
    var error by mutableStateOf<Throwable?>(null)
    var hidden by mutableStateOf(0u)
    var note by mutableStateOf("")
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FollowScreen(app: RubiApp, nav: Nav, state: FollowState, dual: Boolean) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val ctx = LocalContext.current

    suspend fun load(reset: Boolean) {
        if (state.loading) return
        state.loading = true
        val up = state.selected
        try {
            val offset = if (reset) null else state.next
            if (up == null) {
                val page = app.core.followedFeed(offset)
                if (state.selected != null) return
                if (reset) { state.cards.clear(); state.hidden = 0u }
                state.cards.addAll(page.cards.filter { n -> state.cards.none { it.bvid == n.bvid } })
                state.hidden += page.hidden
                state.next = page.next
            } else {
                val page = app.core.upDynamics(up.mid, offset)
                if (state.selected?.mid != up.mid) return
                if (reset) state.posts.clear()
                state.posts.addAll(page.items.filter { n -> state.posts.none { it.id == n.id } })
                state.next = page.next
            }
            state.ended = state.next == null
            state.error = null
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            state.error = e
        } finally {
            state.loading = false
        }
    }

    fun select(up: FfiFollowedUp?) {
        if (state.selected?.mid == up?.mid) {
            scope.launch { state.grid.animateScrollToItem(0) }
            return
        }
        state.selected = up
        state.next = null
        state.ended = false
        state.error = null
        state.loading = false
        scope.launch {
            state.grid.scrollToItem(0)
            load(reset = true)
        }
    }

    LaunchedEffect(Unit) {
        if (state.ups.isEmpty()) runCatching { app.core.followedUps() }.onSuccess { state.ups.addAll(it) }
        if (state.cards.isEmpty() && state.posts.isEmpty() && state.error == null) load(reset = true)
    }
    val shown = if (state.selected == null) state.cards.size else state.posts.size
    val nearEnd by remember { derivedStateOf { (state.grid.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0) >= shown - 3 } }
    LaunchedEffect(state) {
        snapshotFlow { nearEnd && shown > 0 && !state.loading && !state.ended && state.error == null }
            .collect { if (it) load(reset = false) }
    }
    LaunchedEffect(Unit) { nav.reselected.collect { if (it == Tab.Follow) state.grid.animateScrollToItem(0) } }

    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("关注", style = MetroType.pivot, color = c.fg, modifier = Modifier.weight(1f))
            // Copies the following list into the recommender's 关注 UP factor.
            TextAction("同步到推荐", onClick = {
                scope.launch {
                    state.note = runCatching { "已同步 ${app.core.syncFollowings()} 个关注到推荐" }.getOrElse { "同步失败：${it.message}" }
                }
            }, dim = true)
        }
        UpBar(state.ups, state.selected, onSelect = ::select)
        val status = listOfNotNull(
            state.note.takeIf { it.isNotEmpty() },
            "已过滤 ${state.hidden}".takeIf { state.hidden > 0u && state.selected == null },
        ).joinToString(" · ")
        if (status.isNotEmpty()) Meta(status, Modifier.padding(horizontal = 16.dp))
        val isEmpty = shown == 0
        PullToRefreshBox(isRefreshing = state.loading && !isEmpty, onRefresh = { scope.launch { load(reset = true) } }) {
            LazyVerticalGrid(
                // One UP's posts read top to bottom; 全部 follows the home layout.
                columns = GridCells.Fixed(if (dual && state.selected == null) 2 else 1),
                state = state.grid,
                contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 28.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalArrangement = Arrangement.spacedBy(if (dual && state.selected == null) 22.dp else 28.dp),
                modifier = Modifier.fillMaxSize(),
            ) {
                if (state.selected == null) {
                    items(state.cards, key = { it.bvid }) { card ->
                        VideoCardView(card, big = !dual, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) })
                    }
                } else {
                    items(state.posts, key = { it.id }) { post ->
                        DynamicCard(post, onOpenVideo = { nav.push(Screen.Player(it.bvid, it.title)) }, onOpenLink = { url ->
                            ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
                        })
                    }
                }
                item(span = { GridItemSpan(maxLineSpan) }) {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        val e = state.error
                        when {
                            e is RubiException.NotLoggedIn -> {
                                Meta("登录后可查看关注的 UP 的动态")
                                AccentButton("登录") { nav.push(Screen.Login) }
                            }
                            e != null -> {
                                Meta("加载失败：${e.message}")
                                TextAction("重试", onClick = { state.error = null; scope.launch { load(reset = isEmpty) } })
                            }
                            state.loading -> Meta("加载中…")
                            isEmpty -> Meta(if (state.selected == null) "关注的 UP 最近没有发布视频" else "这位 UP 最近没有动态")
                            state.ended -> Meta("没有更多了")
                        }
                    }
                }
            }
        }
    }
}

/** 全部 first, then followed UPs with recent posts; the selected one is underlined in accent. */
@Composable
private fun UpBar(ups: List<FfiFollowedUp>, selected: FfiFollowedUp?, onSelect: (FfiFollowedUp?) -> Unit) {
    val c = LocalMetro.current
    LazyRow(
        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        item(key = "all") {
            BarItem(selected == null, label = "全部", onClick = { onSelect(null) }) {
                Box(Modifier.size(48.dp).background(c.bg2), contentAlignment = Alignment.Center) {
                    Text("全", fontSize = 18.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
                }
            }
        }
        items(ups, key = { it.mid.toString() }) { up ->
            BarItem(selected?.mid == up.mid, label = up.name, onClick = { onSelect(up) }) {
                Box {
                    AsyncImage(model = up.face, contentDescription = up.name, contentScale = ContentScale.Crop, modifier = Modifier.size(48.dp).background(c.bg2))
                    if (up.hasUpdate) Box(Modifier.size(8.dp).background(c.accent).align(Alignment.TopEnd))
                }
            }
        }
    }
}

@Composable
private fun BarItem(on: Boolean, label: String, onClick: () -> Unit, picture: @Composable () -> Unit) {
    val c = LocalMetro.current
    Column(
        Modifier.width(56.dp).clickable(onClick = onClick).drawBehind {
            if (on) drawLine(c.accent, Offset(0f, size.height), Offset(size.width, size.height), 3.dp.toPx())
        }.padding(bottom = 6.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        picture()
        Text(
            label,
            fontSize = 11.sp,
            color = if (on) c.fg else c.fg2,
            fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

/** One post: author and time, then what it holds; a repost shows its original indented. */
@Composable
private fun DynamicCard(post: FfiDynamic, onOpenVideo: (FfiVideoCard) -> Unit, onOpenLink: (String) -> Unit) {
    val c = LocalMetro.current
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(post.body.authorName, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
            Meta(formatAge(post.publishedAt))
            kindLabel(post.body.kind)?.let { Tag(it) }
        }
        DynamicBody(post.body, onOpenVideo, onOpenLink)
        post.orig?.let { orig ->
            Column(
                Modifier.fillMaxWidth().drawBehind {
                    drawLine(c.spine, Offset(0f, 0f), Offset(0f, size.height), 2.dp.toPx())
                }.padding(start = 12.dp),
                verticalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                Meta("@${orig.authorName}")
                DynamicBody(orig, onOpenVideo, onOpenLink)
            }
        }
    }
}

@Composable
private fun DynamicBody(body: FfiDynamicBody, onOpenVideo: (FfiVideoCard) -> Unit, onOpenLink: (String) -> Unit) {
    val c = LocalMetro.current
    if (body.title.isNotBlank()) Text(body.title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
    if (body.text.isNotBlank()) {
        var expanded by remember(body) { mutableStateOf(false) }
        var overflows by remember(body) { mutableStateOf(false) }
        EmoteText(
            body.text,
            color = c.fg,
            fontSize = 14.sp,
            lineHeight = 21.sp,
            maxLines = if (expanded) Int.MAX_VALUE else 6,
            onTextLayout = { if (!expanded) overflows = it.hasVisualOverflow },
        )
        if (overflows || expanded) {
            Text(if (expanded) "收起" else "展开全文", fontSize = 13.sp, color = c.accent, modifier = Modifier.clickable { expanded = !expanded })
        }
    }
    body.video?.let { v -> VideoCardView(v, big = true, onOpen = { onOpenVideo(v) }) }
    if (body.pictures.isNotEmpty() && body.video == null) {
        CommentPictures(body.pictures.map { FfiPicture(it, 0u, 0u) })
    }
    // Articles and posts open in the browser until Rubi reads them itself (文字专栏, phase 2).
    if (body.link.isNotBlank() && body.kind in setOf(FfiDynamicKind.ARTICLE, FfiDynamicKind.OPUS, FfiDynamicKind.OTHER)) {
        TextAction("查看全文", onClick = { onOpenLink(body.link) })
    }
}

private fun kindLabel(kind: FfiDynamicKind): String? = when (kind) {
    FfiDynamicKind.VIDEO -> "视频"
    FfiDynamicKind.ARTICLE -> "专栏"
    FfiDynamicKind.OPUS -> "图文"
    FfiDynamicKind.FORWARD -> "转发"
    else -> null
}
