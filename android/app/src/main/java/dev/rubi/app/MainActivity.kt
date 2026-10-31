package dev.rubi.app

import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import dev.rubi.app.playback.Pip
import dev.rubi.app.ui.RubiRoot
import dev.rubi.app.ui.theme.MetroTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val app = application as RubiApp
        setContent { MetroTheme { RubiRoot(app) } }
    }

    override fun onUserLeaveHint() {
        super.onUserLeaveHint()
        Pip.onUserLeaveHint(this)
    }

    override fun onPictureInPictureModeChanged(isInPictureInPictureMode: Boolean, newConfig: Configuration) {
        super.onPictureInPictureModeChanged(isInPictureInPictureMode, newConfig)
        Pip.active.value = isInPictureInPictureMode
    }
}
