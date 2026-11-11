//! 播放设置: the gear on the video page's and the live room's bar. Picture fit, and how danmaku show: area, speed,
//! size, kinds and opacity. A choice takes at once and is kept for next time (fit for this run).

use std::rc::Rc;

use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::danmaku::{AREAS, Look, SIZES, SPEEDS};
use crate::ui::{keep, load_object};
use crate::xaml::{
    Color, CompositionStretch, FrameworkElement, IButtonBase, IControl, IFlyout, IFlyoutBase, IPanel, IRangeBase,
    RangeBaseValueChangedEventArgs, RoutedEventArgs, SolidColorBrush, UIElement,
};

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const WHITE: Color = Color { a: 255, r: 255, g: 255, b: 255 };
const CLEAR: Color = Color { a: 0, r: 0, g: 0, b: 0 };

/// The picture fits: label and stretch.
pub const FITS: [(&str, CompositionStretch); 3] = [
    ("适应", CompositionStretch::Uniform),
    ("拉伸", CompositionStretch::Fill),
    ("填充", CompositionStretch::UniformToFill),
];

/// What a choice changes, given to the page.
pub enum Change {
    Fit(CompositionStretch),
    Look(Look),
}

/// One row of options; `chosen` says which is on, `pick` what choosing one means.
struct Row {
    title: &'static str,
    labels: Vec<&'static str>,
    /// Several can be on at once (the kinds); otherwise one.
    several: bool,
    chosen: Vec<bool>,
}

fn row_xaml(row: &Row) -> String {
    let buttons: String = row
        .labels
        .iter()
        .map(|label| {
            format!(
                r##"<Button Content="{label}" FontSize="12" Padding="10,3" Margin="0,0,-1,0" BorderThickness="1"
                   BorderBrush="{{ThemeResource SystemControlForegroundBaseLowBrush}}" MinWidth="0" />"##
            )
        })
        .collect();
    format!(
        r##"<TextBlock Text="{}" FontSize="12" Margin="0,10,0,4" Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
           <StackPanel Orientation="Horizontal">{buttons}</StackPanel>"##,
        row.title
    )
}

fn paint(button: &UIElement, on: bool) -> Result<()> {
    let control = button.cast::<IControl>()?;
    control.SetBackground(&SolidColorBrush::CreateInstanceWithColor(if on { COBALT } else { CLEAR })?)?;
    if on {
        control.SetForeground(&SolidColorBrush::CreateInstanceWithColor(WHITE)?)?;
    } else {
        control.cast::<crate::xaml::IDependencyObject>()?.ClearValue(&crate::xaml::Control::ForegroundProperty()?)?;
    }
    Ok(())
}

