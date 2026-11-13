//! 发给好友: tick people from recent chats and followed UPs (searchable), then send the video, one
//! message each. Drawn over the window like the share card.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::ui::contacts::{self, Contact};
use crate::ui::rows::{add_rows, on_row, on_row_click};
use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load, named, on_click, post, set_text};
use crate::xaml::{
    Color, IBorder, IFrameworkElement, IItemsControl, IPanel, ITextBlock, ITextBox, IUIElement, SolidColorBrush,
    TextChangedEventArgs, UIElement, Visibility,
};

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const CLEAR: Color = Color { a: 0, r: 0, g: 0, b: 0 };

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="380" MaxHeight="560" Padding="16"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
      <RowDefinition Height="*" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <TextBlock Text="发给好友" FontSize="20" FontWeight="Light" />
    <TextBlock x:Name="FriendTitle" Grid.Row="1" FontSize="12" Margin="0,2,0,8" TextTrimming="CharacterEllipsis"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    <TextBox x:Name="FriendQuery" Grid.Row="2" PlaceholderText="搜索好友或关注的人" MaxLength="30" />
    <Grid Grid.Row="3" MinHeight="120" Margin="0,6,0,0">
      <ListView x:Name="Friends" SelectionMode="None" IsItemClickEnabled="True">
        <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
        <ListView.ItemContainerStyle>
          <Style TargetType="ListViewItem">
            <Setter Property="Padding" Value="0,6" />
            <Setter Property="MinHeight" Value="0" />
            <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          </Style>
        </ListView.ItemContainerStyle>
        <ListView.ItemTemplate><DataTemplate>
          <Grid Background="Transparent">
            <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
            <StackPanel>
              <TextBlock FontSize="14" TextTrimming="CharacterEllipsis" />
              <TextBlock FontSize="11" Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
            </StackPanel>
            <Border Grid.Column="1" Width="16" Height="16" Margin="8,0,4,0" VerticalAlignment="Center"
                    BorderThickness="2" BorderBrush="{ThemeResource SystemControlForegroundBaseHighBrush}" />
          </Grid>
        </DataTemplate></ListView.ItemTemplate>
      </ListView>
      <TextBlock x:Name="FriendStatus" FontSize="12" Margin="0,8" TextWrapping="Wrap" VerticalAlignment="Top"
                 IsHitTestVisible="False"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    </Grid>
    <StackPanel Grid.Row="4" Orientation="Horizontal" Margin="0,10,0,0">
      <TextBlock x:Name="FriendCount" Text="已选 0 人" FontSize="12" Width="150" VerticalAlignment="Center"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      <Button x:Name="FriendCancel" Content="取消" Margin="0,0,8,0" />
      <Button x:Name="FriendSend" Content="发送" Background="#0050EF" Foreground="White" BorderThickness="0" />
    </StackPanel>
    <TextBlock Grid.Row="5" FontSize="11" Margin="0,8,0,0" TextWrapping="Wrap"
               Text="B 站不收第三方客户端发的视频卡片时，会改发「标题 + 链接」，对方看到的仍是视频。"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </Grid>
</Grid>"##;

struct Sheet {
    root: IFrameworkElement,
    bvid: String,
    /// Who matches the search, as shown.
    shown: RefCell<Vec<Contact>>,
    picked: RefCell<HashSet<u64>>,
    sending: Cell<bool>,
}

thread_local! {
    static SHEET: RefCell<Option<Rc<Sheet>>> = const { RefCell::new(None) };
}

fn sheet() -> Option<Rc<Sheet>> {
    SHEET.with_borrow(Clone::clone)
}

