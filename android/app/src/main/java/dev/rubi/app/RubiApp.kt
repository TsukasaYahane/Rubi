package dev.rubi.app

import android.app.Application
import android.content.ComponentCallbacks2
import coil3.SingletonImageLoader
import dev.rubi.app.data.SessionStore
import androidx.compose.runtime.mutableStateOf
import dev.rubi.app.data.UiPrefs
import dev.rubi.app.playback.Media
import dev.rubi.core.FfiAccount
import dev.rubi.core.RubiCore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob

/** Holds the single Rust core for the process. */
class RubiApp : Application() {
    lateinit var core: RubiCore
        private set
    lateinit var sessions: SessionStore
        private set

    lateinit var ui: UiPrefs
        private set

    /** Players, 听视频, picture-in-picture: built once, after the core. */
    lateinit var media: Media
        private set

    /**
     * The signed-in account, kept for the whole process so pages do not start from 游客 each
     * time; seeded from the last known one when a session is stored.
     */
    val account = mutableStateOf<FfiAccount?>(null)

    /** A video asked for from outside (a Rubi 链接); the root opens it and clears this. */
    val openRequest = mutableStateOf<String?>(null)

    /** 外观 and 动效速度, read by the theme at once when changed in 设置. */
    val themeMode = mutableStateOf(0)
    val motion = mutableStateOf(2)
    val motionStyle = mutableStateOf(0)
    /** 应用图标 (小电视 / R 字): the launcher, 关于 and the share card follow it. */
    val appIcon = mutableStateOf(0)

    /** 节电: freezes after a while in the background or idle. */
    val power = PowerSaver(this)

    /** Re-reads the account; a failed read (offline) keeps what is shown. */
    suspend fun refreshAccount() {
        val a = runCatching { core.account() }.getOrNull() ?: return
        account.value = a
        ui.lastAccount = if (a.loggedIn) a.name to a.mid.toLong() else null
    }

    /** For work that must outlive a page, e.g. the last history report when the player closes. */
    val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    override fun onCreate() {
        super.onCreate()
        dev.rubi.app.ui.clearSharedCards(this)
        sessions = SessionStore(this)
        ui = UiPrefs(this)
        themeMode.value = ui.themeMode
        motion.value = ui.motion
        motionStyle.value = ui.motionStyle
        appIcon.value = ui.appIcon
        // Rubi was left without backing out (swiped away): the launcher catches up on this start.
        dev.rubi.app.ui.AppIcon.syncLauncher(this, ui.appIcon)
        val stored = sessions.load()
        core = RubiCore(filesDir.absolutePath, stored)
        media = Media(this, core)
        if (stored != null) {
            ui.lastAccount?.let { (name, mid) -> account.value = FfiAccount(true, mid.toULong(), name) }
        }
    }

    /**
     * Frees what watching videos piled up: the core's cached cards and comments, and decoded
     * images in memory (they reload from the disk cache when needed again).
     */
    fun releaseVideoCaches() {
        core.releaseCaches()
        SingletonImageLoader.get(this).memoryCache?.clear()
    }

    /** Going to the background or running low: let the caches go. */
    override fun onTrimMemory(level: Int) {
        super.onTrimMemory(level)
        if (level >= ComponentCallbacks2.TRIM_MEMORY_UI_HIDDEN) releaseVideoCaches()
    }

    /** Persists whatever session the core holds now (after login or device-id activation). */
    fun saveSession() = sessions.save(core.sessionJson())

    fun logout() {
        core.logout()
        sessions.clear()
        account.value = null
        ui.lastAccount = null
    }
}
