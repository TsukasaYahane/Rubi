//! Danmaku over the video: a fixed pool of text blocks, moved every frame from the playback
//! position while it moves. Loading and placing happen in the background; only moving touches XAML.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rubi_core::Services;
use rubi_model::Danmaku;
use windows_collections::IVector;
use windows_core::{EventRevoker, IInspectable, Interface, Ref, Result};

use crate::backend::Backend;
use crate::state::danmaku::{Item, Look, items, on_screen, tracks};
use crate::state::emotes::has_emotes;
use crate::ui::emote_text;
use crate::ui::shell::shell;
use crate::ui::{load, named, post, soon};
use crate::xaml::{
    Color, CompositionTarget, CoreDispatcher, IFrameworkElement, IPanel, IRectangleGeometry, IRichTextBlock,
    ITextBlock, ITranslateTransform, IUIElement, RectangleGeometry, SizeChangedEventArgs, SolidColorBrush, UIElement,
    Visibility,
};

/// Most danmaku on screen at once.
const POOL: usize = 72;
/// One danmaku: four black copies around the text make a 1pt outline, moved as one. A danmaku with
/// emotes uses the rich copies instead, the emotes drawn in each (the black ones hide under the
/// top one but for a pixel); plain text keeps to the light ones.
const SLOT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" Visibility="Collapsed">
  <Grid.RenderTransform><TranslateTransform /></Grid.RenderTransform>
  <Grid.Resources>
    <Style TargetType="TextBlock"><Setter Property="FontSize" Value="22" /></Style>
    <Style TargetType="RichTextBlock">
      <Setter Property="FontSize" Value="22" />
      <Setter Property="IsTextSelectionEnabled" Value="False" />
      <Setter Property="Visibility" Value="Collapsed" />
    </Style>
  </Grid.Resources>
  <TextBlock Foreground="Black" Margin="-1,-1,1,1" />
  <TextBlock Foreground="Black" Margin="1,-1,-1,1" />
  <TextBlock Foreground="Black" Margin="-1,1,1,-1" />
  <TextBlock Foreground="Black" Margin="1,1,-1,-1" />
  <TextBlock Foreground="White" />
  <RichTextBlock Foreground="Black" Margin="-1,-1,1,1" />
  <RichTextBlock Foreground="Black" Margin="1,-1,-1,1" />
  <RichTextBlock Foreground="Black" Margin="-1,1,1,-1" />
  <RichTextBlock Foreground="Black" Margin="1,1,-1,-1" />
  <RichTextBlock Foreground="White" />
</Grid>"##;
/// Emotes in a danmaku, square, a little over the text's height.
const EMOTE_PER_FONT: f64 = 1.27;
/// Copies of the text in a slot; the last is the coloured one on top.
const LAYERS: u32 = 5;

/// How long the size must hold still before danmaku are placed again.
const SETTLE: Duration = Duration::from_millis(500);
/// Live danmaku kept for placing.
const LIVE_KEPT: usize = 200;
/// Segments loaded together after the first.
const BATCH: usize = 3;
/// Frames the clock may stand still before frames stop: drawing every frame keeps the whole window
/// redrawing at the screen's rate, paused or not.
const STILL_FRAMES: u32 = 30;
/// How often a stopped overlay looks whether the clock moves again: about a frame, so danmaku set
/// off with the picture. A look only posts a little work; frames redraw the whole window.
const IDLE_LOOK: Duration = Duration::from_millis(16);

struct Slot {
    layers: Vec<ITextBlock>,
    /// The copies for a danmaku with emotes.
    rich: Vec<IRichTextBlock>,
    /// Which copies show: the plain ones, or the rich.
    rich_on: Cell<bool>,
    element: IUIElement,
    shift: ITranslateTransform,
    /// The item it shows, so text and colour change only when that does.
    shows: Cell<Option<usize>>,
    /// On screen. Apart from `shows`, which a new placing clears while the slot stays visible.
    visible: Cell<bool>,
}

