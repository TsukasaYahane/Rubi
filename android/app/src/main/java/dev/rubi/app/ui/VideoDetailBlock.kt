package dev.rubi.app.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiVideoDetail
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * 视频详情 between the UP name and the 推荐 / 评论 bar. Collapsed: engagement numbers and two
 * lines of description. Expanded: full description, tags and identifiers. Tap to toggle.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun VideoDetailBlock(d: FfiVideoDetail, expanded: Boolean, onToggle: () -> Unit) {
    val c = LocalMetro.current
    Column(Modifier.fillMaxWidth().clickable(onClick = onToggle), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Meta(
            "赞 ${formatCount(d.likes)} · 币 ${formatCount(d.coins)} · 藏 ${formatCount(d.favorites)} · " +
                "评论 ${formatCount(d.replies)} · 弹幕 ${formatCount(d.danmaku)}",
            color = c.fg,
        )
        if (d.desc.isNotBlank()) {
            if (expanded) {
                SelectionContainer { Text(d.desc, fontSize = 14.sp, lineHeight = 21.sp, color = c.fg2) }
            } else {
                Text(d.desc, fontSize = 14.sp, lineHeight = 21.sp, color = c.fg2, maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
        }
        if (expanded) {
            if (d.tags.isNotEmpty()) {
                FlowRow(horizontalArrangement = Arrangement.spacedBy(14.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    d.tags.forEach { Text("#$it", fontSize = 13.sp, color = c.fg) }
                }
            }
            val time = SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.CHINA).format(Date(d.pubdate * 1000))
            Meta(listOfNotNull(time, d.zone, "${d.pages} P", formatDuration(d.durationS)).joinToString(" · "))
            SelectionContainer { Meta("${d.bvid} · av${d.aid}") }
        }
        Row(Modifier.fillMaxWidth().heightIn(min = 32.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(
                if (expanded) "收起详情 ▴" else "展开详情 ▾",
                style = MetroType.meta.copy(fontSize = 12.sp),
                color = c.fg,
            )
        }
    }
}
