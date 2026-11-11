//! Two pictures, one above the other, so a new stream can come up behind the one on show and take
//! its place without the picture ever standing empty.
//!
//! A picture is a composition visual inside an empty element, painted with the player's own
//! surface. The surface keeps the video's size and the brush scales it into the visual, so the
//! picture follows a resize at once: a `MediaPlayerElement` remakes its surface on each new size,
//! and many in a row could leave it blank.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use windows_core::{IInspectable, Interface, Ref, Result};
use windows_numerics::Vector2;

use crate::ui::shell::shell;
use crate::ui::{keep, media, named, post};
use crate::xaml::{
    CompositionBrush, CompositionStretch, Compositor, ElementCompositionPreview, IClosable, ICompositionObject,
    ICompositionSurfaceBrush, ICompositor, IFrameworkElement, IMediaPlaybackSession, IMediaPlayer, IMediaPlayer3,
    IMediaPlayer4, IMediaPlayerSurface, ISpriteVisual, IUIElement, IVisual, MediaPlaybackSession, MediaPlayer,
    MediaPlayerSurface, MediaSource, Size, SizeChangedEventArgs, SpriteVisual, UIElement, Visibility,
};

/// How long the old picture stays under the new one: the new one's first frame must be up.
const OVERLAP: Duration = Duration::from_millis(250);
const NAMES: [&str; 2] = ["Video", "VideoNext"];

/// The second element, for a player's layout. Collapsed until wanted: even see-through, an element
/// left in place makes the compositor draw the video with the page, and danmaku stutter with it.
pub const SECOND_ELEMENT: &str = r##"x:Name="VideoNext" Visibility="Collapsed" IsHitTestVisible="False""##;

/// One picture: the visual in its element, and the surface it shows while it has a player.
struct Picture {
    visual: SpriteVisual,
    surface: RefCell<Option<MediaPlayerSurface>>,
}

pub struct TwinVideo {
    root: IFrameworkElement,
    compositor: Compositor,
    pictures: [Picture; 2],
    /// Which of the two is on show.
    active: Cell<usize>,
    retiring: RefCell<Option<(usize, MediaPlayer, MediaSource)>>,
    /// The page's run, and the pair on show: a timer may outlive this one.
    run: fn() -> u64,
    find: fn() -> Option<Rc<TwinVideo>>,
}

/// Puts a visual into the element named `name`, sized with it as it changes.
fn picture(root: &IFrameworkElement, name: &str, compositor: &Compositor) -> Result<Picture> {
    let element = named::<UIElement>(root, name)?;
    let visual = compositor.cast::<ICompositor>()?.CreateSpriteVisual()?;
    ElementCompositionPreview::SetElementChildVisual(&element, &visual)?;
    let sizing = visual.clone();
    let framed = element.cast::<IFrameworkElement>()?;
    fit(&sizing, framed.ActualWidth()?, framed.ActualHeight()?)?;
    keep(framed.SizeChanged(move |sender: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
        if let Ok(element) = sender.ok().and_then(|s| s.cast::<IFrameworkElement>()) {
            let (width, height) = (element.ActualWidth().unwrap_or(0.0), element.ActualHeight().unwrap_or(0.0));
            let _ = fit(&sizing, width, height);
        }
    })?);
    Ok(Picture { visual, surface: RefCell::default() })
}

/// The player draws onto a surface of the video's own size, which the brush scales to the
/// picture: otherwise it picks a size of its own and the picture comes out cut or stretched. The
/// size is set again when the stream's changes (a new quality, a rotated part).
/// Until the size is known the picture stays hidden: the first frames come at a size of the
/// player's own and would show cut.
fn follow_video_size(player: &MediaPlayer, visual: &SpriteVisual) -> Result<()> {
    let session = player.cast::<IMediaPlayer3>()?.PlaybackSession()?.cast::<IMediaPlaybackSession>()?;
    let sized_now = size_surface(player, &session);
    visual.cast::<IVisual>()?.SetOpacity(if sized_now { 1.0 } else { 0.0 })?;
    let Some(dispatcher) = shell().map(|s| s.dispatcher.clone()) else { return Ok(()) };
    let (sized, showing) = (player.clone(), visual.clone());
    keep(session.NaturalVideoSizeChanged(move |session: Ref<MediaPlaybackSession>, _: Ref<IInspectable>| {
        let Ok(session) = session.ok().and_then(|s| s.cast::<IMediaPlaybackSession>()) else { return };
        if size_surface(&sized, &session) {
            let visual = showing.clone();
            post(&dispatcher, move || {
                let _ = visual.cast::<IVisual>().and_then(|v| v.SetOpacity(1.0));
            });
        }
    })?);
    Ok(())
}

/// Sizes the player's surface to the stream; whether the stream's size was known.
fn size_surface(player: &MediaPlayer, session: &IMediaPlaybackSession) -> bool {
    let (Ok(width), Ok(height)) = (session.NaturalVideoWidth(), session.NaturalVideoHeight()) else { return false };
    if width == 0 || height == 0 {
        return false;
    }
    let size = Size { width: width as f32, height: height as f32 };
    player.cast::<IMediaPlayer4>().and_then(|p| p.SetSurfaceSize(size)).is_ok()
}

