package dev.rubi.app.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.google.zxing.BarcodeFormat
import com.google.zxing.common.BitMatrix
import com.google.zxing.qrcode.QRCodeWriter
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiAccount
import dev.rubi.core.FfiFilterLogEntry
import dev.rubi.core.FfiQrState
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext

@Composable
fun MeScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val account by app.account
    var note by remember { mutableStateOf("") }
    var backup by remember { mutableStateOf(false) }
    // Re-read after login / logout (the stack changes); what is known shows meanwhile.
    LaunchedEffect(nav.stack.size) { app.refreshAccount() }

    val list = rememberLazyListState()
    LaunchedEffect(Unit) { nav.reselected.collect { if (it == Tab.Me) list.animateScrollToItem(0) } }
    LazyColumn(Modifier.fillMaxSize(), state = list) {
        item {
            Text("我的", style = MetroType.pivot, color = c.fg, modifier = Modifier.padding(horizontal = 16.dp))
            Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                val a = account
                if (a?.loggedIn == true) {
                    Text(a.name, fontSize = 24.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
                    Meta("mid ${a.mid} · 凭证经 Keystore 加密仅存本机")
                    TextAction("退出登录", onClick = { app.logout() }, dim = true)
                } else {
                    Text("游客", fontSize = 24.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
                    Meta("未登录 · 历史与推荐画像仅保存在本机")
                    AccentButton("登录") { nav.push(Screen.Login) }
                }
            }
        }
        val rows = listOf<Pair<String, () -> Unit>>(
            "历史记录" to { nav.push(Screen.History) },
            "收藏夹" to { nav.push(Screen.FavFolders) },
            "稍后再看" to { nav.push(Screen.WatchLater) },
            "播放列表" to { nav.push(Screen.Playlist) },
            "被过滤内容" to { nav.push(Screen.Filtered) },
            "设置" to { nav.push(Screen.Settings) },
            "备份与迁移" to { backup = true },
        )
        items(rows) { (label, action) ->
            Row(
                Modifier.fillMaxWidth().heightIn(min = 56.dp).clickable(onClick = action).padding(horizontal = 16.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) { Text(label, style = MetroType.section, color = c.fg) }
        }
        item { if (note.isNotEmpty()) Meta(note, Modifier.padding(16.dp)) }
    }
    if (backup) BackupSheet(app, onDismiss = { backup = false }) { note = it; backup = false }
}

@Composable
fun LoginScreen(app: RubiApp, nav: Nav) {
    var tab by remember { mutableStateOf(0) }
    // Every method ends the same way: persist the session (Keystore) and go back.
    val done: (String) -> Unit = { json ->
        app.sessions.save(json)
        nav.pop()
    }
    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "登录")
        Pivot(listOf("扫码", "短信", "密码"), tab, { tab = it }, MetroType.section)
        when (tab) {
            0 -> QrLogin(app, done)
            1 -> SmsLogin(app, done)
            else -> PasswordLogin(app, done)
        }
    }
}

/** QR login; polling runs only while this tab is shown. */
@Composable
private fun QrLogin(app: RubiApp, onLoggedIn: (String) -> Unit) {
    val c = LocalMetro.current
    var matrix by remember { mutableStateOf<BitMatrix?>(null) }
    var status by remember { mutableStateOf("正在获取二维码") }
    var round by remember { mutableStateOf(0) }

    LaunchedEffect(round) {
        val ticket = runCatching { app.core.qrGenerate() }.getOrElse { status = "获取失败：${it.message}"; return@LaunchedEffect }
        matrix = withContext(Dispatchers.Default) { QRCodeWriter().encode(ticket.url, BarcodeFormat.QR_CODE, 0, 0) }
        status = "等待扫码 · 有效期 180s"
        while (true) {
            delay(2000)
            when (val s = runCatching { app.core.qrPoll(ticket.key) }.getOrNull()) {
                FfiQrState.Waiting, null -> {}
                FfiQrState.Scanned -> status = "已扫码，请在手机上确认"
                FfiQrState.Expired -> { status = "二维码已过期"; return@LaunchedEffect }
                is FfiQrState.Success -> {
                    onLoggedIn(s.sessionJson)
                    return@LaunchedEffect
                }
            }
        }
    }

    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
        Box(Modifier.background(Color.White).padding(14.dp)) {
            val m = matrix
            Canvas(Modifier.size(200.dp)) {
                if (m == null) return@Canvas
                val cell = size.width / m.width
                for (y in 0 until m.height) for (x in 0 until m.width) {
                    if (m[x, y]) drawRect(Color.Black, Offset(x * cell, y * cell), Size(cell + 0.5f, cell + 0.5f))
                }
            }
        }
        Text("用手机端 App 扫描二维码", fontSize = 16.sp, color = c.fg)
        Meta("$status · 凭证经 Keystore 加密仅存本机")
        TextAction("刷新二维码", onClick = { round++ })
    }
}

@Composable
fun FilteredScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    var entries by remember { mutableStateOf(emptyList<FfiFilterLogEntry>()) }
    LaunchedEffect(Unit) { entries = withContext(Dispatchers.IO) { runCatching { app.core.filterLog(200u) }.getOrDefault(emptyList()) } }
    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "被过滤内容")
        Meta("规则表 v1 · 最近 ${entries.size} 条", Modifier.padding(horizontal = 16.dp, vertical = 4.dp))
        LazyColumn(Modifier.weight(1f)) {
            items(entries) { e ->
                Column(Modifier.padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text("${e.category} · ${e.ruleId}", style = MetroType.meta, fontWeight = FontWeight.SemiBold, color = c.fg)
                    Text(e.title, fontSize = 15.sp, color = c.fg)
                    Meta("${e.source} · ${e.reason} · ${formatAge(e.at)}")
                }
            }
        }
    }
}
