package dev.rubi.app.ui

import android.os.StatFs
import android.widget.Toast
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.RubiApp
import dev.rubi.app.data.onUnmeteredNetwork
import dev.rubi.app.playback.DownloadService
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiDownload
import dev.rubi.core.FfiDownloadState
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/** Qualities offered for offline copies: 1080P and below. */
val DOWNLOAD_QUALITIES = listOf(80u to "1080P 高清", 64u to "720P 高清", 32u to "480P 清晰", 16u to "360P 流畅")

/** `n` bytes as MB / GB. */
fun formatBytes(n: ULong): String = when {
    n >= 1_073_741_824uL -> "%.1f GB".format(n.toDouble() / 1_073_741_824.0)
    else -> "%.0f MB".format(n.toDouble() / 1_048_576.0)
}

/**
 * Starts an offline copy at quality `qn`, unless 仅 Wi-Fi is on and the network is metered.
 * Returns what to tell the user.
 */
suspend fun startDownload(app: RubiApp, bvid: String, qn: UInt, label: String): String {
    if (app.ui.downloadWifiOnly && !app.onUnmeteredNetwork()) return "当前不是 Wi-Fi：已开启仅 Wi-Fi 下载，可在离线缓存页关闭"
    return runCatching { app.core.downloadStart(bvid, qn) }.fold(
        {
            DownloadService.start(app)
            "已加入离线缓存 · $label"
        },
        { "缓存失败：${it.message}" },
    )
}

/** Picks the quality of an offline copy. */
@Composable
fun DownloadSheet(onDismiss: () -> Unit, onPick: (UInt, String) -> Unit) {
    val c = LocalMetro.current
    PlayerSheet(side = false, onDismiss = onDismiss) {
        Column(Modifier.padding(bottom = 12.dp)) {
            Text("缓存清晰度", fontSize = 18.sp, color = c.fg, modifier = Modifier.padding(start = 20.dp, top = 14.dp))
            Meta("只列 1080P 及以下 · 仅 Wi-Fi 时下载", Modifier.padding(start = 20.dp, bottom = 8.dp))
            DOWNLOAD_QUALITIES.forEach { (qn, label) ->
                Text(
                    label,
                    fontSize = 16.sp,
                    color = c.fg,
                    modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).clickable { onPick(qn, label.substringBefore(' ')) }.padding(horizontal = 20.dp, vertical = 13.dp),
                )
            }
        }
    }
}

/**
 * 我的 → 离线缓存: every copy with its progress, to pause, resume, delete or play. The list is
 * re-read twice a second while anything is downloading.
 */
