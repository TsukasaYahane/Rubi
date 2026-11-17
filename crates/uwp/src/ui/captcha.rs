//! 验证码: the slider that has to be pulled before a password login will go through.
//!
//! The challenge is script the service owns, and it will not run on the page `NavigateToString`
//! builds: that page has no origin (`about:blank`), and the sdk reads one before it draws
//! anything. So the page is served over a loopback http server that lives as long as the sheet.
//! It has an origin, the slider runs, and when it passes the page navigates to `/done?…` — the
//! answer comes back through that request rather than through `window.external.notify`, which a
//! page with a real origin is not allowed to call.

use std::cell::RefCell;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use windows_core::{IInspectable, Ref, Result};

use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load, named, popup, post, trace};
use crate::xaml::{IButtonBase, RoutedEventArgs, Uri, WebView};

const SHEET: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="620" Padding="16"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <StackPanel>
      <TextBlock Text="拖动滑块完成验证" FontSize="16" FontWeight="Light" />
      <WebView x:Name="Page" Height="620" Margin="0,10,0,0" />
      <Button x:Name="Close" Content="取消" HorizontalAlignment="Right" Margin="0,12,0,0" />
    </StackPanel>
  </Grid>
</Grid>"##;

thread_local! {
    /// What the service approved, as the login call wants it.
    static PASSED: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Runs once the slider passes and the sheet is gone.
    static AFTER: RefCell<Option<Box<dyn FnOnce()>>> = const { RefCell::new(None) };
    /// The login token the endpoint handed out with the challenge. It has to go back with the
    /// answer, or the login call is refused with 验证码错误.
    static TOKEN: RefCell<String> = const { RefCell::new(String::new()) };
}

/// What the last passed challenge answered with, if one did.
pub fn passed() -> Option<String> {
    PASSED.with_borrow(Clone::clone)
}

/// The login token that came with this challenge.
pub fn token() -> Option<String> {
    TOKEN.with_borrow(|token| if token.is_empty() { None } else { Some(token.clone()) })
}

/// What to do once the slider passes: the panel that asked for it carries on from there.
pub fn then(job: impl FnOnce() + 'static) {
    AFTER.with_borrow_mut(|slot| *slot = Some(Box::new(job)));
}

/// Asks the captcha endpoint what to render, serves the slider on a loopback page, shows it, and
/// waits for the page to report back.
pub fn start() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let dispatcher = shell.dispatcher.clone();
    backend.spawn(async move {
        let Ok(answer) =
            reqwest::get("https://passport.bilibili.com/x/passport-login/captcha?source=main_web").await
        else {
            post(&dispatcher, || {
                let _ = toast("取不到验证码");
            });
            return;
        };
        let Ok(body) = answer.text().await else { return };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) else { return };
        let text = |path: &str| -> String {
            value.pointer(path).and_then(|item| item.as_str()).unwrap_or_default().to_owned()
        };
        let (gt, challenge) = (text("/data/geetest/gt"), text("/data/geetest/challenge"));
        let login_token = text("/data/token");
        if gt.is_empty() || challenge.is_empty() {
            post(&dispatcher, || {
                let _ = toast("取不到验证码");
            });
            return;
        }
        let Some((port, mut answers)) = serve(gt, challenge).await else {
            post(&dispatcher, || {
                let _ = toast("验证码打不开");
            });
            return;
        };
        let address = format!("http://127.0.0.1:{port}/");
        let show = address.clone();
        post(&dispatcher, move || {
            TOKEN.with_borrow_mut(|slot| *slot = login_token);
            let _ = open(&show);
        });
        // The page answers once, by navigating to /done; the sheet is closed here.
        if let Some(answer) = answers.recv().await {
            trace(&format!("captcha {answer}"));
            // Stored on the ui thread: `PASSED` is thread local, and the job that reads it runs
            // there. Writing it from this worker would put it somewhere nobody looks.
            post(&dispatcher, move || {
                PASSED.with_borrow_mut(|slot| *slot = Some(answer));
                let _ = popup::hide();
                if let Some(job) = AFTER.with_borrow_mut(Option::take) {
                    job();
                }
            });
        }
    });
}

