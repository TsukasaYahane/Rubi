package dev.rubi.app.ui

import androidx.compose.foundation.layout.height
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Spacer
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
import androidx.compose.foundation.lazy.LazyListState
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
import dev.rubi.core.RubiCore
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/** Status rows at the end of a list: loading, error with 重试, empty or 没有更多了. */
internal fun <T, C> LazyListScope.listFooter(nav: Nav, paged: Paged<T, C>) {
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

/**
 * 历史记录 as loaded and scrolled, kept with the page in the back stack: coming back from a video
 * shows the same list; it is read afresh only when the page is opened again.
 */
class HistoryState(core: RubiCore) {
    val pager = Paged<FfiHistoryItem, String>(key = { it.card.bvid + it.viewedAt }) { cursor ->
        val page = core.history(cursor)
        Page(page.items, page.cursor)
    }
    val list = LazyListState()
}

/** 稍后再看, kept the same way as [HistoryState]. */
class WatchLaterState(core: RubiCore) {
    val pager = Paged<FfiHistoryItem, Unit>(key = { it.card.bvid }) { Page(core.watchLaterList(), null) }
    val list = LazyListState()
}

@Composable
fun HistoryScreen(nav: Nav, state: HistoryState) {
    val pager = state.pager
    val list = state.list
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

/**
 * 稍后再看: each video with how far it was watched and 移除; 全部播放 plays them through as a list
 * that is not kept. 管理 ticks videos to take out together, or clears the ones watched.
 */
@Composable
fun WatchLaterScreen(app: RubiApp, nav: Nav, state: WatchLaterState) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val pager = state.pager
    LaunchedEffect(state) { if (pager.items.isEmpty() && !pager.ended) pager.more() }
    var managing by remember { mutableStateOf(false) }
    val picked = remember { mutableStateListOf<ULong>() }
    var note by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }

    // Done: the videos leave the list on the spot (B站 may still list them for a while, so the
    // list is not read again).
    fun act(done: String, gone: (FfiHistoryItem) -> Boolean, block: suspend () -> Unit) {
        if (busy) return
        busy = true
        scope.launch {
            runCatching { block() }
                .onSuccess { note = done; picked.clear(); pager.removeAll(gone) }
                .onFailure { note = if (it is RubiException.NotLoggedIn) "需要登录" else "操作失败：${it.message}" }
            busy = false
        }
    }

    Column(Modifier.fillMaxSize()) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) { PageHeader(nav, "稍后再看") }
            Text(
                if (managing) "完成" else "管理",
                fontSize = 14.sp,
                color = c.fg,
                modifier = Modifier.clickable { managing = !managing; picked.clear(); note = "" }.padding(horizontal = 16.dp, vertical = 10.dp),
            )
        }
        Meta("${pager.items.size} 个视频" + if (managing) " · 已选 ${picked.size}" else "", Modifier.padding(horizontal = 16.dp))
        if (!managing && pager.items.isNotEmpty()) {
            Box(Modifier.padding(start = 16.dp, top = 8.dp, bottom = 4.dp)) {
                AccentButton("▶ 全部播放") { TempPlay.start(app, nav, "稍后再看", pager.items.map { it.card.bvid }) }
            }
        }
        LazyColumn(Modifier.weight(1f), state = state.list, contentPadding = PaddingValues(bottom = 24.dp)) {
            items(pager.items, key = { it.card.bvid }) { h ->
                val card = h.card
                val progress = when {
                    h.progressS < 0 -> "已看完"
                    h.progressS == 0L -> "未看"
                    else -> "看到 ${formatDuration(h.progressS.toUInt())}"
                }
                if (managing) {
                    val on = card.aid in picked
                    Row(
                        Modifier.fillMaxWidth().clickable { if (on) picked.remove(card.aid) else picked.add(card.aid) }.padding(start = 16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Box(Modifier.size(18.dp).border(2.dp, c.fg).padding(3.dp).background(if (on) c.accent else Color.Transparent))
                        Box(Modifier.weight(1f)) {
                            VideoRow(card, progress) { if (on) picked.remove(card.aid) else picked.add(card.aid) }
                        }
                    }
                } else {
                    Box {
                        VideoRow(card, progress) { nav.push(Screen.Player(card.bvid, card.title)) }
                        Text(
                            "移除",
                            fontSize = 13.sp,
                            color = c.fg2,
                            modifier = Modifier.align(Alignment.BottomEnd).clickable {
                                act("已移除「${card.title}」", { it.card.aid == card.aid }) { app.core.removeWatchLater(listOf(card.aid)) }
                            }.padding(horizontal = 16.dp, vertical = 10.dp),
                        )
                    }
                }
            }
            listFooter(nav, pager)
        }
        if (note.isNotEmpty()) Meta(note, Modifier.padding(horizontal = 16.dp, vertical = 4.dp), color = c.accent)
        if (managing) {
            Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
            Row(Modifier.padding(horizontal = 16.dp, vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                val all = pager.items.isNotEmpty() && picked.size == pager.items.size
                SmallButton(if (all) "取消全选" else "全选", null) {
                    picked.clear()
                    if (!all) picked.addAll(pager.items.map { it.card.aid })
                }
                SmallButton("清除已看完", null) { act("已清除看完的视频", { it.progressS < 0 }) { app.core.clearWatchedLater() } }
                Spacer(Modifier.weight(1f))
                val some = picked.isNotEmpty() && !busy
                SmallButton("删除 ${picked.size}", if (some) LiveRed else c.fg2) {
                    if (some) {
                        val aids = picked.toList()
                        act("已删除 ${aids.size} 个视频", { it.card.aid in aids }) { app.core.removeWatchLater(aids) }
                    }
                }
            }
        }
    }
}

