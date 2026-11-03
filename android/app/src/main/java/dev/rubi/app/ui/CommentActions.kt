package dev.rubi.app.ui

import dev.rubi.core.FfiMention
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.Path
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiCommentArea
import dev.rubi.core.FfiCommentRow

/** What a long press on a comment offers. */
enum class CommentAction { Like, Reply, Copy, Share }

/** The web address of a comment, for 分享: the content's page with the reply anchor. */
fun commentLink(area: FfiCommentArea, rpid: ULong): String = when (area.kind) {
    1u -> "https://www.bilibili.com/video/av${area.oid}#reply$rpid"
    12u -> "https://www.bilibili.com/read/cv${area.oid}#reply$rpid"
    else -> "https://www.bilibili.com/opus/${area.oid}#reply$rpid"
}

/** The sheet a long press opens: 点赞 / 回复 / 复制文字 / 分享, square-cornered rows. */
@Composable
fun CommentMenu(row: FfiCommentRow, liked: Boolean, likes: ULong, onDismiss: () -> Unit, onAction: (CommentAction) -> Unit) {
    val c = LocalMetro.current
    PlayerSheet(side = false, onDismiss = onDismiss) {
        Column(Modifier.padding(bottom = 12.dp)) {
            Text(
                "${row.name}：${row.text}",
                fontSize = 12.sp,
                color = c.fg2,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.padding(start = 20.dp, end = 20.dp, top = 14.dp, bottom = 6.dp),
            )
            Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
            MenuRow(if (liked) "取消点赞" else "点赞", glyph = MenuGlyph.Heart, glyphColor = if (liked) c.accent else c.fg, trailing = "$likes") {
                onAction(CommentAction.Like)
            }
            MenuRow("回复", glyph = MenuGlyph.Reply) { onAction(CommentAction.Reply) }
            MenuRow("复制文字", glyph = MenuGlyph.Copy) { onAction(CommentAction.Copy) }
            MenuRow("分享（复制链接）", glyph = MenuGlyph.Share) { onAction(CommentAction.Share) }
        }
    }
}

private enum class MenuGlyph { Heart, Reply, Copy, Share }

/**
 * The menu's icons, drawn on a 24-unit grid with square ends and mitred corners (system
 * symbols turn into colour emoji or missing glyphs on some phones).
 */
@Composable
private fun MenuIcon(glyph: MenuGlyph, color: Color) {
    Canvas(Modifier.size(22.dp)) {
        val u = size.width / 24f
        fun path(vararg pts: Pair<Float, Float>, close: Boolean = false) = Path().apply {
            pts.forEachIndexed { i, (x, y) -> if (i == 0) moveTo(x * u, y * u) else lineTo(x * u, y * u) }
            if (close) close()
        }
        val stroke = Stroke(width = 2f * u, cap = StrokeCap.Square, join = StrokeJoin.Miter)
        when (glyph) {
            MenuGlyph.Heart -> drawPath(
                path(12f to 20f, 3f to 11f, 3f to 6f, 6f to 3f, 9f to 3f, 12f to 6f, 15f to 3f, 18f to 3f, 21f to 6f, 21f to 11f, close = true),
                color,
            )
            MenuGlyph.Reply -> {
                drawPath(path(9f to 5f, 3f to 11f, 9f to 17f), color, style = stroke)
                drawPath(path(3f to 11f, 15f to 11f, 21f to 17f, 21f to 20f), color, style = stroke)
            }
            MenuGlyph.Copy -> {
                drawPath(path(8f to 8f, 21f to 8f, 21f to 21f, 8f to 21f, close = true), color, style = stroke)
                drawPath(path(16f to 4f, 3f to 4f, 3f to 16f), color, style = stroke)
            }
            MenuGlyph.Share -> {
                drawPath(path(11f to 4f, 4f to 4f, 4f to 20f, 20f to 20f, 20f to 13f), color, style = stroke)
                drawPath(path(12f to 12f, 20f to 4f), color, style = stroke)
                drawPath(path(14f to 4f, 20f to 4f, 20f to 10f), color, style = stroke)
            }
        }
    }
}

