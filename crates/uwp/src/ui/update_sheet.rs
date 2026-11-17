//! 发现新版本: what was found, and whether to go and get it.

use std::cell::RefCell;

use windows_core::{IInspectable, Ref, Result};

use crate::ui::{keep, load, named, popup};
use crate::xaml::{IButtonBase, ITextBlock, RoutedEventArgs};

const SHEET: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="400" Padding="20"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <StackPanel>
      <TextBlock Text="发现新版本" FontSize="18" FontWeight="Light" />
      <TextBlock x:Name="Note" FontSize="13" Margin="0,10,0,0" TextWrapping="Wrap" />
      <StackPanel Orientation="Horizontal" HorizontalAlignment="Right" Margin="0,20,0,0">
        <Button x:Name="Later" Content="以后再说" Margin="0,0,8,0" />
        <Button x:Name="Go" Content="前往更新" Background="#0050EF" Foreground="White" BorderThickness="0" />
      </StackPanel>
    </StackPanel>
  </Grid>
</Grid>"##;

thread_local! {
    /// Where the update is fetched from, for the button to open.
    static TARGET: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Asks whether to go and get the version that was found.
pub fn ask(found: &str, url: &str) -> Result<()> {
    let root = load(SHEET)?;
    let note = format!("当前 {}\n最新 {found}", crate::ui::update::version());
    let _ = named::<ITextBlock>(&root, "Note")?.SetText(&note);
    TARGET.with_borrow_mut(|slot| *slot = url.to_owned());
    keep(named::<IButtonBase>(&root, "Later")?.Click(
        |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
            let _ = popup::hide();
        },
    )?);
    keep(named::<IButtonBase>(&root, "Go")?.Click(
        |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
            let _ = popup::hide();
            TARGET.with_borrow(|url| crate::ui::update::go(url));
        },
    )?);
    popup::show(&root)
}
