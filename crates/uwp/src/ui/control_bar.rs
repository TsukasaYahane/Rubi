//! The bar along the foot of a player, for videos and live rooms. In fullscreen it goes after a
//! while without a sign of life, along with whatever else `hide_too` names, and the cursor. A
//! finger's tap on the picture shows or hides it.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::settings::Prefs;
use crate::ui::shell::shell;
use crate::ui::{keep, named, post};
use crate::xaml::{
    Color, CoreCursor, CoreCursorType, CoreWindow, FocusState, IContentControl, IControl, ICoreWindow, IFontIcon,
    IFrameworkElement, IPanel, IRangeBase, ITappedRoutedEventArgs, IUIElement, PointerDeviceType,
    PointerRoutedEventArgs, RangeBaseValueChangedEventArgs, SolidColorBrush, TappedRoutedEventArgs, UIElement,
    Visibility,
};

/// How long the fullscreen bar stays without a sign of life.
const STAYS: Duration = Duration::from_secs(3);
const WHITE: Color = Color { a: 255, r: 255, g: 255, b: 255 };
const DIMMED: Color = Color { a: 110, r: 255, g: 255, b: 255 };
/// A tap this soon after the bar came up is the touch that brought it: it stays.
const JUST_SHOWN: Duration = Duration::from_millis(400);

const MARKUP: &str = r##"
<Grid x:Name="ControlBar" <!--place--> VerticalAlignment="Bottom" Background="#99000000" Padding="4,2,8,2"
      RequestedTheme="Dark">
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" /></Grid.RowDefinitions>
  <Grid x:Name="SeekRow" Visibility="Collapsed" Margin="8,0,4,0">
    <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
    <Slider x:Name="Seek" Minimum="0" Maximum="1" StepFrequency="1" VerticalAlignment="Center"
            IsThumbToolTipEnabled="False" />
    <TextBlock x:Name="Time" Grid.Column="1" FontSize="12" Foreground="White" Margin="12,0,0,0"
               VerticalAlignment="Center" />
  </Grid>
  <Grid Grid.Row="1">
    <Grid.ColumnDefinitions>
      <ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" />
      <ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" />
      <ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" />
    </Grid.ColumnDefinitions>
    <StackPanel Orientation="Horizontal">
      <Button x:Name="Prev" Visibility="Collapsed" Background="Transparent" Foreground="White" BorderThickness="0"
              Padding="8,6" VerticalAlignment="Center"><FontIcon Glyph="&#xE892;" FontSize="16" /></Button>
      <Button x:Name="PlayPause" Visibility="Collapsed" Background="Transparent" Foreground="White" BorderThickness="0"
              Padding="8,6" VerticalAlignment="Center"><FontIcon x:Name="PlayIcon" Glyph="&#xE769;" FontSize="16" /></Button>
      <Button x:Name="Next" Visibility="Collapsed" Background="Transparent" Foreground="White" BorderThickness="0"
              Padding="8,6" VerticalAlignment="Center"><FontIcon Glyph="&#xE893;" FontSize="16" /></Button>
    </StackPanel>
    <Button x:Name="Mute" Grid.Column="1" Background="Transparent" Foreground="White" BorderThickness="0" Padding="8,6"
            VerticalAlignment="Center"><FontIcon x:Name="MuteIcon" Glyph="&#xE767;" FontSize="16" /></Button>
    <Slider x:Name="Volume" Grid.Column="2" Width="96" Minimum="0" Maximum="100" StepFrequency="1"
            VerticalAlignment="Center" IsThumbToolTipEnabled="False" />
    <Button x:Name="DanmakuSwitch" Grid.Column="3" Margin="4,0,0,0" Background="Transparent" Foreground="White"
            BorderThickness="0" Padding="8,6" VerticalAlignment="Center">
      <FontIcon Glyph="&#xE8BD;" FontSize="16" />
    </Button>
    <ContentControl x:Name="SendInFull" Grid.Column="4" IsTabStop="False" HorizontalContentAlignment="Stretch"
                    MaxWidth="560" Margin="12,0" />
    <TextBlock x:Name="Online" Grid.Column="5" VerticalAlignment="Center" FontSize="12" Foreground="White"
               Margin="12,0" />
    <StackPanel Grid.Column="6" Orientation="Horizontal">
      <Button x:Name="Quality" Visibility="Collapsed" Background="Transparent" Foreground="White"
              BorderThickness="0" Padding="8,6" FontSize="12" VerticalAlignment="Center" />
      <Button x:Name="Pip" Visibility="Collapsed" Background="Transparent" Foreground="White" BorderThickness="0"
              Padding="8,6" VerticalAlignment="Center">
        <StackPanel Orientation="Horizontal">
          <FontIcon Glyph="&#xE8A7;" FontSize="14" /><TextBlock Text="小窗" FontSize="12" Margin="6,0,0,0" />
        </StackPanel>
      </Button>
    </StackPanel>
    <StackPanel Grid.Column="7" Orientation="Horizontal">
      <Button x:Name="WindowFull" Visibility="Collapsed" Background="Transparent" Foreground="White"
              BorderThickness="0" Padding="8,6" VerticalAlignment="Center">
        <FontIcon x:Name="WindowFullIcon" Glyph="&#xE9A6;" FontSize="16" />
      </Button>
      <Button x:Name="Full" Background="Transparent" Foreground="White" BorderThickness="0"
              Padding="8,6" VerticalAlignment="Center">
        <FontIcon x:Name="FullIcon" Glyph="&#xE740;" FontSize="16" />
      </Button>
    </StackPanel>
  </Grid>
