//! Danmaku over the video: a fixed pool of text blocks, moved every frame from the playback
//! position. Loading and placing happen in the background; only moving touches XAML.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rubi_core::Services;
use rubi_model::Danmaku;
use windows_collections::IVector;
use windows_core::{EventRevoker, IInspectable, Interface, Ref, Result};

use crate::backend::Backend;
use crate::state::danmaku::{FONT_SIZE, Item, SCROLL_MS, items, on_screen, tracks};
use crate::ui::shell::shell;
use crate::ui::{load, named, player, post, soon};
use crate::xaml::{
    Color, CompositionTarget, CoreDispatcher, IFrameworkElement, IMediaPlaybackSession, IPanel, IRectangleGeometry,
    ITextBlock, ITranslateTransform, IUIElement, MediaPlaybackSession, RectangleGeometry, SizeChangedEventArgs,
    SolidColorBrush, UIElement, Visibility,
};

/// Most danmaku on screen at once.
const POOL: usize = 72;
/// One danmaku: four black copies around the text make a 1pt outline, moved as one.
const SLOT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" Visibility="Collapsed">
  <Grid.RenderTransform><TranslateTransform /></Grid.RenderTransform>
  <Grid.Resources>
    <Style TargetType="TextBlock"><Setter Property="FontSize" Value="22" /></Style>
  </Grid.Resources>
  <TextBlock Foreground="Black" Margin="-1,-1,1,1" />
  <TextBlock Foreground="Black" Margin="1,-1,-1,1" />
  <TextBlock Foreground="Black" Margin="-1,1,1,-1" />
  <TextBlock Foreground="Black" Margin="1,1,-1,-1" />
  <TextBlock Foreground="White" />
</Grid>"##;
/// Copies of the text in a slot; the last is the coloured one on top.
const LAYERS: u32 = 5;

/// How long the size must hold still before danmaku are placed again.
const SETTLE: Duration = Duration::from_millis(200);
/// Segments loaded together after the first.
const BATCH: usize = 3;

struct Slot {
    layers: Vec<ITextBlock>,
    element: IUIElement,
    shift: ITranslateTransform,
    /// The item it shows, so text and colour change only when that does.
    shows: Cell<Option<usize>>,
}

pub struct Overlay {
    canvas: IFrameworkElement,
    slots: Vec<Slot>,
    items: RefCell<Vec<Item>>,
    danmaku: RefCell<Arc<Vec<Danmaku>>>,
    session: MediaPlaybackSession,
    on: Cell<bool>,
    frames: RefCell<Option<EventRevoker>>,
    sizing: RefCell<Option<EventRevoker>>,
    clip_pending: Rc<Cell<bool>>,
    /// Bumped by each size change: only the last one places the danmaku again.
    resizes: Arc<AtomicU64>,
}

