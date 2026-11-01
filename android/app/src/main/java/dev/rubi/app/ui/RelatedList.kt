package dev.rubi.app.ui

import android.os.SystemClock
import android.util.Log
import android.content.Intent
import android.net.Uri
import android.view.LayoutInflater
import android.view.View
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.lazy.rememberLazyListState
import dev.rubi.core.FfiVideoDetail
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.layout.boundsInWindow
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.ui.PlayerView
import dev.rubi.app.R
import dev.rubi.app.RubiApp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import dev.rubi.app.data.DanmakuDisplay
import dev.rubi.app.data.PlaybackPrefs
import dev.rubi.app.playback.AudioQueue
import dev.rubi.app.playback.Pip
import dev.rubi.app.playback.PlayerHolder
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiCommentRef
import dev.rubi.core.FfiCommentRow
import dev.rubi.core.FfiCommentThread
import dev.rubi.core.FfiDanmakuSet
import dev.rubi.core.FfiPlayback
import dev.rubi.core.FfiVideoLink
import dev.rubi.core.FfiStein
import dev.rubi.core.FfiSteinNode
import dev.rubi.core.FfiSignal
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/** 推荐 under the player: related videos, or the local pool when that fails. */
@Composable
internal fun RelatedList(app: RubiApp, nav: Nav, current: String) {
    val c = LocalMetro.current
    val cards = remember { mutableStateListOf<FfiVideoCard>() }
    var note by remember { mutableStateOf("") }
    LaunchedEffect(current) {
        runCatching { app.core.related(current) }
            .onSuccess { list -> cards.addAll(list.filter { it.bvid != current }) }
            .onFailure { e ->
                // Fall back to the local pool so the tab is never empty.
                note = "相关推荐加载失败，显示本地推荐：${e.message}"
                val page = withContext(Dispatchers.IO) { runCatching { app.core.nextPage(6u) }.getOrDefault(emptyList()) }
                cards.addAll(page.filter { it.bvid != current })
            }
    }
    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(24.dp)) {
        if (note.isNotEmpty()) Text(note, color = c.fg2, fontSize = 12.sp)
        cards.forEach { card -> VideoCardView(card, big = false, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) }) }
    }
}
