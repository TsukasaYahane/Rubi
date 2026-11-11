//! A player's frames drawn by the app. The player runs as a frame server: it says when the screen
//! wants a frame, and copies the frame of the moment into a composition surface of ours.
//!
//! A player that draws its own surface keeps showing frames for up to ~0.7 s after a pause, and
//! starts as late after a resume (network streams; measured 2026-11-11), while the sound and the
//! danmaku, which follow the playback position, stop and start at once. Copied by position, the
//! picture is the frame of the position the danmaku follow.

use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use windows_core::{EventRevoker, IInspectable, IUnknown, Interface, Ref, Result};

use crate::xaml::{
    CompositionDrawingSurface, Compositor, CreateDirect3D11SurfaceFromDXGISurface, D3D_DRIVER_TYPE_HARDWARE,
    D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION, D3D11CreateDevice, DirectXAlphaMode, DirectXPixelFormat,
    ICompositionDrawingSurfaceInterop, ICompositionGraphicsDevice, ICompositorInterop, ID3D11Device, ID3D11Multithread,
    IDXGISurface, IDirect3DSurface, IMediaPlaybackSession, IMediaPlayer3, IMediaPlayer5, MediaPlayer, POINT, Rect,
    SIZE, Size,
};

/// Positions closer than this, in 100 ns ticks, are not copied again: 12 ms still takes each
/// frame of a 60 fps video.
const STEP: i64 = 120_000;
/// Copies of one still position: right after a seek the frame there may not be ready at once.
const STILL_COPIES: u32 = 10;

thread_local! {
    /// This thread's compositor and the graphics device that draws for it.
    static GRAPHICS: RefCell<Option<(Compositor, ICompositionGraphicsDevice)>> = const { RefCell::new(None) };
}

/// The graphics device for `compositor`, made on first use.
fn graphics(compositor: &Compositor) -> Result<ICompositionGraphicsDevice> {
    GRAPHICS.with(|graphics| {
        if let Some((known, device)) = graphics.borrow().as_ref()
            && known == compositor
        {
            return Ok(device.clone());
        }
        let device = d3d_device()?;
        let made =
            unsafe { compositor.cast::<ICompositorInterop>()?.CreateGraphicsDevice(&device.cast::<IUnknown>()?)? };
        *graphics.borrow_mut() = Some((compositor.clone(), made.clone()));
        Ok(made)
    })
}

/// A hardware Direct3D 11 device that may be used from the frame server's threads.
fn d3d_device() -> Result<ID3D11Device> {
    let mut raw = core::ptr::null_mut();
    unsafe {
        D3D11CreateDevice(
            core::ptr::null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            core::ptr::null_mut(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT as u32,
            core::ptr::null(),
            0,
            D3D11_SDK_VERSION as u32,
            &mut raw,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )
        .ok()?;
        let device = ID3D11Device::from_raw(raw);
        let _ = device.cast::<ID3D11Multithread>()?.SetMultithreadProtected(true);
        Ok(device)
    }
}

/// What the drawing threads share: the surface's size, and the position last copied.
struct Canvas {
    interop: ICompositionDrawingSurfaceInterop,
    width: u32,
    height: u32,
    last: Option<i64>,
    still: u32,
}

/// One player's frames, drawn into a surface for a brush. Dropping it stops the drawing and gives
/// the player back its own picture, for whoever shows it next (the small window's element shows
/// nothing of a frame server).
pub struct Frames {
    surface: CompositionDrawingSurface,
    canvas: Arc<Mutex<Canvas>>,
    server: IMediaPlayer5,
    _frames: EventRevoker,
}

impl Drop for Frames {
    fn drop(&mut self) {
        let _ = self.server.SetIsVideoFrameServerEnabled(false);
    }
}

impl Frames {
    /// Makes `player` a frame server drawing into a new surface of `compositor`'s. Nothing shows
    /// until [`resize`](Self::resize) gives the video's size.
    pub fn start(compositor: &Compositor, player: &MediaPlayer) -> Result<Self> {
        let surface = graphics(compositor)?.CreateDrawingSurface(
            Size { width: 1.0, height: 1.0 },
            DirectXPixelFormat::B8G8R8A8UIntNormalized,
            DirectXAlphaMode::Premultiplied,
        )?;
        let canvas =
            Arc::new(Mutex::new(Canvas { interop: surface.cast()?, width: 0, height: 0, last: None, still: 0 }));
        let server = player.cast::<IMediaPlayer5>()?;
        server.SetIsVideoFrameServerEnabled(true)?;
        let session = player.cast::<IMediaPlayer3>()?.PlaybackSession()?.cast::<IMediaPlaybackSession>()?;
        let (drawing, copier) = (canvas.clone(), server.clone());
        let frames = server.VideoFrameAvailable(move |_: Ref<MediaPlayer>, _: Ref<IInspectable>| {
            let _ = draw(&copier, &session, &drawing);
        })?;
        Ok(Self { surface, canvas, server, _frames: frames })
    }

    pub fn surface(&self) -> &CompositionDrawingSurface {
        &self.surface
    }

    /// Sizes the surface to the video, from any thread; the next frame is drawn whatever its
    /// position.
    pub fn resizer(&self) -> impl Fn(u32, u32) -> Result<()> + 'static {
        let canvas = self.canvas.clone();
        move |width, height| resize(&canvas, width, height)
    }
}

fn resize(canvas: &Mutex<Canvas>, width: u32, height: u32) -> Result<()> {
    let Ok(mut canvas) = canvas.lock() else { return Ok(()) };
    if (canvas.width, canvas.height) == (width, height) {
        return Ok(());
    }
    unsafe { canvas.interop.Resize(SIZE { cx: width as i32, cy: height as i32 }).ok()? };
    canvas.width = width;
    canvas.height = height;
    canvas.last = None;
    Ok(())
}

/// Copies the frame of the moment, unless the position has hardly moved since the last copy.
fn draw(server: &IMediaPlayer5, session: &IMediaPlaybackSession, canvas: &Mutex<Canvas>) -> Result<()> {
    let Ok(mut canvas) = canvas.lock() else { return Ok(()) };
    if canvas.width == 0 || canvas.height == 0 {
        return Ok(());
    }
    let at = session.Position()?.duration;
    match canvas.last {
        Some(last) if last == at => {
            if canvas.still >= STILL_COPIES {
                return Ok(());
            }
            canvas.still += 1;
        }
        Some(last) if (at - last).abs() < STEP => return Ok(()),
        _ => canvas.still = 0,
    }
    canvas.last = Some(at);
    let mut offset = POINT::default();
    let target: IDXGISurface = unsafe { canvas.interop.BeginDraw(None, &mut offset)? };
    let copied = (|| {
        let mut raw = core::ptr::null_mut();
        unsafe {
            CreateDirect3D11SurfaceFromDXGISurface(target.as_raw(), &mut raw).ok()?;
            let surface = IInspectable::from_raw(raw).cast::<IDirect3DSurface>()?;
            server.CopyFrameToVideoSurfaceWithTargetRectangle(
                &surface,
                Rect {
                    x: offset.x as f32,
                    y: offset.y as f32,
                    width: canvas.width as f32,
                    height: canvas.height as f32,
                },
            )
        }
    })();
    unsafe { canvas.interop.EndDraw().ok()? };
    copied
}
