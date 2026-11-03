package dev.rubi.app.playback

import android.content.Context
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.staticCompositionLocalOf
import dev.rubi.core.RubiCore

/**
 * Everything about playing media for this process, owned by the application and handed to
 * screens through [LocalMedia] (the playback service reaches it through the application).
 * Nothing here is a global: each part is built once, here, from what it needs.
 */
class Media(context: Context, core: RubiCore) {
    /** Real playback: videos and 听视频. */
    val player = MainPlayer(context)

    /** 大图预览 in the feed, kept apart from real playback. */
    val preview = PreviewPlayer(context)

    /** 听视频: what is heard; the core picks what comes next. */
    val listening = Listening(player, core)

    val pip = Pip()

    /** 投屏 to a TV, kept while the remote is closed. */
    val cast = dev.rubi.app.cast.CastSession()

    /**
     * True while the player is fullscreen: the root then paints black edge to edge instead of
     * keeping clear of the camera cutout and system bars.
     */
    val fullscreen = mutableStateOf(false)
}

val LocalMedia = staticCompositionLocalOf<Media> { error("Media is provided by MainActivity") }
