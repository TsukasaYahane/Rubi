package dev.rubi.app.ui

import dev.rubi.core.PreferencesService
import android.content.Context
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * 备份与迁移: export to / import from a zip the user picks with the system file picker.
 * The core reads and writes plain paths, so the zip goes through a temp file in the cache.
 * Login credentials are never part of the bundle.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun BackupSheet(preferences: PreferencesService, context: Context, onDismiss: () -> Unit, onDone: (String) -> Unit) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var withHistory by remember { mutableStateOf(false) }
    var pending by remember { mutableStateOf<File?>(null) }
    var note by remember { mutableStateOf("") }

    val exporter = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("application/zip")) { uri: Uri? ->
        if (uri == null) return@rememberLauncherForActivityResult
        scope.launch {
            note = runCatching {
                withContext(Dispatchers.IO) {
                    val tmp = File.createTempFile("rubi-export", ".zip", context.cacheDir)
                    try {
                        preferences.exportTo(tmp.absolutePath, withHistory)
                        context.contentResolver.openOutputStream(uri)!!.use { out -> tmp.inputStream().use { it.copyTo(out) } }
                    } finally {
                        tmp.delete()
                    }
                }
                "已导出（不含登录凭证）"
            }.getOrElse { "导出失败：${it.message}" }
            onDone(note)
        }
    }
    val importer = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri: Uri? ->
        if (uri == null) return@rememberLauncherForActivityResult
        scope.launch {
            runCatching {
                withContext(Dispatchers.IO) {
                    val tmp = File.createTempFile("rubi-import", ".zip", context.cacheDir)
                    context.contentResolver.openInputStream(uri)!!.use { input -> tmp.outputStream().use { input.copyTo(it) } }
                    tmp
                }
            }.onSuccess { pending = it }.onFailure { note = "读取文件失败：${it.message}" }
        }
    }

    ModalBottomSheet(
        onDismissRequest = { pending?.delete(); onDismiss() },
        containerColor = c.bg2,
        shape = RectangleShape,
        dragHandle = null,
    ) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text("备份与迁移", style = MetroType.section, color = c.fg)
            val file = pending
            if (file == null) {
                Meta("过滤规则、推荐权重、兴趣画像与设置打包为 zip，可在另一台设备导入。登录凭证不会导出。")
                Row(
                    Modifier.fillMaxWidth().heightIn(min = 44.dp).clickable { withHistory = !withHistory },
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Text(if (withHistory) "■" else "□", fontSize = 15.sp, color = if (withHistory) c.accent else c.fg2)
                    Text("包含本地观看历史（推荐信号）", fontSize = 15.sp, color = if (withHistory) c.fg else c.fg2)
                }
                Row(horizontalArrangement = Arrangement.spacedBy(20.dp), verticalAlignment = Alignment.CenterVertically) {
                    AccentButton("导出…") {
                        val day = SimpleDateFormat("yyyyMMdd", Locale.ROOT).format(Date())
                        exporter.launch("rubi-$day.zip")
                    }
                    TextAction("导入…", onClick = { importer.launch(arrayOf("application/zip", "application/octet-stream")) })
                }
            } else {
                Text("导入将覆盖当前的过滤规则、推荐权重、兴趣画像与设置。", fontSize = 15.sp, color = c.fg)
                Meta("文件中的观看历史会追加到本机记录；登录状态不受影响。")
                Row(horizontalArrangement = Arrangement.spacedBy(20.dp), verticalAlignment = Alignment.CenterVertically) {
                    AccentButton("确认导入") {
                        scope.launch {
                            note = runCatching { withContext(Dispatchers.IO) { preferences.importFrom(file.absolutePath) }; "已导入" }
                                .getOrElse { "导入失败：${it.message}" }
                            file.delete()
                            pending = null
                            onDone(note)
                        }
                    }
                    TextAction("取消", onClick = { file.delete(); pending = null }, dim = true)
                }
            }
            if (note.isNotEmpty()) Meta(note)
            Row(Modifier.padding(bottom = 12.dp)) {}
        }
    }
}
