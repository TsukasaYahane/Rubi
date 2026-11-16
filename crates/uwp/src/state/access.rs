//! 文件系统权限: whether Rubi may reach folders outside its own with plain file calls. The
//! answer is the user's to give, in 设置 › 隐私和安全性 › 文件系统; asking the broker for a file
//! outside the app is also what puts Rubi on that page in the first place.

/// Whether a file outside the app's own folders can be read with ordinary file calls.
pub fn granted() -> bool {
    // Ordinary file calls only go through once 文件系统 access has been given. The broker is
    // not asked here: waiting for it on the UI thread blocks the loop it answers on.
    std::fs::metadata(r"C:\Windows\win.ini").is_ok()
}

/// Opens the page where the answer is given, and where Rubi has to be turned on.
#[cfg(target_vendor = "uwp")]
pub fn open_settings() {
    let Ok(uri) = crate::xaml::Uri::CreateUri("ms-settings:privacy-broadfilesystemaccess") else { return };
    // Not waited on: the settings page takes the screen, and the app has nothing to do until it
    // comes back.
    let _ = crate::xaml::Launcher::LaunchUriAsync(&uri);
}

/// Asks once, at the first start, whether Rubi may have 文件系统 access, and opens the page
/// where it is given if the answer is yes.
#[cfg(target_vendor = "uwp")]
pub fn ask_once() {
    if granted() {
        return;
    }
    let Some(shell) = crate::ui::shell::shell() else { return };
    if shell.prefs().access_asked {
        return;
    }
    let mut prefs = shell.prefs();
    prefs.access_asked = true;
    let _ = shell.set_prefs(prefs);

    let Ok(dialog) = crate::xaml::MessageDialog::CreateWithTitle(
        "允许之后，缓存和导出可以直接写进你选的目录。\n不允许也能用，只是每条缓存会多留一份副本在应用自己的目录里。",
        "文件系统权限",
    ) else {
        return;
    };
    let (Ok(commands), Ok(later), Ok(go)) = (
        dialog.Commands(),
        crate::xaml::UICommand::Create("以后再说"),
        crate::xaml::UICommand::Create("打开设置"),
    ) else {
        return;
    };
    let _ = commands.Append(&later);
    let _ = commands.Append(&go);
    let _ = dialog.SetDefaultCommandIndex(1);
    let _ = dialog.SetCancelCommandIndex(0);

    // The dialog is answered on Windows' own thread, and the page has to be told on ours: a
    // wait here would hold the loop that carries the answer.
    let Ok(operation) = dialog.ShowAsync() else { return };
    let dispatcher = shell.dispatcher.clone();
    let handler = windows_future::AsyncOperationCompletedHandler::new(
        move |operation: windows_core::Ref<windows_future::IAsyncOperation<crate::xaml::IUICommand>>,
              _: windows_future::AsyncStatus| {
            if let Ok(operation) = operation.ok()
                && let Ok(chosen) = operation.GetResults()
                && chosen.Label().is_ok_and(|label| label == "打开设置")
            {
                crate::ui::post(&dispatcher, open_settings);
            }
            Ok(())
        },
    );
    let _ = operation.SetCompleted(&handler);
}

/// Nothing to open outside the app package: the page and the asking are the package's.
#[cfg(not(target_vendor = "uwp"))]
pub fn open_settings() {}

#[cfg(not(target_vendor = "uwp"))]
pub fn ask_once() {}
