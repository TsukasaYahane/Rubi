package dev.rubi.app.ui

import androidx.compose.foundation.border
import androidx.compose.foundation.lazy.LazyListState
import kotlinx.coroutines.CoroutineScope
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
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FollowingService
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
class FollowState(private val following: FollowingService, private val scope: CoroutineScope) {
    val ups = mutableStateListOf<FfiFollowedUp>()
    val grid = LazyGridState()
    val bar = LazyListState()
    var note by mutableStateOf("")
    var selected by mutableStateOf<FfiFollowedUp?>(null)
        private set

    val videos = Paged<FfiVideoCard, String>(scope, key = { it.bvid }) { at ->
        following.followedFeed(at).let { Page(it.cards, it.next, it.hidden) }
    }
    var posts by mutableStateOf(postsOf(0uL))
        private set

    /** Shows `up`'s posts, or every UP's videos for null. */
    fun select(up: FfiFollowedUp?) {
        selected = up
        if (up != null) posts = postsOf(up.mid).also { it.more() }
    }

    /** Shows `up`'s posts from outside the bar (全部关注): it joins the bar's front if missing. */
    fun show(up: FfiFollowedUp) {
        if (ups.none { it.mid == up.mid }) ups.add(0, up)
        select(up)
        scope.launch {
            bar.scrollToItem(0)
            grid.scrollToItem(0)
        }
    }

    /** Starts over, after someone signs in or out. */
    fun reload() {
        ups.clear()
        videos.reset()
        loadUps()
    }

    fun loadUps() {
        if (ups.isNotEmpty()) return
        scope.launch { runCatching { following.followedUps() }.onSuccess { ups.addAll(it) } }
    }

    private fun postsOf(mid: ULong) = Paged<FfiDynamic, String>(scope, key = { it.id }) { at ->
        following.upDynamics(mid, at).let { Page(it.items, it.next) }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FollowScreen(followingService: FollowingService, nav: Nav, state: FollowState, dual: Boolean) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val ctx = LocalContext.current
    val all = state.selected == null
    val shown: Paged<*, String> = if (all) state.videos else state.posts

    fun select(up: FfiFollowedUp?) {
        if (state.selected?.mid == up?.mid) {
            scope.launch { state.grid.animateScrollToItem(0) }
            return
        }
        state.select(up)
        scope.launch { state.grid.scrollToItem(0) }
    }

    LaunchedEffect(Unit) {
        state.loadUps()
        if (state.videos.items.isEmpty()) state.videos.more()
    }
    if (all) LoadMoreNearEnd(state.grid, state.videos) else LoadMoreNearEnd(state.grid, state.posts)
    LaunchedEffect(Unit) { nav.reselected.collect { if (it == Tab.Follow) state.grid.animateScrollToItem(0) } }

    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("关注", style = MetroType.pivot, color = c.fg, modifier = Modifier.weight(1f))
            // Copies the following list into the recommender's 关注 UP factor.
            TextAction("同步到推荐", onClick = {
                scope.launch {
                    state.note = runCatching { "已同步 ${followingService.syncFollowings()} 个关注到推荐" }.getOrElse { "同步失败：${it.message}" }
                }
            }, dim = true)
        }
        UpBar(state.bar, state.ups, state.selected, onSelect = ::select, onMore = { nav.push(Screen.Followings) })
        // Swiped to the end of the bar without finding them: the full list is a tap away.
        val atEnd by remember { derivedStateOf { state.ups.isNotEmpty() && state.bar.canScrollBackward && !state.bar.canScrollForward } }
        if (atEnd) {
            Row(
                Modifier.fillMaxWidth().background(c.accent.copy(alpha = 0.15f)).clickable { nav.push(Screen.Followings) }.padding(horizontal = 16.dp, vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("没找到想看的 UP？", fontSize = 13.sp, color = c.fg, modifier = Modifier.weight(1f))
                Text("查看全部关注 ›", fontSize = 13.sp, color = c.accent)
            }
        }
        val status = listOfNotNull(
            state.note.takeIf { it.isNotEmpty() },
            "已过滤 ${state.videos.hidden}".takeIf { state.videos.hidden > 0u && all },
        ).joinToString(" · ")
        if (status.isNotEmpty()) Meta(status, Modifier.padding(horizontal = 16.dp))
        val isEmpty = shown.items.isEmpty()
        PullToRefreshBox(isRefreshing = shown.loading && !isEmpty, onRefresh = { shown.reset() }) {
            LazyVerticalGrid(
                // One UP's posts read top to bottom; 全部 follows the home layout.
                columns = GridCells.Fixed(if (all) LocalScreenLayout.current.videoColumns(dual) else LocalScreenLayout.current.gridColumns(1)),
                state = state.grid,
                contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 28.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalArrangement = Arrangement.spacedBy(if (dual && all) 22.dp else 28.dp),
                modifier = Modifier.fillMaxSize(),
            ) {
                if (all) {
                    items(state.videos.items, key = { it.bvid }) { card ->
                        VideoCardView(card, big = LocalScreenLayout.current.videoColumns(dual) == 1, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) })
                    }
                } else {
                    items(state.posts.items, key = { it.id }) { post ->
                        DynamicCard(post, onOpenVideo = { nav.push(Screen.Player(it.bvid, it.title)) }, onOpenArticle = { nav.push(Screen.Article(it)) }, onComments = { nav.push(Screen.DynamicComments(post)) }, onOpenLink = { url ->
                            ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
                        })
                    }
                }
                item(span = { GridItemSpan(maxLineSpan) }) {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        val e = shown.error
                        when {
                            e is RubiException.NotLoggedIn -> {
                                Meta("登录后可查看关注的 UP 的动态")
                                AccentButton("登录") { nav.push(Screen.Login) }
                            }
                            e != null -> {
                                Meta(errorText(e))
                                TextAction("重试", onClick = shown::retry)
                            }
                            shown.loading -> Meta("加载中…")
                            isEmpty -> Meta(if (all) "关注的 UP 最近没有发布视频" else "这位 UP 最近没有动态")
                            shown.ended -> Meta("没有更多了")
                        }
                    }
                }
            }
        }
    }
}

