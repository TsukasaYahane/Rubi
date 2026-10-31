package dev.rubi.app.ui

import android.annotation.SuppressLint
import android.os.Handler
import android.os.Looper
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.app.ui.theme.MetroType
import dev.rubi.core.FfiLoginCaptcha
import dev.rubi.core.FfiSolvedCaptcha

/**
 * Geetest for password / SMS login. The user solves it here by hand (slide or click); Rubi
 * only hosts Geetest's own widget and passes its result on. Nothing is solved automatically.
 */
@SuppressLint("SetJavaScriptEnabled")
@Composable
fun GeetestDialog(captcha: FfiLoginCaptcha, onSolved: (FfiSolvedCaptcha) -> Unit, onCancel: (String?) -> Unit) {
    val c = LocalMetro.current
    Dialog(onDismissRequest = { onCancel(null) }, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Column(Modifier.fillMaxWidth().padding(16.dp).background(c.bg2).padding(16.dp)) {
            Text("人机验证", style = MetroType.section, color = c.fg)
            Meta("请完成下方的滑动或点选验证")
            AndroidView(
                factory = { ctx ->
                    WebView(ctx).apply {
                        settings.javaScriptEnabled = true
                        setBackgroundColor(android.graphics.Color.WHITE)
                        val main = Handler(Looper.getMainLooper())
                        addJavascriptInterface(
                            object {
                                @JavascriptInterface
                                fun done(challenge: String, validate: String, seccode: String) = main.post {
                                    onSolved(FfiSolvedCaptcha(captcha.token, challenge, validate, seccode))
                                }

                                @JavascriptInterface
                                fun cancel() = main.post { onCancel(null) }

                                @JavascriptInterface
                                fun fail(message: String) = main.post { onCancel(message) }
                            },
                            "Rubi",
                        )
                        loadDataWithBaseURL("https://passport.bilibili.com/", page(captcha), "text/html", "utf-8", null)
                    }
                },
                onRelease = { it.destroy() },
                modifier = Modifier.fillMaxWidth().height(420.dp).padding(top = 8.dp).background(Color.White),
            )
        }
    }
}

/** Geetest's official loader, shown as a popup that opens at once. */
private fun page(c: FfiLoginCaptcha): String {
    fun js(s: String) = s.replace("\\", "\\\\").replace("'", "\\'")
    return """
<!doctype html><html><head><meta name="viewport" content="width=device-width,initial-scale=1">
<script src="https://static.geetest.com/static/tools/gt.js"></script></head>
<body style="margin:0;background:#fff">
<script>
initGeetest({gt:'${js(c.gt)}', challenge:'${js(c.challenge)}', offline:false, new_captcha:true, product:'bind', width:'100%', https:true}, function (o) {
  o.onReady(function () { o.verify(); });
  o.onSuccess(function () { var r = o.getValidate(); Rubi.done(r.geetest_challenge, r.geetest_validate, r.geetest_seccode); });
  o.onClose(function () { Rubi.cancel(); });
  o.onError(function (e) { Rubi.fail((e && (e.msg || e.error_code)) || '验证加载失败'); });
});
</script></body></html>
"""
}
