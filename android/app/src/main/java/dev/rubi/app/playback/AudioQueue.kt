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
 * 听视频: plays the play list as audio only, one item after another by the list's mode, on the
 * shared player (so the media session, notification and background play all apply). Opening a
 * video page stops it: the two would fight over the one player.
 */
object AudioQueue {
    /** The item being heard; null when the queue is idle. */
    val current = mutableStateOf<FfiVideoCard?>(null)
    val playing = mutableStateOf(false)

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private var core: RubiCore? = null
    private var appContext: Context? = null

    private val listener = object : Player.Listener {
        override fun onPlaybackStateChanged(state: Int) {
            if (state == Player.STATE_ENDED) next()
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
        play(card)
    }

    fun toggle() {
        val ctx = appContext ?: return
        val player = PlayerHolder.get(ctx)
        if (player.isPlaying) player.pause() else player.play()
    }

    /** Moves on by the list's mode; stops at the end of a sequential list. */
    fun next() {
        val c = core ?: return
        val now = current.value ?: return
        scope.launch {
            val following = runCatching { c.playlistNext(now.bvid) }.getOrNull()
            if (following == null) stop() else play(following)
        }
    }

    fun stop() {
        val ctx = appContext
        if (current.value == null) return
        current.value = null
        playing.value = false
        if (ctx != null) {
            PlayerHolder.get(ctx).removeListener(listener)
            PlayerHolder.clear()
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
                PlayerHolder.playAudioOnly(ctx, playback, card.title)
            }.onFailure {
                Log.w("RubiAudio", "cannot play ${card.bvid}", it)
                next()
            }
        }
    }
}
