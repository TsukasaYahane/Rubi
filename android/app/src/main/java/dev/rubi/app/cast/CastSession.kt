package dev.rubi.app.cast

import androidx.compose.runtime.mutableStateOf
import dev.rubi.core.FfiCastFile

/**
 * The 投屏 going on, if any: which player, which video, and the relay feeding it. Kept by the app
 * so the remote can be closed and opened again while the TV plays on.
 */
class CastSession {
    val renderer = mutableStateOf<Renderer?>(null)
    val title = mutableStateOf("")
    /** The video being cast and its quality, so 清晰度 can switch it. */
    var bvid: String = ""
        private set
    var cid: ULong? = null
        private set
    val quality = mutableStateOf(0u)
    private val relay = CastRelay()

    /** Starts `file` on `to`; throws when the network address or the player refuses. */
    suspend fun start(to: Renderer, file: FfiCastFile, bvid: String, cid: ULong?, title: String, fromSeconds: Long = 0) {
        val url = relay.serve(file.urls, file.headers.associate { it.name to it.value })
            ?: throw IllegalStateException("没有连接到局域网（Wi-Fi），无法投屏")
        Dlna.play(to, url, title)
        if (fromSeconds > 0) runCatching { Dlna.seek(to, fromSeconds) }
        renderer.value = to
        this.title.value = title
        this.bvid = bvid
        this.cid = cid
        quality.value = file.quality
    }

    suspend fun end() {
        renderer.value?.let { runCatching { Dlna.stop(it) } }
        relay.stop()
        renderer.value = null
        title.value = ""
    }
}
