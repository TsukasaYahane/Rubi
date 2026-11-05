package dev.rubi.app

import android.app.Application
import android.content.ComponentCallbacks2
import android.os.Handler
import android.os.Looper
import coil3.SingletonImageLoader
import dev.rubi.app.data.CoreSettings
import dev.rubi.app.data.SessionStore
import androidx.compose.runtime.mutableStateOf
import dev.rubi.app.data.UiPrefs
import dev.rubi.app.playback.CoreListenSource
import dev.rubi.app.playback.Media
import dev.rubi.core.RubiCore
import java.util.concurrent.ExecutionException
import java.util.concurrent.FutureTask
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob

/** Holds the single Rust core for the process. */
class RubiApp : Application() {
    /**
     * Opened on a thread of its own at start (the database is read then), so the first frame
     * does not wait for it; [coreReady] turns true once it is open. Reading [core] before that
     * waits for it.
     */
    private lateinit var opening: FutureTask<RubiCore>

    val core: RubiCore
        get() = try {
            opening.get()
        } catch (e: ExecutionException) {
            throw e.cause ?: e
        }

    /** Whether [core] is open; the UI shows nothing that needs it before. */
    val coreReady = mutableStateOf(false)

    lateinit var sessions: SessionStore
        private set

    lateinit var ui: UiPrefs
        private set

    /** Players, 听视频, picture-in-picture: built once; they reach the core only when used. */
    lateinit var media: Media
        private set

    /** The core's settings, changed key by key. */
    val settings by lazy { CoreSettings(core.preferences(), scope) }

    /** Who is signed in, and the stored session. */
    lateinit var signin: Signin
        private set

    /** A video asked for from outside (a Rubi 链接); the root opens it and clears this. */
    val openRequest = mutableStateOf<String?>(null)

    /** 外观 as shown now. */
    lateinit var appearance: Appearance
        private set

    /** 节电: freezes after a while in the background or idle. */
    val power = PowerSaver(this)

    /** For work that must outlive a page, e.g. the last history report when the player closes. */
    val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun onCreate() {
        super.onCreate()
        sessions = SessionStore(this)
        ui = UiPrefs(this)
        val stored = sessions.load()
        val dataDir = filesDir.absolutePath
        val main = Handler(Looper.getMainLooper())
        opening = FutureTask { RubiCore(dataDir, stored) }
        Thread({
            opening.run()
            main.post { coreReady.value = true }
        }, "rubi-core-open").start()

        dev.rubi.app.ui.clearSharedCards(this)
        appearance = Appearance(ui)
        signin = Signin({ core.account() }, sessions, ui, stored != null)
        // Rubi was left without backing out (swiped away): the launcher catches up on this start.
        dev.rubi.app.ui.AppIcon.syncLauncher(this, ui.appIcon)
        media = Media(this, CoreListenSource({ core.listening() }, { core.discover() }, { core.videos() }))
    }

    /**
     * Frees what watching videos piled up: the core's cached cards and comments, and decoded
     * images in memory (they reload from the disk cache when needed again).
     */
    fun releaseVideoCaches() {
        if (opening.isDone) runCatching { core.releaseCaches() }
        SingletonImageLoader.get(this).memoryCache?.clear()
    }

    /** Going to the background or running low: let the caches go. */
    override fun onTrimMemory(level: Int) {
        super.onTrimMemory(level)
        if (level >= ComponentCallbacks2.TRIM_MEMORY_UI_HIDDEN) releaseVideoCaches()
    }
}
