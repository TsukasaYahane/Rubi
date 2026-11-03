package dev.rubi.app.ui

import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.RubiApp
import dev.rubi.app.cast.Dlna
import dev.rubi.app.cast.Renderer
import dev.rubi.app.playback.LocalMedia
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiLiveQuality
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * 投屏: finds players on the Wi-Fi, sends the video to the one picked, and is then its remote —
 * play / pause, ±10 秒, the progress bar, volume and 清晰度 (free levels only). 换设备 picks again;
 * 退出投屏 stops the TV.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CastSheet(app: RubiApp, bvid: String, cid: ULong?, title: String, onDismiss: () -> Unit) {
    val c = LocalMetro.current
    val ctx = LocalContext.current
    val media = LocalMedia.current
    val session = media.cast
    val scope = rememberCoroutineScope()
    val casting by session.renderer
    var picking by remember { mutableStateOf(casting == null) }
    var found by remember { mutableStateOf<List<Renderer>?>(null) }
    var round by remember { mutableStateOf(0) }
    var problem by remember { mutableStateOf<String?>(null) }
    var busy by remember { mutableStateOf(false) }
    var qualities by remember { mutableStateOf(emptyList<FfiLiveQuality>()) }

    LaunchedEffect(picking, round) {
        if (!picking) return@LaunchedEffect
        found = null
        found = runCatching { Dlna.discover(ctx) }.getOrDefault(emptyList())
    }

    fun castTo(r: Renderer, qn: UInt?, fromSeconds: Long) {
        if (busy) return
        busy = true
        problem = null
        scope.launch {
            runCatching {
                val file = withContext(Dispatchers.IO) { app.core.castFile(bvid, cid, qn ?: 80u) }
                qualities = file.qualities
                // The phone stops; the TV carries on from where it was.
                media.player.get().pause()
                session.start(r, file, bvid, cid, title, fromSeconds)
            }.onSuccess { picking = false }.onFailure { problem = "投屏失败：${it.message}" }
            busy = false
        }
    }

    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg2, shape = RectangleShape, dragHandle = null) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            val r = casting
            if (picking || r == null) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text("投屏到", style = MetroType.section, color = c.fg, modifier = Modifier.weight(1f))
                    val list = found
                    Meta(if (list == null) "正在搜索同一 Wi-Fi 下的设备…" else "找到 ${list.size} 台")
                }
                found?.forEach { d ->
                    Row(
                        Modifier.fillMaxWidth().clickable { castTo(d, session.quality.value.takeIf { it > 0u }, media.player.get().currentPosition / 1000) }.padding(vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        Box(Modifier.size(width = 36.dp, height = 24.dp).border(2.dp, c.fg))
                        Text(d.name, fontSize = 15.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                }
                if (found?.isEmpty() == true) {
                    Meta("没有找到可投屏的设备。请确认电视与手机连着同一个 Wi-Fi，并已打开电视的投屏（DLNA）功能。")
                    TextAction("重新搜索", onClick = { round++ })
                }
                Meta("只投免费可用的画质；投屏时视频由手机转给电视，手机需保持在同一 Wi-Fi。")
            } else {
                CastRemote(r, title = session.title.value, qualities = qualities, current = session.quality.value,
                    onQuality = { qn ->
                        scope.launch {
                            val at = runCatching { Dlna.position(r).first }.getOrDefault(0L)
                            castTo(r, qn, at)
                        }
                    },
                    onSwitch = { picking = true },
                    onEnd = { scope.launch { session.end(); onDismiss() } },
                )
            }
            if (busy) Meta("正在连接…")
            problem?.let { Meta(it, color = LiveRed) }
            Row(Modifier.padding(bottom = 8.dp)) { TextAction("关闭", onClick = onDismiss, dim = true) }
        }
    }
}

