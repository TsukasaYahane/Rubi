//! 登录的另外两条路: 账密 and 手机号, drawn inside the 我的 page itself rather than in a sheet
//! of their own. The page owns the frame; this owns the two forms and what they call.

use std::cell::RefCell;

use base64::Engine as _;
use rubi_core::{Session, SolvedCaptcha};
use windows_core::Result;

use crate::ui::shell::{shell, toast};
use crate::ui::{captcha, named, post, set_text};
use crate::xaml::{IFrameworkElement, ITextBox};

thread_local! {
    /// The step the slider is standing in front of.
    static WANTED: RefCell<Option<Wanted>> = const { RefCell::new(None) };
    /// The code the user typed, read on the ui thread before the call is handed off.
    static CODE: RefCell<String> = const { RefCell::new(String::new()) };
    /// What 发短信 answered with. The login call takes this instead of asking for a second
    /// slider: the endpoint hands it out once, with the text, and wants it back verbatim.
    static SMS_KEY: RefCell<String> = const { RefCell::new(String::new()) };
}

/// The step the slider is standing in front of.
#[derive(Clone)]
enum Wanted {
    Password { user: String, pass: String },
    Sms { tel: String },
}

/// 账密: the slider first, then the password goes out encrypted.
pub fn ask_password(root: &IFrameworkElement) -> Result<()> {
    let user = named::<ITextBox>(root, "User")?.Text()?.trim().to_owned();
    let pass = named::<crate::xaml::IPasswordBox>(root, "Pass")?.Password()?.to_string();
    if user.is_empty() || pass.is_empty() {
        return set_text(root, "Status", "账号和密码都要填");
    }
    WANTED.with_borrow_mut(|slot| *slot = Some(Wanted::Password { user, pass }));
    captcha::then(finish_password);
    captcha::start();
    Ok(())
}

/// 获取验证码: the slider, then the text. The endpoint answers with the key 登录 needs, so this
/// is the only step that asks for one.
pub fn send_code(root: &IFrameworkElement) -> Result<()> {
    let tel = named::<ITextBox>(root, "Tel")?.Text()?.trim().to_owned();
    if tel.is_empty() {
        return set_text(root, "Status", "先填手机号");
    }
    WANTED.with_borrow_mut(|slot| *slot = Some(Wanted::Sms { tel }));
    captcha::then(finish_sms);
    captcha::start();
    Ok(())
}

/// 登录 with the text's code: no second slider, the key from 发送验证码 is what the call wants.
pub fn sign_in(root: &IFrameworkElement) -> Result<()> {
    let tel = named::<ITextBox>(root, "Tel")?.Text()?.trim().to_owned();
    let code = named::<ITextBox>(root, "SmsCode")?.Text()?.trim().to_owned();
    if tel.is_empty() || code.is_empty() {
        return set_text(root, "Status", "手机号和验证码都要填");
    }
    let key = SMS_KEY.with_borrow(Clone::clone);
    if key.is_empty() {
        return set_text(root, "Status", "先点获取验证码");
    }
    let Some(shell) = shell() else { return Ok(()) };
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    let dispatcher = shell.dispatcher.clone();
    let inner = backend.clone();
    backend.spawn(async move {
        match inner.services.account.sms_login(86, &tel, &code, &key).await {
            Ok(session) => post(&dispatcher, move || saved(session)),
            Err(e) => post(&dispatcher, move || {
                let _ = toast(&format!("登录失败：{e}"));
            }),
        }
    });
    Ok(())
}

/// What the slider approved, as the login call wants it.
fn settled() -> Option<SolvedCaptcha> {
    let answer = captcha::passed()?;
    // The login token is the endpoint's, not the widget's: sending the challenge instead is what
    // the server reports as 验证码错误.
    let token = captcha::token()?;
    let field = |name: &str| -> String {
        answer
            .split('&')
            .find_map(|pair| pair.strip_prefix(&format!("{name}=")))
            .unwrap_or_default()
            .to_owned()
    };
    Some(SolvedCaptcha {
        token,
        challenge: field("challenge"),
        validate: field("validate"),
        seccode: field("seccode"),
    })
}

fn finish_sms() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let dispatcher = shell.dispatcher.clone();
    let wanted = WANTED.with_borrow(Clone::clone);
    let Some(Wanted::Sms { tel }) = wanted else { return };
    let Some(captcha) = settled() else {
        let _ = toast("没有拿到验证结果");
        return;
    };
    let inner = backend.clone();
    backend.spawn(async move {
        match inner.services.account.sms_send(86, &tel, &captcha).await {
            Ok(key) => {
                post(&dispatcher, move || {
                    SMS_KEY.with_borrow_mut(|slot| *slot = key);
                    let _ = toast("验证码已发出");
                });
            }
            Err(e) => post(&dispatcher, move || {
                let _ = toast(&format!("发送失败：{e}"));
            }),
        }
    });
}

fn finish_password() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let dispatcher = shell.dispatcher.clone();
    let wanted = WANTED.with_borrow(Clone::clone);
    let Some(Wanted::Password { user, pass }) = wanted else { return };
    let Some(captcha) = settled() else {
        let _ = toast("没有拿到验证结果");
        return;
    };
    let inner = backend.clone();
    backend.spawn(async move {
        let account = &inner.services.account;
        let key = match account.login_key().await {
            Ok(key) => key,
            Err(e) => {
                post(&dispatcher, move || {
                    let _ = toast(&format!("取公钥失败：{e}"));
                });
                return;
            }
        };
        let Some(sealed) = encrypted(&key, &pass) else {
            post(&dispatcher, || {
                let _ = toast("密码加密失败");
            });
            return;
        };
        match account.password_login(&user, &sealed, &captcha).await {
            Ok(session) => post(&dispatcher, move || saved(session)),
            Err(e) => post(&dispatcher, move || {
                let _ = toast(&format!("登录失败：{e}"));
            }),
        }
    });
}

/// The password as B 站 wants it: the salt in front, then the password, under the public key.
fn encrypted(key: &rubi_core::LoginKey, password: &str) -> Option<String> {
    use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Encrypt, RsaPublicKey};
    let public = RsaPublicKey::from_public_key_pem(&key.key).ok()?;
    let mut rng = rand::thread_rng();
    let plain = format!("{}{}", key.hash, password);
    let sealed = public.encrypt(&mut rng, Pkcs1v15Encrypt, plain.as_bytes()).ok()?;
    Some(base64::engine::general_purpose::STANDARD.encode(sealed))
}

/// Writes the account away and rebuilds what was built for the old one.
fn saved(session: Session) {
    crate::ui::me::account_saved(&session);
    let _ = toast("已登录");
}