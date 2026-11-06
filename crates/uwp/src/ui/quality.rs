//! The 清晰度 button of a player: says what plays, and offers what else there is.

use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::quality::Offer;
use crate::state::settings::escape;
use crate::ui::{keep, load_object, named};
use crate::xaml::{
    FrameworkElement, IContentControl, IFlyoutBase, IFrameworkElement, IMenuFlyout, IMenuFlyoutItem,
    IToggleMenuFlyoutItem, IUIElement, PropertyValue, RoutedEventArgs, Visibility,
};

/// The button named `Quality`.
const BUTTON: &str = "Quality";

/// Shows the button with what plays; with nothing to choose from there is no button.
pub fn show(root: &IFrameworkElement, offer: &Offer) -> Result<()> {
    named::<IContentControl>(root, BUTTON)?.SetContent(&PropertyValue::CreateString(offer.label())?)?;
    let shown = if offer.options.len() > 1 { Visibility::Visible } else { Visibility::Collapsed };
    named::<IUIElement>(root, BUTTON)?.SetVisibility(shown)
}

/// Opens the menu; `pick` gets the qn chosen.
pub fn open(root: &IFrameworkElement, offer: &Offer, pick: fn(u32)) -> Result<()> {
    let items: String = offer
        .options
        .iter()
        .map(|(_, name)| format!(r##"<ToggleMenuFlyoutItem Text="{}" />"##, escape(name)))
        .collect();
    let menu = load_object(&format!(
        r##"<MenuFlyout xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation">{items}</MenuFlyout>"##
    ))?;
    let listed = menu.cast::<IMenuFlyout>()?.Items()?;
    for (at, (qn, _)) in offer.options.iter().enumerate() {
        let item = listed.GetAt(at as u32)?;
        item.cast::<IToggleMenuFlyoutItem>()?.SetIsChecked(*qn == offer.current)?;
        let (qn, playing) = (*qn, *qn == offer.current);
        keep(item.cast::<IMenuFlyoutItem>()?.Click(move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
            if !playing {
                pick(qn);
            }
        })?);
    }
    menu.cast::<IFlyoutBase>()?.ShowAt(&named::<FrameworkElement>(root, BUTTON)?)
}
