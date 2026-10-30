package dev.rubi.app.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

/** Tokens from design/project/Tokens.dc.html. */
@Immutable
data class MetroColors(
    val bg: Color,
    val bg2: Color,
    val fg: Color,
    val fg2: Color,
    val line: Color,
    val spine: Color,
    val accent: Color,
)

val Cobalt = Color(0xFF0050EF)

private val Dark = MetroColors(
    bg = Color.Black, bg2 = Color(0xFF1C1C1C), fg = Color.White, fg2 = Color(0xFFA6A6A6),
    line = Color(0xFF333333), spine = Color(0xFF5C5C5C), accent = Cobalt,
)
private val Light = MetroColors(
    bg = Color.White, bg2 = Color(0xFFF2F2F2), fg = Color.Black, fg2 = Color(0xFF595959),
    line = Color(0xFFD4D4D4), spine = Color(0xFFA6A6A6), accent = Cobalt,
)

val LocalMetro = staticCompositionLocalOf { Dark }

/** Type scale: hierarchy comes from size and weight, never boxes. */
object MetroType {
    val pivot = TextStyle(fontSize = 34.sp, fontWeight = FontWeight.Light, lineHeight = 48.sp)
    val section = TextStyle(fontSize = 22.sp, fontWeight = FontWeight.Light)
    val title = TextStyle(fontSize = 20.sp, fontWeight = FontWeight.SemiBold, lineHeight = 28.sp)
    val body = TextStyle(fontSize = 14.sp, lineHeight = 20.sp)
    val meta = TextStyle(fontSize = 11.sp, fontFamily = FontFamily.Monospace)
    val brand = TextStyle(fontSize = 13.sp, fontFamily = FontFamily.Monospace, fontWeight = FontWeight.SemiBold, letterSpacing = 4.sp)
}

@Composable
fun MetroTheme(content: @Composable () -> Unit) {
    val dark = isSystemInDarkTheme()
    val c = if (dark) Dark else Light
    val scheme = if (dark) {
        darkColorScheme(primary = c.accent, background = c.bg, surface = c.bg2, onBackground = c.fg, onSurface = c.fg)
    } else {
        lightColorScheme(primary = c.accent, background = c.bg, surface = c.bg2, onBackground = c.fg, onSurface = c.fg)
    }
    CompositionLocalProvider(LocalMetro provides c) {
        MaterialTheme(colorScheme = scheme, content = content)
    }
}
