package dev.rubi.app.ui

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.text.InlineTextContent
import androidx.compose.foundation.text.appendInlineContent
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.Placeholder
import androidx.compose.ui.text.PlaceholderVerticalAlign
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.em
import coil3.compose.AsyncImage
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiEmote
import dev.rubi.core.FfiVideoLink

/** The default emote set (`[doge]` → url), loaded once at startup. */
val LocalEmotes = staticCompositionLocalOf<Map<String, String>> { emptyMap() }

/**
 * A `[xxx]` emote token (comments, danmaku). Spaces are allowed: decoration emotes are named
 * like `[香奈美·追寻那道光 应援装扮_啊]`. Only tokens with a known image are replaced.
 */
internal val EMOTE_TOKEN = Regex("""\[[^\[\]\n]{1,40}]""")

/**
 * Text with bilibili emote tokens drawn as inline images. Tokens with no known image
 * stay as text. `extra` adds per-item emotes (comments carry their own map). `links` turns
 * video ids / urls in the text into "▶ title" links handled by `onLink`.
 */
@Composable
fun EmoteText(
    text: String,
    color: Color,
    fontSize: TextUnit,
    modifier: Modifier = Modifier,
    lineHeight: TextUnit = TextUnit.Unspecified,
    maxLines: Int = Int.MAX_VALUE,
    extra: List<FfiEmote> = emptyList(),
    links: List<FfiVideoLink> = emptyList(),
    onLink: (FfiVideoLink) -> Unit = {},
    onTextLayout: (TextLayoutResult) -> Unit = {},
) {
    val global = LocalEmotes.current
    val accent = LocalMetro.current.accent
    val (annotated, inline) = remember(text, global, extra, links) {
        val lookup = if (extra.isEmpty()) global else global + extra.associate { it.text to it.url }
        val inline = mutableMapOf<String, InlineTextContent>()
        // Emote and link spans in text order; a span overlapping an earlier one is skipped.
        val spans = buildList<Pair<IntRange, Any>> {
            for (m in EMOTE_TOKEN.findAll(text)) lookup[m.value]?.let { add(m.range to (m.value to it)) }
            for (l in links) {
                var at = text.indexOf(l.token)
                while (at >= 0) {
                    add(at until at + l.token.length to l)
                    at = text.indexOf(l.token, at + l.token.length)
                }
            }
        }.sortedBy { it.first.first }
        val annotated = buildAnnotatedString {
            var last = 0
            for ((range, what) in spans) {
                if (range.first < last) continue
                append(text, last, range.first)
                when (what) {
                    is FfiVideoLink -> withLink(LinkAnnotation.Clickable(what.token) { onLink(what) }) {
                        withStyle(SpanStyle(color = accent)) { append("▶ ${what.title.ifBlank { what.token }}") }
                    }
                    is Pair<*, *> -> {
                        val token = what.first as String
                        val url = what.second as String
                        appendInlineContent(token, token)
                        inline.getOrPut(token) {
                            InlineTextContent(Placeholder(1.4.em, 1.4.em, PlaceholderVerticalAlign.TextCenter)) {
                                AsyncImage(model = url, contentDescription = token, modifier = Modifier.fillMaxSize())
                            }
                        }
                    }
                }
                last = range.last + 1
            }
            append(text, last, text.length)
        }
        annotated to inline
    }
    Text(
        annotated,
        inlineContent = inline,
        color = color,
        fontSize = fontSize,
        lineHeight = lineHeight,
        maxLines = maxLines,
        overflow = TextOverflow.Ellipsis,
        onTextLayout = onTextLayout,
        modifier = modifier,
    )
}
