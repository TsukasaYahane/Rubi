package dev.rubi.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiFollowedUp
import dev.rubi.core.FfiFollowing
import dev.rubi.core.FfiFollowings

/**
 * 全部关注: every followed UP, those who posted lately first with when, the rest saying how long
 * they have been quiet. Searchable; picking one goes back to 关注 showing that UP's posts.
 */
@Composable
fun FollowingsScreen(app: RubiApp, nav: Nav, follow: FollowState) {
    val c = LocalMetro.current
    var all by remember { mutableStateOf<FfiFollowings?>(null) }
    var error by remember { mutableStateOf<Throwable?>(null) }
    var round by remember { mutableStateOf(0) }
    var query by remember { mutableStateOf("") }
    LaunchedEffect(round) {
        error = null
        runCatching { app.core.allFollowings() }.onSuccess { all = it }.onFailure { error = it }
    }

    Column(Modifier.fillMaxSize()) {
        val count = all?.ups?.size
        PageHeader(nav, if (count != null) "全部关注 ($count)" else "全部关注")
        Box(Modifier.padding(horizontal = 16.dp, vertical = 8.dp).fillMaxWidth().heightIn(min = 40.dp).border(1.dp, c.line).padding(horizontal = 10.dp, vertical = 10.dp)) {
            if (query.isEmpty()) Text("搜索关注的 UP", fontSize = 14.sp, color = c.fg2)
            BasicTextField(
                value = query,
                onValueChange = { query = it },
                singleLine = true,
                textStyle = TextStyle(color = c.fg, fontSize = 14.sp),
                cursorBrush = SolidColor(c.accent),
                modifier = Modifier.fillMaxWidth(),
            )
        }
        val a = all
        if (a == null) {
            Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                val e = error
                if (e == null) Meta("加载中…（读取关注列表和最近的动态）") else {
                    Meta(errorText(e))
                    TextAction("重试", onClick = { round++ })
                }
            }
            return@Column
        }
        Meta("按最近更新排序 · 点选查看该 UP 的动态", Modifier.padding(start = 16.dp, end = 16.dp, bottom = 6.dp))
        val shown = a.ups.filter { query.isBlank() || it.name.contains(query.trim(), ignoreCase = true) }
        if (shown.isEmpty()) Meta("没有找到这个 UP", Modifier.padding(16.dp))
        LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 24.dp)) {
            items(shown, key = { it.mid.toString() }) { up ->
                FollowingRow(up, quietSince = a.checkedSince) {
                    follow.show(FfiFollowedUp(up.mid, up.name, up.face, up.hasUpdate))
                    nav.pop()
                }
            }
        }
    }
}

@Composable
private fun FollowingRow(up: FfiFollowing, quietSince: Long, onClick: () -> Unit) {
    val c = LocalMetro.current
    Row(
        Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Box {
            AsyncImage(model = up.face, contentDescription = null, contentScale = ContentScale.Crop, modifier = Modifier.size(40.dp).background(c.bg2))
            if (up.hasUpdate) Box(Modifier.size(8.dp).background(c.accent).align(Alignment.TopEnd))
        }
        Column(Modifier.weight(1f)) {
            Text(up.name, fontSize = 15.sp, color = c.fg, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Meta(updatedText(up.lastPostAt, quietSince))
        }
        Text("›", fontSize = 16.sp, color = c.fg2)
    }
}

/** 「3 小时前更新」, or for an UP not seen posting, how long that covers: 「5 天内没有更新」. */
private fun updatedText(lastPostAt: Long, quietSince: Long): String {
    val now = System.currentTimeMillis() / 1000
    if (lastPostAt <= 0) {
        val days = ((now - quietSince) / 86_400).coerceAtLeast(1)
        return "$days 天内没有更新"
    }
    val s = (now - lastPostAt).coerceAtLeast(0)
    return when {
        s < 3_600 -> "${maxOf(1, s / 60)} 分钟前更新"
        s < 86_400 -> "${s / 3_600} 小时前更新"
        else -> "${s / 86_400} 天前更新"
    }
}