/** 全部 first, then followed UPs with recent posts; the selected one is underlined in accent. */
@Composable
private fun UpBar(state: LazyListState, ups: List<FfiFollowedUp>, selected: FfiFollowedUp?, onSelect: (FfiFollowedUp?) -> Unit, onMore: () -> Unit) {
    val c = LocalMetro.current
    LazyRow(
        state = state,
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
        // The bar lists only UPs who posted lately; everyone else is in 全部关注.
        item(key = "more") {
            BarItem(false, label = "全部关注", onClick = onMore) {
                Box(Modifier.size(48.dp).border(2.dp, c.fg), contentAlignment = Alignment.Center) {
                    GlyphIcon(Glyph.More)
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
internal fun DynamicCard(
    post: FfiDynamic,
    onOpenVideo: (FfiVideoCard) -> Unit,
    onOpenArticle: (String) -> Unit,
    onOpenLink: (String) -> Unit,
    onComments: () -> Unit,
) {
    val c = LocalMetro.current
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(post.body.authorName, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
            Meta(formatAge(post.publishedAt))
            kindLabel(post.body.kind)?.let { Tag(it) }
        }
        DynamicBody(post.body, onOpenVideo, onOpenArticle, onOpenLink)
        post.orig?.let { orig ->
            Column(
                Modifier.fillMaxWidth().drawBehind {
                    drawLine(c.spine, Offset(0f, 0f), Offset(0f, size.height), 2.dp.toPx())
                }.padding(start = 12.dp),
                verticalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                Meta("@${orig.authorName}")
                DynamicBody(orig, onOpenVideo, onOpenArticle, onOpenLink)
            }
        }
        // The post's own comments, on a page of their own.
        if (post.comments != null) {
            TextAction(if (post.commentCount > 0u) "评论 ${formatCount(post.commentCount)}" else "评论", onClick = onComments, dim = true)
        }
    }
}

@Composable
private fun DynamicBody(body: FfiDynamicBody, onOpenVideo: (FfiVideoCard) -> Unit, onOpenArticle: (String) -> Unit, onOpenLink: (String) -> Unit) {
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
    // Articles and 图文 open in the reader; other kinds of post only on the web.
    if (body.link.isNotBlank() && body.kind in setOf(FfiDynamicKind.ARTICLE, FfiDynamicKind.OPUS)) {
        TextAction("查看全文", onClick = { onOpenArticle(body.link) })
    } else if (body.link.isNotBlank() && body.kind == FfiDynamicKind.OTHER) {
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
