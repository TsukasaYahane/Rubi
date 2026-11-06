//! Two picture elements, one above the other, so a new stream can come up behind the one on show
//! and take its place without the element ever standing empty.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use windows_core::{Interface, Result};

use crate::ui::shell::shell;
use crate::ui::{media, named, post};
use crate::xaml::{IFrameworkElement, IMediaPlayer, IMediaPlayerElement, IUIElement, MediaPlayer, MediaSource};

/// How long the old picture stays under the new one: the new one's first frame must be up.
const OVERLAP: Duration = Duration::from_millis(250);
const FIRST: &str = "Video";
const SECOND: &str = "VideoNext";

/// The second element, for a player's layout.
pub const SECOND_ELEMENT: &str = r##"x:Name="VideoNext" AreTransportControlsEnabled="False" Stretch="Uniform"
  Opacity="0" IsHitTestVisible="False""##;

pub struct TwinVideo {
    root: IFrameworkElement,
    active: Cell<&'static str>,
    retiring: RefCell<Option<(&'static str, MediaPlayer, MediaSource)>>,
    /// The page's run, and the pair on show: a timer may outlive this one.
    run: fn() -> u64,
    find: fn() -> Option<Rc<TwinVideo>>,
}

impl TwinVideo {
    pub fn new(root: &IFrameworkElement, run: fn() -> u64, find: fn() -> Option<Rc<TwinVideo>>) -> Rc<Self> {
        Rc::new(Self { root: root.clone(), active: Cell::new(FIRST), retiring: RefCell::default(), run, find })
    }

    /// The name of the element on show.
    pub fn active(&self) -> &'static str {
        self.active.get()
    }

    fn next(&self) -> &'static str {
        if self.active.get() == FIRST { SECOND } else { FIRST }
    }

    /// Gives `player` to the element out of sight.
    pub fn prepare(&self, player: &MediaPlayer) -> Result<()> {
        named::<IMediaPlayerElement>(&self.root, self.next())?.SetMediaPlayer(player)
    }

    /// Takes the waiting player back from the element out of sight.
    pub fn clear_next(&self) -> Result<()> {
        named::<IMediaPlayerElement>(&self.root, self.next())?.SetMediaPlayer(None::<&MediaPlayer>)
    }

    /// Shows the element out of sight, and retires what played in the other.
    pub fn switch(&self, old: Option<MediaPlayer>, old_source: Option<MediaSource>) -> Result<()> {
        let leaving = self.active.replace(self.next());
        named::<IUIElement>(&self.root, self.active.get())?.SetOpacity(1.0)?;
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
        let Some((name, player, source)) = self.retiring.borrow_mut().take() else { return Ok(()) };
        named::<IUIElement>(&self.root, name)?.SetOpacity(0.0)?;
        named::<IMediaPlayerElement>(&self.root, name)?.SetMediaPlayer(None::<&MediaPlayer>)?;
        media::close(Some(player), Some(source));
        Ok(())
    }

    /// Both elements let go, as they must before their players close.
    pub fn clear(&self) {
        for name in [FIRST, SECOND] {
            let element = named::<IMediaPlayerElement>(&self.root, name);
            let _ = element.and_then(|e| e.SetMediaPlayer(None::<&MediaPlayer>));
        }
        if let Some((_, player, source)) = self.retiring.borrow_mut().take() {
            media::close(Some(player), Some(source));
        }
    }
}