@Composable
fun DownloadsScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val ctx = LocalContext.current
    val scope = rememberCoroutineScope()
    var items by remember { mutableStateOf(emptyList<FfiDownload>()) }
    var wifiOnly by remember { mutableStateOf(app.ui.downloadWifiOnly) }
    var version by remember { mutableStateOf(0) }
    LaunchedEffect(version) {
        while (true) {
            items = runCatching { app.core.downloads() }.getOrDefault(items)
            if (items.none { it.state == FfiDownloadState.RUNNING || it.state == FfiDownloadState.QUEUED }) break
            delay(500)
        }
    }
    fun say(msg: String) = Toast.makeText(ctx, msg, Toast.LENGTH_SHORT).show()
    fun change(block: () -> Unit) {
        runCatching(block).onFailure { say("操作失败：${it.message}") }
        version++
    }
    val used = items.sumOf { it.doneBytes }
    val free = remember { runCatching { StatFs(ctx.filesDir.path).availableBytes.toULong() }.getOrDefault(0uL) }

    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "离线缓存")
        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp), verticalAlignment = Alignment.CenterVertically) {
            Meta("已用 ${formatBytes(used)} · 手机剩余 ${formatBytes(free)}", Modifier.weight(1f))
            Row(
                Modifier.clickable {
                    wifiOnly = !wifiOnly
                    app.ui.downloadWifiOnly = wifiOnly
                }.padding(vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Meta("仅 Wi-Fi 下载")
                SquareSwitch(wifiOnly)
            }
        }
        if (items.isEmpty()) Meta("在播放页点「缓存」选择清晰度，就能没网也看", Modifier.padding(16.dp))
        LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 24.dp)) {
            items(items, key = { it.card.bvid }) { d ->
                DownloadRow(
                    d,
                    onMain = {
                        when (d.state) {
                            FfiDownloadState.DONE -> nav.push(Screen.Player(d.card.bvid, d.card.title))
                            FfiDownloadState.RUNNING, FfiDownloadState.QUEUED -> change { app.core.downloadPause(d.card.bvid) }
                            else -> {
                                if (wifiOnly && !ctx.onUnmeteredNetwork()) say("当前不是 Wi-Fi：已开启仅 Wi-Fi 下载")
                                else scope.launch {
                                    // Returns once the copy is under way; it carries on without this page.
                                    runCatching { app.core.downloadStart(d.card.bvid, d.quality) }
                                        .onSuccess { DownloadService.start(app) }
                                        .onFailure { say("继续失败：${it.message}") }
                                    version++
                                }
                            }
                        }
                    },
                    onDelete = {
                        change { app.core.downloadRemove(d.card.bvid) }
                        say("已删除，释放 ${formatBytes(d.doneBytes)}")
                    },
                )
            }
        }
    }
}

@Composable
private fun DownloadRow(d: FfiDownload, onMain: () -> Unit, onDelete: () -> Unit) {
    val c = LocalMetro.current
    val label = when (d.state) {
        FfiDownloadState.DONE -> "已完成"
        FfiDownloadState.RUNNING -> "下载中"
        FfiDownloadState.QUEUED -> "等待中"
        FfiDownloadState.PAUSED -> "已暂停"
        FfiDownloadState.FAILED -> "失败：${d.error ?: "未知原因"}，点继续从断点重试"
    }
    val quality = DOWNLOAD_QUALITIES.firstOrNull { it.first == d.quality }?.second?.substringBefore(' ') ?: "${d.quality}"
    val total = maxOf(d.totalBytes, d.doneBytes)
    Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Box(Modifier.width(84.dp).aspectRatio(16f / 9f).background(c.bg2), contentAlignment = Alignment.Center) {
            AsyncImage(model = d.card.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
            if (d.state == FfiDownloadState.DONE) Text("▶", color = Color.White, fontSize = 14.sp)
        }
        Column(Modifier.weight(1f)) {
            Text(d.card.title, fontSize = 14.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
            val size = if (d.state == FfiDownloadState.DONE) formatBytes(total) else "${formatBytes(d.doneBytes)} / ${formatBytes(total)}"
            Text("$quality · $size · $label", fontSize = 12.sp, color = if (d.state == FfiDownloadState.FAILED) Color(0xFFFF8080) else c.fg2, maxLines = 2)
            if (d.state != FfiDownloadState.DONE) {
                val f = if (total > 0u) (d.doneBytes.toDouble() / total.toDouble()).toFloat() else 0f
                Box(Modifier.padding(top = 6.dp).fillMaxWidth().height(3.dp).background(c.line)) {
                    Box(Modifier.fillMaxWidth(f.coerceIn(0f, 1f)).fillMaxHeight().background(c.accent))
                }
            }
        }
        Column(horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(6.dp)) {
            val main = when (d.state) {
                FfiDownloadState.DONE -> "播放"
                FfiDownloadState.RUNNING, FfiDownloadState.QUEUED -> "暂停"
                else -> "继续"
            }
            Text(main, fontSize = 13.sp, color = c.accent, modifier = Modifier.clickable(onClick = onMain).padding(4.dp))
            Text("删除", fontSize = 13.sp, color = c.fg2, modifier = Modifier.clickable(onClick = onDelete).padding(4.dp))
        }
    }
}
