package dev.rubi.app.playback

import android.os.SystemClock
import android.util.Log
import androidx.compose.runtime.mutableStateOf
import androidx.media3.common.Player
import dev.rubi.core.VideosService
import dev.rubi.core.DiscoverService
import dev.rubi.core.ListeningService
import dev.rubi.core.FfiPlayback
import dev.rubi.core.FfiVideoCard
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * What 听视频 needs from the core: the listening queue's moves and a video's audio. The core
 * decides what comes next; nothing else of it is reachable from here.
 */
interface ListenSource {
    suspend fun start(list: Long, bvid: String): FfiVideoCard
    suspend fun startRelated(bvid: String): FfiVideoCard
    suspend fun jump(bvid: String): FfiVideoCard
    /** Reads the stored list: call off the main thread. */
    fun next(byUser: Boolean): FfiVideoCard?
    /** Reads the stored list: call off the main thread. */
    fun previous(): FfiVideoCard?
    fun stop()
    fun relatedLeft(): UInt?
    suspend fun relatedOf(bvid: String): List<String>
    suspend fun append(bvids: List<String>)
    suspend fun audio(bvid: String): FfiPlayback
}

/**
 * [ListenSource] over the core's 听视频 queue, 相关推荐 and streams, each resolved on first
 * use (the core may still be opening).
 */
class CoreListenSource(
    private val queue: () -> ListeningService,
    private val discover: () -> DiscoverService,
    private val videos: () -> VideosService,
) : ListenSource {
    override suspend fun start(list: Long, bvid: String) = queue().listenFrom(list, bvid)
    override suspend fun startRelated(bvid: String) = queue().listenRelated(bvid)
    override suspend fun jump(bvid: String) = queue().listenJump(bvid)
    override fun next(byUser: Boolean) = queue().listenNext(byUser)
    override fun previous() = queue().listenPrevious()
    override fun stop() = queue().listenStop()
    override fun relatedLeft() = queue().relatedLeft()
    override suspend fun relatedOf(bvid: String) = discover().related(bvid).map { it.bvid }
    override suspend fun append(bvids: List<String>) = queue().listenAppend(bvids)
    // Audio streams are the same at any quality; ask for the lowest.
    override suspend fun audio(bvid: String) = videos().playback(bvid, 16u)
}

/**
 * 听视频 on this device: plays what the core's listening queue picks, audio only, on the shared
 * player (so the media session, notification and background play all apply). Which item comes
 * next or before is the core's call; this object only plays it. Opening a video page stops it:
 * the two would fight over the one player.
 *
 * Every move (start, jump, next, previous, an item ending) replaces the one before it: a slow
 * answer for an earlier pick is dropped, never played over the later one, and nothing plays
 * after [stop].
 */
class Listening(private val player: MainPlayer, private val source: ListenSource) {
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

    /** The move under way; a new one cancels it. */
    private var moving: Job? = null

    private val listener = object : Player.Listener {
        override fun onPlaybackStateChanged(state: Int) {
            if (state == Player.STATE_ENDED) move { source.next(false) }
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
        replaceMove { picked({ source.start(list, card.bvid) }, onError = ::stop)?.let { play(it) } }
    }

    /**
     * Starts hearing one video from `fromMs` (where its video page was), then its 相关推荐 one
     * after another, topped up as they are heard.
     */
    fun startRelated(card: FfiVideoCard, fromMs: Long) {
        attach()
        related.value = true
        current.value = card
        replaceMove { picked({ source.startRelated(card.bvid) }, onError = ::stop)?.let { play(it, fromMs) } }
    }

    private fun attach() {
        val exo = player.get()
        exo.removeListener(listener)
        exo.addListener(listener)
    }

    /** Switches to `card` from the list on the page. */
    fun jump(card: FfiVideoCard) {
        replaceMove { picked({ source.jump(card.bvid) })?.let { play(it) } }
    }

    fun toggle() {
        val exo = player.get()
        if (exo.isPlaying) exo.pause() else exo.play()
    }

    /** 下一首: always moves on (the core decides where to). */
    fun next() = move { source.next(true) }

    /** 上一首: back to the start when a few seconds in, else the previous item. */
    fun previous() {
        val exo = player.get()
        if (exo.currentPosition > RESTART_WITHIN_MS) exo.seekTo(0) else move { source.previous() }
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
        moving?.cancel()
        moving = null
        cancelSleep()
        if (current.value == null) return
        current.value = null
        playing.value = false
        related.value = false
        source.stop()
        player.get().removeListener(listener)
        player.clear()
    }

    /**
     * Following 相关推荐: when few are left after `bvid`, adds its own related videos (already
     * through the rule table) so hearing goes on.
     */
    private fun topUp(bvid: String) {
        val left = source.relatedLeft() ?: return
        if (left >= RELATED_AHEAD) return
        scope.launch {
            val more = runCatching { source.relatedOf(bvid) }.getOrDefault(emptyList())
            source.append(more.take(RELATED_BATCH))
        }
    }

    /** Runs `block` as the move under way, cancelling the one before. */
    private fun replaceMove(block: suspend () -> Unit) {
        moving?.cancel()
        moving = scope.launch { block() }
    }

    /** Asks the core (off the main thread) for the item to hear next; none means listening is over. */
    private fun move(pick: () -> FfiVideoCard?) {
        replaceMove {
            val card = picked({ withContext(Dispatchers.IO) { pick() } })
            if (card == null) stop() else play(card)
        }
    }

    /** The result of `pick`, or null after `onError`; a cancelled move stays cancelled. */
    private suspend fun picked(pick: suspend () -> FfiVideoCard?, onError: () -> Unit = {}): FfiVideoCard? =
        try {
            pick()
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            onError()
            null
        }

    /** Plays `card`; one that cannot be played is skipped for the next. Runs inside a move. */
    private suspend fun play(card: FfiVideoCard, fromMs: Long = 0L) {
        var item: FfiVideoCard? = card
        var at = fromMs
        while (item != null) {
            current.value = item
            topUp(item.bvid)
            try {
                val playback = source.audio(item.bvid)
                currentCoroutineContext().ensureActive()
                player.playAudioOnly(playback, item.title, item.upName, item.cover)
                if (at > 0) player.get().seekTo(at)
                return
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w("RubiAudio", "cannot play ${item.bvid}", e)
                item = picked({ withContext(Dispatchers.IO) { source.next(false) } })
                at = 0L
            }
        }
        stop()
    }
}

/** Fewer than this many still to come after the one heard: fetch more. */
private const val RELATED_AHEAD = 3u
/** Related videos added each time. */
private const val RELATED_BATCH = 8

/** Within this much of the start, 上一首 goes to the previous item instead of restarting. */
private const val RESTART_WITHIN_MS = 3_000L