fn fit(visual: &SpriteVisual, width: f64, height: f64) -> Result<()> {
    visual.cast::<IVisual>()?.SetSize(Vector2 { x: width as f32, y: height as f32 })
}

impl TwinVideo {
    pub fn new(root: &IFrameworkElement, run: fn() -> u64, find: fn() -> Option<Rc<TwinVideo>>) -> Result<Rc<Self>> {
        let first = named::<UIElement>(root, NAMES[0])?;
        let compositor =
            ElementCompositionPreview::GetElementVisual(&first)?.cast::<ICompositionObject>()?.Compositor()?;
        let pictures = [picture(root, NAMES[0], &compositor)?, picture(root, NAMES[1], &compositor)?];
        Ok(Rc::new(Self {
            root: root.clone(),
            compositor,
            pictures,
            active: Cell::new(0),
            retiring: RefCell::default(),
            run,
            find,
        }))
    }

    fn next(&self) -> usize {
        1 - self.active.get()
    }

    fn element(&self, at: usize) -> Result<IUIElement> {
        named(&self.root, NAMES[at])
    }

    /// Shows `player` in picture `at`.
    fn attach(&self, at: usize, player: &MediaPlayer) -> Result<()> {
        self.detach(at);
        let surface = player.cast::<IMediaPlayer4>()?.GetSurface(&self.compositor)?;
        follow_video_size(player, &self.pictures[at].visual)?;
        let content = surface.cast::<IMediaPlayerSurface>()?.CompositionSurface()?;
        let brush = self.compositor.cast::<ICompositor>()?.CreateSurfaceBrushWithSurface(&content)?;
        brush.cast::<ICompositionSurfaceBrush>()?.SetStretch(CompositionStretch::Uniform)?;
        self.pictures[at].visual.cast::<ISpriteVisual>()?.SetBrush(&brush.cast::<CompositionBrush>()?)?;
        *self.pictures[at].surface.borrow_mut() = Some(surface);
        Ok(())
    }

    /// Picture `at` lets go of its player, as it must before the player closes.
    fn detach(&self, at: usize) {
        let picture = &self.pictures[at];
        let _ = picture.visual.cast::<ISpriteVisual>().and_then(|v| v.SetBrush(None::<&CompositionBrush>));
        if let Some(surface) = picture.surface.borrow_mut().take() {
            let _ = surface.cast::<IClosable>().and_then(|s| s.Close());
        }
    }

    /// Shows `player` in the picture on show.
    pub fn show(&self, player: &MediaPlayer) -> Result<()> {
        self.attach(self.active.get(), player)
    }

    /// Gives `player` to the picture out of sight.
    pub fn prepare(&self, player: &MediaPlayer) -> Result<()> {
        let next = self.element(self.next())?;
        next.SetOpacity(0.0)?;
        next.SetVisibility(Visibility::Visible)?;
        self.attach(self.next(), player)
    }

    /// Takes the waiting player back from the picture out of sight.
    pub fn clear_next(&self) -> Result<()> {
        self.detach(self.next());
        self.element(self.next())?.SetVisibility(Visibility::Collapsed)
    }

    /// Shows the picture out of sight, and retires what played in the other.
    pub fn switch(&self, old: Option<MediaPlayer>, old_source: Option<MediaSource>) -> Result<()> {
        let leaving = self.active.replace(self.next());
        self.element(self.active.get())?.SetOpacity(1.0)?;
        if let Some(old) = &old {
            old.cast::<IMediaPlayer>()?.SetIsMuted(true)?;
        }
        let (Some(old), Some(old_source)) = (old, old_source) else { return Ok(()) };
        *self.retiring.borrow_mut() = Some((leaving, old, old_source));
        let Some(shell) = shell() else { return Ok(()) };
        let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
        let (dispatcher, opening, run, find) = (shell.dispatcher.clone(), (self.run)(), self.run, self.find);
        backend.spawn(async move {
            tokio::time::sleep(OVERLAP).await;
            post(&dispatcher, move || {
                if opening == run()
                    && let Some(twin) = find()
                {
                    let _ = twin.finish();
                }
            });
        });
        Ok(())
    }

    fn finish(&self) -> Result<()> {
        let Some((at, player, source)) = self.retiring.borrow_mut().take() else { return Ok(()) };
        self.detach(at);
        self.element(at)?.SetVisibility(Visibility::Collapsed)?;
        media::close(Some(player), Some(source));
        Ok(())
    }

    /// Both pictures let go, as they must before their players close.
    pub fn clear(&self) {
        self.detach(0);
        self.detach(1);
        if let Some((_, player, source)) = self.retiring.borrow_mut().take() {
            media::close(Some(player), Some(source));
        }
    }
}
