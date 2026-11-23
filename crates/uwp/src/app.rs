//! The application object. XAML owns the process; `OnLaunched` hands over to the shell.

use windows_core::{Interface, Ref, Result, implement};

use crate::ui::shell;
use crate::xaml::{
    Application, ApplicationInitializationCallback, ApplicationInitializationCallbackParams,
    CachedFileUpdaterActivatedEventArgs, FileActivatedEventArgs, FileOpenPickerActivatedEventArgs,
    FileSavePickerActivatedEventArgs, IActivatedEventArgs, IApplicationOverrides, IApplicationOverrides_Impl,
    IProtocolActivatedEventArgs,
    LaunchActivatedEventArgs, SearchActivatedEventArgs, ShareTargetActivatedEventArgs, WindowCreatedEventArgs,
};

windows_link::link!("api-ms-win-core-winrt-l1-1-0.dll" "system" fn RoInitialize(init_type: i32) -> i32);
const RO_INIT_MULTITHREADED: i32 = 1;

/// Starts XAML; returns when the app closes.
pub fn run() -> Result<()> {
    windows_core::HRESULT(unsafe { RoInitialize(RO_INIT_MULTITHREADED) }).ok()?;
    Application::Start(&ApplicationInitializationCallback::new(|_: Ref<ApplicationInitializationCallbackParams>| {
        // XAML holds the application for the life of the process.
        std::mem::forget(Application::compose(App));
    }))
}

#[implement(IApplicationOverrides)]
struct App;

impl IApplicationOverrides_Impl for App_Impl {
    fn OnLaunched(&self, _: Ref<LaunchActivatedEventArgs>) -> Result<()> {
        let started = shell::start();
        // A start that fails leaves the frame empty and the window black; say why in the log.
        if let Err(problem) = &started {
            crate::ui::trace(&format!("launch failed: {problem}"));
        }
        started
    }

    fn OnActivated(&self, args: Ref<IActivatedEventArgs>) -> Result<()> {
        // `rubi://` comes in here rather than in OnLaunched: the link names the video to open.
        // Anything else that reaches this is left to OnLaunched.
        let Ok(args) = args.ok() else { return Ok(()) };
        crate::ui::trace("activated");
        // Only a protocol activation is taken here. Anything else — the media panel bringing the
        // window back, say — is left alone: starting the shell again would put 首页 over the page
        // the user was on.
        if let Ok(protocol) = args.cast::<IProtocolActivatedEventArgs>()
            && let Ok(link) = protocol.Uri().and_then(|uri| uri.AbsoluteUri())
        {
            crate::ui::shell::open_link(&link);
        }
        Ok(())
    }

    fn OnFileActivated(&self, _: Ref<FileActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnSearchActivated(&self, _: Ref<SearchActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnShareTargetActivated(&self, _: Ref<ShareTargetActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnFileOpenPickerActivated(&self, _: Ref<FileOpenPickerActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnFileSavePickerActivated(&self, _: Ref<FileSavePickerActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnCachedFileUpdaterActivated(&self, _: Ref<CachedFileUpdaterActivatedEventArgs>) -> Result<()> {
        Ok(())
    }

    fn OnWindowCreated(&self, _: Ref<WindowCreatedEventArgs>) -> Result<()> {
        Ok(())
    }
}
