package dev.rubi.app.ui

import dev.rubi.app.playback.LocalMedia
import android.widget.Toast
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiPlaylist
import dev.rubi.core.FfiPlaylistMode
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/** Long-press on any video card or row: put it on the play list. Provided by the root. */
val LocalAddToPlaylist = compositionLocalOf<(FfiVideoCard) -> Unit> { {} }

/** The id of 默认, the list that always exists. */
const val DEFAULT_PLAYLIST = 1L

/** Longest list name, as the core allows. */
private const val MAX_NAME = 20

/** Adds `card` to 默认 (the quick long-press add) and says how it went. */
fun addToPlaylist(app: RubiApp, scope: CoroutineScope, card: FfiVideoCard) {
    scope.launch {
        val msg = runCatching { if (app.core.playlistAdd(DEFAULT_PLAYLIST, card.bvid)) "已加入「默认」播放列表" else "已在「默认」播放列表中" }
            .getOrElse { "加入失败：${it.message}" }
        Toast.makeText(app, msg, Toast.LENGTH_SHORT).show()
    }
}

private val MODES = listOf(
    "顺序" to FfiPlaylistMode.SEQUENTIAL,
    "列表循环" to FfiPlaylistMode.LOOP_ALL,
    "单曲循环" to FfiPlaylistMode.LOOP_ONE,
    "随机" to FfiPlaylistMode.SHUFFLE,
)

/** The list shown last on the play list page, kept while the app runs. */
private var shownList = DEFAULT_PLAYLIST

private enum class ListSheet { None, Create, Rename, Delete }

