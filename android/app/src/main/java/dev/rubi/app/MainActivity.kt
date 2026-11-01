package dev.rubi.app

import android.content.Intent
import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.CompositionLocalProvider
import dev.rubi.app.playback.ACTION_LISTEN
import dev.rubi.app.playback.LocalMedia
import dev.rubi.app.ui.RubiRoot
import dev.rubi.app.ui.theme.MetroTheme

class MainActivity : ComponentActivity() {
    private val media get() = (application as RubiApp).media

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val app = application as RubiApp
        setContent { MetroTheme { CompositionLocalProvider(LocalMedia provides app.media) { RubiRoot(app) } } }
        handle(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handle(intent)
    }

    /** The playback notification: open the 听视频 page when something is being heard. */
    private fun handle(intent: Intent?) {
        if (intent?.action == ACTION_LISTEN && media.listening.current.value != null) media.listening.pageRequested.value = true
    }

    override fun onUserLeaveHint() {
        super.onUserLeaveHint()
        media.pip.onUserLeaveHint(this)
    }

    override fun onPictureInPictureModeChanged(isInPictureInPictureMode: Boolean, newConfig: Configuration) {
        super.onPictureInPictureModeChanged(isInPictureInPictureMode, newConfig)
        media.pip.active.value = isInPictureInPictureMode
    }
}