@Composable
private fun MenuRow(label: String, glyph: MenuGlyph, glyphColor: Color = LocalMetro.current.fg, trailing: String = "", onClick: () -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().heightIn(min = 52.dp).clickable(onClick = onClick).padding(horizontal = 20.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        MenuIcon(glyph, glyphColor)
        Text(label, fontSize = 16.sp, color = c.fg, modifier = Modifier.weight(1f))
        if (trailing.isNotEmpty()) Text(trailing, fontSize = 13.sp, color = c.fg2)
    }
}

/**
 * The reply box: `回复 @名字`, a square field and 发送. An empty reply is refused on the spot;
 * `error` shows what the last send ran into.
 */
@Composable
fun ReplyComposer(to: String, sending: Boolean, error: String?, mentions: Mentions, onDismiss: () -> Unit, onSend: (String, List<FfiMention>) -> Unit) {
    val c = LocalMetro.current
    var field by remember { mutableStateOf(TextFieldValue("")) }
    val text = field.text
    var hint by remember { mutableStateOf<String?>(null) }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { runCatching { focus.requestFocus() } }
    PlayerSheet(side = false, onDismiss = onDismiss) {
        Column(Modifier.imePadding().padding(12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text("回复 @$to", fontSize = 12.sp, color = c.fg2)
            MentionPicker(field, mentions) { field = it }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Box(
                    Modifier.weight(1f).heightIn(min = 40.dp).border(1.dp, c.line).padding(horizontal = 8.dp, vertical = 10.dp),
                    contentAlignment = Alignment.CenterStart,
                ) {
                    if (text.isEmpty()) Text("说点什么", fontSize = 14.sp, color = c.fg2)
                    BasicTextField(
                        value = field,
                        onValueChange = { field = it; hint = null },
                        textStyle = TextStyle(color = c.fg, fontSize = 14.sp),
                        cursorBrush = SolidColor(c.accent),
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                        keyboardActions = KeyboardActions(onSend = { if (text.isBlank()) hint = "先写点内容" else onSend(text, mentions.picked) }),
                        modifier = Modifier.fillMaxWidth().focusRequester(focus),
                    )
                }
                Box(
                    Modifier.height(40.dp).background(if (sending) c.line else c.accent).clickable(enabled = !sending) {
                        if (text.isBlank()) hint = "先写点内容" else onSend(text, mentions.picked)
                    }.padding(horizontal = 16.dp),
                    contentAlignment = Alignment.Center,
                ) { Text(if (sending) "发送中" else "发送", color = Color.White, fontSize = 14.sp) }
            }
            Text(hint ?: error.orEmpty(), fontSize = 12.sp, color = Color(0xFFFF6B6B), modifier = Modifier.heightIn(min = 16.dp))
        }
    }
}

/** Longest comment B站 takes. */
const val MAX_COMMENT = 1000

/** What a comment area belongs to, for 「评论视频 / 专栏 / 动态」. */
fun areaNoun(area: FfiCommentArea): String = when (area.kind) {
    1u -> "视频"
    12u -> "专栏"
    else -> "动态"
}

/** The entry at the top of a comment area: a square field that opens the composer. */
@Composable
fun PostCommentEntry(onClick: () -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().padding(vertical = 8.dp).border(1.dp, c.line).clickable(onClick = onClick).padding(10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Box(Modifier.size(28.dp).background(c.accent))
        Text("发一条友善的评论", fontSize = 14.sp, color = c.fg2)
    }
}

/**
 * Writing a new comment (a new floor, not a reply): several lines, a count against the
 * limit, 取消 and 发表. Empty or too long text is stopped here.
 */
