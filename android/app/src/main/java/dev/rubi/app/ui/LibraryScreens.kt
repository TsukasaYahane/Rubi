package dev.rubi.app.ui

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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiFavFolder
import dev.rubi.core.FfiHistoryItem
import dev.rubi.core.FfiVideoCard
import dev.rubi.core.RubiException
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/**
 * A paged list: `load(page)` returns the items and whether more follow. Shared by history and
 * favourite folders; the next page loads near the end and stops after an error until 重试.
 *
 * Loads run in the screen's scope, not in the effect that asked for them: that effect restarts
 * whenever the list grows, and restarting must not cancel a page half way.
 */
private class Pager<T>(private val scope: CoroutineScope, private val load: suspend (Int) -> Pair<List<T>, Boolean>) {
    val items = mutableStateListOf<T>()
    var page by mutableStateOf(1)
    var hasMore by mutableStateOf(true)
    var loading by mutableStateOf(false)
    var error by mutableStateOf<Throwable?>(null)

    fun more() {
        if (loading || !hasMore) return
        loading = true
        scope.launch {
            try {
                val (got, more) = load(page)
                items.addAll(got)
                hasMore = more && got.isNotEmpty()
                page++
                error = null
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                error = e
            } finally {
                loading = false
            }
        }
    }
}

private fun describe(e: Throwable): String = when (e) {
    is RubiException.NotLoggedIn -> "登录后可查看"
    is RubiException.RiskControl -> "被 B 站风控拦截，稍等片刻再重试"
    else -> "加载失败：${e.message}"
}

/** Status rows at the end of a list: loading, error with 重试, empty or 没有更多了. */
private fun LazyListScope.listFooter(nav: Nav, empty: Boolean, loading: Boolean, hasMore: Boolean, error: Throwable?, retry: () -> Unit) {
    item(key = "footer") {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            when {
                error != null -> {
                    Meta(describe(error))
                    if (error is RubiException.NotLoggedIn) AccentButton("登录") { nav.push(Screen.Login) }
                    else TextAction("重试", onClick = retry)
                }
                loading -> Meta("加载中…")
                empty -> Meta("这里还是空的")
                !hasMore -> Meta("没有更多了")
            }
        }
    }
}

@Composable
fun HistoryScreen(app: RubiApp, nav: Nav) {
    val scope = rememberCoroutineScope()
    var cursor by remember { mutableStateOf<String?>(null) }
    val pager = remember {
        Pager<FfiHistoryItem>(scope) { _ ->
            val page = app.core.history(cursor)
            cursor = page.cursor
            page.items to (page.cursor != null)
        }
    }
    val list = rememberLazyListState()
    val nearEnd by remember { derivedStateOf { (list.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0) >= pager.items.size - 4 } }
    LaunchedEffect(nearEnd, pager.items.size) { if (nearEnd && pager.error == null) pager.more() }

    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "历史记录")
        LazyColumn(Modifier.weight(1f), state = list, contentPadding = PaddingValues(bottom = 24.dp)) {
            items(pager.items, key = { it.card.bvid + it.viewedAt }) { h ->
                val progress = when {
                    h.progressS < 0 -> "已看完"
                    h.progressS == 0L -> "刚开始"
                    else -> "看到 ${formatDuration(h.progressS.toUInt())}"
                }
                VideoRow(h.card, "$progress · ${formatAge(h.viewedAt)}") { nav.push(Screen.Player(h.card.bvid, h.card.title)) }
            }
            listFooter(nav, pager.items.isEmpty(), pager.loading, pager.hasMore, pager.error) { pager.more() }
        }
    }
}

@Composable
fun WatchLaterScreen(app: RubiApp, nav: Nav) {
    val scope = rememberCoroutineScope()
    val pager = remember { Pager<FfiVideoCard>(scope) { _ -> app.core.watchLaterList() to false } }
    LaunchedEffect(Unit) { pager.more() }
    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "稍后再看")
        LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 24.dp)) {
            items(pager.items, key = { it.bvid }) { card ->
                VideoRow(card, "${formatCount(card.views)} 播放 · ${formatAge(card.pubdate)}") { nav.push(Screen.Player(card.bvid, card.title)) }
            }
            listFooter(nav, pager.items.isEmpty(), pager.loading, pager.hasMore, pager.error) { pager.more() }
        }
    }
}

@Composable
fun FavFoldersScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val pager = remember { Pager<FfiFavFolder>(scope) { _ -> app.core.myFavFolders() to false } }
    LaunchedEffect(Unit) { pager.more() }
    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "收藏夹")
        LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 24.dp)) {
            items(pager.items, key = { it.id.toString() }) { f ->
                Column(
                    Modifier.fillMaxWidth().heightIn(min = 64.dp).clickable { nav.push(Screen.FavFolder(f.id, f.title)) }
                        .padding(horizontal = 16.dp, vertical = 10.dp),
                ) {
                    Text(f.title, style = MetroType.section, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Meta("${f.count} 个视频")
                }
            }
            listFooter(nav, pager.items.isEmpty(), pager.loading, pager.hasMore, pager.error) { pager.more() }
        }
    }
}

@Composable
fun FavFolderScreen(app: RubiApp, nav: Nav, id: ULong, title: String) {
    val scope = rememberCoroutineScope()
    val pager = remember {
        Pager<FfiVideoCard>(scope) { pn ->
            val page = app.core.favItems(id, pn.toUInt())
            page.items to page.hasMore
        }
    }
    val list = rememberLazyListState()
    val nearEnd by remember { derivedStateOf { (list.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0) >= pager.items.size - 4 } }
    LaunchedEffect(nearEnd, pager.items.size) { if (nearEnd && pager.error == null) pager.more() }
    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, title)
        LazyColumn(Modifier.weight(1f), state = list, contentPadding = PaddingValues(bottom = 24.dp)) {
            items(pager.items, key = { it.bvid.ifEmpty { it.aid.toString() } }) { card ->
                VideoRow(card, "${formatCount(card.views)} 播放 · ${formatAge(card.pubdate)}") { nav.push(Screen.Player(card.bvid, card.title)) }
            }
            listFooter(nav, pager.items.isEmpty(), pager.loading, pager.hasMore, pager.error) { pager.more() }
        }
    }
}

/** Compact list row: small cover on the left, title / UP / one meta line on the right. */
@Composable
private fun VideoRow(card: FfiVideoCard, meta: String, onClick: () -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Box(Modifier.size(width = 136.dp, height = 76.dp).background(c.bg2)) {
            AsyncImage(model = card.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
            if (card.durationS > 0u) {
                Text(
                    formatDuration(card.durationS),
                    style = MetroType.meta,
                    color = Color.White,
                    modifier = Modifier.align(Alignment.BottomEnd).background(Color(0xBF000000)).padding(horizontal = 4.dp),
                )
            }
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(card.title, style = MetroType.body, color = c.fg, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Text(card.upName, fontSize = 12.sp, color = c.fg2, maxLines = 1)
            Meta(meta)
        }
    }
}