impl Overlay {
    pub fn attach(root: &IFrameworkElement, session: MediaPlaybackSession) -> Result<Rc<Self>> {
        let canvas = named::<IFrameworkElement>(root, "Danmaku")?;
        let children = canvas.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
        let mut slots = Vec::with_capacity(POOL);
        for _ in 0..POOL {
            let slot = load(SLOT)?;
            let parts = slot.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
            let layers = (0..LAYERS).map(|at| parts.GetAt(at)?.cast()).collect::<Result<Vec<ITextBlock>>>()?;
            let element = slot.cast::<IUIElement>()?;
            let shift = element.RenderTransform()?.cast::<ITranslateTransform>()?;
            children.Append(&slot.cast::<UIElement>()?)?;
            slots.push(Slot { layers, element, shift, shows: Cell::new(None) });
        }
        let overlay = Rc::new(Self {
            canvas,
            slots,
            items: RefCell::default(),
            danmaku: RefCell::default(),
            session,
            on: Cell::new(true),
            frames: RefCell::new(None),
            sizing: RefCell::new(None),
            clip_pending: Rc::default(),
            resizes: Arc::default(),
        });
        let drawing = Rc::downgrade(&overlay);
        let frames = CompositionTarget::Rendering(move |_: Ref<IInspectable>, _: Ref<IInspectable>| {
            if let Some(overlay) = drawing.upgrade() {
                let _ = overlay.draw();
            }
        })?;
        *overlay.frames.borrow_mut() = Some(frames);
        let resizing = Rc::downgrade(&overlay);
        let sizing = overlay.canvas.SizeChanged(move |_: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
            let Some(overlay) = resizing.upgrade() else { return };
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
        Ok(overlay)
    }

    /// Shows or hides the danmaku; returns whether they are on.
    pub fn toggle(&self) -> bool {
        let on = !self.on.get();
        self.on.set(on);
        on
    }

    /// Loads the video's danmaku, the part being watched first.
    pub fn load(&self, backend: Arc<Backend>, dispatcher: CoreDispatcher, cid: u64, duration_ms: u32) {
        let run = player::run();
        let core = backend.clone();
        let size = self.size();
        backend.spawn(async move {
            let services = &core.services;
            let Ok(session) = services.danmaku.session(cid, duration_ms) else { return };
            let segments: Vec<u32> = (1..=session.segment_count()).collect();
            let (first, rest) = segments.split_at(1.min(segments.len()));
            for batch in std::iter::once(first).chain(rest.chunks(BATCH)) {
                let Ok(output) = session.load(batch).await else { continue };
                let placed = place(services, &output.kept, size);
                let danmaku = Arc::new(output.kept);
                post(&dispatcher, move || {
                    if let Some(overlay) = player::overlay(run) {
                        overlay.set(danmaku, placed);
                    }
                });
            }
        });
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
        let (resizes, dispatcher, run) = (self.resizes.clone(), shell.dispatcher.clone(), player::run());
        backend.spawn(async move {
            tokio::time::sleep(SETTLE).await;
            if resizes.load(Ordering::Relaxed) == turn {
                post(&dispatcher, move || {
                    if let Some(overlay) = player::overlay(run) {
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
        let (danmaku, size, run) = (self.danmaku.borrow().clone(), self.size(), player::run());
        if danmaku.is_empty() {
            return;
        }
        let dispatcher = shell.dispatcher.clone();
        let core = backend.clone();
        backend.spawn(async move {
            let placed = place(&core.services, &danmaku, size);
            post(&dispatcher, move || {
                if let Some(overlay) = player::overlay(run) {
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
        let session = self.session.cast::<IMediaPlaybackSession>()?;
        let now_ms = (session.Position()?.duration / 10_000) as u32;
        let (width, height) = self.size();
        let items = self.items.borrow();
        let mut used = 0;
        if self.on.get() {
            for (at, (x, y)) in on_screen(&items, now_ms, width, height) {
                let Some(slot) = self.slots.get(used) else { break };
                used += 1;
                if slot.shows.replace(Some(at)) != Some(at) {
                    let item = &items[at];
                    for layer in &slot.layers {
                        layer.SetText(&item.text)?;
                    }
                    if let Some(top) = slot.layers.last() {
                        top.SetForeground(&brush(item.color)?)?;
                    }
                    slot.element.SetVisibility(Visibility::Visible)?;
                }
                slot.shift.SetX(x)?;
                slot.shift.SetY(y)?;
            }
        }
        for slot in &self.slots[used..] {
            if slot.shows.take().is_some() {
                slot.element.SetVisibility(Visibility::Collapsed)?;
            }
        }
        Ok(())
    }
}

fn brush(rgb: u32) -> Result<SolidColorBrush> {
    SolidColorBrush::CreateInstanceWithColor(Color { a: 255, r: (rgb >> 16) as u8, g: (rgb >> 8) as u8, b: rgb as u8 })
}

fn place(services: &Services, danmaku: &[Danmaku], (width, height): (f64, f64)) -> Vec<Item> {
    let placements =
        services.danmaku.layout(danmaku, Vec::new(), width as f32, FONT_SIZE as f32, tracks(height), SCROLL_MS);
    items(danmaku, &placements)
}