@Composable
fun FavFoldersScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val pager = remember { Paged<FfiFavFolder, Unit>(scope, key = { it.id }) { Page(app.core.myFavFolders(), null) } }
    LaunchedEffect(Unit) { pager.more() }
    // The folder being edited (null id: a new one), or null with the panel closed.
    var editing by remember { mutableStateOf<FolderDraft?>(null) }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            PageHeader(nav, "收藏夹")
            Row(Modifier.padding(horizontal = 16.dp, vertical = 6.dp)) {
                SmallButton("+ 新建收藏夹", c.accent) { editing = FolderDraft(null, "", "", false, false) }
            }
            LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 24.dp)) {
                items(pager.items, key = { it.id.toString() }) { f ->
                    Row(
                        Modifier.fillMaxWidth().heightIn(min = 64.dp).clickable { nav.push(Screen.FavFolder(f.id, f.title)) }
                            .padding(horizontal = 16.dp, vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(Modifier.weight(1f)) {
                            Text(f.title, style = MetroType.section, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            Meta("${f.count} 个视频 · ${if (f.private) "私密" else "公开"}${if (f.default) " · 默认" else ""}")
                        }
                        TextAction("编辑", onClick = {
                            val draft = FolderDraft(f.id, f.title, "", f.private, f.default)
                            editing = draft
                            // The list has no 简介; fetch it into the open panel.
                            scope.launch {
                                runCatching { app.core.favFolderIntro(f.id) }.onSuccess { intro ->
                                    if (editing?.id == f.id && editing?.intro.isNullOrEmpty()) editing = editing?.copy(intro = intro)
                                }
                            }
                        }, dim = true)
                    }
                }
                listFooter(nav, pager)
            }
        }
        editing?.let { d ->
            FolderPanel(
                draft = d,
                onChange = { editing = it },
                onCancel = { editing = null },
                onSave = { done ->
                    scope.launch {
                        runCatching {
                            val id = d.id
                            if (id == null) app.core.createFavFolder(d.title, d.intro, d.private) else app.core.editFavFolder(id, d.title, d.intro, d.private)
                        }.onSuccess { editing = null; pager.reset() }.onFailure { done(writeError(it)) }
                    }
                },
                onDelete = { done ->
                    val id = d.id ?: return@FolderPanel
                    scope.launch {
                        runCatching { app.core.deleteFavFolder(id) }.onSuccess { editing = null; pager.reset() }.onFailure { done(writeError(it)) }
                    }
                },
            )
        }
    }
}

