package dev.rubi.app.ui

import androidx.compose.runtime.rememberUpdatedState
import dev.rubi.app.playback.LocalMedia
import android.os.SystemClock
import android.util.Log
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.media3.common.Player
import dev.rubi.app.RubiApp
import dev.rubi.app.data.PlaybackPrefs
import dev.rubi.core.FfiDanmakuSet
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

private const val TAG = "RubiPlayer"

/** Seconds between two 正在播放 heartbeats. */
private const val HEARTBEAT_S = 15

/**
 * What the video page means for the shared player while it is open: it may go into
 * picture-in-picture, it takes the player over from 听视频, playback stops when the page is
 * left, and going to the background pauses unless 后台播放 is on (read each time, so the toggle
 * applies at once).
 */
@Composable
fun PlayerPageLifecycle(app: RubiApp) {
    val media = LocalMedia.current
    val ctx = LocalContext.current
    val activity = remember(ctx) { ctx.activity() }
    DisposableEffect(Unit) {
        media.pip.onPlayer = true
        media.pip.autoEnter = PlaybackPrefs.autoPip(app.settings)
        activity?.let(media.pip::sync)
        onDispose {
            media.pip.onPlayer = false
            activity?.let(media.pip::sync)
            media.player.clear()
        }
    }
    LaunchedEffect(Unit) { media.listening.stop() }
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    DisposableEffect(lifecycle) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_STOP && !PlaybackPrefs.backgroundPlay(app.settings)) {
                media.player.get().pause()
            }
        }
        lifecycle.addObserver(observer)
        onDispose { lifecycle.removeObserver(observer) }
    }
}

/**
 * 历史记录: a heartbeat when part `cid` starts, every 15 s while it plays, and when it is left
 * (another branch or the page closing). Positions are sampled every second so the last report
 * carries this part's own progress.
 */
@Composable
fun ReportWatching(app: RubiApp, bvid: String, cid: ULong?) {
    val media = LocalMedia.current
    val watched = remember { longArrayOf(0L, 0L) } // played s, real s
    LaunchedEffect(cid) {
        val part = cid ?: return@LaunchedEffect
        val player = media.player.get()
        val started = SystemClock.elapsedRealtime()
        watched[0] = 0L
        watched[1] = 0L
        runCatching { app.core.reportWatch(bvid, part, 0u, 0u, 1u) }
        var tick = 0
        while (true) {
            delay(1_000)
            if (player.isPlaying) watched[0] = player.currentPosition / 1000
            watched[1] = (SystemClock.elapsedRealtime() - started) / 1000
            if (++tick % HEARTBEAT_S == 0 && player.isPlaying) {
                runCatching { app.core.reportWatch(bvid, part, watched[0].toULong(), watched[1].toULong(), 0u) }
            }
        }
    }
    DisposableEffect(cid) {
        val part = cid
        onDispose {
            if (part != null) {
                val played = watched[0].toULong()
                val real = watched[1].toULong()
                app.scope.launch { runCatching { app.core.reportWatch(bvid, part, played, real, 2u) } }
            }
        }
    }
}

/** Calls `onEnded` each time playback reaches the end, while `enabled`. */
@Composable
fun OnPlaybackEnded(enabled: Boolean, onEnded: () -> Unit) {
    val media = LocalMedia.current
    val latest = rememberUpdatedState(onEnded)
    DisposableEffect(enabled) {
        if (!enabled) return@DisposableEffect onDispose {}
        val player = media.player.get()
        val listener = object : Player.Listener {
            override fun onPlaybackStateChanged(state: Int) {
                if (state == Player.STATE_ENDED) latest.value()
            }
        }
        player.addListener(listener)
        onDispose { player.removeListener(listener) }
    }
}

/** Opened from the play list: when the video ends, the list's next one (by its mode) follows. */
@Composable
fun ContinuePlaylistOnEnd(app: RubiApp, nav: Nav, bvid: String, enabled: Boolean) {
    val media = LocalMedia.current
    val scope = rememberCoroutineScope()
    DisposableEffect(enabled) {
        if (!enabled) return@DisposableEffect onDispose {}
        val player = media.player.get()
        val listener = object : Player.Listener {
            override fun onPlaybackStateChanged(state: Int) {
                if (state != Player.STATE_ENDED) return
                scope.launch {
                    val next = offMain { runCatching { app.core.playlistNext(bvid) }.getOrNull() } ?: return@launch
                    if (next.bvid == bvid) {
                        player.seekTo(0)
                        player.play()
                    } else {
                        nav.replaceTop(Screen.Player(next.bvid, next.title, fromPlaylist = true))
                    }
                }
            }
        }
        player.addListener(listener)
        onDispose { player.removeListener(listener) }
    }
}

/**
 * Loads a part's danmaku segment by segment: the one at `positionMs` first, then the rest from
 * nearest to farthest, reporting the set so far after each batch. A failure reports a null set
 * and what went wrong (and whether some danmaku were already shown).
 */
suspend fun loadDanmakuNearFirst(
    app: RubiApp,
    cid: ULong,
    durationMs: UInt,
    positionMs: Long,
    onUpdate: (FfiDanmakuSet?, String?) -> Unit,
) {
    var shown = false
    runCatching {
        // Closed when done (or cancelled): the session holds every loaded danmaku in the core,
        // and left to the garbage collector each video entered again kept another copy.
        app.core.danmakuSession(cid, durationMs).use { session ->
            val total = session.segmentCount()
            val here = session.segmentAt(positionMs.toUInt())
            val order = (1u..total).sortedBy { kotlin.math.abs(it.toInt() - here.toInt()) }
            for (batch in listOf(order.take(1)) + order.drop(1).chunked(3)) {
                onUpdate(session.load(batch), null)
                shown = true
            }
        }
        Log.i(TAG, "danmaku loaded")
    }.onFailure { e ->
        onUpdate(null, if (shown) "部分弹幕加载失败：${e.message}" else "弹幕加载失败：${e.message}")
        Log.w(TAG, "danmaku load failed", e)
    }
}
