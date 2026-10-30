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
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.em
import coil3.compose.AsyncImage
import dev.rubi.core.FfiEmote

/** The default emote set (`[doge]` → url), loaded once at startup. */
val LocalEmotes = staticCompositionLocalOf<Map<String, String>> { emptyMap() }

private val TOKEN = Regex("""\[[^\[\]\s]{1,24}]""")

/**
 * Text with bilibili emote tokens drawn as inline images. Tokens with no known image
 * stay as text. `extra` adds per-item emotes (comments carry their own map).
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
) {
    val global = LocalEmotes.current
    val (annotated, inline) = remember(text, global, extra) {
        val lookup = if (extra.isEmpty()) global else global + extra.associate { it.text to it.url }
        val inline = mutableMapOf<String, InlineTextContent>()
        val annotated = buildAnnotatedString {
            var last = 0
            for (m in TOKEN.findAll(text)) {
                val url = lookup[m.value] ?: continue
                append(text, last, m.range.first)
                appendInlineContent(m.value, m.value)
                inline.getOrPut(m.value) {
                    InlineTextContent(Placeholder(1.4.em, 1.4.em, PlaceholderVerticalAlign.TextCenter)) {
                        AsyncImage(model = url, contentDescription = m.value, modifier = Modifier.fillMaxSize())
                    }
                }
                last = m.range.last + 1
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
        modifier = modifier,
    )
}
