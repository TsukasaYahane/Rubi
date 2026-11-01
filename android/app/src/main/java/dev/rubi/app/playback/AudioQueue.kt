package dev.rubi.app.playback

import android.content.Context
import android.util.Log
import androidx.compose.runtime.mutableStateOf
import androidx.media3.common.Player
import dev.rubi.core.FfiPlaylistMode
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

    /** Set when the notification is tapped; the root opens the 听视频 page and clears it. */
    val pageRequested = mutableStateOf(false)

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private var core: RubiCore? = null
    private var appContext: Context? = null

    /** What was heard before, so 上一首 in shuffle goes back the way it came. */
    private val heard = ArrayDeque<FfiVideoCard>()

    private val listener = object : Player.Listener {
        override fun onPlaybackStateChanged(state: Int) {
            if (state == Player.STATE_ENDED) advance(byUser = false)
        }

        override fun onIsPlayingChanged(isPlaying: Boolean) {
            playing.value = isPlaying
        }
    }

    /** Starts hearing the list from `card`. */
    fun start(context: Context, core: RubiCore, card: FfiVideoCard) {
        this.core = core
        appContext = context.applicationContext
        heard.clear()
        val player = PlayerHolder.get(context)
        player.removeListener(listener)
        player.addListener(listener)
        play(card)
    }

    /** Switches to `card` from the list on the page. */
    fun jump(card: FfiVideoCard) {
        current.value?.let(heard::addLast)
        play(card)
    }

    fun toggle() {
        val ctx = appContext ?: return
        val player = PlayerHolder.get(ctx)
        if (player.isPlaying) player.pause() else player.play()
    }

    /** 下一首: always moves on, even in 单曲循环 (which only repeats when an item ends by itself). */
    fun next() = advance(byUser = true)

    /** 上一首: back to the start when more than 3 s in, else the previous item. */
    fun previous() {
        val c = core ?: return
        val ctx = appContext ?: return
        val now = current.value ?: return
        val player = PlayerHolder.get(ctx)
        if (player.currentPosition > 3_000) {
            player.seekTo(0)
            return
        }
        scope.launch {
            val shuffle = runCatching { c.playlistMode() }.getOrNull() == FfiPlaylistMode.SHUFFLE
            val back = if (shuffle && heard.isNotEmpty()) heard.removeLast() else runCatching { c.playlistPrevious(now.bvid) }.getOrNull()
            back?.let(::play)
        }
    }

    fun seekTo(ms: Long) {
        appContext?.let { PlayerHolder.get(it).seekTo(ms) }
    }

    /** Position and length of what is heard, in ms. */
    fun progress(): Pair<Long, Long> {
        val ctx = appContext ?: return 0L to 0L
        val p = PlayerHolder.get(ctx)
        return p.currentPosition to p.duration.coerceAtLeast(0L)
    }

    fun stop() {
        val ctx = appContext
        if (current.value == null) return
        current.value = null
        playing.value = false
        heard.clear()
        if (ctx != null) {
            PlayerHolder.get(ctx).removeListener(listener)
            PlayerHolder.clear()
        }
    }

    private fun advance(byUser: Boolean) {
        val c = core ?: return
        val now = current.value ?: return
        scope.launch {
            val mode = runCatching { c.playlistMode() }.getOrNull()
            val following = if (byUser && mode == FfiPlaylistMode.LOOP_ONE) {
                runCatching { c.playlistItems() }.getOrNull()?.let { items ->
                    val at = items.indexOfFirst { it.bvid == now.bvid }
                    items.getOrNull((at + 1) % items.size.coerceAtLeast(1))
                }
            } else {
                runCatching { c.playlistNext(now.bvid) }.getOrNull()
            }
            when {
                following == null && byUser -> runCatching { c.playlistFirst() }.getOrNull()?.let { heard.addLast(now); play(it) }
                following == null -> stop()
                else -> { if (following.bvid != now.bvid) heard.addLast(now); play(following) }
            }
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
                advance(byUser = false)
            }
        }
    }
}
