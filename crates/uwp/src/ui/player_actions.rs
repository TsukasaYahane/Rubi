//! The video page's action row: 赞, 币, 藏, 稍后再看, 关注. Each change shows at once and is
//! taken back if B站 refuses it. A guest is sent to 我的 to sign in.

use std::cell::{Cell, RefCell};
use std::future::Future;
use std::rc::Rc;

use rubi_model::{FavFolder, VideoRelation};
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::ui::keep;
use crate::ui::shell::{Tab, shell, show_tab};
use crate::ui::{load_object, named, on_click, player, post, set_text};
use crate::xaml::{
    FrameworkElement, HoldingRoutedEventArgs, HoldingState, IFlyoutBase, IFrameworkElement, IHoldingRoutedEventArgs,
    IMenuFlyout, IMenuFlyoutItem, IToggleMenuFlyoutItem, IUIElement, RightTappedRoutedEventArgs, RoutedEventArgs,
    ToggleMenuFlyoutItem,
};

pub struct Actions {
    root: IFrameworkElement,
    bvid: String,
    relation: Cell<Option<VideoRelation>>,
    counts: RefCell<(u64, u64, u64)>,
}

impl Actions {
    pub fn attach(root: &IFrameworkElement, bvid: &str) -> Result<Rc<Self>> {
        let actions = Rc::new(Self {
            root: root.clone(),
            bvid: bvid.to_owned(),
            relation: Cell::new(None),
            counts: RefCell::new((0, 0, 0)),
        });
        let on = |name: &str, act: fn(&Self)| {
            let weak = Rc::downgrade(&actions);
            on_click(root, name, move || {
                if let Some(a) = weak.upgrade() {
                    act(&a);
                }
            })
        };
        on("ActLike", Self::toggle_like)?;
        // 一键三连: a right click, or a press held with a finger or pen.
        let like = named::<IUIElement>(root, "ActLike")?;
        let weak = Rc::downgrade(&actions);
        keep(like.RightTapped(move |_: Ref<IInspectable>, _: Ref<RightTappedRoutedEventArgs>| {
            if let Some(a) = weak.upgrade() {
                a.triple();
            }
        })?);
        let weak = Rc::downgrade(&actions);
        keep(like.Holding(move |_: Ref<IInspectable>, args: Ref<HoldingRoutedEventArgs>| {
            let started = args
                .ok()
                .and_then(|a| a.cast::<IHoldingRoutedEventArgs>()?.HoldingState())
                .is_ok_and(|s| s == HoldingState::Started);
            if let (true, Some(a)) = (started, weak.upgrade()) {
                a.triple();
            }
        })?);
        on("ActFav", Self::open_folders)?;
        on("ActLater", Self::watch_later)?;
        on("ActFollow", Self::toggle_follow)?;
        for (name, coins) in [("Coin1", 1u32), ("Coin2", 2)] {
            let weak = Rc::downgrade(&actions);
            let item = named::<IMenuFlyoutItem>(root, name)?;
            let revoker = item.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                if let Some(a) = weak.upgrade() {
                    a.give_coins(coins);
                }
            })?;
            keep(revoker);
        }
        Ok(actions)
    }

    /// The counts from the video's details, and the account's relation when signed in.
    pub fn set_counts(&self, likes: u64, coins: u64, favorites: u64) {
        *self.counts.borrow_mut() = (likes, coins, favorites);
        self.show();
    }

    pub fn set_relation(&self, relation: VideoRelation) {
        self.relation.set(Some(relation));
        self.show();
    }

    fn show(&self) {
        let (likes, coins, favorites) = *self.counts.borrow();
        let relation = self.relation.take();
        self.relation.set(relation);
        let relation = relation.unwrap_or_default();
        let _ = set_text(&self.root, "LikeText", &format!("{}{likes}", if relation.liked { "已赞 " } else { "赞 " }));
        let _ =
            set_text(&self.root, "CoinText", &format!("{}{coins}", if relation.coins > 0 { "已投 " } else { "币 " }));
        let _ = set_text(
            &self.root,
            "FavText",
            &format!("{}{favorites}", if relation.favorited { "已藏 " } else { "藏 " }),
        );
        let _ = set_text(&self.root, "FollowText", if relation.following_up { "已关注" } else { "关注" });
        for (name, on) in [("LikeText", relation.liked), ("CoinText", relation.coins > 0), ("FavText", relation.favorited)] {
            let _ = tint(&self.root, name, on);
        }
    }

    /// The relation, if someone is signed in; a guest goes to sign in.
    fn signed_in(&self) -> Option<VideoRelation> {
        let relation = self.relation.take();
        self.relation.set(relation);
        if relation.is_none() {
            show_tab(Tab::Me);
        }
        relation
    }

    /// Runs a write in the background; on failure `undo` restores what was shown.
    fn write<F>(
        &self,
        label: &'static str,
        work: impl FnOnce(std::sync::Arc<crate::backend::Backend>, String) -> F + Send + 'static,
        undo: impl FnOnce(&Self) + Send + 'static,
    ) where
        F: Future<Output = Result2> + Send + 'static,
    {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let dispatcher = shell.dispatcher.clone();
        let bvid = self.bvid.clone();
        let run = player::run();
        let core = backend.clone();
        backend.spawn(async move {
            let failed = work(core, bvid).await.err();
            post(&dispatcher, move || {
                let Some(problem) = failed else { return };
                if let Some(actions) = player::actions(run) {
                    undo(&actions);
                }
                player::say(&format!("{label}失败：{problem}"));
            });
        });
    }

    fn toggle_like(&self) {
        let Some(relation) = self.signed_in() else { return };
        let on = !relation.liked;
        let step = |a: &Self, on: bool| {
            let mut counts = a.counts.borrow_mut();
            counts.0 = if on { counts.0 + 1 } else { counts.0.saturating_sub(1) };
        };
        step(self, on);
        self.set_relation(VideoRelation { liked: on, ..relation });
        self.write(
            "点赞",
            move |core, bvid| async move { core.services.actions.like(&bvid, on).await.map_err(|e| e.to_string()) },
            move |a| {
                step(a, !on);
                a.set_relation(relation);
            },
        );
    }

    fn give_coins(&self, coins: u32) {
        let Some(relation) = self.signed_in() else { return };
        if relation.coins + coins > 2 {
            player::say("每个视频最多投 2 枚硬币");
            return;
        }
        // 同时点赞, unless it is liked already.
        let also_like = !relation.liked
            && named::<IToggleMenuFlyoutItem>(&self.root, "CoinLike").and_then(|i| i.IsChecked()).unwrap_or(false);
        {
            let mut counts = self.counts.borrow_mut();
            counts.1 += coins as u64;
            counts.0 += u64::from(also_like);
        }
        self.set_relation(VideoRelation {
            coins: relation.coins + coins,
            liked: relation.liked || also_like,
            ..relation
        });
        self.write(
            "投币",
            move |core, bvid| async move {
                core.services.actions.coin(&bvid, coins, also_like).await.map_err(|e| e.to_string())
            },
            move |a| {
                let mut counts = a.counts.borrow_mut();
                counts.1 = counts.1.saturating_sub(coins as u64);
                counts.0 = counts.0.saturating_sub(u64::from(also_like));
                drop(counts);
                a.set_relation(relation);
            },
        );
    }

    /// 一键三连: like, two coins and the default folder at once. What B站 did shows when it answers.
    fn triple(&self) {
        let Some(relation) = self.signed_in() else { return };
        if relation.liked && relation.coins >= 2 && relation.favorited {
            player::say("已经三连过了");
            return;
        }
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, bvid, run, core) =
            (shell.dispatcher.clone(), self.bvid.clone(), player::run(), backend.clone());
        player::say("正在三连…");
        backend.spawn(async move {
            let done = core.services.actions.triple(&bvid).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                let Some(actions) = player::actions(run) else { return };
                match done {
                    Ok(t) => {
                        let Some(now) = actions.relation.get() else { return };
                        {
                            let mut counts = actions.counts.borrow_mut();
                            counts.0 += u64::from(t.liked && !now.liked);
                            counts.1 += u64::from(t.coins);
                            counts.2 += u64::from(t.favorited && !now.favorited);
                        }
                        actions.set_relation(VideoRelation {
                            liked: now.liked || t.liked,
                            coins: (now.coins + t.coins).min(2),
                            favorited: now.favorited || t.favorited,
                            ..now
                        });
                        player::say(if t.liked && t.coined && t.favorited {
                            "三连成功"
                        } else {
                            "三连完成（部分已做过）"
                        });
                    }
                    Err(problem) => player::say(&format!("三连失败：{problem}")),
                }
            });
        });
    }

    fn watch_later(&self) {
        if self.signed_in().is_none() {
            return;
        }
        let _ = set_text(&self.root, "LaterText", "已加入");
        self.write(
            "加入稍后再看",
            |core, bvid| async move { core.services.actions.watch_later(&bvid).await.map_err(|e| e.to_string()) },
            |a| {
                let _ = set_text(&a.root, "LaterText", "稍后");
            },
        );
    }

    fn toggle_follow(&self) {
        let Some(relation) = self.signed_in() else { return };
        let on = !relation.following_up;
        self.set_relation(VideoRelation { following_up: on, ..relation });
        self.write(
            "关注",
            move |core, bvid| async move { core.services.actions.follow_up(&bvid, on).await.map_err(|e| e.to_string()) },
            move |a| a.set_relation(relation),
        );
    }

    /// Reads the folders, then shows them beside the button.
    fn open_folders(&self) {
        if self.signed_in().is_none() {
            return;
        }
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let dispatcher = shell.dispatcher.clone();
        let (bvid, run, core) = (self.bvid.clone(), player::run(), backend.clone());
        backend.spawn(async move {
            let folders = core.services.actions.fav_folders(&bvid).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                let Some(actions) = player::actions(run) else { return };
                match folders {
                    Ok(folders) => {
                        let _ = actions.show_folders(folders);
                    }
                    Err(problem) => player::say(&format!("读取收藏夹失败：{problem}")),
                }
            });
        });
    }

    fn show_folders(self: Rc<Self>, folders: Vec<FavFolder>) -> Result<()> {
        let mut xaml =
            String::from(r#"<MenuFlyout xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">"#);
        for folder in &folders {
            xaml.push_str(&format!(
                r#"<ToggleMenuFlyoutItem Text="{}" IsChecked="{}" />"#,
                escape(&folder.title),
                folder.contains
            ));
        }
        xaml.push_str("</MenuFlyout>");
        let menu = load_object(&xaml)?;
        let items = menu.cast::<IMenuFlyout>()?.Items()?;
        for (at, folder) in folders.iter().enumerate() {
            let item = items.GetAt(at as u32)?.cast::<ToggleMenuFlyoutItem>()?;
            let weak = Rc::downgrade(&self);
            let id = folder.id;
            let revoker =
                item.cast::<IMenuFlyoutItem>()?.Click(move |sender: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                    let checked = sender.ok().and_then(|s| s.cast::<IToggleMenuFlyoutItem>()?.IsChecked());
                    if let (Some(a), Ok(on)) = (weak.upgrade(), checked) {
                        a.set_folder(id, on);
                    }
                })?;
            keep(revoker);
        }
        menu.cast::<IFlyoutBase>()?.ShowAt(&named::<FrameworkElement>(&self.root, "ActFav")?)
    }

    fn set_folder(&self, id: u64, on: bool) {
        self.write(
            "收藏",
            move |core, bvid| async move {
                let (add, remove) = if on { (vec![id], vec![]) } else { (vec![], vec![id]) };
                core.services.actions.favorite(&bvid, &add, &remove).await.map_err(|e| e.to_string())
            },
            |_| {},
        );
        if let Some(relation) = self.relation.get() {
            self.set_relation(VideoRelation { favorited: on || relation.favorited, ..relation });
        }
    }
}

type Result2 = std::result::Result<(), String>;

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

const COBALT: crate::xaml::Color = crate::xaml::Color { a: 255, r: 0, g: 0x50, b: 0xEF };

/// What has been done shows in blue; otherwise the text keeps the theme's colour.
fn tint(root: &IFrameworkElement, name: &str, on: bool) -> Result<()> {
    let text = named::<crate::xaml::ITextBlock>(root, name)?;
    if on {
        text.SetForeground(&crate::xaml::SolidColorBrush::CreateInstanceWithColor(COBALT)?)
    } else {
        text.cast::<crate::xaml::IDependencyObject>()?.ClearValue(&crate::xaml::TextBlock::ForegroundProperty()?)
    }
}
