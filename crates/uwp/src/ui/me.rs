//! 我的: who is signed in, or the QR code to sign in with. The rest of the page comes later.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rubi_core::QrState;
use windows_core::{Interface, Result};

use crate::session;
use crate::state::qr;
use crate::ui::shell::{Shell, shell};
use crate::ui::{load, named, post, set_text};
use crate::xaml::{IContentControl, IFrameworkElement, UIElement};

/// How often the login is asked about.
const POLL_EVERY: Duration = Duration::from_secs(2);
/// Bumped when the page is left or shown again: a poll from an earlier showing stops.
static RUN: AtomicU64 = AtomicU64::new(0);

const LAYOUT: &str = r##"
<StackPanel xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
            xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Margin="16">
  <TextBlock x:Name="Name" Text="我的" FontSize="20" FontWeight="Light" />
  <TextBlock x:Name="Status" Margin="0,8,0,12" FontSize="14" TextWrapping="Wrap"
             Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  <ContentControl x:Name="Code" IsTabStop="False" HorizontalAlignment="Left" />
</StackPanel>"##;

/// Stops a login under way.
pub fn leave() {
    RUN.fetch_add(1, Ordering::Relaxed);
}

pub fn show(shell: &Shell) -> Result<()> {
    let root = load(LAYOUT)?;
    shell.set_page(&root)?;
    let Some(backend) = shell.backend.get().cloned() else {
        return set_text(&root, "Status", "正在打开…");
    };
    let run = RUN.load(Ordering::Relaxed);
    let dispatcher = shell.dispatcher.clone();
    let core = backend.clone();
    let tell = move |update: Update| {
        post(&dispatcher, move || {
            if RUN.load(Ordering::Relaxed) == run {
                let _ = update.apply();
            }
        });
    };
    backend.spawn(async move {
        let account = &core.services.account;
        if account.is_logged_in() {
            match account.whoami().await {
                Ok(who) => tell(Update::SignedIn(who.uname)),
                Err(e) => tell(Update::Status(format!("读取账号失败：{e}"))),
            }
            return;
        }
        let ticket = match account.qr_generate().await {
            Ok(t) => t,
            Err(e) => return tell(Update::Status(format!("获取二维码失败：{e}"))),
        };
        tell(Update::Code(ticket.url.clone()));
        while RUN.load(Ordering::Relaxed) == run {
            tokio::time::sleep(POLL_EVERY).await;
            match account.qr_poll(&ticket.key).await {
                Ok(QrState::Waiting) => {}
                Ok(QrState::Scanned) => tell(Update::Status("已扫码，请在手机上确认".into())),
                Ok(QrState::Expired) => return tell(Update::Status("二维码已过期，重新打开这一页".into())),
                Ok(QrState::Success(session)) => {
                    let who = account.whoami().await.map(|w| w.uname).unwrap_or_default();
                    return tell(Update::Saved(session, who));
                }
                Err(e) => return tell(Update::Status(format!("登录失败：{e}"))),
            }
        }
    });
    Ok(())
}

/// What the background login has to say; applied on the UI thread.
enum Update {
    Status(String),
    Code(String),
    SignedIn(String),
    Saved(rubi_core::Session, String),
}

impl Update {
    fn apply(self) -> Result<()> {
        let Some(root) = current_page() else { return Ok(()) };
        match self {
            Self::Status(text) => set_text(&root, "Status", &text),
            Self::Code(url) => show_code(&root, &url),
            Self::SignedIn(name) => signed_in(&root, &name),
            Self::Saved(session, name) => {
                let stored = session::save(&session);
                if let Some(home) = shell().and_then(|s| s.home_page()) {
                    home.reload();
                }
                signed_in(&root, &name)?;
                if stored.is_err() {
                    set_text(&root, "Status", "已登录，但保存失败：下次启动要重新登录")?;
                }
                Ok(())
            }
        }
    }
}

fn current_page() -> Option<IFrameworkElement> {
    shell()?.current_page().ok()
}

fn signed_in(root: &IFrameworkElement, name: &str) -> Result<()> {
    set_text(root, "Name", name)?;
    set_text(root, "Status", "已登录")?;
    named::<IContentControl>(root, "Code")?.SetContent(None::<&windows_core::IInspectable>)
}

/// The QR code, on white with its quiet zone, whatever the theme.
fn show_code(root: &IFrameworkElement, url: &str) -> Result<()> {
    let Some((side, data)) = qr::path_data(url) else { return set_text(root, "Status", "二维码生成失败") };
    let code = load(&format!(
        r##"<Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" Background="White" Padding="14">
          <Viewbox Width="220" Height="220"><Canvas Width="{side}" Height="{side}">
            <Path Fill="Black" Data="{data}" />
          </Canvas></Viewbox></Border>"##
    ))?;
    named::<IContentControl>(root, "Code")?.SetContent(&code.cast::<UIElement>()?)?;
    set_text(root, "Status", "用 B 站手机客户端扫码登录")
}