/// Shows the settings under `button`: `fit` (when the picture has one to choose) and `look` as they
/// are now, then `extra` below them; `apply` takes each change.
pub fn show(
    button: &FrameworkElement,
    fit: Option<CompositionStretch>,
    look: Look,
    extra: Option<UIElement>,
    apply: impl Fn(Change) + 'static,
) -> Result<()> {
    let fit_row = fit.map(|fit| Row {
        title: "画面比例",
        labels: FITS.map(|f| f.0).to_vec(),
        several: false,
        chosen: FITS.map(|f| f.1 == fit).to_vec(),
    });
    // Rows after the fit are numbered as if it were there.
    let skipped = usize::from(fit_row.is_none());
    let rows = [
        Row {
            title: "弹幕 · 显示区域",
            labels: AREAS.map(|a| a.0).to_vec(),
            several: false,
            chosen: AREAS.map(|a| a.1 == look.area).to_vec(),
        },
        Row {
            title: "弹幕 · 滚动速度",
            labels: SPEEDS.map(|s| s.0).to_vec(),
            several: false,
            chosen: SPEEDS.map(|s| s.1 == look.speed).to_vec(),
        },
        Row {
            title: "弹幕 · 字号",
            labels: SIZES.map(|s| s.0).to_vec(),
            several: false,
            chosen: SIZES.map(|s| s.1 == look.size).to_vec(),
        },
        Row {
            title: "弹幕 · 显示类型",
            labels: vec!["滚动", "顶部", "底部", "彩色"],
            several: true,
            chosen: vec![look.scroll, look.top, look.bottom, look.colored],
        },
    ];
    let rows: Vec<Row> = fit_row.into_iter().chain(rows).collect();
    let body: String = rows.iter().map(row_xaml).collect();
    let percent = (look.opacity * 100.0).round();
    let flyout = load_object(&format!(
        r##"<Flyout xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" Placement="Top">
          <StackPanel Width="300">{body}
            <TextBlock Text="弹幕 · 不透明度" FontSize="12" Margin="0,10,0,0"
                       Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
            <Slider Minimum="20" Maximum="100" StepFrequency="5" Value="{percent}" />
          </StackPanel></Flyout>"##
    ))?;
    let panel = flyout.cast::<IFlyout>()?.Content()?.cast::<IPanel>()?;
    let parts = panel.Children()?.cast::<IVector<UIElement>>()?;
    let apply: Rc<dyn Fn(Change)> = Rc::new(apply);
    let look = Rc::new(std::cell::Cell::new(look));
    // Each row is a title, then its buttons.
    for (index, row) in rows.into_iter().enumerate() {
        let buttons =
            parts.GetAt((index * 2 + 1) as u32)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
        let state = Rc::new(std::cell::RefCell::new(row.chosen.clone()));
        for at in 0..row.labels.len() {
            paint(&buttons.GetAt(at as u32)?, row.chosen[at])?;
        }
        for at in 0..row.labels.len() {
            let (buttons, state, apply, look) = (buttons.clone(), state.clone(), apply.clone(), look.clone());
            let several = row.several;
            keep(buttons.GetAt(at as u32)?.cast::<IButtonBase>()?.Click(
                move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                    {
                        let mut chosen = state.borrow_mut();
                        if several {
                            chosen[at] = !chosen[at];
                        } else {
                            chosen.iter_mut().enumerate().for_each(|(i, c)| *c = i == at);
                        }
                        for (i, on) in chosen.iter().enumerate() {
                            if let Ok(b) = buttons.GetAt(i as u32) {
                                let _ = paint(&b, *on);
                            }
                        }
                    }
                    let chosen = state.borrow().clone();
                    let now = look.get();
                    let change = match index + skipped {
                        0 => Change::Fit(FITS[at].1),
                        1 => Change::Look(Look { area: AREAS[at].1, ..now }),
                        2 => Change::Look(Look { speed: SPEEDS[at].1, ..now }),
                        3 => Change::Look(Look { size: SIZES[at].1, ..now }),
                        _ => Change::Look(Look {
                            scroll: chosen[0],
                            top: chosen[1],
                            bottom: chosen[2],
                            colored: chosen[3],
                            ..now
                        }),
                    };
                    if let Change::Look(next) = &change {
                        look.set(*next);
                    }
                    apply(change);
                },
            )?);
        }
    }
    let slider = parts.GetAt(parts.Size()? - 1)?.cast::<IRangeBase>()?;
    if let Some(extra) = extra {
        parts.Append(&extra)?;
    }
    keep(slider.ValueChanged(move |_: Ref<IInspectable>, args: Ref<RangeBaseValueChangedEventArgs>| {
        let value = args.ok().and_then(|a| a.cast::<crate::xaml::IRangeBaseValueChangedEventArgs>()?.NewValue());
        if let Ok(value) = value {
            let next = Look { opacity: value / 100.0, ..look.get() };
            look.set(next);
            apply(Change::Look(next));
        }
    })?);
    flyout.cast::<IFlyoutBase>()?.ShowAt(button)
}

