package dev.rubi.app.ui

import android.content.ComponentName
import android.content.Context
import android.content.pm.PackageManager
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Path
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.drawscope.drawIntoCanvas
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.Dp
import dev.rubi.app.ui.theme.Cobalt

/**
 * Rubi's icon, two to choose from in 设置 → 外观: 小电视 (the default: a TV with two blocks on top
 * and a play mark cut out) and R 字 (an R of blocks). Both are a Cobalt square, no rounding; the
 * launcher icon, the 关于 page and the share card all follow the choice.
 */
object AppIcon {
    const val TV = 0
    const val R = 1
    val LABELS = listOf("小电视", "R 字")

    private const val TV_ALIAS = "dev.rubi.app.LauncherTv"
    private const val R_ALIAS = "dev.rubi.app.LauncherR"

    /**
     * Puts the chosen icon on the launcher if it is not there yet. Called when Rubi is left or
     * started, never while it is in use: switching the entry under a running app makes some
     * launchers close it.
     */
    fun syncLauncher(context: Context, which: Int) {
        val pm = context.packageManager
        val shown = pm.getComponentEnabledSetting(ComponentName(context, R_ALIAS)) == PackageManager.COMPONENT_ENABLED_STATE_ENABLED
        if (shown != (which == R)) applyLauncher(context, which)
    }

    /** Shows `which` on the launcher (one entry on, the other off; the app keeps running). */
    private fun applyLauncher(context: Context, which: Int) {
        val pm = context.packageManager
        fun set(alias: String, on: Boolean) = pm.setComponentEnabledSetting(
            ComponentName(context, alias),
            if (on) PackageManager.COMPONENT_ENABLED_STATE_ENABLED else PackageManager.COMPONENT_ENABLED_STATE_DISABLED,
            PackageManager.DONT_KILL_APP,
        )
        // The new one first, so the launcher is never left without an entry.
        if (which == R) {
            set(R_ALIAS, true)
            set(TV_ALIAS, false)
        } else {
            set(TV_ALIAS, true)
            set(R_ALIAS, false)
        }
    }
}

/** Draws icon `which` as a `size`-pixel square at `x`, `y`: the same shapes as the launcher's. */
fun drawRubiLogo(canvas: Canvas, x: Float, y: Float, size: Float, which: Int) {
    val u = size / 100f
    canvas.drawRect(x, y, x + size, y + size, Paint().apply { color = Cobalt.toArgb() })
    val white = Paint(Paint.ANTI_ALIAS_FLAG).apply { color = 0xFFFFFFFF.toInt() }
    fun block(l: Float, t: Float, w: Float, h: Float) = canvas.drawRect(x + l * u, y + t * u, x + (l + w) * u, y + (t + h) * u, white)
    fun shape(vararg p: Float) = Path().apply {
        moveTo(x + p[0] * u, y + p[1] * u)
        var i = 2
        while (i < p.size) {
            lineTo(x + p[i] * u, y + p[i + 1] * u)
            i += 2
        }
        close()
    }
    if (which == AppIcon.R) {
        block(28f, 22f, 12f, 56f)
        block(28f, 22f, 40f, 12f)
        block(56f, 22f, 12f, 32f)
        block(28f, 44f, 40f, 10f)
        canvas.drawPath(shape(44f, 54f, 58f, 54f, 74f, 78f, 60f, 78f), white)
    } else {
        block(28f, 20f, 10f, 12f)
        block(62f, 20f, 10f, 12f)
        block(18f, 30f, 64f, 48f)
        // The play mark, cut out in Cobalt.
        canvas.drawPath(shape(42f, 42f, 42f, 66f, 62f, 54f), Paint(Paint.ANTI_ALIAS_FLAG).apply { color = Cobalt.toArgb() })
    }
}

/** The chosen icon as a square of `size`. */
@Composable
fun RubiLogo(size: Dp, which: Int) {
    androidx.compose.foundation.Canvas(Modifier.size(size)) {
        drawIntoCanvas { drawRubiLogo(it.nativeCanvas, 0f, 0f, this.size.width, which) }
    }
}
