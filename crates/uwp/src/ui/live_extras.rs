//! What a live room shows besides its danmaku: 关注 for the anchor, the room's filter switches
//! with what they have held back, and what the switches let through: gifts on a gold line over the
//! picture, effects on a blue one under it, popups on a card that can only be closed.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use rubi_core::LiveFilter;
use rubi_model::LiveEvent;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load, named, on_click, post, set_text};
use crate::xaml::{
    Color, IButtonBase, IControl, IFrameworkElement, IPanel, IUIElement, RoutedEventArgs, SolidColorBrush, UIElement,
    Visibility,
};

/// How long a gift or an effect stays over the picture.
const SHOWS: Duration = Duration::from_secs(5);
const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };

/// The extras of the room on show. `current` finds them again from a callback.
pub struct Extras {
    root: IFrameworkElement,
    current: fn() -> Option<Rc<Extras>>,
    /// Held back this session: gift banners, popups, effects.
    blocked: Cell<(u32, u32, u32)>,
    followed: Cell<bool>,
    /// Bumped by each gift and effect shown, so only the last one hides it.
    gift_turn: Cell<u64>,
    effect_turn: Cell<u64>,
    /// The line in the open settings that says what was held back.
    detail_text: std::cell::RefCell<Option<crate::xaml::ITextBlock>>,
}

impl Extras {
    /// Wires the room's 关注 for `uid` (none when 0) and the popup's 关闭.
    pub fn new(root: &IFrameworkElement, uid: u64, current: fn() -> Option<Rc<Extras>>) -> Result<Rc<Self>> {
        let extras = Rc::new(Self {
            root: root.clone(),
            current,
            blocked: Cell::default(),
            followed: Cell::new(false),
            gift_turn: Cell::default(),
            effect_turn: Cell::default(),
            detail_text: std::cell::RefCell::default(),
        });
        on_click(root, "PopupClose", move || {
            if let Some(extras) = current() {
                let _ = extras.shown("PopupCard", false);
            }
        })?;
        if uid > 0 {
            on_click(root, "Follow", move || {
                if let Some(extras) = current() {
                    extras.toggle_follow(uid);
                }
            })?;
            extras.load_follow(uid);
        }
        Ok(extras)
    }