/// Where a danmaku sent goes: label and mode.
pub const SEND_MODES: [(&str, u8); 3] = [("滚动", 1), ("顶部", 5), ("底部", 4)];
/// The colours a danmaku sent may take, as B站 offers them.
const SEND_COLORS: [u32; 8] = [0xFF_FFFF, 0xFE_0302, 0xFF_7204, 0xFF_AA02, 0xFF_D302, 0x00_CD00, 0x00_A2FF, 0xCC_0273];

/// The 样式 of a danmaku to send under `button`: `pick` gets the mode and colour chosen.
pub fn show_send_style(
    button: &FrameworkElement,
    mode: u8,
    color: u32,
    pick: impl Fn(u8, u32) + 'static,
) -> Result<()> {
    let modes: String = SEND_MODES
        .iter()
        .map(|(label, _)| {
            format!(
                r##"<Button Content="{label}" FontSize="12" Padding="10,3" Margin="0,0,-1,0" BorderThickness="1"
                   BorderBrush="{{ThemeResource SystemControlForegroundBaseLowBrush}}" MinWidth="0" />"##
            )
        })
        .collect();
    let colors: String = SEND_COLORS
        .iter()
        .map(|c| {
            format!(
                r##"<Button Width="24" Height="24" Padding="0" Margin="0,0,6,0" MinWidth="0" MinHeight="0" Background="#{c:06X}"
                   BorderThickness="2" BorderBrush="Transparent" AutomationProperties.Name="#{c:06X}" />"##
            )
        })
        .collect();
    let flyout = load_object(&format!(
        r##"<Flyout xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" Placement="Top">
          <StackPanel Width="240">
            <TextBlock Text="位置" FontSize="12" Margin="0,0,0,4" Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
            <StackPanel Orientation="Horizontal">{modes}</StackPanel>
            <TextBlock Text="颜色" FontSize="12" Margin="0,10,0,4" Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
            <StackPanel Orientation="Horizontal">{colors}</StackPanel>
          </StackPanel></Flyout>"##
    ))?;
    let parts = flyout.cast::<IFlyout>()?.Content()?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
    let mode_buttons = parts.GetAt(1)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
    let color_buttons = parts.GetAt(3)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
    let chosen = Rc::new(std::cell::Cell::new((mode, color)));
    let pick: Rc<dyn Fn(u8, u32)> = Rc::new(pick);
    let mark = {
        let (mode_buttons, color_buttons) = (mode_buttons.clone(), color_buttons.clone());
        move |(mode, color): (u8, u32)| -> Result<()> {
            for (at, (_, m)) in SEND_MODES.iter().enumerate() {
                paint(&mode_buttons.GetAt(at as u32)?, *m == mode)?;
            }
            for (at, c) in SEND_COLORS.iter().enumerate() {
                let edge = if *c == color { COBALT } else { CLEAR };
                color_buttons
                    .GetAt(at as u32)?
                    .cast::<IControl>()?
                    .SetBorderBrush(&SolidColorBrush::CreateInstanceWithColor(edge)?)?;
            }
            Ok(())
        }
    };
    mark((mode, color))?;
    let mark = Rc::new(mark);
    for (at, (_, m)) in SEND_MODES.iter().enumerate() {
        let (chosen, pick, mark, m) = (chosen.clone(), pick.clone(), mark.clone(), *m);
        keep(mode_buttons.GetAt(at as u32)?.cast::<IButtonBase>()?.Click(
            move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                let next = (m, chosen.get().1);
                chosen.set(next);
                let _ = mark(next);
                pick(next.0, next.1);
            },
        )?);
    }
    for (at, c) in SEND_COLORS.iter().enumerate() {
        let (chosen, pick, mark, c) = (chosen.clone(), pick.clone(), mark.clone(), *c);
        keep(color_buttons.GetAt(at as u32)?.cast::<IButtonBase>()?.Click(
            move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
                let next = (chosen.get().0, c);
                chosen.set(next);
                let _ = mark(next);
                pick(next.0, next.1);
            },
        )?);
    }
    flyout.cast::<IFlyoutBase>()?.ShowAt(button)
}
