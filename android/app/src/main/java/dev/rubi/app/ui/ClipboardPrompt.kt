package dev.rubi.app.ui

import dev.rubi.core.VideosService
import dev.rubi.app.data.UiPrefs
import android.content.ClipboardManager
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import coil3.compose.AsyncImage
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext

/** How long the offer stays up untouched. */
private const val OFFER_MS = 8_000L

/** Text Rubi itself copies carries this label; it is never offered back. */
private const val OWN_LABEL = "Rubi"

/**
 * 识别剪贴板链接: each time Rubi comes to the front, a video link on the clipboard (BV / av /
 * b23.tv) is offered as a strip along the bottom: 打开 or ✕. The same text is offered once.
 */
@Composable
fun ClipboardPrompt(videos: VideosService, uiPrefs: UiPrefs, nav: Nav, modifier: Modifier) {
    val c = LocalMetro.current
    val ctx = LocalContext.current
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    var offer by remember { mutableStateOf<FfiVideoCard?>(null) }

    LaunchedEffect(lifecycle) {
        lifecycle.currentStateFlow.collect { state ->
            if (state != Lifecycle.State.RESUMED) return@collect
            // The clipboard can only be read once the window has focus.
            delay(400)
            val clip = runCatching { ctx.getSystemService(ClipboardManager::class.java).primaryClip }.getOrNull() ?: return@collect
            if (clip.description?.label == OWN_LABEL || clip.itemCount == 0) return@collect
            val text = clip.getItemAt(0).coerceToText(ctx)?.toString()?.trim().orEmpty()
            if (text.isEmpty() || text == uiPrefs.lastClipboard) return@collect
            uiPrefs.lastClipboard = text
            offer = withContext(Dispatchers.IO) { videos.videoInText(text.take(2_000)) }
        }
    }
    LaunchedEffect(offer) {
        if (offer != null) {
            delay(OFFER_MS)
            offer = null
        }
    }

    val card = offer ?: return
    Row(
        modifier.padding(start = 12.dp, end = 12.dp, bottom = 68.dp).fillMaxWidth().background(c.bg2).border(1.dp, c.line).padding(10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        AsyncImage(model = card.cover, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.size(width = 72.dp, height = 40.dp).background(c.bg))
        Column(Modifier.weight(1f)) {
            Meta("剪贴板里有一个视频链接")
            Text(card.title, fontSize = 13.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        Box(
            Modifier.height(30.dp).background(c.accent).clickable {
                offer = null
                nav.push(Screen.Player(card.bvid, card.title))
            }.padding(horizontal = 12.dp),
            contentAlignment = Alignment.Center,
        ) { Text("打开", fontSize = 13.sp, color = Color.White) }
        Text("✕", fontSize = 15.sp, color = c.fg2, modifier = Modifier.clickable { offer = null }.padding(6.dp))
    }
}
