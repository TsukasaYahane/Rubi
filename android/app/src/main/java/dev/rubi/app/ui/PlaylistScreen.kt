package dev.rubi.app.ui

import android.widget.Toast
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.RubiApp
import dev.rubi.app.playback.AudioQueue
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiPlaylistMode
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/** Long-press on any video card or row: put it on the play list. Provided by the root. */
val LocalAddToPlaylist = compositionLocalOf<(FfiVideoCard) -> Unit> { {} }

/** Adds `card` to the play list and says how it went. */
fun addToPlaylist(app: RubiApp, scope: CoroutineScope, card: FfiVideoCard) {
    scope.launch {
        val msg = runCatching { if (app.core.playlistAdd(card.bvid)) "已加入播放列表" else "已在播放列表中" }
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

@Composable
fun PlaylistScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val ctx = LocalContext.current
    var items by remember { mutableStateOf(emptyList<FfiVideoCard>()) }
    var mode by remember { mutableStateOf(FfiPlaylistMode.SEQUENTIAL) }
    var version by remember { mutableStateOf(0) }
    val hearing by AudioQueue.current
    val audioPlaying by AudioQueue.playing

    LaunchedEffect(version) {
        items = runCatching { app.core.playlistItems() }.getOrDefault(emptyList())
        mode = runCatching { app.core.playlistMode() }.getOrDefault(FfiPlaylistMode.SEQUENTIAL)
    }
    fun change(block: () -> Unit) {
        runCatching(block)
        version++
    }
    fun play(card: FfiVideoCard) = nav.push(Screen.Player(card.bvid, card.title, fromPlaylist = true))

    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "播放列表")
        Row(Modifier.padding(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(18.dp)) {
            MODES.forEach { (label, m) ->
                Text(
                    label,
                    fontSize = 15.sp,
                    fontWeight = if (m == mode) FontWeight.SemiBold else FontWeight.Normal,
                    color = if (m == mode) c.fg else c.fg2,
                    modifier = Modifier.heightIn(min = 44.dp).clickable { change { app.core.setPlaylistMode(m) } }.padding(top = 12.dp),
                )
            }
        }
        Row(Modifier.padding(horizontal = 16.dp, vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(20.dp), verticalAlignment = Alignment.CenterVertically) {
            AccentButton("播放全部") { runCatching { app.core.playlistFirst() }.getOrNull()?.let(::play) }
            TextAction("听全部", onClick = {
                runCatching { app.core.playlistFirst() }.getOrNull()?.let {
                    AudioQueue.start(ctx, app.core, it)
                    nav.push(Screen.Listen)
                }
            })
            if (items.isNotEmpty()) TextAction("清空", onClick = { change { app.core.playlistClear() } }, dim = true)
        }
        hearing?.let { now ->
            // Tapping the 正在听 block opens the 听视频 page.
            Column(Modifier.clickable { nav.push(Screen.Listen) }.padding(horizontal = 16.dp, vertical = 4.dp)) {
                Meta("正在听 · 点按打开播放页")
                Text(now.title, fontSize = 15.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Row(horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                    TextAction(if (audioPlaying) "暂停" else "继续", onClick = { AudioQueue.toggle() })
                    TextAction("下一首", onClick = { AudioQueue.next() })
                    TextAction("停止", onClick = { AudioQueue.stop() }, dim = true)
                }
            }
        }
        if (items.isEmpty()) {
            Meta("长按任意视频卡片，或在播放页点「列表」加入", Modifier.padding(16.dp))
        }
        LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 24.dp)) {
            itemsIndexed(items, key = { _, it -> it.bvid }) { i, card ->
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Column(Modifier.weight(1f)) {
                        VideoRow(card, "${if (hearing?.bvid == card.bvid) "▶ 正在听 · " else ""}${formatDuration(card.durationS)}") { play(card) }
                    }
                    Column(Modifier.width(44.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                        if (i > 0) RowAction("↑") { change { app.core.playlistMove(i.toUInt(), (i - 1).toUInt()) } }
                        if (i < items.lastIndex) RowAction("↓") { change { app.core.playlistMove(i.toUInt(), (i + 1).toUInt()) } }
                        RowAction("✕") { change { app.core.playlistRemove(card.bvid) } }
                    }
                }
            }
        }
    }
}

@Composable
private fun RowAction(label: String, onClick: () -> Unit) {
    val c = LocalMetro.current
    Text(label, fontSize = 16.sp, color = c.fg2, modifier = Modifier.heightIn(min = 32.dp).clickable(onClick = onClick).padding(8.dp))
}
