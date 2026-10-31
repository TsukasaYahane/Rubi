package dev.rubi.app.playback

import android.content.ComponentName
import android.content.Context
import android.os.SystemClock
import android.util.Log
import androidx.annotation.OptIn
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.MergingMediaSource
import androidx.media3.exoplayer.source.ProgressiveMediaSource
import androidx.media3.session.MediaController
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import androidx.media3.session.SessionToken
import dev.rubi.core.FfiPlayback
import dev.rubi.core.FfiStream

private const val TAG = "RubiPlayback"

/**
 * One ExoPlayer for the whole process. The activity draws it; [PlaybackService] exposes
 * it through a MediaSession, which gives background play, the notification and headset keys.
 */
object PlayerHolder {
    @Volatile
    private var player: ExoPlayer? = null

    /** What is playing now, so a failing CDN node can be swapped for the next url. */
    private class Current(val playback: FfiPlayback, val title: String, val video: FfiStream, val audio: FfiStream?) {
        var attempt = 0
        val attempts = maxOf(1 + video.backupUrls.size, 1 + (audio?.backupUrls?.size ?: 0))
        val startedAt = SystemClock.elapsedRealtime()
        var firstFrameLogged = false
    }

    @Volatile
    private var current: Current? = null

    @OptIn(UnstableApi::class)
    fun get(context: Context): ExoPlayer =
        player ?: synchronized(this) {
            player ?: ExoPlayer.Builder(context.applicationContext)
                // Start after 0.5 s of media instead of the default 2.5 s; keep a normal buffer after that.
                .setLoadControl(
                    DefaultLoadControl.Builder()
                        .setBufferDurationsMs(15_000, 50_000, 500, 1_500)
                        .build(),
                )
                .build()
                .also { p ->
                    p.addListener(Listener(context.applicationContext))
                    player = p
                }
        }

    /** Plays the DASH pair (video + audio) chosen from [playback], from `positionMs`. */
    fun play(context: Context, playback: FfiPlayback, title: String, positionMs: Long = 0L) {
        val video = pickVideo(playback) ?: return
        val c = Current(playback, title, video, pickAudio(playback))
        current = c
        load(context, c, positionMs)
        bindService(context)
    }

    /** Audio-only entry point reserved for the 听视频 mode (no UI yet). */
    @OptIn(UnstableApi::class)
    fun playAudioOnly(context: Context, playback: FfiPlayback, title: String) {
        val audio = pickAudio(playback) ?: return
        current = null
        val p = get(context)
        p.setMediaSource(ProgressiveMediaSource.Factory(http(playback)).createMediaSource(item(audio.url, title)))
        p.prepare()
        p.playWhenReady = true
        bindService(context)
    }

    /**
     * Stops and empties the player, so nothing of the previous video keeps playing or stays
     * on screen while the next one loads. The view then shows its black shutter.
     */
    fun clear() {
        current = null
        player?.run {
            stop()
            clearMediaItems()
        }
    }

    fun release() = synchronized(this) {
        player?.release()
        player = null
        current = null
    }

    @OptIn(UnstableApi::class)
    private fun load(context: Context, c: Current, positionMs: Long) {
        val factory = ProgressiveMediaSource.Factory(http(c.playback))
        val sources = buildList {
            add(factory.createMediaSource(item(urlAt(c.video, c.attempt), c.title)))
            c.audio?.let { add(factory.createMediaSource(item(urlAt(it, c.attempt), c.title))) }
        }
        val p = get(context)
        p.setMediaSource(MergingMediaSource(*sources.toTypedArray()), positionMs)
        p.prepare()
        p.playWhenReady = true
        Log.i(TAG, "load attempt ${c.attempt + 1}/${c.attempts}: ${host(urlAt(c.video, c.attempt))}")
    }

    /** The url for `attempt`: the best-ranked first, then backups (wrapping if a list is shorter). */
    private fun urlAt(s: FfiStream, attempt: Int): String {
        val all = listOf(s.url) + s.backupUrls
        return all[attempt % all.size]
    }

    private class Listener(private val context: Context) : Player.Listener {
        override fun onRenderedFirstFrame() {
            val c = current ?: return
            if (!c.firstFrameLogged) {
                c.firstFrameLogged = true
                Log.i(TAG, "first frame ${SystemClock.elapsedRealtime() - c.startedAt} ms after play()")
            }
        }

        override fun onPlayerError(error: PlaybackException) {
            val c = current ?: return
            Log.w(TAG, "playback error on attempt ${c.attempt + 1}: ${error.errorCodeName}", error)
            if (c.attempt + 1 < c.attempts) {
                c.attempt++
                load(context, c, player?.currentPosition ?: 0L)
            }
        }
    }

    @OptIn(UnstableApi::class)
    private fun http(p: FfiPlayback) =
        DefaultHttpDataSource.Factory().setDefaultRequestProperties(p.headers.associate { it.name to it.value })

    private fun item(url: String, title: String) = MediaItem.Builder()
        .setUri(url)
        .setMediaMetadata(MediaMetadata.Builder().setTitle(title).build())
        .build()

    private fun host(url: String) = url.substringAfter("://").substringBefore('/')

    /** Highest quality the server granted, preferring AVC for the widest device support. */
    private fun pickVideo(p: FfiPlayback): FfiStream? {
        val best = p.video.filter { it.id == p.quality }.ifEmpty { p.video }
        return best.firstOrNull { it.codecs.startsWith("avc1") } ?: best.maxByOrNull { it.bandwidth }
    }

    private fun pickAudio(p: FfiPlayback): FfiStream? = p.audio.maxByOrNull { it.bandwidth }

    private fun bindService(context: Context) {
        val token = SessionToken(context, ComponentName(context, PlaybackService::class.java))
        MediaController.Builder(context, token).buildAsync()
    }
}

class PlaybackService : MediaSessionService() {
    private var session: MediaSession? = null

    override fun onCreate() {
        super.onCreate()
        session = MediaSession.Builder(this, PlayerHolder.get(this)).build()
    }

    override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? = session

    override fun onDestroy() {
        session?.release()
        session = null
        PlayerHolder.release()
        super.onDestroy()
    }
}
