package dev.rubi.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import dev.rubi.app.RubiApp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiUpOrder
import dev.rubi.core.FfiUpProfile
import dev.rubi.core.FfiVideoCard
import dev.rubi.core.RubiException
import kotlinx.coroutines.launch

private val ORDERS = listOf("最新" to FfiUpOrder.NEWEST, "最多播放" to FfiUpOrder.MOST_VIEWED, "最多收藏" to FfiUpOrder.MOST_FAVORITED)

/** An UP's page: profile, follow, and uploads in two columns (充电专属 videos left out). */
@Composable
fun UpScreen(app: RubiApp, nav: Nav, mid: ULong, name: String) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    var profile by remember { mutableStateOf<FfiUpProfile?>(null) }
    var order by remember { mutableStateOf(FfiUpOrder.NEWEST) }
    val cards = remember { mutableStateListOf<FfiVideoCard>() }
    var page by remember { mutableStateOf(1u) }
    var hasMore by remember { mutableStateOf(true) }
    var loading by remember { mutableStateOf(false) }
    var hiddenCharging by remember { mutableStateOf(0u) }
    var error by remember { mutableStateOf<String?>(null) }
    var note by remember { mutableStateOf("") }
    var confirmUnfollow by remember { mutableStateOf(false) }
    val grid = rememberLazyGridState()

    suspend fun more() {
        if (loading || !hasMore) return
        loading = true
        val asked = order
        runCatching { app.core.upVideos(mid, page, asked) }
            .onSuccess { got ->
                if (asked != order) return@onSuccess
                cards.addAll(got.items.filter { n -> cards.none { it.bvid == n.bvid } })
                hiddenCharging += got.hiddenCharging
                hasMore = got.hasMore
                page++
                error = null
            }
            .onFailure {
                error = if (it is RubiException.RiskControl) "被 B 站风控拦截（请求过于频繁或未登录），稍等片刻再重试" else "加载失败：${it.message}"
            }
        loading = false
    }

    LaunchedEffect(mid, nav.stack.size) { profile = runCatching { app.core.upProfile(mid) }.getOrNull() ?: profile }
    LaunchedEffect(mid, order) {
        cards.clear(); page = 1u; hasMore = true; hiddenCharging = 0u
        more()
    }
    val nearEnd by remember { derivedStateOf { (grid.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0) >= cards.size - 4 } }
    LaunchedEffect(nearEnd, cards.size) { if (nearEnd && cards.isNotEmpty() && error == null) more() }

    fun toggleFollow(p: FfiUpProfile) {
        if (p.followed && !confirmUnfollow) {
            confirmUnfollow = true
            note = "再点一次取消关注"
            return
        }
        confirmUnfollow = false
        val on = !p.followed
        profile = p.copy(followed = on, fans = if (on) p.fans + 1u else p.fans - minOf(p.fans, 1uL))
        scope.launch {
            runCatching { app.core.followMid(mid, p.name, on) }
                .onSuccess { note = if (on) "已关注" else "已取消关注" }
                .onFailure {
                    profile = p
                    note = if (it is RubiException.NotLoggedIn) "登录后才能关注" else "操作失败：${it.message}"
                }
        }
    }

    Column(Modifier.fillMaxSize()) {
        Row(Modifier.fillMaxWidth().padding(start = 4.dp, end = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clickable { nav.pop() }, contentAlignment = Alignment.Center) { GlyphIcon(Glyph.Back) }
            Meta("UP 主 · mid $mid")
        }
        LazyVerticalGrid(
            columns = GridCells.Fixed(2),
            state = grid,
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = 28.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalArrangement = Arrangement.spacedBy(22.dp),
            modifier = Modifier.fillMaxSize(),
        ) {
            item(span = { GridItemSpan(maxLineSpan) }) {
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    val p = profile
                    Row(horizontalArrangement = Arrangement.spacedBy(14.dp), verticalAlignment = Alignment.CenterVertically) {
                        AsyncImage(
                            model = p?.face,
                            contentDescription = null,
                            contentScale = ContentScale.Crop,
                            modifier = Modifier.size(64.dp).background(c.bg2),
                        )
                        Column(Modifier.weight(1f)) {
                            Text(p?.name ?: name, style = MetroType.pivot.copy(fontSize = 28.sp, lineHeight = 36.sp), color = c.fg, maxLines = 1)
                            if (p != null) Meta(listOf("Lv${p.level}", p.official).filter { it.isNotBlank() }.joinToString(" · "))
                        }
                    }
                    if (p != null) {
                        if (p.sign.isNotBlank()) Text(p.sign, fontSize = 14.sp, lineHeight = 21.sp, color = c.fg2)
                        Row(horizontalArrangement = Arrangement.spacedBy(28.dp)) {
                            Stat("粉丝", formatCount(p.fans))
                            Stat("关注", formatCount(p.followings))
                            Stat("投稿", formatCount(p.archives))
                        }
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                            if (p.followed) TextAction(if (confirmUnfollow) "确认取消关注" else "已关注", onClick = { toggleFollow(p) }, dim = !confirmUnfollow)
                            else AccentButton("关注") { toggleFollow(p) }
                            if (note.isNotEmpty()) Meta(note)
                        }
                    }
                    Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                        ORDERS.forEach { (label, o) ->
                            val on = o == order
                            Text(
                                label,
                                fontSize = 16.sp,
                                fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
                                color = if (on) c.fg else c.fg2,
                                modifier = Modifier.heightIn(min = 44.dp).clickable { order = o }.padding(top = 10.dp),
                            )
                        }
                    }
                    if (hiddenCharging > 0u) Meta("已隐藏充电专属视频 $hiddenCharging 个")
                    error?.let { e ->
                        Text(e, color = c.fg2, fontSize = 13.sp)
                        TextAction("重试", onClick = { scope.launch { more() } })
                    }
                }
            }
            items(cards, key = { it.bvid }) { card ->
                VideoCardView(card, big = false, onOpen = { nav.push(Screen.Player(card.bvid, card.title)) })
            }
            if (!hasMore && cards.isNotEmpty()) {
                item(span = { GridItemSpan(maxLineSpan) }) { Meta("没有更多了") }
            }
        }
    }
}

@Composable
private fun Stat(label: String, value: String) {
    val c = LocalMetro.current
    Column {
        Text(value, fontSize = 20.sp, fontWeight = FontWeight.SemiBold, color = c.fg)
        Meta(label)
    }
}
