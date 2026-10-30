package dev.rubi.app.playback

import android.content.ComponentName
import android.content.Context
import androidx.annotation.OptIn
import androidx.media3.common.MediaItem
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.MergingMediaSource
import androidx.media3.exoplayer.source.ProgressiveMediaSource
import androidx.media3.session.MediaController
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import androidx.media3.session.SessionToken
import dev.rubi.core.FfiPlayback
import dev.rubi.core.FfiStream

/**
 * One ExoPlayer for the whole process. The activity draws it; [PlaybackService] exposes
 * it through a MediaSession, which gives background play, the notification and headset keys.
 */
object PlayerHolder {
    @Volatile
    private var player: ExoPlayer? = null

    fun get(context: Context): ExoPlayer =
        player ?: synchronized(this) {
            player ?: ExoPlayer.Builder(context.applicationContext).build().also { player = it }
        }

    /** Plays the DASH pair (video + audio) chosen from [playback]. */
    @OptIn(UnstableApi::class)
    fun play(context: Context, playback: FfiPlayback, title: String) {
        val headers = playback.headers.associate { it.name to it.value }
        val http = DefaultHttpDataSource.Factory().setDefaultRequestProperties(headers)
        val video = pickVideo(playback) ?: return
        val sources = buildList {
            add(ProgressiveMediaSource.Factory(http).createMediaSource(item(video, title)))
            pickAudio(playback)?.let { add(ProgressiveMediaSource.Factory(http).createMediaSource(item(it, title))) }
        }
        val p = get(context)
        p.setMediaSource(MergingMediaSource(*sources.toTypedArray()))
        p.prepare()
        p.playWhenReady = true
        bindService(context)
    }

    /** Audio-only entry point reserved for the 听视频 mode (no UI yet). */
    @OptIn(UnstableApi::class)
    fun playAudioOnly(context: Context, playback: FfiPlayback, title: String) {
        val http = DefaultHttpDataSource.Factory().setDefaultRequestProperties(playback.headers.associate { it.name to it.value })
        val audio = pickAudio(playback) ?: return
        val p = get(context)
        p.setMediaSource(ProgressiveMediaSource.Factory(http).createMediaSource(item(audio, title)))
        p.prepare()
        p.playWhenReady = true
        bindService(context)
    }

    fun release() = synchronized(this) {
        player?.release()
        player = null
    }

    private fun item(s: FfiStream, title: String) = MediaItem.Builder()
        .setUri(s.url)
        .setMediaMetadata(androidx.media3.common.MediaMetadata.Builder().setTitle(title).build())
        .build()

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