/// Who holds an overlay: answers placed in the background find it through here.
#[derive(Clone, Copy)]
pub struct Host {
    pub run: fn() -> u64,
    pub find: fn(u64) -> Option<Rc<Overlay>>,
}

pub struct Overlay {
    host: Host,
    /// Where in the danmaku's time it is now, in ms.
    clock: Box<dyn Fn() -> u32>,
    canvas: IFrameworkElement,
    slots: Vec<Slot>,
    items: RefCell<Vec<Item>>,
    danmaku: RefCell<Arc<Vec<Danmaku>>>,
    /// A placing is under way, and more came in meanwhile.
    placing: Cell<bool>,
    placed_stale: Cell<bool>,
    on: Cell<bool>,
    /// Size, speed, area and kinds, as the viewer set them.
    look: Cell<Look>,
    /// The 空降点 found so far, and who wants to know of them.
    marks: RefCell<Vec<rubi_model::JumpMark>>,
    on_marks: RefCell<Option<Box<dyn Fn(&[rubi_model::JumpMark])>>>,
    /// Emotes the danmaku use beyond the common set: a live room's own.
    emotes: RefCell<BTreeMap<String, String>>,
    /// Itself, for the frame handler it starts again after stopping.
    me: Weak<Self>,
    /// Drawing every frame; `None` while the clock stands still or the danmaku are off.
    frames: RefCell<Option<EventRevoker>>,
    /// The clock at the last frame, and for how many frames it has not moved.
    last_ms: Cell<u32>,
    still: Cell<u32>,
    /// A look at the clock is waiting, while frames are stopped.
    looking: Cell<bool>,
    sizing: RefCell<Option<EventRevoker>>,
    clip_pending: Rc<Cell<bool>>,
    /// The size the clip was last set for: a frame at another size sets it again.
    clipped: Cell<(f64, f64)>,
    /// Bumped by each size change: only the last one places the danmaku again.
    resizes: Arc<AtomicU64>,
}

