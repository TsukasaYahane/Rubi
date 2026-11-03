package dev.rubi.app

import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import android.util.Log
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.mutableStateOf

/** Background this long and Rubi freezes. */
private const val BACKGROUND_MS = 12_000L

/** No touch for this long in the foreground, with nothing playing, and Rubi freezes too. */
private const val IDLE_MS = 3 * 60_000L

private const val TAG = "RubiPower"

/**
 * 节电: after [BACKGROUND_MS] in the background, or [IDLE_MS] in the foreground without a touch
 * while nothing plays, Rubi freezes until it is woken (back to the foreground, or a touch).
 *
 * Frozen means only what keeps the current page as it is: the image memory cache and the core's
 * card / comment caches are let go, the feed preview player is released, a paused player drops
 * its decoders and buffers (in the background only, where its frame is not on screen), and pages
 * stop their frame loops and live channels (they read [frozen]). Playback that is running, 听视频
 * and downloads carry on.
 */
class PowerSaver(private val app: RubiApp) {
    /** Pages pause timers, frame loops and sockets while this is true. */
    val frozen = mutableStateOf(false)

    private val main = Handler(Looper.getMainLooper())
    private var foreground = false
    private var lastTouch = SystemClock.uptimeMillis()
    /** The paused player was stopped on freezing and is prepared again on waking. */
    private var playerStopped = false

    private val freezeNow = Runnable { freeze() }
    private val idleCheck = Runnable { checkIdle() }

    fun onForeground() {
        foreground = true
        main.removeCallbacks(freezeNow)
        wake()
        touched()
    }

    fun onBackground() {
        foreground = false
        main.removeCallbacks(idleCheck)
        main.removeCallbacks(freezeNow)
        main.postDelayed(freezeNow, BACKGROUND_MS)
    }

    /** Any touch or key: wakes a frozen app and restarts the idle clock. */
    fun touched() {
        lastTouch = SystemClock.uptimeMillis()
        if (frozen.value) wake()
        main.removeCallbacks(idleCheck)
        if (foreground) main.postDelayed(idleCheck, IDLE_MS)
    }

    private fun checkIdle() {
        if (!foreground) return
        val quiet = SystemClock.uptimeMillis() - lastTouch
        when {
            quiet < IDLE_MS -> main.postDelayed(idleCheck, IDLE_MS - quiet)
            // Watching or listening without touching is not idle.
            app.media.player.get().isPlaying -> main.postDelayed(idleCheck, IDLE_MS)
            else -> freeze()
        }
    }

    private fun freeze() {
        if (frozen.value) return
        frozen.value = true
        app.media.preview.release()
        if (!foreground) {
            val player = app.media.player.get()
            if (!player.isPlaying && player.mediaItemCount > 0) {
                // Keeps the item and position; prepare() picks it up again.
                player.stop()
                playerStopped = true
            }
        }
        app.releaseVideoCaches()
        Log.i(TAG, if (foreground) "frozen after ${IDLE_MS / 1000} s idle" else "frozen in the background")
    }

    private fun wake() {
        if (!frozen.value) return
        frozen.value = false
        if (playerStopped) {
            playerStopped = false
            // Paused before, paused after: only the frame and buffers come back.
            app.media.player.get().prepare()
        }
        Log.i(TAG, "woken")
    }
}

/** Whether Rubi is frozen ([PowerSaver]); pages stop frame loops and sockets while it is true. */
val LocalFrozen = compositionLocalOf { false }