/** A 收藏夹 being made or edited. */
private data class FolderDraft(val id: ULong?, val title: String, val intro: String, val private: Boolean, val default: Boolean)

private fun writeError(e: Throwable): String = if (e is RubiException.NotLoggedIn) "需要登录" else e.message ?: "操作失败"

/** The panel along the bottom: name, 简介, 公开, then 保存 / 取消 (and 删除 for a folder that is not 默认). */
@Composable
private fun androidx.compose.foundation.layout.BoxScope.FolderPanel(
    draft: FolderDraft,
    onChange: (FolderDraft) -> Unit,
    onCancel: () -> Unit,
    onSave: ((String) -> Unit) -> Unit,
    onDelete: ((String) -> Unit) -> Unit,
) {
    val c = LocalMetro.current
    var problem by remember(draft.id) { mutableStateOf<String?>(null) }
    var confirmDelete by remember(draft.id) { mutableStateOf(false) }
    BottomPanel {
        Text(if (draft.id == null) "新建收藏夹" else "编辑收藏夹", fontSize = 14.sp, color = c.fg)
        NameField(draft.title, "名称", Modifier.fillMaxWidth().padding(top = 8.dp)) { onChange(draft.copy(title = it)); problem = null }
        NameField(draft.intro, "简介（可不填）", Modifier.fillMaxWidth().padding(top = 6.dp), max = 200) { onChange(draft.copy(intro = it)) }
        Row(Modifier.fillMaxWidth().clickable { onChange(draft.copy(private = !draft.private)) }.padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("公开", fontSize = 14.sp, color = c.fg, modifier = Modifier.weight(1f))
            SquareSwitch(!draft.private)
        }
        problem?.let { Meta(it, color = LiveRed) }
        Row(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(14.dp), verticalAlignment = Alignment.CenterVertically) {
            SmallButton("保存", c.accent) {
                if (draft.title.isBlank()) problem = "请输入名称" else onSave { problem = it }
            }
            TextAction("取消", onClick = onCancel, dim = true)
            if (draft.id != null && !draft.default) {
                Spacer(Modifier.weight(1f))
                SmallButton(if (confirmDelete) "确认删除" else "删除", LiveRed) {
                    if (confirmDelete) onDelete { problem = it } else confirmDelete = true
                }
            }
        }
    }
}