@Composable
private fun CastRemote(
    r: Renderer,
    title: String,
    qualities: List<FfiLiveQuality>,
    current: UInt,
    onQuality: (UInt) -> Unit,
    onSwitch: () -> Unit,
    onEnd: () -> Unit,
) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var pos by remember { mutableStateOf(0L) }
    var len by remember { mutableStateOf(0L) }
    var playing by remember { mutableStateOf(true) }
    var volume by remember { mutableStateOf<Int?>(null) }
    LaunchedEffect(r) {
        volume = runCatching { Dlna.volume(r) }.getOrNull()
        while (true) {
            runCatching { Dlna.position(r) }.onSuccess { (p, l) -> pos = p; len = l }
            delay(1_000)
        }
    }
    Text("正在投屏到 ${r.name}", fontSize = 13.sp, color = c.accent)
    Text(title, fontSize = 16.sp, color = c.fg, maxLines = 2, overflow = TextOverflow.Ellipsis)
    // The bar: tap a spot to go there.
    var barWidth by remember { mutableStateOf(1) }
    Box(
        Modifier.fillMaxWidth().height(20.dp)
            .onSizeChanged { barWidth = it.width.coerceAtLeast(1) }
            .pointerInput(len) {
                detectTapGestures { o -> if (len > 0) scope.launch { runCatching { Dlna.seek(r, (o.x / barWidth.toFloat() * len).toLong()) } } }
            },
        contentAlignment = Alignment.CenterStart,
    ) {
        Box(Modifier.fillMaxWidth().height(4.dp).background(c.line))
        Box(Modifier.fillMaxWidth(if (len > 0) (pos.toFloat() / len).coerceIn(0f, 1f) else 0f).height(4.dp).background(c.accent))
    }
    Row(Modifier.fillMaxWidth()) {
        Meta(clockText(pos), Modifier.weight(1f))
        Meta(clockText(len))
    }
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp, Alignment.CenterHorizontally)) {
        RemoteKey("-10") { scope.launch { runCatching { Dlna.seek(r, (pos - 10).coerceAtLeast(0)) } } }
        RemoteKey(if (playing) "❚❚" else "▶", filled = true) {
            scope.launch { runCatching { if (playing) Dlna.pause(r) else Dlna.resume(r) }.onSuccess { playing = !playing } }
        }
        RemoteKey("+10") { scope.launch { runCatching { Dlna.seek(r, pos + 10) } } }
    }
    volume?.let { v ->
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Text("音量", fontSize = 13.sp, color = c.fg)
            RemoteKey("−") { scope.launch { runCatching { Dlna.setVolume(r, v - 5) }.onSuccess { volume = (v - 5).coerceAtLeast(0) } } }
            Text("$v", fontSize = 14.sp, color = c.fg, modifier = Modifier.width(36.dp))
            RemoteKey("+") { scope.launch { runCatching { Dlna.setVolume(r, v + 5) }.onSuccess { volume = (v + 5).coerceAtMost(100) } } }
        }
    }
    if (qualities.size > 1) {
        Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            qualities.forEach { q ->
                val on = q.qn == current
                Text(
                    q.name,
                    fontSize = 12.sp,
                    color = if (on) Color.White else c.fg,
                    modifier = Modifier.then(if (on) Modifier.background(c.accent) else Modifier.border(1.dp, c.line))
                        .clickable { if (!on) onQuality(q.qn) }
                        .padding(horizontal = 9.dp, vertical = 4.dp),
                )
            }
        }
    }
    Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        SmallButton("换设备", null, onSwitch)
        SmallButton("退出投屏", LiveRed, onEnd)
    }
}

@Composable
private fun RemoteKey(label: String, filled: Boolean = false, onClick: () -> Unit) {
    val c = LocalMetro.current
    Box(
        Modifier.size(52.dp).then(if (filled) Modifier.background(c.accent) else Modifier.border(1.dp, c.line)).clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) { Text(label, fontSize = 16.sp, color = if (filled) Color.White else c.fg) }
}

private fun clockText(s: Long) = if (s >= 3600) "%d:%02d:%02d".format(s / 3600, s / 60 % 60, s % 60) else "%02d:%02d".format(s / 60, s % 60)
