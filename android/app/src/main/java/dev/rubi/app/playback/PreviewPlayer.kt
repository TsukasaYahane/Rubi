package dev.rubi.app.playback

import android.content.Context
import androidx.annotation.OptIn
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.MergingMediaSource
import androidx.media3.exoplayer.source.ProgressiveMediaSource
import dev.rubi.core.FfiPlayback

/** Quality asked for previews: 360P keeps them light. */
const val PREVIEW_QN: UInt = 16u

/**
 * A second, small ExoPlayer for 大图预览: no media session, no notification, short buffers,
 * looping. It never touches [MainPlayer], so a preview cannot disturb real playback.
 */
class PreviewPlayer(context: Context) {
    private val context = context.applicationContext

    @Volatile
    private var player: ExoPlayer? = null

    @OptIn(UnstableApi::class)
    fun get(): ExoPlayer =
        player ?: synchronized(this) {
            player ?: ExoPlayer.Builder(context, Decoders.renderers(context))
                .setLoadControl(DefaultLoadControl.Builder().setBufferDurationsMs(2_000, 8_000, 500, 1_000).build())
                .build()
                .also {
                    it.repeatMode = Player.REPEAT_MODE_ONE
                    player = it
                }
        }

    /** Plays the lightest video stream of [playback]; audio only when `volume` > 0. */
    @OptIn(UnstableApi::class)
    fun play(playback: FfiPlayback, volume: Float) {
        val video = playback.video.filter { it.codecs.startsWith("avc1") }.ifEmpty { playback.video }.minByOrNull { it.bandwidth } ?: return
        val audio = if (volume > 0f) playback.audio.minByOrNull { it.bandwidth } else null
        val http = DefaultHttpDataSource.Factory().setDefaultRequestProperties(playback.headers.associate { it.name to it.value })
        val factory = ProgressiveMediaSource.Factory(http)
        val sources = listOfNotNull(video, audio).map { factory.createMediaSource(MediaItem.fromUri(it.url)) }
        val p = get()
        p.volume = volume.coerceIn(0f, 1f)
        p.setMediaSource(MergingMediaSource(*sources.toTypedArray()))
        p.prepare()
        p.playWhenReady = true
    }

    fun stop() {
        player?.run {
            stop()
            clearMediaItems()
        }
    }
}