@Composable
fun FavFolderScreen(app: RubiApp, nav: Nav, id: ULong, title: String) {
    val c = LocalMetro.current
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
    // 编辑: tick videos, then take them out or move / copy them to another folder.
    var editing by remember { mutableStateOf(false) }
    val picked = remember { mutableStateListOf<ULong>() }
    var target by remember { mutableStateOf<Boolean?>(null) } // true: copy, false: move
    var note by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }

    fun act(done: String, block: suspend () -> Unit) {
        if (busy) return
        busy = true
        scope.launch {
            runCatching { block() }
                .onSuccess {
                    note = done
                    picked.clear()
                    pager.reset()
                }
                .onFailure { note = if (it is RubiException.NotLoggedIn) "需要登录" else "操作失败：${it.message}" }
            busy = false
        }
    }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) { PageHeader(nav, title) }
                Text(
                    if (editing) "完成" else "编辑",
                    fontSize = 14.sp,
                    color = c.fg,
                    modifier = Modifier.clickable { editing = !editing; picked.clear(); note = "" }.padding(horizontal = 16.dp, vertical = 10.dp),
                )
            }
            Meta(
                "${pager.items.size} 个视频" + if (editing) " · 已选 ${picked.size}" else "",
                Modifier.padding(horizontal = 16.dp),
            )
            if (!editing && pager.items.isNotEmpty()) {
                Box(Modifier.padding(start = 16.dp, top = 8.dp, bottom = 4.dp)) {
                    // The whole folder as a list that is not kept; later pages join as they load.
                    AccentButton("▶ 一键播放") {
                        TempPlay.start(app, nav, title, pager.items.map { it.bvid }) { pn ->
                            val page = app.core.favItems(id, pn.toUInt())
                            page.items.map { it.bvid } to page.hasMore
                        }
                    }
                }
            }
            if (editing && pager.items.isNotEmpty()) {
                val all = picked.size == pager.items.size
                Box(Modifier.padding(horizontal = 16.dp)) {
                    TextAction(if (all) "取消全选" else "全选", onClick = {
                        picked.clear()
                        if (!all) picked.addAll(pager.items.map { it.aid })
                    }, dim = true)
                }
            }
            LazyColumn(Modifier.weight(1f), state = list, contentPadding = PaddingValues(bottom = 24.dp)) {
                items(pager.items, key = { it.bvid.ifEmpty { it.aid.toString() } }) { card ->
                    if (editing) {
                        val on = card.aid in picked
                        Row(
                            Modifier.fillMaxWidth().clickable { if (on) picked.remove(card.aid) else picked.add(card.aid) }.padding(start = 16.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Box(Modifier.size(18.dp).border(2.dp, c.fg).padding(3.dp).background(if (on) c.accent else Color.Transparent))
                            Box(Modifier.weight(1f)) {
                                VideoRow(card, "${formatCount(card.views)} 播放 · ${formatAge(card.pubdate)}") {
                                    if (on) picked.remove(card.aid) else picked.add(card.aid)
                                }
                            }
                        }
                    } else {
                        VideoRow(card, "${formatCount(card.views)} 播放 · ${formatAge(card.pubdate)}") { nav.push(Screen.Player(card.bvid, card.title)) }
                    }
                }
                listFooter(nav, pager)
            }
            if (note.isNotEmpty()) Meta(note, Modifier.padding(horizontal = 16.dp, vertical = 4.dp), color = c.accent)
            if (editing) {
                Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
                Row(Modifier.padding(horizontal = 16.dp, vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    val some = picked.isNotEmpty() && !busy
                    SmallButton("移出收藏夹", if (some) LiveRed else c.fg2) {
                        if (some) {
                            val aids = picked.toList()
                            act("已移出 ${aids.size} 个视频") { app.core.removeFromFavFolder(id, aids) }
                        }
                    }
                    SmallButton("移动到…", null) { if (some) target = false }
                    SmallButton("复制到…", null) { if (some) target = true }
                }
            }
        }
        target?.let { keep ->
            FolderPicker(app, exclude = id, title = "${if (keep) "复制" else "移动"} ${picked.size} 个视频到", onDismiss = { target = null }) { to ->
                target = null
                val aids = picked.toList()
                act("已${if (keep) "复制" else "移动"} ${aids.size} 个视频到「${to.title}」") { app.core.transferFav(id, to.id, aids, keep) }
            }
        }
    }
}

/** The user's other folders, to move or copy videos into. */
@Composable
private fun androidx.compose.foundation.layout.BoxScope.FolderPicker(
    app: RubiApp,
    exclude: ULong,
    title: String,
    onDismiss: () -> Unit,
    onPick: (FfiFavFolder) -> Unit,
) {
    val c = LocalMetro.current
    var folders by remember { mutableStateOf<List<FfiFavFolder>?>(null) }
    LaunchedEffect(Unit) { folders = runCatching { app.core.myFavFolders() }.getOrDefault(emptyList()).filter { it.id != exclude } }
    BottomPanel {
        Text(title, fontSize = 14.sp, color = c.fg)
        val list = folders
        when {
            list == null -> Meta("读取收藏夹…", Modifier.padding(vertical = 8.dp))
            list.isEmpty() -> Meta("没有别的收藏夹，先新建一个", Modifier.padding(vertical = 8.dp))
            else -> list.forEach { f ->
                Row(Modifier.fillMaxWidth().clickable { onPick(f) }.padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(f.title, fontSize = 15.sp, color = c.fg, modifier = Modifier.weight(1f))
                    Meta("${f.count}")
                }
            }
        }
        TextAction("取消", onClick = onDismiss, dim = true)
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
            UpName(card.upName, card.upMid, 12.sp, c.fg2)
            Meta(meta)
        }
    }
}