impl Overlay {
    pub fn attach(root: &IFrameworkElement, clock: Box<dyn Fn() -> u32>, host: Host) -> Result<Rc<Self>> {
        let canvas = named::<IFrameworkElement>(root, "Danmaku")?;
        let children = canvas.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
        let mut slots = Vec::with_capacity(POOL);
        for _ in 0..POOL {
            let slot = load(SLOT)?;
            let parts = slot.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
            let layers = (0..LAYERS).map(|at| parts.GetAt(at)?.cast()).collect::<Result<Vec<ITextBlock>>>()?;
            let rich =
                (LAYERS..2 * LAYERS).map(|at| parts.GetAt(at)?.cast()).collect::<Result<Vec<IRichTextBlock>>>()?;
            let element = slot.cast::<IUIElement>()?;
            let shift = element.RenderTransform()?.cast::<ITranslateTransform>()?;
            children.Append(&slot.cast::<UIElement>()?)?;
            slots.push(Slot {
                layers,
                rich,
                rich_on: Cell::new(false),
                element,
                shift,
                shows: Cell::new(None),
                visible: Cell::new(false),
            });
        }
        let overlay = Rc::new_cyclic(|me| Self {
            host,
            clock,
            placing: Cell::new(false),
            placed_stale: Cell::new(false),
            canvas,
            slots,
            items: RefCell::default(),
            danmaku: RefCell::default(),
            on: Cell::new(true),
            look: Cell::new(shell().map(|s| s.prefs().danmaku).unwrap_or_default()),
            marks: RefCell::default(),
            on_marks: RefCell::default(),
            emotes: RefCell::default(),
            me: me.clone(),
            frames: RefCell::new(None),
            last_ms: Cell::new(u32::MAX),
            still: Cell::new(0),
            looking: Cell::new(false),
            sizing: RefCell::new(None),
            clip_pending: Rc::default(),
            clipped: Cell::new((0.0, 0.0)),
            resizes: Arc::default(),
        });
        overlay.wake();
        let resizing = Rc::downgrade(&overlay);
        let sizing = overlay.canvas.SizeChanged(move |_: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
            let Some(overlay) = resizing.upgrade() else { return };
            overlay.wake();
            let pane = Rc::downgrade(&overlay);
            soon(&overlay.clip_pending, move || {
                if let Some(overlay) = pane.upgrade() {
                    let _ = overlay.clip();
                }
            });
            overlay.place_when_settled();
        })?;
        *overlay.sizing.borrow_mut() = Some(sizing);
        overlay.clip()?;
        overlay.dress_all()?;
        Ok(overlay)
    }

    /// Shows or hides the danmaku; returns whether they are on.
    pub fn toggle(&self) -> bool {
        let on = !self.on.get();
        self.on.set(on);
        // Off, the next frame hides what shows and then frames stop.
        self.wake();
        on
    }

    /// Draws every frame again, until the clock stands still.
    fn wake(&self) {
        self.still.set(0);
        if self.frames.borrow().is_some() {
            return;
        }
        let drawing = self.me.clone();
        let frames = CompositionTarget::Rendering(move |_: Ref<IInspectable>, _: Ref<IInspectable>| {
            if let Some(overlay) = drawing.upgrade() {
                let _ = overlay.draw();
                overlay.rest_if_still();
            }
        });
        if let Ok(frames) = frames {
            *self.frames.borrow_mut() = Some(frames);
        }
    }

    /// Stops drawing every frame once nothing on screen can move, and looks at the clock now and
    /// then instead.
    fn rest_if_still(&self) {
        let now_ms = (self.clock)();
        let still = if now_ms == self.last_ms.replace(now_ms) { self.still.get() + 1 } else { 0 };
        self.still.set(still);
        let nothing_shows = !self.on.get() && self.slots.iter().all(|slot| !slot.visible.get());
        if nothing_shows || still >= STILL_FRAMES {
            self.frames.borrow_mut().take();
            self.look_later();
        }
    }

    /// Looks at the clock in a little while, and draws every frame again if it moved.
    fn look_later(&self) {
        if self.looking.replace(true) {
            return;
        }
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, run, find) = (shell.dispatcher.clone(), (self.host.run)(), self.host.find);
        backend.spawn(async move {
            tokio::time::sleep(IDLE_LOOK).await;
            post(&dispatcher, move || {
                let Some(overlay) = find(run) else { return };
                overlay.looking.set(false);
                if overlay.frames.borrow().is_some() {
                    return;
                }
                if overlay.on.get() && (overlay.clock)() != overlay.last_ms.get() {
                    overlay.wake();
                } else {
                    overlay.look_later();
                }
            });
        });
    }

    /// Loads the video's danmaku, the part being watched first.
    pub fn load(&self, backend: Arc<Backend>, dispatcher: CoreDispatcher, cid: u64, duration_ms: u32) {
        let (run, find) = ((self.host.run)(), self.host.find);
        let core = backend.clone();
        let (size, look) = (self.size(), self.look.get());
        backend.spawn(async move {
            let services = &core.services;
            let Ok(session) = services.danmaku.session(cid, duration_ms) else { return };
            let segments: Vec<u32> = (1..=session.segment_count()).collect();
            let (first, rest) = segments.split_at(1.min(segments.len()));
            for batch in std::iter::once(first).chain(rest.chunks(BATCH)) {
                let Ok(mut output) = session.load(batch).await else { continue };
                let placed = place(services, &output.kept, size, look);
                let marks = std::mem::take(&mut output.jump_marks);
                let danmaku = Arc::new(output.kept);
                post(&dispatcher, move || {
                    if let Some(overlay) = find(run) {
                        overlay.set(danmaku, placed);
                        overlay.add_marks(marks);
                    }
                });
            }
        });
    }

    /// Adds what a live room just said, keeping the latest.
    pub fn add_live(&self, fresh: Vec<Danmaku>) {
        if fresh.is_empty() {
            return;
        }
        let mut all = self.danmaku.borrow().as_ref().clone();
        all.extend(fresh);
        let extra = all.len().saturating_sub(LIVE_KEPT);
        all.drain(..extra);
        *self.danmaku.borrow_mut() = Arc::new(all);
        self.place();
    }

    /// Adds one just sent, so its sender sees it.
    pub fn add(&self, sent: Danmaku) {
        let mut all = self.danmaku.borrow().as_ref().clone();
        all.push(sent);
        *self.danmaku.borrow_mut() = Arc::new(all);
        self.place();
    }

    fn set(&self, danmaku: Arc<Vec<Danmaku>>, placed: Vec<Item>) {
        *self.danmaku.borrow_mut() = danmaku;
        *self.items.borrow_mut() = placed;
        self.forget_slots();
        self.wake();
        self.placing.set(false);
        if self.placed_stale.take() {
            self.place();
        }
    }

    fn forget_slots(&self) {
        for slot in &self.slots {
            slot.shows.set(None);
        }
    }

    /// Places everything again once the size stops changing: placing is the slow part, and
    /// danmaku in flight only need the new width, which [`draw`](Self::draw) reads every frame.
    fn place_when_settled(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let turn = self.resizes.fetch_add(1, Ordering::Relaxed) + 1;
        let (resizes, dispatcher, run) = (self.resizes.clone(), shell.dispatcher.clone(), (self.host.run)());
        let find = self.host.find;
        backend.spawn(async move {
            tokio::time::sleep(SETTLE).await;
            if resizes.load(Ordering::Relaxed) == turn {
                post(&dispatcher, move || {
                    if let Some(overlay) = find(run) {
                        overlay.place();
                    }
                });
            }
        });
    }

    /// Places everything again for the picture's size.
    fn place(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (danmaku, size, run, look) =
            (self.danmaku.borrow().clone(), self.size(), (self.host.run)(), self.look.get());
        if danmaku.is_empty() {
            return;
        }
        if self.placing.replace(true) {
            self.placed_stale.set(true);
            return;
        }
        let find = self.host.find;
        let dispatcher = shell.dispatcher.clone();
        let core = backend.clone();
        backend.spawn(async move {
            let placed = place(&core.services, &danmaku, size, look);
            post(&dispatcher, move || {
                if let Some(overlay) = find(run) {
                    overlay.set(danmaku, placed);
                }
            });
        });
    }

    fn size(&self) -> (f64, f64) {
        (self.canvas.ActualWidth().unwrap_or(0.0), self.canvas.ActualHeight().unwrap_or(0.0))
    }

    /// Keeps danmaku inside the picture.
    fn clip(&self) -> Result<()> {
        let (width, height) = self.size();
        self.clipped.set((width, height));
        let geometry = RectangleGeometry::new()?;
        geometry.cast::<IRectangleGeometry>()?.SetRect(crate::xaml::Rect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        })?;
        self.canvas.cast::<IUIElement>()?.SetClip(&geometry)
    }

    fn draw(&self) -> Result<()> {
        let now_ms = (self.clock)();
        let (width, height) = self.size();
        // A size change can come without its event (into fullscreen inside the scrolling page):
        // the clip and the placing follow the size seen here.
        if self.clipped.get() != (width, height) {
            self.clip()?;
            // Place at once for the size as it is now, and again once it settles: a fullscreen
            // switch animates its layout, so the size seen mid-way is not the one to keep.
            self.place();
            self.place_when_settled();
        }
        let items = self.items.borrow();
        let mut used = 0;
        if self.on.get() {
            for (at, (x, y)) in on_screen(&items, now_ms, width, height, self.look.get()) {
                let Some(slot) = self.slots.get(used) else { break };
                used += 1;
                if slot.shows.replace(Some(at)) != Some(at) {
                    self.dress(slot, &items[at])?;
                    slot.element.SetVisibility(Visibility::Visible)?;
                    slot.visible.set(true);
                }
                slot.shift.SetX(x)?;
                slot.shift.SetY(y)?;
            }
        }
        for slot in &self.slots[used..] {
            slot.shows.set(None);
            if slot.visible.take() {
                slot.element.SetVisibility(Visibility::Collapsed)?;
            }
        }
        Ok(())
    }
}

