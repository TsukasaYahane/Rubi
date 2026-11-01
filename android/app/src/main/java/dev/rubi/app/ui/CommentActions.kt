package dev.rubi.app.ui

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
fun ReplyComposer(to: String, sending: Boolean, error: String?, onDismiss: () -> Unit, onSend: (String) -> Unit) {
    val c = LocalMetro.current
    var text by remember { mutableStateOf("") }
    var hint by remember { mutableStateOf<String?>(null) }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { runCatching { focus.requestFocus() } }
    PlayerSheet(side = false, onDismiss = onDismiss) {
        Column(Modifier.imePadding().padding(12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text("回复 @$to", fontSize = 12.sp, color = c.fg2)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Box(
                    Modifier.weight(1f).heightIn(min = 40.dp).border(1.dp, c.line).padding(horizontal = 8.dp, vertical = 10.dp),
                    contentAlignment = Alignment.CenterStart,
                ) {
                    if (text.isEmpty()) Text("说点什么", fontSize = 14.sp, color = c.fg2)
                    BasicTextField(
                        value = text,
                        onValueChange = { text = it; hint = null },
                        textStyle = TextStyle(color = c.fg, fontSize = 14.sp),
                        cursorBrush = SolidColor(c.accent),
                        keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                        keyboardActions = KeyboardActions(onSend = { if (text.isBlank()) hint = "先写点内容" else onSend(text) }),
                        modifier = Modifier.fillMaxWidth().focusRequester(focus),
                    )
                }
                Box(
                    Modifier.height(40.dp).background(if (sending) c.line else c.accent).clickable(enabled = !sending) {
                        if (text.isBlank()) hint = "先写点内容" else onSend(text)
                    }.padding(horizontal = 16.dp),
                    contentAlignment = Alignment.Center,
                ) { Text(if (sending) "发送中" else "发送", color = Color.White, fontSize = 14.sp) }
            }
            Text(hint ?: error.orEmpty(), fontSize = 12.sp, color = Color(0xFFFF6B6B), modifier = Modifier.heightIn(min = 16.dp))
        }
    }
}
