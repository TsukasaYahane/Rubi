package dev.rubi.app.ui

import android.view.LayoutInflater
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.media3.common.Player
import androidx.media3.ui.PlayerView
import dev.rubi.app.R
import dev.rubi.app.RubiApp
import dev.rubi.app.data.DanmakuDisplay
import dev.rubi.app.data.PreviewPrefs
import dev.rubi.app.playback.PREVIEW_QN
import dev.rubi.app.playback.PreviewPlayer
import dev.rubi.core.FfiDanmakuSet
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collectLatest

/** How long the list must rest before the centred card starts its preview. */
private const val SETTLE_MS = 600L

/**
 * The bvid of the card to preview: the one nearest the vertical centre of `grid`, once
 * scrolling has rested for a moment. Null while scrolling or when previews are off.
 */
@Composable
fun rememberPreviewTarget(grid: LazyGridState, enabled: Boolean): String? {
    val target by produceState<String?>(null, grid, enabled) {
        snapshotFlow {
            if (!enabled || grid.isScrollInProgress) {
                null
            } else {
                val info = grid.layoutInfo
                val centre = (info.viewportStartOffset + info.viewportEndOffset) / 2
                // The card whose middle is nearest the screen's middle; the centre line may
                // well fall in the gap between two cards.
                info.visibleItemsInfo
                    .filter { it.key is String }
                    .minByOrNull { kotlin.math.abs(it.offset.y + it.size.height / 2 - centre) }
                    ?.key as? String
            }
        }.collectLatest { bvid ->
            if (bvid == null) {
                value = null
            } else {
                delay(SETTLE_MS)
                value = bvid
            }
        }
    }
    return target
}

/**
 * The preview drawn over a card's cover: muted (or at the set volume), low quality, looping,
 * shown only once its first frame is out, so the cover stays until then. Stops when the card
 * leaves the centre, the list scrolls, the page closes or the app goes to the background.
 */
@Composable
fun FeedPreview(app: RubiApp, card: FfiVideoCard, prefs: PreviewPrefs) {
    val ctx = LocalContext.current
    val player = remember { PreviewPlayer.get(ctx) }
    var ready by remember { mutableStateOf(false) }
    var danmaku by remember { mutableStateOf<FfiDanmakuSet?>(null) }
    // Bumped when the app returns to the foreground, to start the preview again.
    var round by remember { mutableIntStateOf(0) }

    val lifecycle = LocalLifecycleOwner.current.lifecycle
    DisposableEffect(lifecycle) {
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_STOP -> { ready = false; PreviewPlayer.stop() }
                Lifecycle.Event.ON_START -> round++
                else -> {}
            }
        }
        lifecycle.addObserver(observer)
        onDispose { lifecycle.removeObserver(observer) }
    }
    DisposableEffect(card.bvid) {
        val listener = object : Player.Listener {
            override fun onRenderedFirstFrame() { ready = true }
        }
        player.addListener(listener)
        onDispose {
            player.removeListener(listener)
            PreviewPlayer.stop()
        }
    }
    LaunchedEffect(card.bvid, round) {
        ready = false
        runCatching {
            val playback = app.core.playback(card.bvid, PREVIEW_QN)
            PreviewPlayer.play(ctx, playback, prefs.volume)
            if (prefs.danmaku) {
                val durationMs = maxOf(card.durationS.toULong() * 1000u, playback.durationMs).toUInt()
                danmaku = app.core.danmakuSession(playback.cid, durationMs).load(listOf(1u))
            }
        }
    }

    AndroidView(
        factory = {
            (LayoutInflater.from(it).inflate(R.layout.player_view, null) as PlayerView).apply {
                useController = false
                this.player = player
            }
        },
        // Detach so the player never keeps rendering into a view that is gone.
        onRelease = { it.player = null },
        modifier = Modifier.fillMaxSize().alpha(if (ready) 1f else 0f),
    )
    val set = danmaku
    if (ready && set != null) {
        val display = remember { DanmakuDisplay.load(app.core) }
        DanmakuLayer(app.core, set, display, Modifier.fillMaxSize(), source = player)
    }
}