impl Overlay {
    /// Calls `show` with the 空降点 found, now and as more come.
    pub fn watch_marks(&self, show: impl Fn(&[rubi_model::JumpMark]) + 'static) {
        show(&self.marks.borrow());
        *self.on_marks.borrow_mut() = Some(Box::new(show));
    }

    fn add_marks(&self, more: Vec<rubi_model::JumpMark>) {
        if more.is_empty() {
            return;
        }
        let mut marks = self.marks.borrow_mut();
        for mark in more {
            if !marks.iter().any(|m| m.target_ms == mark.target_ms) {
                marks.push(mark);
            }
        }
        marks.sort_by_key(|m| m.target_ms);
        let marks = marks.clone();
        if let Some(show) = self.on_marks.borrow().as_ref() {
            show(&marks);
        }
    }

    /// The size, speed, area and kinds on show.
    pub fn look(&self) -> Look {
        self.look.get()
    }

    /// Shows the danmaku as `look` says, at once.
    pub fn set_look(&self, look: Look) -> Result<()> {
        self.look.set(look);
        self.dress_all()?;
        self.forget_slots();
        self.place();
        self.wake();
        Ok(())
    }

    /// The opacity and text size of every slot.
    fn dress_all(&self) -> Result<()> {
        let look = self.look.get();
        self.canvas.cast::<IUIElement>()?.SetOpacity(look.opacity)?;
        for slot in &self.slots {
            for layer in &slot.layers {
                layer.SetFontSize(look.size)?;
            }
            for layer in &slot.rich {
                layer.SetFontSize(look.size)?;
            }
        }
        Ok(())
    }

