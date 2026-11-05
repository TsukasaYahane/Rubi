package dev.rubi.app.ui

import dev.rubi.core.AccountService
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.rubi.app.data.PasswordCrypto
import dev.rubi.app.ui.theme.LocalMetro
import dev.rubi.core.FfiLoginCaptcha
import dev.rubi.core.FfiSolvedCaptcha
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

/** A login error in words, for the codes the passport endpoints document. */
private fun loginError(e: Throwable): String {
    val msg = e.message.orEmpty()
    val known = listOf(
        "-629" to "账号或密码错误",
        "-105" to "人机验证未通过，请重试",
        "-2100" to "该账号需要手机或邮箱二次验证，请改用短信或扫码登录",
        "-662" to "提交超时，请重试",
        "2400" to "登录密钥失效，请重试",
        "2406" to "验证服务出错，请稍后再试",
        "86000" to "密码加密失败，请重试",
        "1002" to "手机号格式不正确",
        "86203" to "今日短信发送次数已达上限",
        "1003" to "验证码已发送，请稍后再试",
        "1025" to "该手机号已被封禁",
        "1006" to "短信验证码错误",
        "1007" to "短信验证码已过期，请重新获取",
    )
    return known.firstOrNull { (code, _) -> "error $code:" in msg }?.second ?: "登录失败：$msg"
}

/** Runs the Geetest step, then `next` with the solved captcha. */
private class CaptchaStep {
    var pending by mutableStateOf<FfiLoginCaptcha?>(null)
    var then: (suspend (FfiSolvedCaptcha) -> Unit)? = null
}

@Composable
private fun CaptchaHost(step: CaptchaStep, onError: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    step.pending?.let { cap ->
        GeetestDialog(
            cap,
            onSolved = { solved ->
                step.pending = null
                val next = step.then
                step.then = null
                if (next != null) scope.launch { next(solved) }
            },
            onCancel = { reason ->
                step.pending = null
                step.then = null
                if (reason != null) onError("人机验证失败：$reason")
            },
        )
    }
}

@Composable
fun SmsLogin(account: AccountService, onLoggedIn: (String) -> Unit) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val captcha = remember { CaptchaStep() }
    var cid by remember { mutableStateOf("86") }
    var tel by remember { mutableStateOf("") }
    var code by remember { mutableStateOf("") }
    var captchaKey by remember { mutableStateOf<String?>(null) }
    var cooldown by remember { mutableIntStateOf(0) }
    var note by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }

    LaunchedEffect(cooldown > 0) {
        while (cooldown > 0) { delay(1000); cooldown-- }
    }

    fun requestCode() {
        if (tel.isBlank() || busy || cooldown > 0) return
        busy = true
        note = "正在获取人机验证…"
        scope.launch {
            runCatching { account.loginCaptcha() }
                .onSuccess { cap ->
                    note = ""
                    captcha.then = { solved ->
                        runCatching { account.smsSend(cid.toUIntOrNull() ?: 86u, tel.trim(), solved) }
                            .onSuccess { captchaKey = it; cooldown = 60; note = "验证码已发送，5 分钟内有效" }
                            .onFailure { note = loginError(it) }
                        busy = false
                    }
                    captcha.pending = cap
                }
                .onFailure { note = loginError(it); busy = false }
        }
    }

    fun login() {
        val key = captchaKey ?: run { note = "请先获取验证码"; return }
        if (code.isBlank() || busy) return
        busy = true
        scope.launch {
            runCatching { account.smsLogin(cid.toUIntOrNull() ?: 86u, tel.trim(), code.trim(), key) }
                .onSuccess(onLoggedIn)
                .onFailure { note = loginError(it) }
            busy = false
        }
    }

    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.Bottom) {
            LineField(cid, { cid = it.filter(Char::isDigit).take(4) }, "区号", KeyboardType.Number, Modifier.width(64.dp), prefix = "+")
            LineField(tel, { tel = it.filter(Char::isDigit).take(15) }, "手机号", KeyboardType.Phone, Modifier.weight(1f))
        }
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.Bottom) {
            LineField(code, { code = it.filter(Char::isDigit).take(6) }, "短信验证码", KeyboardType.Number, Modifier.weight(1f))
            TextAction(if (cooldown > 0) "${cooldown}s 后重发" else "获取验证码", onClick = { requestCode() }, dim = cooldown > 0 || tel.isBlank())
        }
        Row(Modifier.padding(top = 8.dp)) { AccentButton(if (busy) "请稍候…" else "登录") { login() } }
        if (note.isNotEmpty()) Text(note, fontSize = 13.sp, color = c.fg2)
        Meta("发送前需完成一次人机验证；凭证经 Keystore 加密仅存本机")
    }
    CaptchaHost(captcha) { note = it; busy = false }
}