@Composable
fun PlaylistScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val listening = LocalMedia.current.listening
    var lists by remember { mutableStateOf(emptyList<FfiPlaylist>()) }
    var current by remember { mutableStateOf(shownList) }
    var items by remember { mutableStateOf(emptyList<FfiVideoCard>()) }
    var mode by remember { mutableStateOf(FfiPlaylistMode.SEQUENTIAL) }
    var version by remember { mutableStateOf(0) }
    var sheet by remember { mutableStateOf(ListSheet.None) }
    val hearing by listening.current
    val audioPlaying by listening.playing

    LaunchedEffect(version, current) {
        lists = runCatching { app.core.playlists() }.getOrDefault(emptyList())
        // A list deleted elsewhere (or from an older run) falls back to 默认.
        if (lists.isNotEmpty() && lists.none { it.id == current }) current = DEFAULT_PLAYLIST
        shownList = current
        items = runCatching { app.core.playlistItems(current) }.getOrDefault(emptyList())
        mode = runCatching { app.core.playlistMode() }.getOrDefault(FfiPlaylistMode.SEQUENTIAL)
    }
    fun change(block: () -> Unit) {
        runCatching(block)
        version++
    }
    fun play(card: FfiVideoCard) {
        app.core.playlistPlay(current)
        nav.push(Screen.Player(card.bvid, card.title, fromPlaylist = true))
    }
    val list = lists.firstOrNull { it.id == current }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            PageHeader(nav, "播放列表")
            // The lists, then 新建.
            Row(
                Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                lists.forEach { l -> Chip("${l.name} ${l.count}", l.id == current) { current = l.id; sheet = ListSheet.None } }
                Chip("+ 新建", false) { sheet = ListSheet.Create }
            }
            Row(Modifier.padding(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(18.dp)) {
                MODES.forEach { (label, m) ->
                    Text(
                        label,
                        fontSize = 15.sp,
                        fontWeight = if (m == mode) FontWeight.SemiBold else FontWeight.Normal,
                        color = if (m == mode) c.fg else c.fg2,
                        modifier = Modifier.heightIn(min = 40.dp).clickable { change { app.core.setPlaylistMode(m) } }.padding(top = 10.dp),
                    )
                }
            }
            Row(Modifier.padding(horizontal = 16.dp, vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(18.dp), verticalAlignment = Alignment.CenterVertically) {
                AccentButton("播放全部") { runCatching { app.core.playlistFirst(current) }.getOrNull()?.let(::play) }
                TextAction("听全部", onClick = {
                    runCatching { app.core.playlistFirst(current) }.getOrNull()?.let {
                        listening.start(it, current)
                        nav.push(Screen.Listen)
                    }
                })
                if (list != null && !list.fixed) {
                    TextAction("重命名", onClick = { sheet = ListSheet.Rename }, dim = true)
                    TextAction("删除列表", onClick = { sheet = ListSheet.Delete }, dim = true)
                } else if (items.isNotEmpty()) {
                    TextAction("清空", onClick = { change { app.core.playlistClear(current) } }, dim = true)
                }
            }
            hearing?.let { now ->
                // Tapping the 正在听 block opens the 听视频 page.
                Column(Modifier.clickable { nav.push(Screen.Listen) }.padding(horizontal = 16.dp, vertical = 4.dp)) {
                    Meta("正在听 · 点按打开播放页")
                    Text(now.title, fontSize = 15.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                        TextAction(if (audioPlaying) "暂停" else "继续", onClick = { listening.toggle() })
                        TextAction("下一首", onClick = { listening.next() })
                        TextAction("停止", onClick = { listening.stop() }, dim = true)
                    }
                }
            }
            if (items.isEmpty()) {
                Meta(
                    if (current == DEFAULT_PLAYLIST) "长按任意视频卡片，或在播放页点「列表」加入" else "这个列表还是空的。在播放页点「列表」加入",
                    Modifier.padding(16.dp),
                )
            }
            LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 24.dp)) {
                itemsIndexed(items, key = { _, it -> it.bvid }) { i, card ->
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Column(Modifier.weight(1f)) {
                            VideoRow(card, "${if (hearing?.bvid == card.bvid) "▶ 正在听 · " else ""}${formatDuration(card.durationS)}") { play(card) }
                        }
                        Column(Modifier.width(44.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                            if (i > 0) RowAction("↑") { change { app.core.playlistMove(current, i.toUInt(), (i - 1).toUInt()) } }
                            if (i < items.lastIndex) RowAction("↓") { change { app.core.playlistMove(current, i.toUInt(), (i + 1).toUInt()) } }
                            RowAction("✕") { change { app.core.playlistRemove(current, card.bvid) } }
                        }
                    }
                }
            }
        }
        // 新建 / 重命名 / 删除, as a panel along the bottom.
        when (sheet) {
            ListSheet.Create, ListSheet.Rename -> {
                val renaming = sheet == ListSheet.Rename
                NamePanel(
                    title = if (renaming) "重命名" else "新建播放列表",
                    initial = if (renaming) list?.name.orEmpty() else "",
                    onCancel = { sheet = ListSheet.None },
                ) { name ->
                    runCatching { if (renaming) app.core.playlistRename(current, name) else { current = app.core.playlistCreate(name) } }
                        .map { sheet = ListSheet.None; version++; null }
                        .getOrElse { it.message ?: "操作失败" }
                }
            }
            ListSheet.Delete -> list?.let { l ->
                BottomPanel {
                    Text("删除「${l.name}」？里面的 ${l.count} 个视频会一起移出。", fontSize = 14.sp, color = c.fg)
                    Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(18.dp), verticalAlignment = Alignment.CenterVertically) {
                        SmallButton("删除", LiveRed) {
                            change { app.core.playlistDelete(l.id) }
                            current = DEFAULT_PLAYLIST
                            sheet = ListSheet.None
                        }
                        TextAction("取消", onClick = { sheet = ListSheet.None })
                    }
                }
            }
            ListSheet.None -> {}
        }
    }
}

