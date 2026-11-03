package dev.rubi.app.playback

import android.os.SystemClock
import android.util.Log
import androidx.compose.runtime.mutableStateOf
import androidx.media3.common.Player
import dev.rubi.core.FfiVideoCard
import dev.rubi.core.RubiCore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
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
    /** Hearing one video followed by its 相关推荐, not a play list. */
    val related = mutableStateOf(false)

    /** 定时暂停: when hearing pauses (elapsedRealtime ms), or null with no timer. */
    val sleepAt = mutableStateOf<Long?>(null)
    private var sleepJob: Job? = null

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

    /** Starts hearing play list `list` from `card`. */
    fun start(card: FfiVideoCard, list: Long) {
        attach()
        related.value = false
        // Shown at once, so the 听视频 page opened next has something to show.
        current.value = card
        scope.launch { runCatching { core.listenFrom(list, card.bvid) }.onSuccess { play(it) }.onFailure { stop() } }
    }

    /**
     * Starts hearing one video from `fromMs` (where its video page was), then its 相关推荐 one
     * after another, topped up as they are heard.
     */
    fun startRelated(card: FfiVideoCard, fromMs: Long) {
        attach()
        related.value = true
        current.value = card
        scope.launch { runCatching { core.listenRelated(card.bvid) }.onSuccess { play(it, fromMs) }.onFailure { stop() } }
    }

    private fun attach() {
        val exo = player.get()
        exo.removeListener(listener)
        exo.addListener(listener)
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

    /** Pauses hearing after `minutes`; a new timer replaces the old one. */
    fun sleepIn(minutes: Int) {
        sleepJob?.cancel()
        val ms = minutes * 60_000L
        sleepAt.value = SystemClock.elapsedRealtime() + ms
        sleepJob = scope.launch {
            delay(ms)
            player.get().pause()
            sleepAt.value = null
        }
    }

    fun cancelSleep() {
        sleepJob?.cancel()
        sleepJob = null
        sleepAt.value = null
    }

    fun stop() {
        cancelSleep()
        if (current.value == null) return
        current.value = null
        playing.value = false
        related.value = false
        core.listenStop()
        player.get().removeListener(listener)
        player.clear()
    }

    /**
     * Following 相关推荐: when few are left after `bvid`, adds its own related videos (already
     * through the rule table) so hearing goes on.
     */
    private fun topUp(bvid: String) {
        val left = core.relatedLeft() ?: return
        if (left >= RELATED_AHEAD) return
        scope.launch {
            val more = runCatching { core.related(bvid) }.getOrDefault(emptyList())
            core.listenAppend(more.take(RELATED_BATCH).map { it.bvid })
        }
    }

    /** Asks the core for the item to hear next; none means listening is over. */
    private fun move(pick: () -> FfiVideoCard?) {
        scope.launch {
            val card = runCatching { pick() }.getOrNull()
            if (card == null) stop() else play(card)
        }
    }

    private fun play(card: FfiVideoCard, fromMs: Long = 0L) {
        current.value = card
        topUp(card.bvid)
        scope.launch {
            runCatching {
                // Audio streams are the same at any quality; ask for the lowest.
                val playback = core.playback(card.bvid, 16u)
                player.playAudioOnly(playback, card.title, card.upName, card.cover)
                if (fromMs > 0) player.get().seekTo(fromMs)
            }.onFailure {
                Log.w("RubiAudio", "cannot play ${card.bvid}", it)
                move { core.listenNext(false) }
            }
        }
    }
}

/** Fewer than this many still to come after the one heard: fetch more. */
private const val RELATED_AHEAD = 3u
/** Related videos added each time. */
private const val RELATED_BATCH = 8

/** Within this much of the start, 上一首 goes to the previous item instead of restarting. */
private const val RESTART_WITHIN_MS = 3_000L
