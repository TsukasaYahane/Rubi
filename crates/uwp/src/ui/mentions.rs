//! @ 提到 in a text box: typing `@` lists the followed UPs and recent chats, narrowed as the name
//! is typed; ↑↓ move, Enter or a click puts `@name ` in. Who was picked is kept, and the core
//! sends only those whose `@name` is still in the text.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::settings::escape;
use crate::ui::contacts::{self, Contact};
use crate::ui::{keep, load, named};
use crate::xaml::{
    IButtonBase, IContentControl, IFrameworkElement, IKeyRoutedEventArgs, IPanel, ITextBox, IUIElement,
    KeyRoutedEventArgs, RoutedEventArgs, TextChangedEventArgs, UIElement, VirtualKey, Visibility,
};

/// Names the list shows at once.
const SHOWN: usize = 6;

pub struct Mentions {
    text: ITextBox,
    list: IFrameworkElement,
    /// Matches on show and the one chosen.
    matches: RefCell<Vec<Contact>>,
    chosen: Cell<usize>,
    /// Where the `@` being typed is, in UTF-16 units.
    at: Cell<Option<usize>>,
    picked: RefCell<Vec<(String, u64)>>,
}

impl Mentions {
    /// Watches the text box `text` and lists names in `list` (a `ContentControl`) under `root`.
    pub fn attach(root: &IFrameworkElement, text: &str, list: &str) -> Result<Rc<Self>> {
        let mentions = Rc::new(Self {
            text: named::<ITextBox>(root, text)?,
            list: named::<IFrameworkElement>(root, list)?,
            matches: RefCell::default(),
            chosen: Cell::new(0),
            at: Cell::new(None),
            picked: RefCell::default(),
        });
        let weak = Rc::downgrade(&mentions);
        keep(mentions.text.TextChanged(move |_: Ref<IInspectable>, _: Ref<TextChangedEventArgs>| {
            if let Some(mentions) = weak.upgrade() {
                let _ = mentions.update();
            }
        })?);
        let weak = Rc::downgrade(&mentions);
        keep(mentions.text.cast::<IUIElement>()?.KeyDown(move |_: Ref<IInspectable>, args: Ref<KeyRoutedEventArgs>| {
            let (Some(mentions), Ok(args)) = (weak.upgrade(), args.ok().and_then(|a| a.cast::<IKeyRoutedEventArgs>()))
            else {
                return;
            };
            if mentions.is_open() && args.Key().is_ok_and(|k| mentions.key(k)) {
                let _ = args.SetHandled(true);
            }
        })?);
        Ok(mentions)
    }

    /// The list is on show: Enter picks rather than sends.
    pub fn is_open(&self) -> bool {
        self.at.get().is_some() && !self.matches.borrow().is_empty()
    }

    /// Who was picked, for the core to keep those still named.
    pub fn picked(&self) -> Vec<(String, u64)> {
        self.picked.borrow().clone()
    }

    /// After sending: nobody is picked.
    pub fn clear(&self) {
        self.picked.borrow_mut().clear();
        let _ = self.close();
    }

    fn key(self: &Rc<Self>, key: VirtualKey) -> bool {
        let count = self.matches.borrow().len();
        match key {
            VirtualKey::Down => self.chosen.set((self.chosen.get() + 1) % count),
            VirtualKey::Up => self.chosen.set((self.chosen.get() + count - 1) % count),
            VirtualKey::Enter => {
                let _ = self.pick(self.chosen.get());
                return true;
            }
            VirtualKey::Escape => {
                let _ = self.close();
                return true;
            }
            _ => return false,
        }
        let _ = self.draw();
        true
    }

    /// The `@` before the caret and what follows it, when a name is being typed.
    fn typed(&self) -> Result<Option<(usize, String)>> {
        let text: Vec<u16> = self.text.Text()?.encode_utf16().collect();
        let caret = (self.text.SelectionStart()?.max(0) as usize).min(text.len());
        let before = &text[..caret];
        let Some(at) = before.iter().rposition(|&c| c == u16::from(b'@')) else { return Ok(None) };
        let query = String::from_utf16_lossy(&before[at + 1..]);
        if query.chars().any(char::is_whitespace) || query.chars().count() > 20 {
            return Ok(None);
        }
        Ok(Some((at, query)))
    }