    /// Adds emotes a live room's danmaku use, by token.
    pub fn add_emotes(&self, more: BTreeMap<String, String>) {
        self.emotes.borrow_mut().extend(more);
    }

    /// Puts `item`'s text and colour in `slot`, on the rich copies when it has emotes.
    fn dress(&self, slot: &Slot, item: &Item) -> Result<()> {
        let own = self.emotes.borrow();
        let rich = has_emotes(&item.text, emote_text::lookup(&own));
        if slot.rich_on.replace(rich) != rich {
            let (on, off) = if rich {
                (Visibility::Visible, Visibility::Collapsed)
            } else {
                (Visibility::Collapsed, Visibility::Visible)
            };
            for layer in &slot.rich {
                layer.cast::<IUIElement>()?.SetVisibility(on)?;
            }
            for layer in &slot.layers {
                layer.cast::<IUIElement>()?.SetVisibility(off)?;
            }
        }
        if rich {
            for layer in &slot.rich {
                emote_text::show(layer, &item.text, &own, self.look.get().size * EMOTE_PER_FONT)?;
            }
            if let Some(top) = slot.rich.last() {
                top.SetForeground(&brush(item.color)?)?;
            }
        } else {
            for layer in &slot.layers {
                layer.SetText(&item.text)?;
            }
            if let Some(top) = slot.layers.last() {
                top.SetForeground(&brush(item.color)?)?;
            }
        }
        Ok(())
    }
}

fn brush(rgb: u32) -> Result<SolidColorBrush> {
    SolidColorBrush::CreateInstanceWithColor(Color { a: 255, r: (rgb >> 16) as u8, g: (rgb >> 8) as u8, b: rgb as u8 })
}

fn place(services: &Services, danmaku: &[Danmaku], (width, height): (f64, f64), look: Look) -> Vec<Item> {
    let shown: Vec<Danmaku> = danmaku.iter().filter(|d| look.shows(d)).cloned().collect();
    let placements = services.danmaku.layout(
        &shown,
        Vec::new(),
        width as f32,
        look.size as f32,
        tracks(height, &look),
        look.scroll_ms(),
    );
    items(&shown, &placements)
}
