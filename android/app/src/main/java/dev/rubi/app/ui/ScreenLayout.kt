package dev.rubi.app.ui

import androidx.compose.ui.Modifier
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.LazyListScope
import android.app.Activity
import android.content.Context
import android.content.pm.ActivityInfo
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.platform.LocalConfiguration

/** How wide the window is, in Android's window size classes. */
enum class WindowWidth { COMPACT, MEDIUM, EXPANDED }

/**
 * 宽屏: what the device and the window allow. A tablet is a screen whose shorter side is at
 * least 600dp (Android's own line; an unfolded foldable counts, folded it is a phone). Only a
 * tablet gets the wide layout, and only while its window is wide enough: a phone stays on the
 * phone layout even turned sideways (fullscreen video) or in a wide split window.
 */
data class ScreenLayout(val tablet: Boolean, val width: WindowWidth) {
    /** Use the wide layout. */
    val wide: Boolean get() = tablet && width != WindowWidth.COMPACT

    /** Wide enough for two panes side by side (a tablet held sideways). */
    val twoPane: Boolean get() = tablet && width == WindowWidth.EXPANDED
}

val LocalScreenLayout = staticCompositionLocalOf { ScreenLayout(tablet = false, width = WindowWidth.COMPACT) }

/**
 * Columns of a video grid: the 单列 / 双列 choice on a phone; on a wide screen 3 upright and 4
 * held sideways (the choice does not apply there).
 */
fun ScreenLayout.videoColumns(dual: Boolean): Int = when {
    twoPane -> 4
    wide -> 3
    dual -> 2
    else -> 1
}

/** Columns of a grid that has `phone` on a phone: one or two more on a wide screen. */
fun ScreenLayout.gridColumns(phone: Int): Int = when {
    twoPane -> phone + 2
    wide -> phone + 1
    else -> phone
}

/** Columns of a list of video rows (history, 收藏夹, 稍后再看, 缓存…): two on a wide screen. */
fun ScreenLayout.rowColumns(): Int = if (wide) 2 else 1

/**
 * `items` in rows of `columns` inside a LazyColumn, so a list keeps its LazyListState (and
 * the place it is restored to) while laid out as a grid on a wide screen. A short last row
 * keeps its cells the width of the others.
 */
fun <T> LazyListScope.gridItems(items: List<T>, columns: Int, key: (T) -> Any, content: @Composable (T) -> Unit) {
    if (columns <= 1) {
        items(items, key = key) { content(it) }
        return
    }
    items(items.chunked(columns), key = { row -> key(row.first()) }) { row ->
        Row(Modifier.fillMaxWidth()) {
            row.forEach { Box(Modifier.weight(1f)) { content(it) } }
            repeat(columns - row.size) { Spacer(Modifier.weight(1f)) }
        }
    }
}

/** Shorter side, in dp, from which a screen is a tablet. */
private const val TABLET_MIN_DP = 600
/** Window widths where the classes change, in dp. */
private const val MEDIUM_MIN_DP = 600
private const val EXPANDED_MIN_DP = 840

private fun isTablet(context: Context): Boolean = context.resources.configuration.smallestScreenWidthDp >= TABLET_MIN_DP

/** The layout for the window as it is now; follows turning, folding and window resizing. */
@Composable
fun rememberScreenLayout(): ScreenLayout {
    val conf = LocalConfiguration.current
    val tablet = conf.smallestScreenWidthDp >= TABLET_MIN_DP
    val width = when {
        conf.screenWidthDp >= EXPANDED_MIN_DP -> WindowWidth.EXPANDED
        conf.screenWidthDp >= MEDIUM_MIN_DP -> WindowWidth.MEDIUM
        else -> WindowWidth.COMPACT
    }
    return remember(tablet, width) { ScreenLayout(tablet, width) }
}

/**
 * The orientation outside fullscreen video: a tablet turns freely (as the user's rotation lock
 * allows), a phone stays upright.
 */
fun Activity.applyDeviceOrientation() {
    requestedOrientation =
        if (isTablet(this)) ActivityInfo.SCREEN_ORIENTATION_FULL_USER else ActivityInfo.SCREEN_ORIENTATION_PORTRAIT
}
