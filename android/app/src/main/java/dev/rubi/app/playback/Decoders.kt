package dev.rubi.app.playback

import android.content.Context
import android.os.Build
import androidx.annotation.OptIn
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.DefaultRenderersFactory
import androidx.media3.exoplayer.RenderersFactory
import androidx.media3.exoplayer.mediacodec.MediaCodecSelector
import androidx.media3.exoplayer.mediacodec.MediaCodecUtil
import java.io.File

/**
 * Which decoders the players use. Emulators such as MuMu offer a fake hardware H.264 decoder
 * that starts without complaint but only ever outputs green frames, and playback then stalls
 * after a few seconds; there the software decoders go first. Phones keep hardware first.
 */
object Decoders {
    /** Files only the MuMu emulator ships, and the Android emulator's hardware names. */
    private val emulatorMarkers = listOf("/system/lib64/libnemuinitaidl.so", "/system/lib/libnemuinitaidl.so")

    val onEmulator: Boolean by lazy {
        Build.HARDWARE in setOf("goldfish", "ranchu") ||
            Build.FINGERPRINT.startsWith("generic") ||
            emulatorMarkers.any { runCatching { File(it).exists() }.getOrDefault(false) }
    }

    /** Renderers for a player: software decoders first on emulators, and falling back to the
     *  next decoder whenever one fails to start. */
    @OptIn(UnstableApi::class)
    fun renderers(context: Context): RenderersFactory {
        val selector = if (onEmulator) {
            MediaCodecSelector { mime, secure, tunneling ->
                MediaCodecUtil.getDecoderInfos(mime, secure, tunneling).sortedBy { if (it.softwareOnly) 0 else 1 }
            }
        } else {
            MediaCodecSelector.DEFAULT
        }
        return DefaultRenderersFactory(context).setMediaCodecSelector(selector).setEnableDecoderFallback(true)
    }
}
