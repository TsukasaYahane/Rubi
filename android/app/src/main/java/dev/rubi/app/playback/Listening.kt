package dev.rubi.app.playback

import android.util.Log
import androidx.compose.runtime.mutableStateOf
import androidx.media3.common.Player
import dev.rubi.core.FfiVideoCard
import dev.rubi.core.RubiCore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

/**
 * 听视频 on this device: plays what the core's listening queue picks, audio only, on the shared
 * player (so the media session, notification and background play all apply). Which item comes
 * next or before is the core's call; this object only plays it. Opening a video page stops it:
 * the two would fight over the one player.
 */
class Listening(private val player: MainPlayer, private val core: RubiCore) {
    /** The item being heard; null when the queue is idle. */
    val current = mutableStateOf<FfiVideoCard?>(null)
    val playing = mutableStateOf(false)

    /** Set when the notification is tapped; the root opens the 听视频 page and clears it. */
    val pageRequested = mutableStateOf(false)

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)

    private val listener = object : Player.Listener {
        override fun onPlaybackStateChanged(state: Int) {
            if (state == Player.STATE_ENDED) move { core.listenNext(false) }
        }

        override fun onIsPlayingChanged(isPlaying: Boolean) {
            playing.value = isPlaying
        }
    }

    /** Starts hearing the list from `card`. */
    fun start(card: FfiVideoCard) {
        val exo = player.get()
        exo.removeListener(listener)
        exo.addListener(listener)
        // Shown at once, so the 听视频 page opened next has something to show.
        current.value = card
        scope.launch { runCatching { core.listenFrom(card.bvid) }.onSuccess(::play).onFailure { stop() } }
    }

    /** Switches to `card` from the list on the page. */
    fun jump(card: FfiVideoCard) {
        scope.launch { runCatching { core.listenJump(card.bvid) }.onSuccess(::play) }
    }

    fun toggle() {
        val exo = player.get()
        if (exo.isPlaying) exo.pause() else exo.play()
    }

    /** 下一首: always moves on (the core decides where to). */
    fun next() = move { core.listenNext(true) }

    /** 上一首: back to the start when a few seconds in, else the previous item. */
    fun previous() {
        val exo = player.get()
        if (exo.currentPosition > RESTART_WITHIN_MS) exo.seekTo(0) else move { core.listenPrevious() }
    }

    fun seekTo(ms: Long) {
        player.get().seekTo(ms)
    }

    /** Position and length of what is heard, in ms. */
    fun progress(): Pair<Long, Long> {
        val exo = player.get()
        return exo.currentPosition to exo.duration.coerceAtLeast(0L)
    }

    fun stop() {
        if (current.value == null) return
        current.value = null
        playing.value = false
        core.listenStop()
        player.get().removeListener(listener)
        player.clear()
    }

    /** Asks the core for the item to hear next; none means listening is over. */
    private fun move(pick: () -> FfiVideoCard?) {
        scope.launch {
            val card = runCatching { pick() }.getOrNull()
            if (card == null) stop() else play(card)
        }
    }

    private fun play(card: FfiVideoCard) {
        current.value = card
        scope.launch {
            runCatching {
                // Audio streams are the same at any quality; ask for the lowest.
                val playback = core.playback(card.bvid, 16u)
                player.playAudioOnly(playback, card.title, card.upName, card.cover)
            }.onFailure {
                Log.w("RubiAudio", "cannot play ${card.bvid}", it)
                move { core.listenNext(false) }
            }
        }
    }
}

/** Within this much of the start, 上一首 goes to the previous item instead of restarting. */
private const val RESTART_WITHIN_MS = 3_000L
