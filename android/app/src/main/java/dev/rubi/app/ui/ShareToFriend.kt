package dev.rubi.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.RubiException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** Someone to send to: their mid, name, and which list they came from. */
private data class Friend(val mid: ULong, val name: String, val from: String)

/**
 * 发给好友: tick people from recent chats and followed UPs (searchable), then send the video as
 * B站's own share card, one message each.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ShareToFriendSheet(app: RubiApp, bvid: String, title: String, onDismiss: () -> Unit, done: (String) -> Unit) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var people by remember { mutableStateOf<List<Friend>?>(null) }
    var problem by remember { mutableStateOf<String?>(null) }
    var query by remember { mutableStateOf("") }
    var sending by remember { mutableStateOf(false) }
    val picked = remember { mutableStateListOf<ULong>() }
    LaunchedEffect(Unit) {
        people = runCatching {
            withContext(Dispatchers.IO) {
                val chats = app.core.chatSessions().sessions.map { Friend(it.talkerMid, it.name, "最近聊天") }
                val follows = runCatching { app.core.allFollowings().ups }.getOrDefault(emptyList())
                    .filter { f -> chats.none { it.mid == f.mid } }
                    .map { Friend(it.mid, it.name, "关注") }
                chats + follows
            }
        }.onFailure { problem = if (it is RubiException.NotLoggedIn) "登录后才能发给好友" else "读取失败：${errorText(it)}" }.getOrNull()
    }
    ModalBottomSheet(onDismissRequest = onDismiss, containerColor = c.bg2, shape = RectangleShape, dragHandle = null) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("发给好友", style = MetroType.section, color = c.fg)
            Text(title, fontSize = 13.sp, color = c.fg2, maxLines = 1, overflow = TextOverflow.Ellipsis)
            NameField(query, "搜索好友或关注的人", Modifier.fillMaxWidth(), max = 30) { query = it }
            val list = people
            when {
                problem != null -> Meta(problem!!, color = LiveRed)
                list == null -> Meta("读取聊天与关注…")
                else -> {
                    val shown = list.filter { query.isBlank() || it.name.contains(query.trim(), ignoreCase = true) }
                    LazyColumn(Modifier.fillMaxWidth().heightIn(max = 360.dp)) {
                        items(shown, key = { it.mid.toString() }) { f ->
                            val on = f.mid in picked
                            Row(
                                Modifier.fillMaxWidth().clickable { if (on) picked.remove(f.mid) else picked.add(f.mid) }.padding(vertical = 9.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Column(Modifier.weight(1f)) {
                                    Text(f.name, fontSize = 15.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                    Meta(f.from)
                                }
                                Box(Modifier.size(18.dp).border(2.dp, c.fg).padding(3.dp).background(if (on) c.accent else Color.Transparent))
                            }
                        }
                    }
                }
            }
            Row(Modifier.padding(top = 4.dp, bottom = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Meta(if (sending) "发送中…" else "已选 ${picked.size} 人", Modifier.weight(1f))
                TextAction("取消", onClick = onDismiss, dim = true)
                SmallButton("发送", if (picked.isEmpty() || sending) c.fg2 else c.accent) {
                    if (picked.isEmpty() || sending) return@SmallButton
                    sending = true
                    scope.launch {
                        val targets = picked.toList()
                        var failed: Throwable? = null
                        var sent = 0
                        var asLink = false
                        for (mid in targets) {
                            runCatching { withContext(Dispatchers.IO) { app.core.shareVideoTo(mid, bvid) } }
                                .onSuccess { card -> sent++; if (!card) asLink = true }
                                .onFailure { failed = it }
                        }
                        sending = false
                        val f = failed
                        if (f == null) {
                            // B站 refuses share cards from outside its own clients: the link went instead.
                            done("已发给 $sent 位好友" + if (asLink) "（B 站不接受第三方发送视频卡片，已以标题和链接发送）" else "")
                            onDismiss()
                        } else {
                            problem = if (f is RubiException.NotLoggedIn) "登录后才能发给好友" else "已发 $sent 人，有 ${targets.size - sent} 人发送失败：${errorText(f)}"
                        }
                    }
                }
            }
        }
    }
}
