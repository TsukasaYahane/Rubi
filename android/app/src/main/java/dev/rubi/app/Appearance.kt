package dev.rubi.app

import androidx.compose.runtime.mutableStateOf
import dev.rubi.app.data.UiPrefs

/**
 * 外观 as the app shows it now: read by the theme at once when changed in 设置, and saved in
 * [UiPrefs] by whoever changes it.
 */
class Appearance(ui: UiPrefs) {
    /** 0 跟随系统, 1 浅色, 2 深色. */
    val themeMode = mutableStateOf(ui.themeMode)
    /** 动效速度. */
    val motion = mutableStateOf(ui.motion)
    val motionStyle = mutableStateOf(ui.motionStyle)
    /** 竖屏视频封面: 0 适应, 1 填充 ([UiPrefs.portraitCover]). */
    val portraitCover = mutableStateOf(ui.portraitCover)
    /** 界面风格: 0 图形, 1 文字 ([UiPrefs.uiStyle]). */
    val uiStyle = mutableStateOf(ui.uiStyle)
    /** 应用图标 (小电视 / R 字): the launcher, 关于 and the share card follow it. */
    val appIcon = mutableStateOf(ui.appIcon)
}