@Composable
fun PostComposer(noun: String, sending: Boolean, error: String?, mentions: Mentions, onDismiss: () -> Unit, onSend: (String, List<FfiMention>) -> Unit) {
    val c = LocalMetro.current
    var field by remember { mutableStateOf(TextFieldValue("")) }
    val text = field.text
    var hint by remember { mutableStateOf<String?>(null) }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { runCatching { focus.requestFocus() } }
    fun submit() {
        hint = when {
            text.isBlank() -> "先写点内容"
            text.length > MAX_COMMENT -> "最多 $MAX_COMMENT 字"
            else -> null
        }
        if (hint == null) onSend(text, mentions.picked)
    }
    PlayerSheet(side = false, onDismiss = onDismiss) {
        Column(Modifier.imePadding().padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("评论$noun", fontSize = 12.sp, color = c.fg2)
            MentionPicker(field, mentions) { field = it }
            Box(Modifier.fillMaxWidth().heightIn(min = 88.dp).border(1.dp, c.line).padding(8.dp)) {
                if (text.isEmpty()) Text("发一条友善的评论", fontSize = 14.sp, color = c.fg2)
                BasicTextField(
                    value = field,
                    onValueChange = { field = it; hint = null },
                    textStyle = TextStyle(color = c.fg, fontSize = 14.sp),
                    cursorBrush = SolidColor(c.accent),
                    modifier = Modifier.fillMaxWidth().focusRequester(focus),
                )
            }
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                Text(
                    "@",
                    fontSize = 16.sp,
                    color = c.fg,
                    modifier = Modifier.clickable {
                        val t = field.text + "@"
                        field = TextFieldValue(t, TextRange(t.length))
                    }.padding(horizontal = 6.dp, vertical = 4.dp),
                )
                Text(
                    "${text.length} / $MAX_COMMENT",
                    fontSize = 12.sp,
                    color = if (text.length > MAX_COMMENT) Color(0xFFFF6B6B) else c.fg2,
                    modifier = Modifier.weight(1f),
                )
                Text("取消", fontSize = 14.sp, color = c.fg2, modifier = Modifier.clickable(onClick = onDismiss).padding(8.dp))
                Box(
                    Modifier.height(36.dp).background(if (sending) c.line else c.accent).clickable(enabled = !sending, onClick = ::submit).padding(horizontal = 18.dp),
                    contentAlignment = Alignment.Center,
                ) { Text(if (sending) "发表中" else "发表", color = Color.White, fontSize = 14.sp) }
            }
            Text(hint ?: error.orEmpty(), fontSize = 12.sp, color = Color(0xFFFF6B6B), modifier = Modifier.heightIn(min = 16.dp))
        }
    }
}

/**
 * @ 提到: who can be picked (followed UPs and friends, read on the first @) and who has been
 * picked in this composer.
 */
class Mentions(private val load: suspend () -> List<FfiMention>) {
    var all by mutableStateOf<List<FfiMention>?>(null)
        private set
    val picked = mutableListOf<FfiMention>()

    suspend fun ensure() {
        if (all == null) all = runCatching { load() }.getOrDefault(emptyList())
    }
}

/** How many names the @ list shows at once. */
private const val MENTION_ROWS = 6

/**
 * While the word before the cursor starts with @, a list of matching people; picking one puts
 * `@名字 ` in its place and remembers them for the send.
 */
@Composable
fun MentionPicker(field: TextFieldValue, mentions: Mentions, onPick: (TextFieldValue) -> Unit) {
    val c = LocalMetro.current
    val before = field.text.substring(0, field.selection.start.coerceIn(0, field.text.length))
    val typed = Regex("@([^\\s@]*)$").find(before)
    LaunchedEffect(typed != null) { if (typed != null) mentions.ensure() }
    if (typed == null) return
    val word = typed.groupValues[1]
    val all = mentions.all
    Column(Modifier.fillMaxWidth().border(1.dp, c.line)) {
        when {
            all == null -> Meta("读取关注列表…", Modifier.padding(8.dp))
            else -> {
                val hits = all.filter { word.isEmpty() || it.name.contains(word, ignoreCase = true) }.take(MENTION_ROWS)
                if (hits.isEmpty()) Meta(if (all.isEmpty()) "登录后可以 @ 关注的人" else "没有找到「$word」", Modifier.padding(8.dp))
                hits.forEach { m ->
                    Text(
                        m.name,
                        fontSize = 14.sp,
                        color = c.fg,
                        maxLines = 1,
                        modifier = Modifier.fillMaxWidth().clickable {
                            val start = typed.range.first
                            val replaced = field.text.substring(0, start) + "@${m.name} " + field.text.substring(before.length)
                            if (mentions.picked.none { it.mid == m.mid }) mentions.picked.add(m)
                            onPick(TextFieldValue(replaced, TextRange(start + m.name.length + 2)))
                        }.padding(horizontal = 10.dp, vertical = 8.dp),
                    )
                }
            }
        }
    }
}
