//! 后台下载: keeping the process out of suspension while a cache is being written. The window may
//! be minimised or left behind another app; a download in progress is a reason for it to keep
//! running.
//!
//! `ExtendedExecutionSession` has no default constructor in the generated bindings, and the
//! factory interface makes the generator panic, so the class is activated through
//! `RoActivateInstance` and the result is cast to the interface. The interface itself is
//! generated, so nothing below the activation is hand-written.

use std::cell::RefCell;

use windows_core::{HSTRING, Interface};
use windows_link::link;

use crate::xaml::{
    ExtendedExecutionReason, ExtendedExecutionRevokedReason, IClosable, IExtendedExecutionSession,
};

link!(
    "combase.dll" "system" fn RoActivateInstance(
        activatable_class_id: *mut core::ffi::c_void,
        instance: *mut *mut core::ffi::c_void,
    ) -> i32
);

const SESSION_CLASS: &str = "Windows.ApplicationModel.ExtendedExecution.ExtendedExecutionSession";

thread_local! {
    static HELD: RefCell<Option<IExtendedExecutionSession>> = const { RefCell::new(None) };
}

/// Activates the class the bindings cannot build.
fn activate() -> Option<IExtendedExecutionSession> {
    let name = HSTRING::from(SESSION_CLASS);
    let mut raw: *mut core::ffi::c_void = core::ptr::null_mut();
    // `HSTRING` is a transparent wrapper over its handle; `RoActivateInstance` takes that handle.
    let hr = unsafe { RoActivateInstance(core::mem::transmute_copy(&name), &mut raw) };
    if hr < 0 || raw.is_null() {
        return None;
    }
    // The pointer is a new reference to an inspectable of the class named above.
    Some(unsafe { IExtendedExecutionSession::from_raw(raw) })
}

/// Asks not to be suspended. Called when a download starts; doing it again while one is held is
/// free.
pub fn keep_awake() {
    HELD.with_borrow_mut(|slot| {
        if slot.is_some() {
            return;
        }
        let Some(session) = activate() else { return };
        let _ = session.SetReason(ExtendedExecutionReason::Unspecified);
        let _ = session.SetDescription("缓存");
        // Not waited on: the answer arrives through the ui thread's own message pump.
        let _ = session.RequestExtensionAsync();
        *slot = Some(session);
    });
}

/// Lets the session go: nothing is being written any more, so the app may be suspended.
pub fn let_go() {
    HELD.with_borrow_mut(|slot| {
        if let Some(session) = slot.take() {
            if let Ok(closable) = session.cast::<IClosable>() {
                let _ = closable.Close();
            }
        }
    });
}