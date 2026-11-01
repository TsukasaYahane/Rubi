package dev.rubi.app.playback

import android.app.PendingIntent
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.SystemClock
import android.util.Log
import androidx.annotation.OptIn
import androidx.media3.common.MediaItem
import androidx.media3.common.ForwardingPlayer
import androidx.media3.common.MediaMetadata
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultDataSource
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.MergingMediaSource
import androidx.media3.exoplayer.source.ProgressiveMediaSource
import androidx.media3.session.MediaController
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import androidx.media3.session.SessionToken
import dev.rubi.app.RubiApp
import dev.rubi.core.FfiPlayback
import dev.rubi.core.FfiStream

private const val TAG = "RubiPlayback"

/**
 * The one ExoPlayer for real playback (videos and 听视频). The activity draws it;
 * [PlaybackService] exposes it through a MediaSession, which gives background play, the
 * notification and headset keys. Owned by [Media]; created on first use.
 */
class MainPlayer(context: Context) {
    private val context = context.applicationContext

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
    fun get(): ExoPlayer =
        player ?: synchronized(this) {
            player ?: ExoPlayer.Builder(context, Decoders.renderers(context))
                // Start after 0.5 s of media instead of the default 2.5 s; keep a normal buffer after that.
                .setLoadControl(
                    DefaultLoadControl.Builder()
                        .setBufferDurationsMs(15_000, 50_000, 500, 1_500)
                        .build(),
                )
                .build()
                .also { p ->
                    p.addListener(Listener())
                    player = p
                }
        }

    /** Plays the DASH pair (video + audio) chosen from [playback], from `positionMs`. */
    fun play(playback: FfiPlayback, title: String, positionMs: Long = 0L) {
        val video = pickVideo(playback) ?: return
        val c = Current(playback, title, video, pickAudio(playback))
        current = c
        load(c, positionMs)
        bindService()
    }

    /** An offline copy: the video file and its audio file, from `positionMs`. */
    @OptIn(UnstableApi::class)
    fun playLocal(videoPath: String, audioPath: String?, title: String, positionMs: Long = 0L) {
        current = null
        val factory = ProgressiveMediaSource.Factory(DefaultDataSource.Factory(context))
        val sources = listOfNotNull(videoPath, audioPath).map {
            factory.createMediaSource(item(Uri.fromFile(java.io.File(it)).toString(), title))
        }
        val p = get()
        p.setMediaSource(MergingMediaSource(*sources.toTypedArray()), positionMs)
        p.prepare()
        p.playWhenReady = true
        bindService()
    }

    /** 听视频: the audio stream only, with UP and cover for the notification and lock screen. */
    @OptIn(UnstableApi::class)
    fun playAudioOnly(playback: FfiPlayback, title: String, artist: String, cover: String) {
        val audio = pickAudio(playback) ?: return
        current = null
        val p = get()
        val meta = MediaMetadata.Builder().setTitle(title).setArtist(artist).setArtworkUri(Uri.parse(cover)).build()
        val media = MediaItem.Builder().setUri(audio.url).setMediaMetadata(meta).build()
        p.setMediaSource(ProgressiveMediaSource.Factory(http(playback)).createMediaSource(media))
        p.prepare()
        p.playWhenReady = true
        bindService()
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
    private fun load(c: Current, positionMs: Long) {
        val factory = ProgressiveMediaSource.Factory(http(c.playback))
        val sources = buildList {
            add(factory.createMediaSource(item(urlAt(c.video, c.attempt), c.title)))
            c.audio?.let { add(factory.createMediaSource(item(urlAt(it, c.attempt), c.title))) }
        }
        val p = get()
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

    private inner class Listener : Player.Listener {
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
                load(c, player?.currentPosition ?: 0L)
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

    private fun bindService() {
        val token = SessionToken(context, ComponentName(context, PlaybackService::class.java))
        MediaController.Builder(context, token).buildAsync()
    }
}

class PlaybackService : MediaSessionService() {
    private var session: MediaSession? = null

    override fun onCreate() {
        super.onCreate()
        // Tapping the notification opens the app; while listening, on the 听视频 page.
        val open = PendingIntent.getActivity(
            this,
            0,
            Intent(this, dev.rubi.app.MainActivity::class.java).setAction(ACTION_LISTEN).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val media = (application as RubiApp).media
        session = MediaSession.Builder(this, QueuePlayer(media.player.get(), media.listening)).setSessionActivity(open).build()
    }

    override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? = session

    override fun onDestroy() {
        session?.release()
        session = null
        (application as RubiApp).media.player.release()
        super.onDestroy()
    }
}

const val ACTION_LISTEN = "dev.rubi.app.LISTEN"

/**
 * The player as the notification, lock screen and headset see it: while 听视频 runs, 上一首 /
 * 下一首 are offered and go to [Listening] instead of the player's own one-item timeline.
 */
private class QueuePlayer(player: Player, private val queue: Listening) : ForwardingPlayer(player) {
    private val listening get() = queue.current.value != null

    override fun getAvailableCommands(): Player.Commands {
        val base = super.getAvailableCommands()
        if (!listening) return base
        return base.buildUpon()
            .addAll(Player.COMMAND_SEEK_TO_NEXT, Player.COMMAND_SEEK_TO_NEXT_MEDIA_ITEM, Player.COMMAND_SEEK_TO_PREVIOUS, Player.COMMAND_SEEK_TO_PREVIOUS_MEDIA_ITEM)
            .build()
    }

    override fun isCommandAvailable(command: Int): Boolean = getAvailableCommands().contains(command)

    // The session calls these on the main thread, where Listening lives.
    override fun seekToNext() = if (listening) queue.next() else super.seekToNext()
    override fun seekToNextMediaItem() = if (listening) queue.next() else super.seekToNextMediaItem()
    override fun seekToPrevious() = if (listening) queue.previous() else super.seekToPrevious()
    override fun seekToPreviousMediaItem() = if (listening) queue.previous() else super.seekToPreviousMediaItem()
}