@Composable
fun PasswordLogin(account: AccountService, onLoggedIn: (String) -> Unit) {
    val c = LocalMetro.current
    val scope = rememberCoroutineScope()
    val captcha = remember { CaptchaStep() }
    var user by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var note by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }

    fun login() {
        if (user.isBlank() || password.isEmpty() || busy) return
        busy = true
        note = "正在获取人机验证…"
        scope.launch {
            runCatching { account.loginCaptcha() }
                .onSuccess { cap ->
                    note = ""
                    captcha.then = { solved ->
                        // The salt lives 20 s: fetch it only after the captcha is done.
                        runCatching {
                            val key = account.loginKey()
                            val encrypted = PasswordCrypto.encrypt(key.hash, key.publicKeyPem, password)
                            account.passwordLogin(user.trim(), encrypted, solved)
                        }
                            .onSuccess { password = ""; onLoggedIn(it) }
                            .onFailure { note = loginError(it) }
                        busy = false
                    }
                    captcha.pending = cap
                }
                .onFailure { note = loginError(it); busy = false }
        }
    }

    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
        LineField(user, { user = it.trim().take(64) }, "手机号或邮箱", KeyboardType.Email, Modifier.fillMaxWidth())
        LineField(password, { password = it.take(64) }, "密码", KeyboardType.Password, Modifier.fillMaxWidth(), secret = true)
        Row(Modifier.padding(top = 8.dp)) { AccentButton(if (busy) "请稍候…" else "登录") { login() } }
        if (note.isNotEmpty()) Text(note, fontSize = 13.sp, color = c.fg2)
        Meta("密码在本机用 B 站公钥加密后提交，不保存、不上传到其他地方；登录前需完成一次人机验证")
    }
    CaptchaHost(captcha) { note = it; busy = false }
}

/** A Metro text field: text over a hairline, no box. */
@Composable
private fun LineField(
    value: String,
    onChange: (String) -> Unit,
    hint: String,
    keyboard: KeyboardType,
    modifier: Modifier,
    secret: Boolean = false,
    prefix: String = "",
) {
    val c = LocalMetro.current
    Column(modifier) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            if (prefix.isNotEmpty()) Text(prefix, fontSize = 17.sp, color = c.fg2)
            BasicTextField(
                value = value,
                onValueChange = onChange,
                singleLine = true,
                textStyle = TextStyle(fontSize = 17.sp, color = c.fg),
                cursorBrush = SolidColor(c.accent),
                keyboardOptions = KeyboardOptions(keyboardType = keyboard),
                visualTransformation = if (secret) PasswordVisualTransformation() else VisualTransformation.None,
                modifier = Modifier.weight(1f).heightIn(min = 40.dp).padding(top = 8.dp),
                decorationBox = { inner ->
                    if (value.isEmpty()) Text(hint, fontSize = 17.sp, color = c.fg2)
                    inner()
                },
            )
        }
        HorizontalDivider(color = c.line)
    }
}
