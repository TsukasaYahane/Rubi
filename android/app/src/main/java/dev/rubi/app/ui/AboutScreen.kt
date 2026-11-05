package dev.rubi.app.ui

import dev.rubi.app.Appearance
import android.content.Intent
import android.net.Uri
import android.os.Build
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.background
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.ui.theme.LocalMetro
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

/** Where Rubi is published: its releases and issues. */
private const val PROJECT = "https://github.com/TsukasaYahane/Rubi"
private const val LATEST_RELEASE = "https://api.github.com/repos/TsukasaYahane/Rubi/releases/latest"

private val FAQ = listOf(
    "为什么有些视频不显示？" to "Rubi 会按过滤规则拦下广告、推广和你屏蔽的内容。被拦下的都能在「我的 → 被过滤内容」里看到原因。",
    "能看大会员 / 付费内容吗？" to "不能。Rubi 只提供免费内容，画质最高到免费可用的档位；付费内容请在 B 站网站或官方 App 开通。",
    "登录安全吗？" to "账号凭证只加密保存在本机，不会上传或导出；密码只发给 B 站登录接口，Rubi 不保存。",
    "推荐是怎么算的？" to "推荐在本机计算：结合你的观看、点赞、不感兴趣等反馈。首页卡片上的「为什么推荐」能看到每一项得分。",
)

private const val RULES_HELP = "过滤规则按顺序判断每张卡片、每条评论和弹幕：广告与推广卡片、商品评论、你屏蔽的 UP 与关键词会被拦下，" +
    "拦下的内容记在「被过滤内容」里。规则可以在「设置 → 内容过滤 / 屏蔽关键词」里开关和增减；导出备份时规则一并带上。"

private val LIBRARIES = listOf(
    "Jetpack Compose · Material 3（Apache-2.0）",
    "AndroidX Media3 / ExoPlayer（Apache-2.0）",
    "Coil（Apache-2.0）",
    "ZXing（Apache-2.0）",
    "UniFFI（MPL-2.0）",
    "reqwest · tokio · tokio-tungstenite（MIT / Apache-2.0）",
    "rusqlite / SQLite（MIT / Public Domain）",
    "serde · serde_json · prost（MIT / Apache-2.0）",
)

/**
 * 关于 Rubi: the icon and version (always on the left), checking for a newer release, the project
 * page, the 声明 with open-source licences, and 帮助.
 */
@Composable
fun AboutScreen(appearance: Appearance, nav: Nav) {
    val c = LocalMetro.current
    val ctx = LocalContext.current
    val scope = rememberCoroutineScope()
    val version = remember {
        runCatching {
            val info = ctx.packageManager.getPackageInfo(ctx.packageName, 0)
            val build = if (Build.VERSION.SDK_INT >= 28) info.longVersionCode else @Suppress("DEPRECATION") info.versionCode.toLong()
            (info.versionName ?: "?") to build
        }.getOrDefault("?" to 0L)
    }
    var note by remember { mutableStateOf("") }
    val open = remember { mutableStateMapOf<String, Boolean>() }
    fun browse(url: String) = runCatching { ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url))) }

    Column(Modifier.fillMaxSize()) {
        PageHeader(nav, "关于 Rubi")
        LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = 24.dp)) {
            item {
                Row(Modifier.fillMaxWidth().padding(top = 14.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                    RubiLogo(64.dp, appearance.appIcon.value)
                    Column {
                        Text("RUBI", fontSize = 22.sp, letterSpacing = 4.sp, fontWeight = FontWeight.Light, color = c.fg)
                        Meta("版本 ${version.first}（构建 ${version.second}）")
                    }
                }
                Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    SmallButton("检查更新", c.accent) {
                        note = "正在检查…"
                        scope.launch { note = checkUpdate(version.first) }
                    }
                    SmallButton("项目发布页", null) { browse("$PROJECT/releases") }
                }
                if (note.isNotEmpty()) Meta(note, Modifier.padding(top = 6.dp), color = c.accent)

                AboutSection("声明")
                Text(
                    "Rubi 是第三方哔哩哔哩客户端，与哔哩哔哩官方无关，内容版权归原作者与平台所有。Rubi 不提供、不绕过任何付费内容与鉴权，" +
                        "离线缓存遵守官方规则。使用时请遵守哔哩哔哩用户协议。",
                    fontSize = 13.sp,
                    lineHeight = 21.sp,
                    color = c.fg2,
                )
                Expandable("开源许可", open["lic"] == true, { open["lic"] = open["lic"] != true }) {
                    LIBRARIES.forEach { Meta(it, Modifier.padding(vertical = 2.dp)) }
                }

                AboutSection("帮助")
                FAQ.forEach { (q, a) ->
                    Expandable(q, open[q] == true, { open[q] = open[q] != true }) {
                        Text(a, fontSize = 13.sp, lineHeight = 21.sp, color = c.fg2)
                    }
                }
                Expandable("过滤规则说明", open["rules"] == true, { open["rules"] = open["rules"] != true }) {
                    Text(RULES_HELP, fontSize = 13.sp, lineHeight = 21.sp, color = c.fg2)
                }
                Row(Modifier.fillMaxWidth().clickable { browse("$PROJECT/issues") }.padding(vertical = 12.dp)) {
                    Text("反馈问题", fontSize = 14.sp, color = c.fg, modifier = Modifier.weight(1f))
                    Meta("在项目页提 Issue ›")
                }
            }
        }
    }
}

@Composable
private fun AboutSection(title: String) {
    Text(title, fontSize = 20.sp, fontWeight = FontWeight.Light, color = LocalMetro.current.fg, modifier = Modifier.padding(top = 20.dp, bottom = 6.dp))
}

@Composable
private fun Expandable(title: String, opened: Boolean, onToggle: () -> Unit, body: @Composable () -> Unit) {
    val c = LocalMetro.current
    Box(Modifier.fillMaxWidth().height(1.dp).background(c.line))
    Row(Modifier.fillMaxWidth().clickable(onClick = onToggle).padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(title, fontSize = 14.sp, color = c.fg, modifier = Modifier.weight(1f))
        Text(if (opened) "−" else "+", fontSize = 16.sp, color = c.fg2)
    }
    if (opened) Column(Modifier.padding(bottom = 10.dp)) { body() }
}

/** Asks GitHub for the latest release and says how this version compares. */
private suspend fun checkUpdate(current: String): String = withContext(Dispatchers.IO) {
    runCatching {
        val conn = (URL(LATEST_RELEASE).openConnection() as HttpURLConnection).apply {
            connectTimeout = 8_000
            readTimeout = 8_000
            setRequestProperty("Accept", "application/vnd.github+json")
        }
        try {
            if (conn.responseCode != 200) return@runCatching "暂时无法检查更新（发布页未公开或暂无发布）"
            val tag = JSONObject(conn.inputStream.bufferedReader().readText()).optString("tag_name").removePrefix("v")
            when {
                tag.isEmpty() -> "暂时无法检查更新"
                tag == current -> "已是最新版本"
                else -> "有新版本 $tag，可在项目发布页下载"
            }
        } finally {
            conn.disconnect()
        }
    }.getOrElse { "检查失败：${it.message}" }
}
