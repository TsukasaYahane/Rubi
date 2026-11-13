//! Who can be named or sent to: recent chats first, then followed UPs. Read once a run, on first
//! need, and shared by @ 提到 and 发给好友.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::ui::post;
use crate::ui::shell::shell;

#[derive(Clone)]
pub struct Contact {
    pub name: String,
    pub mid: u64,
    /// Where they came from: 最近聊天 or 关注.
    pub from: &'static str,
}

type Waiting = Box<dyn FnOnce(Rc<Vec<Contact>>)>;

thread_local! {
    static READ: RefCell<Option<Rc<Vec<Contact>>>> = const { RefCell::new(None) };
    static READING: Cell<bool> = const { Cell::new(false) };
    static WAITING: RefCell<Vec<Waiting>> = const { RefCell::new(Vec::new()) };
}

/// The contacts, if read already.
pub fn known() -> Option<Rc<Vec<Contact>>> {
    READ.with_borrow(Clone::clone)
}

/// Calls `then` with the contacts, at once when read, otherwise once they are.
pub fn with(then: impl FnOnce(Rc<Vec<Contact>>) + 'static) {
    if let Some(read) = known() {
        then(read);
        return;
    }
    WAITING.with_borrow_mut(|w| w.push(Box::new(then)));
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    if READING.replace(true) {
        return;
    }
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let chats = core.services.messages.sessions().await.map(|(s, _)| s).unwrap_or_default();
        let follows = core.services.following.all().await.map(|a| a.ups).unwrap_or_default();
        let mut read: Vec<(String, u64, &'static str)> =
            chats.into_iter().map(|s| (s.name, s.talker_mid, "最近聊天")).collect();
        for up in follows {
            if !read.iter().any(|c| c.1 == up.mid) {
                read.push((up.name, up.mid, "关注"));
            }
        }
        post(&dispatcher, move || {
            READING.set(false);
            let read: Rc<Vec<Contact>> =
                Rc::new(read.into_iter().map(|(name, mid, from)| Contact { name, mid, from }).collect());
            READ.with_borrow_mut(|r| *r = Some(read.clone()));
            for then in WAITING.with_borrow_mut(std::mem::take) {
                then(read.clone());
            }
        });
    });
}