/// Offers video `bvid` (`title`) to be sent to friends.
pub fn open(bvid: &str, title: &str) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    if !shell.backend.get().is_some_and(|b| b.services.account.is_logged_in()) {
        toast("登录后才能发给好友");
        return Ok(());
    }
    let root = load(LAYOUT)?;
    set_text(&root, "FriendTitle", title)?;
    set_text(&root, "FriendStatus", "正在读取聊天与关注…")?;
    let opened = Rc::new(Sheet {
        root: root.clone(),
        bvid: bvid.to_owned(),
        shown: RefCell::default(),
        picked: RefCell::default(),
        sending: Cell::new(false),
    });
    SHEET.with_borrow_mut(|s| *s = Some(opened));
    let list = named::<IFrameworkElement>(&root, "Friends")?;
    on_row(&list, |at, parts| sheet().map_or(Ok(()), |s| s.fill(at, parts)))?;
    on_row_click(&list, |at| {
        if let Some(sheet) = sheet() {
            let _ = sheet.toggle(at);
        }
    })?;
    keep(named::<ITextBox>(&root, "FriendQuery")?.TextChanged(
        |_: Ref<IInspectable>, _: Ref<TextChangedEventArgs>| {
            if let Some(sheet) = sheet() {
                let _ = sheet.filter();
            }
        },
    )?);
    on_click(&root, "FriendCancel", close)?;
    on_click(&root, "FriendSend", || {
        if let Some(sheet) = sheet() {
            sheet.send();
        }
    })?;
    let host = shell.viewer_host()?;
    let children = host.Children()?.cast::<IVector<UIElement>>()?;
    children.Clear()?;
    children.Append(&root.cast::<UIElement>()?)?;
    host.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)?;
    contacts::with(|_| {
        if let Some(sheet) = sheet() {
            let _ = sheet.filter();
        }
    });
    Ok(())
}

fn close() {
    SHEET.with_borrow_mut(|s| *s = None);
    crate::ui::share_card::close();
}

impl Sheet {
    fn filter(&self) -> Result<()> {
        let Some(all) = contacts::known() else { return Ok(()) };
        let query = named::<ITextBox>(&self.root, "FriendQuery")?.Text()?.to_string();
        let query = query.trim().to_lowercase();
        let found: Vec<Contact> =
            all.iter().filter(|c| query.is_empty() || c.name.to_lowercase().contains(&query)).cloned().collect();
        let count = found.len();
        *self.shown.borrow_mut() = found;
        let list = named::<IFrameworkElement>(&self.root, "Friends")?;
        list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        add_rows(&list, 0, count)?;
        let said = if all.is_empty() {
            "没有读到聊天或关注"
        } else if count == 0 {
            "没有匹配的人"
        } else {
            ""
        };
        set_text(&self.root, "FriendStatus", said)
    }

    fn fill(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let shown = self.shown.borrow();
        let Some(contact) = shown.get(at) else { return Ok(()) };
        let words = parts.GetAt(0)?.cast::<IPanel>()?.Children()?;
        words.GetAt(0)?.cast::<ITextBlock>()?.SetText(&contact.name)?;
        words.GetAt(1)?.cast::<ITextBlock>()?.SetText(contact.from)?;
        let on = self.picked.borrow().contains(&contact.mid);
        parts.GetAt(1)?.cast::<IBorder>()?.SetBackground(&SolidColorBrush::CreateInstanceWithColor(if on {
            COBALT
        } else {
            CLEAR
        })?)
    }

    fn toggle(&self, at: usize) -> Result<()> {
        let Some(mid) = self.shown.borrow().get(at).map(|c| c.mid) else { return Ok(()) };
        {
            let mut picked = self.picked.borrow_mut();
            if !picked.remove(&mid) {
                picked.insert(mid);
            }
        }
        let list = named::<IItemsControl>(&self.root, "Friends")?.Items()?.cast::<IVector<IInspectable>>()?;
        list.SetAt(at as u32, &crate::xaml::PropertyValue::CreateInt32(at as i32)?)?;
        set_text(&self.root, "FriendCount", &format!("已选 {} 人", self.picked.borrow().len()))
    }

    fn send(&self) {
        let targets: Vec<u64> = self.picked.borrow().iter().copied().collect();
        if targets.is_empty() || self.sending.replace(true) {
            return;
        }
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let _ = set_text(&self.root, "FriendCount", "发送中…");
        let (dispatcher, core, bvid) = (shell.dispatcher.clone(), backend.clone(), self.bvid.clone());
        backend.spawn(async move {
            let (mut sent, mut as_link, mut failed) = (0, false, None);
            for mid in targets {
                match core.services.messages.share_video(mid, &bvid).await {
                    Ok(card) => {
                        sent += 1;
                        as_link |= !card;
                    }
                    Err(problem) => failed = Some(problem.to_string()),
                }
            }
            post(&dispatcher, move || {
                close();
                let how = if as_link { "（以标题和链接发出）" } else { "" };
                match failed {
                    Some(problem) if sent == 0 => toast(&format!("发送失败：{problem}")),
                    Some(problem) => toast(&format!("已发给 {sent} 人{how}，有的没发出：{problem}")),
                    None => toast(&format!("已发给 {sent} 人{how}")),
                }
            });
        });
    }
}
