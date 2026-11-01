package dev.rubi.app.playback

import android.content.Context
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
object AudioQueue {
    /** The item being heard; null when the queue is idle. */
    val current = mutableStateOf<FfiVideoCard?>(null)
    val playing = mutableStateOf(false)

    /** Set when the notification is tapped; the root opens the 听视频 page and clears it. */
    val pageRequested = mutableStateOf(false)

    /** Within this much of the start, 上一首 goes to the previous item instead of restarting. */
    private const val RESTART_WITHIN_MS = 3_000L

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private var core: RubiCore? = null
    private var appContext: Context? = null

    private val listener = object : Player.Listener {
        override fun onPlaybackStateChanged(state: Int) {
            if (state == Player.STATE_ENDED) move { it.listenNext(false) }
        }

        override fun onIsPlayingChanged(isPlaying: Boolean) {
            playing.value = isPlaying
        }
    }

    /** Starts hearing the list from `card`. */
    fun start(context: Context, core: RubiCore, card: FfiVideoCard) {
        this.core = core
        appContext = context.applicationContext
        val player = PlayerHolder.get(context)
        player.removeListener(listener)
        player.addListener(listener)
        // Shown at once, so the 听视频 page opened next has something to show.
        current.value = card
        scope.launch { runCatching { core.listenFrom(card.bvid) }.onSuccess(::play).onFailure { stop() } }
    }

    /** Switches to `card` from the list on the page. */
    fun jump(card: FfiVideoCard) {
        val c = core ?: return
        scope.launch { runCatching { c.listenJump(card.bvid) }.onSuccess(::play) }
    }

    fun toggle() {
        val player = player() ?: return
        if (player.isPlaying) player.pause() else player.play()
    }

    /** 下一首: always moves on (the core decides where to). */
    fun next() = move { it.listenNext(true) }

    /** 上一首: back to the start when a few seconds in, else the previous item. */
    fun previous() {
        val player = player() ?: return
        if (player.currentPosition > RESTART_WITHIN_MS) player.seekTo(0) else move { it.listenPrevious() }
    }

    fun seekTo(ms: Long) {
        player()?.seekTo(ms)
    }

    /** Position and length of what is heard, in ms. */
    fun progress(): Pair<Long, Long> {
        val p = player() ?: return 0L to 0L
        return p.currentPosition to p.duration.coerceAtLeast(0L)
    }

    fun stop() {
        if (current.value == null) return
        current.value = null
        playing.value = false
        core?.listenStop()
        player()?.let {
            it.removeListener(listener)
            PlayerHolder.clear()
        }
    }

    /** Asks the core for the item to hear next; none means listening is over. */
    private fun move(pick: (RubiCore) -> FfiVideoCard?) {
        val c = core ?: return
        scope.launch {
            val card = runCatching { pick(c) }.getOrNull()
            if (card == null) stop() else play(card)
        }
    }

    private fun play(card: FfiVideoCard) {
        val c = core ?: return
        val ctx = appContext ?: return
        current.value = card
        scope.launch {
            runCatching {
                // Audio streams are the same at any quality; ask for the lowest.
                val playback = c.playback(card.bvid, 16u)
                PlayerHolder.playAudioOnly(ctx, playback, card.title, card.upName, card.cover)
            }.onFailure {
                Log.w("RubiAudio", "cannot play ${card.bvid}", it)
                move { core -> core.listenNext(false) }
            }
        }
    }

    private fun player(): Player? = appContext?.let { PlayerHolder.get(it) }
}
