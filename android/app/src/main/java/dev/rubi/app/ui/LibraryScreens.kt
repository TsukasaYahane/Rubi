package dev.rubi.app.ui

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
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

/** Status rows at the end of a list: loading, error with 重试, empty or 没有更多了. */
private fun <T, C> LazyListScope.listFooter(nav: Nav, paged: Paged<T, C>) {
    val error = paged.error
    // No key: a keyed footer is what the list would hold on to while the first page is
    // inserted above it, opening the list at the bottom instead of the newest entry.
    item {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            when {
                error != null -> {
                    Meta(errorText(error))
                    if (error is RubiException.NotLoggedIn) AccentButton("登录") { nav.push(Screen.Login) }
                    else TextAction("重试", onClick = paged::retry)
                }
                paged.loading -> Meta("加载中…")
                paged.items.isEmpty() -> Meta("这里还是空的")
                paged.ended -> Meta("没有更多了")
            }
        }
    }
}

@Composable
fun HistoryScreen(app: RubiApp, nav: Nav) {
    val scope = rememberCoroutineScope()
    val pager = remember {
        Paged<FfiHistoryItem, String>(scope, key = { it.card.bvid + it.viewedAt }) { cursor ->
            val page = app.core.history(cursor)
            Page(page.items, page.cursor)
        }
    }
    val list = rememberLazyListState()
    LoadMoreNearEnd(list, pager)

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
            listFooter(nav, pager)
        }
    }
}

@Composable
fun WatchLaterScreen(app: RubiApp, nav: Nav) {
    val scope = rememberCoroutineScope()
    val pager = remember { Paged<FfiVideoCard, Unit>(scope, key = { it.bvid }) { Page(app.core.watchLaterList(), null) } }
    LaunchedEffect(Unit) { pager.more() }
    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "稍后再看")
        LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 24.dp)) {
            items(pager.items, key = { it.bvid }) { card ->
                VideoRow(card, "${formatCount(card.views)} 播放 · ${formatAge(card.pubdate)}") { nav.push(Screen.Player(card.bvid, card.title)) }
            }
            listFooter(nav, pager)
        }
    }
}

@Composable
fun FavFoldersScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val pager = remember { Paged<FfiFavFolder, Unit>(scope, key = { it.id }) { Page(app.core.myFavFolders(), null) } }
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
            listFooter(nav, pager)
        }
    }
}

@Composable
fun FavFolderScreen(app: RubiApp, nav: Nav, id: ULong, title: String) {
    val scope = rememberCoroutineScope()
    val pager = remember {
        Paged<FfiVideoCard, Int>(scope, key = { it.bvid.ifEmpty { it.aid.toString() } }) { pn ->
            val at = pn ?: 1
            val page = app.core.favItems(id, at.toUInt())
            Page(page.items, if (page.hasMore && page.items.isNotEmpty()) at + 1 else null)
        }
    }
    val list = rememberLazyListState()
    LoadMoreNearEnd(list, pager)
    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, title)
        LazyColumn(Modifier.weight(1f), state = list, contentPadding = PaddingValues(bottom = 24.dp)) {
            items(pager.items, key = { it.bvid.ifEmpty { it.aid.toString() } }) { card ->
                VideoRow(card, "${formatCount(card.views)} 播放 · ${formatAge(card.pubdate)}") { nav.push(Screen.Player(card.bvid, card.title)) }
            }
            listFooter(nav, pager)
        }
    }
}

/** Compact list row: small cover on the left, title / UP / one meta line on the right.
 *  Long-press puts the video on the play list. */
@OptIn(ExperimentalFoundationApi::class)
@Composable
internal fun VideoRow(card: FfiVideoCard, meta: String, onClick: () -> Unit) {
    val c = LocalMetro.current
    val addToList = LocalAddToPlaylist.current
    Row(
        Modifier.fillMaxWidth().combinedClickable(onLongClick = { addToList(card) }, onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 8.dp),
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