    fn shown(&self, name: &str, on: bool) -> Result<()> {
        named::<IUIElement>(&self.root, name)?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })
    }

    fn load_follow(&self, uid: u64) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, core, current) = (shell.dispatcher.clone(), backend.clone(), self.current);
        backend.spawn(async move {
            let Ok(profile) = core.services.space.profile(uid).await else { return };
            post(&dispatcher, move || {
                if let Some(extras) = current() {
                    extras.followed.set(profile.followed);
                    let _ = extras.show_follow();
                }
            });
        });
    }

    fn show_follow(&self) -> Result<()> {
        let on = self.followed.get();
        set_text(&self.root, "FollowText", if on { "已关注" } else { "+ 关注" })?;
        let button = named::<IControl>(&self.root, "Follow")?;
        let (fill, text) = if on {
            (Color { a: 0, ..COBALT }, COBALT)
        } else {
            (COBALT, Color { a: 255, r: 255, g: 255, b: 255 })
        };
        button.SetBackground(&SolidColorBrush::CreateInstanceWithColor(fill)?)?;
        button.SetForeground(&SolidColorBrush::CreateInstanceWithColor(text)?)?;
        self.shown("Follow", true)
    }

    fn toggle_follow(&self, uid: u64) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let on = !self.followed.get();
        self.followed.set(on);
        let _ = self.show_follow();
        let name = named::<IFrameworkElement>(&self.root, "UpLink")
            .and_then(|l| l.cast::<crate::xaml::IContentControl>()?.Content())
            .and_then(|c| c.cast::<crate::xaml::IPropertyValue>()?.GetString())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let (dispatcher, core, current) = (shell.dispatcher.clone(), backend.clone(), self.current);
        backend.spawn(async move {
            let done = core.services.actions.follow_mid(uid, &name, on).await.map_err(|e| e.to_string());
            post(&dispatcher, move || match done {
                Ok(()) => toast(if on { "已关注" } else { "已取消关注" }),
                Err(problem) => {
                    toast(&format!("关注失败：{problem}"));
                    if let Some(extras) = current() {
                        extras.followed.set(!on);
                        let _ = extras.show_follow();
                    }
                }
            });
        });
    }

    /// Shows a gift, an effect or a popup the switches let through.
    pub fn heard(&self, event: &LiveEvent) -> Result<()> {
        match event {
            LiveEvent::Gift { text } => {
                set_text(&self.root, "GiftText", &format!("🎁 {text}"))?;
                self.show_for_a_while("GiftLine", &self.gift_turn)
            }
            LiveEvent::Effect { text } => {
                set_text(&self.root, "EffectText", text)?;
                self.show_for_a_while("EffectLine", &self.effect_turn)
            }
            LiveEvent::Popup { text } => {
                set_text(&self.root, "PopupText", text)?;
                self.shown("PopupCard", true)
            }
            _ => Ok(()),
        }
    }

    fn show_for_a_while(&self, name: &'static str, turn: &Cell<u64>) -> Result<()> {
        self.shown(name, true)?;
        let mine = turn.get() + 1;
        turn.set(mine);
        let Some(shell) = shell() else { return Ok(()) };
        let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
        let (dispatcher, current) = (shell.dispatcher.clone(), self.current);
        let gift = name == "GiftLine";
        backend.spawn(async move {
            tokio::time::sleep(SHOWS).await;
            post(&dispatcher, move || {
                let Some(extras) = current() else { return };
                let now = if gift { extras.gift_turn.get() } else { extras.effect_turn.get() };
                if now == mine {
                    let _ = extras.shown(name, false);
                }
            });
        });
        Ok(())
    }

    /// What the switches have held back so far this session.
    pub fn held_back(&self, gifts: u32, popups: u32, effects: u32) -> Result<()> {
        if self.blocked.replace((gifts, popups, effects)) == (gifts, popups, effects) {
            return Ok(());
        }
        set_text(&self.root, "Blocked", &format!("已拦下 {}", gifts + popups + effects))?;
        // The settings, while open, say it by kind.
        if let Some(detail) = self.detail_text.borrow().as_ref() {
            detail.SetText(&self.detail())?;
        }
        Ok(())
    }

    fn detail(&self) -> String {
        let (g, p, e) = self.blocked.get();
        format!("本场已拦下：礼物横幅 {g} · 弹窗 {p} · 特效 {e}")
    }

    /// The switches, for the bottom of the settings; `apply` takes each change.
    pub fn switches(&self, filter: LiveFilter, apply: impl Fn(LiveFilter) + 'static) -> Result<UIElement> {
        let row = |label: &str, on: bool| {
            format!(
                r##"<CheckBox Content="{label}" IsChecked="{}" Margin="0,2,0,0" />"##,
                if on { "True" } else { "False" }
            )
        };
        let panel = load(&format!(
            r##"<StackPanel xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">
              <Border Height="1" Margin="0,12,0,6" Background="{{ThemeResource SystemControlForegroundBaseLowBrush}}" />
              <TextBlock Text="直播间过滤 · 打开即拦下" FontSize="12"
                         Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
              {}{}{}
              <TextBlock FontSize="12"
                         Margin="0,6,0,0" TextWrapping="Wrap" Text="{}"
                         Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
            </StackPanel>"##,
            row("礼物横幅", filter.gifts),
            row("抽奖 / 红包 / 购物弹窗", filter.popups),
            row("高能弹幕与进场特效", filter.effects),
            self.detail(),
        ))?;
        let parts = panel.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
        *self.detail_text.borrow_mut() = Some(parts.GetAt(parts.Size()? - 1)?.cast()?);
        let filter = Rc::new(Cell::new(filter));
        let apply: Rc<dyn Fn(LiveFilter)> = Rc::new(apply);
        for at in 0..3u32 {
            let switch = parts.GetAt(2 + at)?;
            let held = switch.cast::<crate::xaml::IToggleButton>()?;
            let (filter, apply, current) = (filter.clone(), apply.clone(), self.current);
            keep(switch.cast::<IButtonBase>()?.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                let on = held.IsChecked().unwrap_or(false);
                let mut next = filter.get();
                let hide = match at {
                    0 => {
                        next.gifts = on;
                        "GiftLine"
                    }
                    1 => {
                        next.popups = on;
                        "PopupCard"
                    }
                    _ => {
                        next.effects = on;
                        "EffectLine"
                    }
                };
                filter.set(next);
                if on && let Some(extras) = current() {
                    let _ = extras.shown(hide, false);
                }
                apply(next);
            })?);
        }
        panel.cast()
    }
}