    fn update(self: &Rc<Self>) -> Result<()> {
        let Some((at, query)) = self.typed()? else { return self.close() };
        self.at.set(Some(at));
        let Some(people) = contacts::known() else {
            let weak = Rc::downgrade(self);
            contacts::with(move |_| {
                if let Some(mentions) = weak.upgrade() {
                    let _ = mentions.update();
                }
            });
            return self.say("正在读取关注和最近聊天…");
        };
        let query = query.to_lowercase();
        let found: Vec<Contact> =
            people.iter().filter(|p| p.name.to_lowercase().contains(&query)).take(SHOWN).cloned().collect();
        if found.is_empty() {
            return self.close();
        }
        *self.matches.borrow_mut() = found;
        self.chosen.set(0);
        self.draw()
    }

    fn say(&self, text: &str) -> Result<()> {
        self.matches.borrow_mut().clear();
        let line = load(&format!(
            r##"<TextBlock xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" Text="{}" FontSize="12"
                 Margin="8,6" Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />"##,
            escape(text)
        ))?;
        self.list.cast::<IContentControl>()?.SetContent(&line.cast::<IInspectable>()?)?;
        self.list.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)
    }

    fn draw(self: &Rc<Self>) -> Result<()> {
        let matches = self.matches.borrow();
        let chosen = self.chosen.get();
        let rows: String = matches
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let fill = if i == chosen { "{ThemeResource SystemControlBackgroundBaseLowBrush}" } else { "Transparent" };
                format!(
                    r##"<Button HorizontalAlignment="Stretch" HorizontalContentAlignment="Stretch" Background="{fill}"
                         BorderThickness="0" Padding="8,5" AllowFocusOnInteraction="False">
                       <Grid><TextBlock Text="{}" FontSize="13" />
                         <TextBlock Text="{}" FontSize="11" HorizontalAlignment="Right" VerticalAlignment="Center"
                                    Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" /></Grid></Button>"##,
                    escape(&p.name),
                    p.from
                )
            })
            .collect();
        drop(matches);
        let panel = load(&format!(
            r##"<StackPanel xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">{rows}</StackPanel>"##
        ))?;
        let buttons = panel.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
        for at in 0..buttons.Size()? {
            let weak = Rc::downgrade(self);
            keep(buttons.GetAt(at)?.cast::<IButtonBase>()?.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                if let Some(mentions) = weak.upgrade() {
                    let _ = mentions.pick(at as usize);
                }
            })?);
        }
        self.list.cast::<IContentControl>()?.SetContent(&panel.cast::<IInspectable>()?)?;
        self.list.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)
    }

    fn pick(&self, at: usize) -> Result<()> {
        let Some(person) = self.matches.borrow().get(at).cloned() else { return Ok(()) };
        let Some(start) = self.at.get() else { return Ok(()) };
        let text: Vec<u16> = self.text.Text()?.encode_utf16().collect();
        let caret = (self.text.SelectionStart()?.max(0) as usize).min(text.len());
        let inserted: Vec<u16> = format!("@{} ", person.name).encode_utf16().collect();
        let mut next = text[..start].to_vec();
        next.extend_from_slice(&inserted);
        next.extend_from_slice(&text[caret..]);
        self.close()?;
        self.text.SetText(&String::from_utf16_lossy(&next))?;
        self.text.SetSelectionStart((start + inserted.len()) as i32)?;
        let mut picked = self.picked.borrow_mut();
        if !picked.iter().any(|p| p.1 == person.mid) {
            picked.push((person.name, person.mid));
        }
        Ok(())
    }

    fn close(&self) -> Result<()> {
        self.at.set(None);
        self.matches.borrow_mut().clear();
        self.list.cast::<IUIElement>()?.SetVisibility(Visibility::Collapsed)
    }
}
