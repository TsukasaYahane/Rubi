package dev.rubi.app

import dev.rubi.app.ui.UI_WORDS
import dev.rubi.app.ui.LocalWordsUi
import android.content.Intent
import android.content.res.Configuration
import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.SystemBarStyle
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.LaunchedEffect
import android.graphics.Color
import dev.rubi.app.ui.LocalMotion
import dev.rubi.app.ui.AppIcon
import dev.rubi.app.ui.Motion
import dev.rubi.app.ui.theme.isDark
import androidx.compose.runtime.CompositionLocalProvider
import dev.rubi.app.playback.ACTION_LISTEN
import dev.rubi.app.playback.LocalMedia
import dev.rubi.app.ui.RubiRoot
import dev.rubi.app.ui.LocalPortraitCover
import dev.rubi.app.ui.PORTRAIT_FILL
import dev.rubi.app.ui.LocalScreenLayout
import dev.rubi.app.ui.applyDeviceOrientation
import dev.rubi.app.ui.rememberScreenLayout
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.ui.Modifier

class MainActivity : ComponentActivity() {
    private val media get() = (application as RubiApp).media

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        applyDeviceOrientation()
        val app = application as RubiApp
        setContent {
            val mode = app.appearance.themeMode.value
            val dark = isDark(mode)
            // Status and navigation bar icons follow the chosen 外观, not only the system's.
            LaunchedEffect(dark) {
                val bars = if (dark) SystemBarStyle.dark(Color.TRANSPARENT) else SystemBarStyle.light(Color.TRANSPARENT, Color.TRANSPARENT)
                enableEdgeToEdge(statusBarStyle = bars, navigationBarStyle = bars)
            }
            MetroTheme(mode) {
                CompositionLocalProvider(
                    LocalMedia provides app.media,
                    LocalFrozen provides app.power.frozen.value,
                    LocalMotion provides Motion(app.appearance.motion.value, app.appearance.motionStyle.value),
                    // 宽屏: tablet or phone, and how wide the window is now.
                    LocalScreenLayout provides rememberScreenLayout(),
                    LocalPortraitCover provides (app.appearance.portraitCover.value == PORTRAIT_FILL),
                    LocalWordsUi provides (app.appearance.uiStyle.value == UI_WORDS),
                ) {
                    // The core opens on its own thread at start; until then, the page background.
                    if (app.coreReady.value) RubiRoot(app)
                    else Box(Modifier.fillMaxSize().background(LocalMetro.current.bg))
                }
            }
        }
        handle(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handle(intent)
    }

    /** The playback notification: open the 听视频 page when something is being heard. */
    private fun handle(intent: Intent?) {
        // rubi://video/BV…: open that video.
        val link = intent?.data
        if (intent?.action == Intent.ACTION_VIEW && link?.scheme == RUBI_SCHEME && link.host == "video") {
            link.lastPathSegment?.takeIf { it.startsWith("BV") }?.let { (application as RubiApp).openRequest.value = it }
        }
        if (intent?.action == ACTION_LISTEN && media.listening.current.value != null) media.listening.pageRequested.value = true
    }

    override fun onStart() {
        super.onStart()
        (application as RubiApp).power.onForeground()
    }

    override fun onStop() {
        super.onStop()
        // In picture-in-picture the window is still on screen: not background.
        if (!isInPictureInPictureMode) (application as RubiApp).power.onBackground()
    }

    override fun dispatchTouchEvent(ev: MotionEvent): Boolean {
        if (ev.actionMasked == MotionEvent.ACTION_DOWN) (application as RubiApp).power.touched()
        return super.dispatchTouchEvent(ev)
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        (application as RubiApp).power.touched()
        return super.dispatchKeyEvent(event)
    }

    override fun onDestroy() {
        // Left for good (backed out): now the launcher may show the chosen icon.
        if (isFinishing && !isChangingConfigurations) AppIcon.syncLauncher(this, (application as RubiApp).ui.appIcon)
        super.onDestroy()
    }

    override fun onUserLeaveHint() {
        super.onUserLeaveHint()
        media.pip.onUserLeaveHint(this)
    }

    override fun onConfigurationChanged(newConfig: Configuration) {
        super.onConfigurationChanged(newConfig)
        // Folded or unfolded: a foldable becomes a phone or a tablet. Fullscreen video keeps landscape.
        if (!media.fullscreen.value) applyDeviceOrientation()
    }

    override fun onPictureInPictureModeChanged(isInPictureInPictureMode: Boolean, newConfig: Configuration) {
        super.onPictureInPictureModeChanged(isInPictureInPictureMode, newConfig)
        media.pip.active.value = isInPictureInPictureMode
    }
}

/** Rubi 链接: `rubi://video/<BV id>` opens the video in Rubi. */
const val RUBI_SCHEME = "rubi"

fun rubiLink(bvid: String) = "$RUBI_SCHEME://video/$bvid"