</Grid>"##;

/// The bar's XAML; `place` holds its attached properties in the layout.
pub fn markup(place: &str) -> String {
    MARKUP.replace("<!--place-->", place)
}

pub struct ControlBar {
    root: IFrameworkElement,
    hide_too: &'static [&'static str],
    /// The page's run, and the bar of the page on show: a timer may outlive this one.
    run: fn() -> u64,
    find: fn() -> Option<Rc<ControlBar>>,
    /// Sets what the player plays at: level from 0 to 1, muted.
    set_sound: Box<dyn Fn(f64, bool)>,
    full: Cell<bool>,
    muted: Cell<bool>,
    idle: Cell<bool>,
    /// When the bar last came up.
    shown_at: Cell<Option<Instant>>,
    /// Where the pointer last was.
    pointer: Cell<(f32, f32)>,
    over_bar: Cell<bool>,
    /// Bumped by each sign of life: only the last one may hide the bar.
    turn: Arc<AtomicU64>,
}

impl ControlBar {
    /// Wires the bar in `root`, whose element `stage` is the picture.
    pub fn attach(
        root: &IFrameworkElement,
        stage: &str,
        hide_too: &'static [&'static str],
        run: fn() -> u64,
        find: fn() -> Option<Rc<ControlBar>>,
        set_sound: impl Fn(f64, bool) + 'static,
    ) -> Result<Rc<Self>> {
        let bar = Rc::new(Self {
            root: root.clone(),
            hide_too,
            run,
            find,
            set_sound: Box::new(set_sound),
            full: Cell::new(false),
            muted: Cell::new(false),
            idle: Cell::new(false),
            shown_at: Cell::new(None),
            pointer: Cell::default(),
            over_bar: Cell::new(false),
            turn: Arc::default(),
        });
        bar.wake()?;
        let volume = named::<IRangeBase>(root, "Volume")?;
        volume.SetValue(f64::from(shell().map(|s| s.prefs()).unwrap_or_default().volume))?;
        let weak = Rc::downgrade(&bar);
        keep(volume.ValueChanged(move |_: Ref<IInspectable>, _: Ref<RangeBaseValueChangedEventArgs>| {
            if let Some(bar) = weak.upgrade() {
                bar.muted.set(false);
                let _ = bar.apply_sound();
            }
        })?);
        let weak = Rc::downgrade(&bar);
        crate::ui::on_click(root, "Mute", move || {
            if let Some(bar) = weak.upgrade() {
                bar.muted.set(!bar.muted.get());
                let _ = bar.apply_sound();
            }
        })?;
        let weak = Rc::downgrade(&bar);
        keep(named::<IUIElement>(root, stage)?.PointerMoved(
            move |_: Ref<IInspectable>, args: Ref<PointerRoutedEventArgs>| {
                let at = args.ok().and_then(|a| a.GetCurrentPoint(None::<&UIElement>)?.Position());
                if let (Some(bar), Ok(at)) = (weak.upgrade(), at) {
                    // Hiding what lay under the pointer makes this fire too, with the pointer still.
                    if bar.pointer.replace((at.x, at.y)) != (at.x, at.y) {
                        let _ = bar.wake();
                    }
                }
            },
        )?);
        let weak = Rc::downgrade(&bar);
        keep(named::<IUIElement>(root, stage)?.Tapped(
            move |_: Ref<IInspectable>, args: Ref<TappedRoutedEventArgs>| {
                if let (Some(bar), true) = (weak.upgrade(), is_touch(&args)) {
                    let _ = bar.touched();
                }
            },
        )?);
        let element = named::<IUIElement>(root, "ControlBar")?;
        let weak = Rc::downgrade(&bar);
        keep(element.PointerEntered(move |_: Ref<IInspectable>, _: Ref<PointerRoutedEventArgs>| {
            if let Some(bar) = weak.upgrade() {
                bar.over_bar.set(true);
            }
        })?);
        let weak = Rc::downgrade(&bar);
        keep(element.PointerExited(move |_: Ref<IInspectable>, _: Ref<PointerRoutedEventArgs>| {
            if let Some(bar) = weak.upgrade() {
                bar.over_bar.set(false);
            }
        })?);
        Ok(bar)
    }

    /// The volume on the slider, from 0 to 1.
    fn level(&self) -> Result<f64> {
        Ok(named::<IRangeBase>(&self.root, "Volume")?.Value()? / 100.0)
    }

    /// Gives the player what the bar says.
    pub fn apply_sound(&self) -> Result<()> {
        let level = self.level()?;
        let silent = self.muted.get() || level <= 0.0;
        named::<IFontIcon>(&self.root, "MuteIcon")?.SetGlyph(if silent { "\u{E74F}" } else { "\u{E767}" })?;
        (self.set_sound)(level, self.muted.get());
        Ok(())
    }

    /// Remembers the volume for the next player.
    pub fn keep_volume(&self) {
        let (Some(shell), Ok(level)) = (shell(), self.level()) else { return };
        let volume = (level * 100.0) as u8;
        let prefs = shell.prefs();
        if prefs.volume != volume {
            let _ = shell.set_prefs(Prefs { volume, ..prefs });
        }
    }

    pub fn show_danmaku_state(&self, on: bool) -> Result<()> {
        let color = if on { WHITE } else { DIMMED };
        named::<IControl>(&self.root, "DanmakuSwitch")?.SetForeground(&SolidColorBrush::CreateInstanceWithColor(color)?)
    }

    /// Follows the page in or out of fullscreen.
    pub fn set_full(&self, on: bool) -> Result<()> {
        self.full.set(on);
        named::<IFontIcon>(&self.root, "FullIcon")?.SetGlyph(if on { "\u{E73F}" } else { "\u{E740}" })?;
        self.wake()
    }

    /// A sign of life: shows the bar, and sets it to go again after a still moment.
    pub fn wake(&self) -> Result<()> {
        self.set_idle(false)?;
        let turn = self.turn.fetch_add(1, Ordering::Relaxed) + 1;
        self.hide_after(turn);
        Ok(())
    }

    /// A finger tapped the picture: brings the bar up, or puts it away.
    fn touched(&self) -> Result<()> {
        let just_shown = self.shown_at.get().is_some_and(|at| at.elapsed() < JUST_SHOWN);
        if self.idle.get() || just_shown {
            return self.wake();
        }
        // Any hiding still waiting is for a sign of life before this.
        self.turn.fetch_add(1, Ordering::Relaxed);
        self.set_idle(true)
    }

    fn hide_after(&self, turn: u64) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (turns, dispatcher, opening, run, find) =
            (self.turn.clone(), shell.dispatcher.clone(), (self.run)(), self.run, self.find);
        backend.spawn(async move {
            tokio::time::sleep(STAYS).await;
            if turns.load(Ordering::Relaxed) == turn {
                post(&dispatcher, move || {
                    if opening == run()
                        && let Some(bar) = find()
                    {
                        let _ = bar.hide(turn);
                    }
                });
            }
        });
    }

    /// Hides the bars, unless someone is at them: then it looks again later.
    fn hide(&self, turn: u64) -> Result<()> {
        let typing = named::<IControl>(&self.root, "SendText")?.FocusState()? != FocusState::Unfocused;
        if typing || self.over_bar.get() {
            self.hide_after(turn);
            return Ok(());
        }
        self.set_idle(true)
    }

    fn set_idle(&self, idle: bool) -> Result<()> {
        if self.idle.replace(idle) == idle {
            return Ok(());
        }
        if !idle {
            self.shown_at.set(Some(Instant::now()));
        }
        let shown = if idle { Visibility::Collapsed } else { Visibility::Visible };
        named::<IUIElement>(&self.root, "ControlBar")?.SetVisibility(shown)?;
        for name in self.hide_too {
            named::<IUIElement>(&self.root, name)?.SetVisibility(shown)?;
        }
        // The pointer goes too only in fullscreen: in a window it may be wanted elsewhere.
        if idle && !self.full.get() {
            return Ok(());
        }
        let window = CoreWindow::GetForCurrentThread()?.cast::<ICoreWindow>()?;
        if idle {
            window.SetPointerCursor(None::<&CoreCursor>)
        } else {
            window.SetPointerCursor(&CoreCursor::CreateCursor(CoreCursorType::Arrow, 0)?)
        }
    }
}

/// The tap came from a finger, not a mouse or pen.
pub fn is_touch(args: &Ref<TappedRoutedEventArgs>) -> bool {
    args.ok()
        .and_then(|a| a.cast::<ITappedRoutedEventArgs>()?.PointerDeviceType())
        .is_ok_and(|kind| kind == PointerDeviceType::Touch)
}

/// Moves the send bar from one host to another, in the colour that suits where it lands.
pub fn move_send_bar(root: &IFrameworkElement, from: &str, to: &str, color: Color) -> Result<()> {
    let bar = named::<UIElement>(root, "SendBar")?;
    named::<IContentControl>(root, from)?.SetContent(None::<&IInspectable>)?;
    named::<IContentControl>(root, to)?.SetContent(&bar)?;
    bar.cast::<IPanel>()?.SetBackground(&SolidColorBrush::CreateInstanceWithColor(color)?)
}
