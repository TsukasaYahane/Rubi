package dev.rubi.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
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
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiChatMessage
import dev.rubi.core.FfiChatSession
import dev.rubi.core.FfiNotice
import dev.rubi.core.FfiNoticeKind
import dev.rubi.core.FfiVideoCard
import dev.rubi.core.RubiException
import kotlinx.coroutines.launch

/** Underlined single-line input (Metro text box). */
@Composable
private fun MetroField(value: String, onChange: (String) -> Unit, hint: String, modifier: Modifier = Modifier, onSubmit: () -> Unit) {
    val c = LocalMetro.current
    Box(modifier.height(44.dp), contentAlignment = Alignment.CenterStart) {
        if (value.isEmpty()) Text(hint, color = c.fg2, fontSize = 16.sp)
        BasicTextField(
            value = value,
            onValueChange = onChange,
            singleLine = true,
            textStyle = TextStyle(color = c.fg, fontSize = 16.sp),
            cursorBrush = SolidColor(c.accent),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
            keyboardActions = KeyboardActions(onAny = { onSubmit() }),
            modifier = Modifier.fillMaxWidth(),
        )
        Box(Modifier.align(Alignment.BottomStart).fillMaxWidth().height(2.dp).background(c.line))
    }
}

@Composable
fun SearchScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var query by remember { mutableStateOf("") }
    var results by remember { mutableStateOf<List<FfiVideoCard>?>(null) }
    var hot by remember { mutableStateOf(emptyList<String>()) }
    var note by remember { mutableStateOf("") }

    LaunchedEffect(Unit) {
        runCatching { app.core.hotWords() }.onSuccess { hot = it.words; note = "已过滤推广热搜词 ${it.hidden} 条" }
    }
    fun search(q: String) {
        query = q
        scope.launch {
            runCatching { app.core.search(q, 1u) }
                .onSuccess { results = it.cards; note = "结果 ${it.cards.size} · 已过滤 ${it.hidden}" }
                .onFailure { note = "搜索失败：${it.message}" }
        }
    }

    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().padding(end = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
            MetroField(query, { query = it }, "搜索视频、UP 主", Modifier.weight(1f)) { if (query.isNotBlank()) search(query) }
        }
        Meta(note, Modifier.padding(horizontal = 16.dp, vertical = 8.dp))
        LazyColumn(Modifier.weight(1f)) {
            val r = results
            if (r == null) {
                item { SectionTitle("热搜") }
                items(hot.withIndex().toList()) { (i, w) ->
                    Row(
                        Modifier.fillMaxWidth().height(48.dp).clickable { search(w) }.padding(horizontal = 16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text("%02d".format(i + 1), style = MetroType.meta.copy(fontSize = 16.sp), color = if (i < 3) c.fg else c.fg2, modifier = Modifier.widthIn(min = 32.dp))
                        Text(w, fontSize = 16.sp, color = c.fg)
                    }
                }
            } else {
                items(r, key = { it.bvid }) { card ->
                    Box(Modifier.padding(horizontal = 16.dp, vertical = 12.dp)) {
                        VideoCardView(card, big = true, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) })
                    }
                }
            }
        }
    }
}

@Composable
fun InboxScreen(app: RubiApp, nav: Nav) {
    val c = LocalMetro.current
    var tab by remember { mutableStateOf(0) }
    var notices by remember { mutableStateOf(emptyList<FfiNotice>()) }
    var sessions by remember { mutableStateOf(emptyList<FfiChatSession>()) }
    var note by remember { mutableStateOf("") }

    LaunchedEffect(tab) {
        note = ""
        runCatching {
            if (tab < 3) {
                val kind = listOf(FfiNoticeKind.REPLY, FfiNoticeKind.AT, FfiNoticeKind.LIKE)[tab]
                notices = app.core.notices(kind, null).items
                note = "仅显示真人互动"
            } else {
                val s = app.core.chatSessions()
                sessions = s.sessions
                note = "已过滤系统推广通知与营销私信 ${s.hidden} 个"
            }
        }.onFailure { note = if (it is RubiException.NotLoggedIn) "登录后可查看消息" else "加载失败：${it.message}" }
    }

    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "消息")
        Pivot(listOf("回复", "@我", "赞", "私信"), tab, { tab = it }, MetroType.section)
        Meta(note, Modifier.padding(horizontal = 16.dp, vertical = 4.dp))
        LazyColumn(Modifier.weight(1f)) {
            if (tab < 3) {
                items(notices, key = { it.id.toString() }) { n ->
                    Column(Modifier.padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text(n.from, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
                        Text(n.text, fontSize = 15.sp, lineHeight = 22.sp, color = c.fg)
                        Text(n.context, fontSize = 12.sp, color = c.fg2)
                        Meta(formatAge(n.at))
                    }
                }
            } else {
                items(sessions, key = { it.talkerMid.toString() }) { s ->
                    Row(
                        Modifier.fillMaxWidth().clickable { nav.push(Screen.Chat(s.talkerMid, s.name)) }.padding(16.dp, 12.dp),
                        verticalAlignment = Alignment.Top,
                    ) {
                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                            Text(s.name, fontSize = 20.sp, fontWeight = if (s.unread > 0u) FontWeight.SemiBold else FontWeight.Light, color = c.fg)
                            Text((if (s.lastFromMe) "我：" else "") + s.lastText, fontSize = 14.sp, color = c.fg2, maxLines = 1)
                        }
                        Column(horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            Meta(formatAge(s.lastAt))
                            if (s.unread > 0u) Tag(s.unread.toString())
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun ChatScreen(app: RubiApp, nav: Nav, mid: ULong, name: String) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var messages by remember { mutableStateOf(emptyList<FfiChatMessage>()) }
    var draft by remember { mutableStateOf("") }
    var note by remember { mutableStateOf("") }

    suspend fun reload() {
        runCatching { app.core.chatMessages(mid) }
            .onSuccess { msgs ->
                messages = msgs
                msgs.lastOrNull()?.let { runCatching { app.core.markRead(mid, it.seqno) } }
            }
            .onFailure { note = "加载失败：${it.message}" }
    }
    LaunchedEffect(mid) { reload() }

    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().height(56.dp).padding(start = 4.dp, end = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
            Text(name, fontSize = 18.sp, fontWeight = FontWeight.SemiBold, color = c.fg, modifier = Modifier.weight(1f))
            Meta("私信")
        }
        LazyColumn(Modifier.weight(1f).padding(horizontal = 16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            items(messages, key = { it.seqno.toString() }) { m ->
                Box(Modifier.fillMaxWidth(), contentAlignment = if (m.fromMe) Alignment.CenterEnd else Alignment.CenterStart) {
                    Text(
                        m.text,
                        fontSize = 15.sp,
                        lineHeight = 22.sp,
                        color = if (m.fromMe) Color.White else c.fg,
                        modifier = Modifier.widthIn(max = 300.dp).background(if (m.fromMe) c.accent else c.bg2).padding(12.dp, 10.dp),
                    )
                }
            }
        }
        if (note.isNotEmpty()) Meta(note, Modifier.padding(horizontal = 16.dp))
        Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
        Row(Modifier.padding(16.dp, 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            val send = {
                val text = draft.trim()
                if (text.isNotEmpty()) scope.launch {
                    runCatching { app.core.sendMessage(mid, text) }
                        .onSuccess { draft = ""; reload() }
                        .onFailure { note = "发送失败：${it.message}" }
                }
                Unit
            }
            MetroField(draft, { draft = it }, "发消息", Modifier.weight(1f), onSubmit = send)
            AccentButton("发送", send)
        }
    }
}