/**
 * 加入播放列表 from the video page: tick the lists that should hold the video, or make a new one
 * with it in. Changes apply at once.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PlaylistPickSheet(app: RubiApp, bvid: String, onDismiss: () -> Unit) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var lists by remember { mutableStateOf(emptyList<FfiPlaylist>()) }
    var holding by remember { mutableStateOf(emptySet<Long>()) }
    var version by remember { mutableStateOf(0) }
    var problem by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(version) {
        lists = runCatching { app.core.playlists() }.getOrDefault(emptyList())
        holding = runCatching { app.core.playlistsWith(bvid).toSet() }.getOrDefault(emptySet())
    }
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg2, shape = RectangleShape, dragHandle = null) {
        Column(Modifier.padding(16.dp)) {
            Text("加入播放列表", style = MetroType.section, color = c.fg)
            lists.forEach { l ->
                val on = l.id in holding
                Row(
                    Modifier.fillMaxWidth().clickable {
                        scope.launch {
                            runCatching { if (on) app.core.playlistRemove(l.id, bvid) else app.core.playlistAdd(l.id, bvid) }
                                .onFailure { problem = "操作失败：${it.message}" }
                            version++
                        }
                    }.padding(vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(l.name, fontSize = 15.sp, color = c.fg)
                    Text("  ${l.count}", fontSize = 12.sp, color = c.fg2, modifier = Modifier.weight(1f))
                    Box(Modifier.size(18.dp).border(2.dp, c.fg).padding(3.dp).background(if (on) c.accent else Color.Transparent))
                }
            }
            var name by remember { mutableStateOf("") }
            Row(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                NameField(name, "新列表名称", Modifier.weight(1f)) { name = it; problem = null }
                SmallButton("新建并加入", null) {
                    scope.launch {
                        runCatching { app.core.playlistAdd(app.core.playlistCreate(name), bvid) }
                            .onSuccess { name = ""; version++ }
                            .onFailure { problem = it.message }
                    }
                }
            }
            problem?.let { Meta(it, Modifier.padding(top = 4.dp), color = LiveRed) }
            Row(Modifier.padding(top = 8.dp, bottom = 8.dp)) { TextAction("完成", onClick = onDismiss) }
        }
    }
}

/** A name to type for a list; `onDone` returns what is wrong with it, or null when it took. */
@Composable
private fun androidx.compose.foundation.layout.BoxScope.NamePanel(title: String, initial: String, onCancel: () -> Unit, onDone: (String) -> String?) {
    val c = LocalMetro.current
    var name by remember(initial) { mutableStateOf(initial) }
    var problem by remember { mutableStateOf<String?>(null) }
    val submit = { problem = onDone(name) }
    BottomPanel {
        Text(title, fontSize = 14.sp, color = c.fg)
        Row(Modifier.padding(top = 8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            NameField(name, "列表名称", Modifier.weight(1f), onDone = { submit() }) { name = it; problem = null }
            SmallButton("确定", c.accent) { submit() }
            TextAction("取消", onClick = onCancel, dim = true)
        }
        problem?.let { Meta(it, Modifier.padding(top = 4.dp), color = LiveRed) }
    }
}

@Composable
internal fun androidx.compose.foundation.layout.BoxScope.BottomPanel(content: @Composable () -> Unit) {
    val c = LocalMetro.current
    Column(Modifier.align(Alignment.BottomCenter).fillMaxWidth().background(c.bg2).padding(horizontal = 16.dp, vertical = 12.dp)) {
        Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
        Column(Modifier.padding(top = 10.dp)) { content() }
    }
}

/** A low, square single-line field. */
@Composable
internal fun NameField(value: String, hint: String, modifier: Modifier, max: Int = MAX_NAME, onDone: () -> Unit = {}, onChange: (String) -> Unit) {
    val c = LocalMetro.current
    Box(modifier.height(30.dp).border(1.dp, c.line).padding(horizontal = 8.dp), contentAlignment = Alignment.CenterStart) {
        if (value.isEmpty()) Text(hint, fontSize = 13.sp, color = c.fg2)
        BasicTextField(
            value = value,
            onValueChange = { onChange(it.take(max)) },
            singleLine = true,
            textStyle = TextStyle(color = c.fg, fontSize = 13.sp),
            cursorBrush = SolidColor(c.accent),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { onDone() }),
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

/** As tall as [NameField]: filled with `fill`, or outlined when it is null. */
@Composable
internal fun SmallButton(text: String, fill: Color?, onClick: () -> Unit) {
    val c = LocalMetro.current
    Box(
        Modifier.height(30.dp).then(if (fill != null) Modifier.background(fill) else Modifier.border(1.dp, c.fg)).clickable(onClick = onClick).padding(horizontal = 12.dp),
        contentAlignment = Alignment.Center,
    ) { Text(text, fontSize = 13.sp, color = if (fill != null) Color.White else c.fg) }
}

@Composable
private fun Chip(label: String, on: Boolean, onClick: () -> Unit) {
    val c = LocalMetro.current
    Text(
        label,
        fontSize = 13.sp,
        color = if (on) Color.White else c.fg,
        modifier = Modifier.then(if (on) Modifier.background(c.accent) else Modifier.border(1.dp, c.line))
            .clickable(onClick = onClick)
            .padding(horizontal = 10.dp, vertical = 6.dp),
    )
}

@Composable
private fun RowAction(label: String, onClick: () -> Unit) {
    val c = LocalMetro.current
    Text(label, fontSize = 16.sp, color = c.fg2, modifier = Modifier.heightIn(min = 32.dp).clickable(onClick = onClick).padding(8.dp))
}