fn open(address: &str) -> Result<()> {
    let root = load(SHEET)?;
    let page = named::<WebView>(&root, "Page")?;
    page.Navigate(&Uri::CreateUri(address)?)?;
    keep(named::<IButtonBase>(&root, "Close")?.Click(
        |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
            let _ = popup::hide();
        },
    )?);
    popup::show(&root)
}

/// A one-page server: it hands out the slider and keeps going until the page comes back with an
/// answer at `/done`.
async fn serve(gt: String, challenge: String) -> Option<(u16, mpsc::Receiver<String>)> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.ok()?;
    let port = listener.local_addr().ok()?.port();
    let (sender, answers) = mpsc::channel(1);
    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else { return };
            let mut buffer = vec![0u8; 4096];
            let Ok(read) = socket.read(&mut buffer).await else { continue };
            let request = String::from_utf8_lossy(&buffer[..read]).to_string();
            let line = request.lines().next().unwrap_or_default().to_owned();
            let done = line.starts_with("GET /done");
            let body = if done {
                // The slider has passed: hand the answer over and let the page stop.
                let _ = sender.send(query_of(&line)).await;
                "<html><body style=\"font-family:Segoe UI\">验证通过，可以关掉了。</body></html>".to_owned()
            } else {
                slider_page(&gt, &challenge)
            };
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = socket.write_all(header.as_bytes()).await;
            let _ = socket.write_all(body.as_bytes()).await;
            let _ = socket.flush().await;
            if done {
                return;
            }
        }
    });
    Some((port, answers))
}

/// `validate=…&seccode=…` out of the request line, as one record.
fn query_of(line: &str) -> String {
    let query = line.split('?').nth(1).unwrap_or_default();
    let query = query.split(' ').next().unwrap_or_default();
    let mut fields = Vec::new();
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or_default();
        let value = decode(parts.next().unwrap_or_default());
        if !key.is_empty() {
            fields.push(format!("{key}={value}"));
        }
    }
    fields.join("&")
}

/// `%XX` and `+` back to what they stand for; the page encodes its answer.
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'%' if at + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[at + 1..at + 3]).unwrap_or_default();
                match u8::from_str_radix(hex, 16) {
                    Ok(value) => {
                        out.push(value);
                        at += 3;
                    }
                    Err(_) => {
                        out.push(bytes[at]);
                        at += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                at += 1;
            }
            other => {
                out.push(other);
                at += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The service's own slider, told what challenge to run. When it passes, the page leaves for
/// `/done` with the answer in the query, which is the one channel the server can read.
fn slider_page(gt: &str, challenge: &str) -> String {
    let page = r##"<html><head><meta charset="utf-8" />
<style>html,body{margin:0;overflow:hidden;font-family:"Segoe UI",sans-serif;background:#fff}</style></head>
<body>
<div id="captcha">正在加载验证…</div>
<script src="https://static.geetest.com/static/tools/gt.js"></script>
<script>
initGeetest({
  gt: "@GT@", challenge: "@CHALLENGE@", offline: false, new_captcha: true,
  product: "popup"
}, function (captcha) {
  captcha.onSuccess(function () {
    var v = captcha.getValidate() || {};
    location.href = "/done?validate=" + encodeURIComponent(v.geetest_validate || "")
      + "&seccode=" + encodeURIComponent(v.geetest_seccode || "")
      + "&challenge=" + encodeURIComponent(v.geetest_challenge || "");
  });
  captcha.onError(function () {
    document.getElementById("captcha").textContent = "验证失败，请重试";
  });
  captcha.appendTo("#captcha");
});
</script>
</body></html>"##;
    page.replace("@GT@", gt).replace("@CHALLENGE@", challenge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_answer_out_of_the_request_line() {
        let line = "GET /done?validate=abc%2C123&seccode=xyz&challenge=ch%2B1 HTTP/1.1";
        assert_eq!(query_of(line), "validate=abc,123&seccode=xyz&challenge=ch 1");
    }

    #[test]
    fn turns_plus_into_a_space() {
        assert_eq!(decode("a+b%20c"), "a b c");
    }
}