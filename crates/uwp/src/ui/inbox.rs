//! 消息: 回复, @我 and 赞 as lists that open the video at the comment, and 私信 as chats beside
//! their list on a wider window. The unread count is read now and then for the 消息 icon.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rubi_model::{ChatMessage, ChatSession, Notice, NoticeKind, UnreadCounts, VideoRef};
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Result};

use crate::state::layout::WindowClass;
use crate::state::library::age;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::{shell, toast};
use crate::ui::{load, named, on_click, on_enter, player, post, set_text};
use crate::xaml::{
    Color, GridLength, GridUnitType, HorizontalAlignment, IBorder, IColumnDefinition, IControl, IFrameworkElement,
    IImage, IItemsControl, IPanel, IRichTextBlock, ITextBlock, ITextBox, IUIElement, SolidColorBrush, UIElement,
    Visibility,
};

/// How often the unread count is read while the app runs.
const UNREAD_EVERY: Duration = Duration::from_secs(120);
const TABS: [(&str, Option<NoticeKind>); 4] =
    [("回复", Some(NoticeKind::Reply)), ("@我", Some(NoticeKind::At)), ("赞", Some(NoticeKind::Like)), ("私信", None)];
const LOAD_AHEAD: usize = 6;
const EMOTE_SIZE: f64 = 20.0;
const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const WHITE: Color = Color { a: 255, r: 255, g: 255, b: 255 };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };
const PICKED: Color = Color { a: 0x26, r: 0, g: 0x50, b: 0xEF };
const CLEAR: Color = Color { a: 0, r: 0, g: 0, b: 0 };
const COVER_DECODE_WIDTH: i32 = 200;

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <StackPanel Orientation="Horizontal" Margin="8,8,8,0">
    <StackPanel.Resources>
      <Style TargetType="Button">
        <Setter Property="Background" Value="Transparent" />
        <Setter Property="BorderThickness" Value="0" />
        <Setter Property="Padding" Value="8,4" />
        <Setter Property="FontSize" Value="20" />
        <Setter Property="FontWeight" Value="Light" />
      </Style>
      <Style TargetType="Border">
        <Setter Property="Background" Value="#E33333" />
        <Setter Property="MinWidth" Value="16" />
        <Setter Property="Height" Value="16" />
        <Setter Property="Padding" Value="4,0" />
        <Setter Property="Margin" Value="6,0,0,0" />
        <Setter Property="VerticalAlignment" Value="Center" />
        <Setter Property="Visibility" Value="Collapsed" />
      </Style>
    </StackPanel.Resources>
    <!--tabs-->
  </StackPanel>
  <TextBlock x:Name="Status" Grid.Row="1" FontSize="12" Margin="16,2,16,6" TextWrapping="Wrap"
             Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  <ListView x:Name="Notices" Grid.Row="2" SelectionMode="None" IsItemClickEnabled="True">
    <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
    <ListView.ItemContainerStyle>
      <Style TargetType="ListViewItem">
        <Setter Property="Padding" Value="16,10" />
        <Setter Property="HorizontalContentAlignment" Value="Stretch" />
      </Style>
    </ListView.ItemContainerStyle>
    <ListView.ItemTemplate><DataTemplate>
      <StackPanel>
        <TextBlock FontSize="14" FontWeight="SemiBold" TextTrimming="CharacterEllipsis" />
        <RichTextBlock FontSize="14" TextWrapping="Wrap" Margin="0,3,0,0" IsTextSelectionEnabled="False" />
        <RichTextBlock FontSize="12" TextWrapping="Wrap" Margin="0,3,0,0" MaxLines="2" IsTextSelectionEnabled="False"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        <TextBlock FontSize="12" Margin="0,4,0,0" Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      </StackPanel>
    </DataTemplate></ListView.ItemTemplate>
  </ListView>
  <Grid x:Name="Chats" Grid.Row="2" Visibility="Collapsed" BorderThickness="0,1,0,0"
        BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <Grid.ColumnDefinitions>
      <ColumnDefinition x:Name="SessionsColumn" Width="38*" /><ColumnDefinition x:Name="ChatColumn" Width="62*" />
    </Grid.ColumnDefinitions>
    <ListView x:Name="Sessions" SelectionMode="None" IsItemClickEnabled="True">
      <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
      <ListView.ItemContainerStyle>
        <Style TargetType="ListViewItem">
          <Setter Property="Padding" Value="0" />
          <Setter Property="HorizontalContentAlignment" Value="Stretch" />
        </Style>
      </ListView.ItemContainerStyle>
      <ListView.ItemTemplate><DataTemplate>
        <Grid Padding="16,10">
          <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
          <StackPanel>
            <TextBlock FontSize="17" FontWeight="Light" TextTrimming="CharacterEllipsis" />
            <TextBlock FontSize="13" TextTrimming="CharacterEllipsis" Margin="0,2,0,0"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          </StackPanel>
          <StackPanel Grid.Column="1" Margin="10,0,0,0">
            <TextBlock FontSize="11" HorizontalAlignment="Right"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
            <Border Background="#E33333" MinWidth="16" Height="16" Padding="4,0" Margin="0,6,0,0"
                    HorizontalAlignment="Right"><TextBlock FontSize="10" Foreground="White"
                    HorizontalAlignment="Center" VerticalAlignment="Center" /></Border>
          </StackPanel>
        </Grid>
      </DataTemplate></ListView.ItemTemplate>
    </ListView>
    <Grid x:Name="ChatPane" Grid.Column="1" BorderThickness="1,0,0,0"
          BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
      <Grid.RowDefinitions>
        <RowDefinition Height="Auto" /><RowDefinition Height="*" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
      </Grid.RowDefinitions>
      <StackPanel Orientation="Horizontal" Padding="4,6,16,6" BorderThickness="0,0,0,1"
                  BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
        <Button x:Name="ChatBack" Visibility="Collapsed" Background="Transparent" BorderThickness="0" Padding="10,6">
          <FontIcon Glyph="&#xE72B;" FontSize="14" /></Button>
        <TextBlock x:Name="Talker" FontSize="16" FontWeight="SemiBold" Margin="12,0,0,0" VerticalAlignment="Center" />
      </StackPanel>
      <ListView x:Name="Messages" Grid.Row="1" SelectionMode="None" IsItemClickEnabled="True">
        <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
        <ListView.ItemsPanel><ItemsPanelTemplate>
          <ItemsStackPanel ItemsUpdatingScrollMode="KeepLastItemInView" />
        </ItemsPanelTemplate></ListView.ItemsPanel>
        <ListView.ItemContainerStyle>
          <Style TargetType="ListViewItem">
            <Setter Property="Padding" Value="16,5" />
            <Setter Property="MinHeight" Value="0" />
            <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          </Style>
        </ListView.ItemContainerStyle>
        <ListView.ItemTemplate><DataTemplate>
          <StackPanel>
            <Border Padding="12,8" MaxWidth="360">
              <RichTextBlock FontSize="14" TextWrapping="Wrap" IsTextSelectionEnabled="False" /></Border>
            <Grid MaxWidth="320" Padding="8" Background="{ThemeResource SystemControlBackgroundBaseLowBrush}">
              <Grid.ColumnDefinitions><ColumnDefinition Width="104" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
              <Image Width="96" Height="54" Stretch="UniformToFill" HorizontalAlignment="Left" />
              <StackPanel Grid.Column="1">
                <TextBlock FontSize="13" TextWrapping="Wrap" MaxLines="2" />
                <TextBlock FontSize="11" Margin="0,4,0,0"
                           Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
              </StackPanel>
            </Grid>
            <TextBlock Text="▷ 打开视频" FontSize="12" Margin="0,4,0,0" Foreground="#0050EF" />
          </StackPanel>
        </DataTemplate></ListView.ItemTemplate>
      </ListView>
      <TextBlock x:Name="ChatNote" Grid.Row="2" FontSize="12" Margin="16,2" TextWrapping="Wrap" Visibility="Collapsed"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      <Grid Grid.Row="3" Padding="16,8" BorderThickness="0,1,0,0"
            BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
        <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
        <Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
        <TextBox x:Name="ChatText" PlaceholderText="发消息" MaxLength="500" />
        <Button x:Name="ChatSend" Grid.Column="1" Content="发送" Margin="8,0,0,0" Background="#0050EF"
                Foreground="White" BorderThickness="0" Padding="14,5" />
      </Grid>
    </Grid>
  </Grid>
</Grid>"##;

pub struct Inbox {
    root: IFrameworkElement,
    tab: Cell<usize>,
    notices: RefCell<Vec<Notice>>,
    cursor: RefCell<Option<String>>,
    loading: Cell<bool>,
    sessions: RefCell<Vec<ChatSession>>,
    /// The chat on show, by its index in the sessions.
    picked: Cell<Option<usize>>,
    messages: RefCell<Vec<ChatMessage>>,
    sending: Cell<bool>,
    /// Bumped by each tab or chat opened: answers for an earlier one are dropped.
    turn: Cell<u64>,
    covers: Covers,
}

thread_local! {
    static PAGE: RefCell<Option<Rc<Inbox>>> = const { RefCell::new(None) };
    static UNREAD: RefCell<UnreadCounts> = RefCell::default();
}

fn held() -> Option<Rc<Inbox>> {
    PAGE.with_borrow(Clone::clone)
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

fn shown(on: bool) -> Visibility {
    if on { Visibility::Visible } else { Visibility::Collapsed }
}

/// Reads the unread count now, then every [`UNREAD_EVERY`].
pub fn watch_unread() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        loop {
            if core.services.account.is_logged_in()
                && let Ok(counts) = core.services.messages.unread().await
            {
                post(&dispatcher, move || took_unread(counts));
            }
            tokio::time::sleep(UNREAD_EVERY).await;
        }
    });
}

/// Reads the unread count once, as the window comes back.
pub fn read_unread() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    if !backend.services.account.is_logged_in() {
        return;
    }
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        if let Ok(counts) = core.services.messages.unread().await {
            post(&dispatcher, move || took_unread(counts));
        }
    });
}

fn took_unread(counts: UnreadCounts) {
    let total = counts.total();
    UNREAD.set(counts);
    if let Some(shell) = shell() {
        let _ = shell.set_unread(total);
    }
    if let Some(page) = held() {
        let _ = page.mark_tabs();
    }
}

fn unread_of(tab: usize) -> u32 {
    let counts = UNREAD.with_borrow(Clone::clone);
    [counts.reply, counts.at, counts.like, counts.chat][tab]
}

/// What has been looked at is no longer unread here; the next reading says what B站 thinks.
fn seen(tab: usize) {
    let mut counts = UNREAD.with_borrow(Clone::clone);
    match tab {
        0 => counts.reply = 0,
        1 => counts.at = 0,
        2 => counts.like = 0,
        _ => {}
    }
    took_unread(counts);
}

/// Shows the 消息 tab.
pub fn show() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let page = match held() {
        Some(page) => page,
        None => {
            let page = Inbox::new()?;
            PAGE.with_borrow_mut(|p| *p = Some(page.clone()));
            page.open(0)?;
            page
        }
    };
    shell.set_page(&page.root)?;
    page.fit()?;
    page.mark_tabs()
}

/// Follows the window: on a narrow one a chat takes the whole page.
pub fn resized() {
    if let Some(page) = held() {
        let _ = page.fit();
    }
}

impl Inbox {
    fn new() -> Result<Rc<Self>> {
        let tabs: String = TABS
            .iter()
            .enumerate()
            .map(|(i, (label, _))| {
                format!(
                    r##"<Button x:Name="Tab{i}"><StackPanel Orientation="Horizontal"><TextBlock Text="{label}" />
                    <Border x:Name="TabBadge{i}"><TextBlock FontSize="10" FontWeight="Normal" Foreground="White"
                    HorizontalAlignment="Center" VerticalAlignment="Center" /></Border></StackPanel></Button>"##
                )
            })
            .collect();
        let root = load(&LAYOUT.replace("<!--tabs-->", &tabs))?;
        let page = Rc::new(Self {
            root: root.clone(),
            tab: Cell::new(0),
            notices: RefCell::default(),
            cursor: RefCell::default(),
            loading: Cell::new(false),
            sessions: RefCell::default(),
            picked: Cell::new(None),
            messages: RefCell::default(),
            sending: Cell::new(false),
            turn: Cell::new(0),
            covers: Covers::default(),
        });
        for at in 0..TABS.len() {
            on_click(&root, &format!("Tab{at}"), move || {
                if let Some(page) = held() {
                    let _ = page.open(at);
                }
            })?;
        }
        let notices = named::<IFrameworkElement>(&root, "Notices")?;
        on_row(&notices, |at, parts| held().map_or(Ok(()), |p| p.fill_notice(at, parts)))?;
        on_row_click(&notices, |at| {
            if let Some(page) = held() {
                page.open_notice(at);
            }
        })?;
        let sessions = named::<IFrameworkElement>(&root, "Sessions")?;
        on_row(&sessions, |at, parts| held().map_or(Ok(()), |p| p.fill_session(at, parts)))?;
        on_row_click(&sessions, |at| {
            if let Some(page) = held() {
                let _ = page.pick(at);
            }
        })?;
        let messages = named::<IFrameworkElement>(&root, "Messages")?;
        on_row(&messages, |at, parts| held().map_or(Ok(()), |p| p.fill_message(at, parts)))?;
        on_row_click(&messages, |at| {
            let Some(page) = held() else { return };
            let video = page.messages.borrow().get(at).and_then(|m| m.video.clone());
            if let Some(shared) = video {
                open_video(shared.video, &shared.title, None);
            }
        })?;
        on_click(&root, "ChatSend", || {
            if let Some(page) = held() {
                let _ = page.send();
            }
        })?;
        on_enter(&root, "ChatText", || {
            if let Some(page) = held() {
                let _ = page.send();
            }
        })?;
        on_click(&root, "ChatBack", || {
            if let Some(page) = held() {
                page.picked.set(None);
                let _ = page.fit();
            }
        })?;
        Ok(page)
    }

    fn say(&self, text: &str) {
        let _ = set_text(&self.root, "Status", text);
    }

    fn mark_tabs(&self) -> Result<()> {
        for at in 0..TABS.len() {
            let color = if at == self.tab.get() { COBALT } else { DIM };
            named::<IControl>(&self.root, &format!("Tab{at}"))?
                .SetForeground(&SolidColorBrush::CreateInstanceWithColor(color)?)?;
            let count = unread_of(at);
            let badge = named::<IBorder>(&self.root, &format!("TabBadge{at}"))?;
            badge.Child()?.cast::<ITextBlock>()?.SetText(&count.to_string())?;
            badge.cast::<IUIElement>()?.SetVisibility(shown(count > 0))?;
        }
        Ok(())
    }

    fn open(&self, tab: usize) -> Result<()> {
        self.tab.set(tab);
        self.turn.set(self.turn.get() + 1);
        self.loading.set(false);
        let chats = TABS[tab].1.is_none();
        named::<IUIElement>(&self.root, "Notices")?.SetVisibility(shown(!chats))?;
        named::<IUIElement>(&self.root, "Chats")?.SetVisibility(shown(chats))?;
        let signed = shell().and_then(|s| s.backend.get().cloned()).is_some_and(|b| b.services.account.is_logged_in());
        if !signed {
            self.say("登录后可查看消息");
            return self.mark_tabs();
        }
        self.say("正在读取…");
        if chats {
            self.load_sessions();
        } else {
            self.notices.borrow_mut().clear();
            *self.cursor.borrow_mut() = None;
            self.list("Notices")?.Clear()?;
            self.load_notices();
            seen(tab);
        }
        self.mark_tabs()
    }

    fn list(&self, name: &str) -> Result<IVector<IInspectable>> {
        named::<IItemsControl>(&self.root, name)?.Items()?.cast()
    }

    fn load_notices(&self) {
        let Some(kind) = TABS[self.tab.get()].1 else { return };
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.loading.replace(true) {
            return;
        }
        let (turn, cursor, dispatcher, core) =
            (self.turn.get(), self.cursor.borrow().clone(), shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let found = core.services.messages.notices(kind, cursor.as_deref()).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(page) = held().filter(|p| p.turn.get() == turn) {
                    let _ = page.took_notices(found);
                }
            });
        });
    }

    fn took_notices(&self, found: std::result::Result<(Vec<Notice>, Option<String>), String>) -> Result<()> {
        self.loading.set(false);
        match found {
            Ok((notices, cursor)) => {
                let from = self.notices.borrow().len();
                self.notices.borrow_mut().extend(notices);
                let to = self.notices.borrow().len();
                *self.cursor.borrow_mut() = cursor.filter(|_| to > from);
                add_rows(&named(&self.root, "Notices")?, from, to)?;
                self.say(if to == 0 { "还没有消息" } else { "仅显示真人互动" });
            }
            Err(problem) => self.say(&format!("读取失败：{problem}")),
        }
        Ok(())
    }

    fn fill_notice(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let notices = self.notices.borrow();
        let Some(notice) = notices.get(at) else { return Ok(()) };
        parts.GetAt(0)?.cast::<ITextBlock>()?.SetText(&notice.from)?;
        let none = BTreeMap::new();
        crate::ui::emote_text::show(&parts.GetAt(1)?.cast::<IRichTextBlock>()?, &notice.text, &none, EMOTE_SIZE)?;
        crate::ui::emote_text::show(&parts.GetAt(2)?.cast::<IRichTextBlock>()?, &notice.context, &none, 16.0)?;
        let open = if notice.video.is_some() { " · ▷ 打开视频" } else { "" };
        parts.GetAt(3)?.cast::<ITextBlock>()?.SetText(&format!("{}{open}", age(notice.at, now())))?;
        let near_end = at + LOAD_AHEAD >= notices.len();
        drop(notices);
        if near_end && self.cursor.borrow().is_some() {
            self.load_notices();
        }
        Ok(())
    }

    fn open_notice(&self, at: usize) {
        let notice = self.notices.borrow().get(at).cloned();
        let Some(notice) = notice else { return };
        match notice.video {
            // The notice's text is not the video's title: the page fills that in itself.
            Some(video) => open_video(video, "", notice.comment),
            None => toast("这条消息不在视频下，暂时打不开"),
        }
    }

    fn load_sessions(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (turn, dispatcher, core) = (self.turn.get(), shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let found = core.services.messages.sessions().await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(page) = held().filter(|p| p.turn.get() == turn) {
                    let _ = page.took_sessions(found);
                }
            });
        });
    }

    fn took_sessions(&self, found: std::result::Result<(Vec<ChatSession>, usize), String>) -> Result<()> {
        match found {
            Ok((sessions, hidden)) => {
                let count = sessions.len();
                // The chat on show stays, by who it is with.
                let talker = self.picked.get().and_then(|i| self.sessions.borrow().get(i).map(|s| s.talker_mid));
                *self.sessions.borrow_mut() = sessions;
                let list = self.list("Sessions")?;
                list.Clear()?;
                add_rows(&named(&self.root, "Sessions")?, 0, count)?;
                self.say(&if hidden > 0 {
                    format!("已过滤营销私信 {hidden} 个（未关注账号的推广卡片）")
                } else if count == 0 {
                    "没有私信".to_owned()
                } else {
                    String::new()
                });
                let again = talker.and_then(|mid| self.sessions.borrow().iter().position(|s| s.talker_mid == mid));
                let wide = shell().is_some_and(|s| s.class() != WindowClass::Narrow);
                match again.or(if wide && count > 0 { Some(0) } else { None }) {
                    Some(at) => self.pick(at)?,
                    None => self.fit()?,
                }
            }
            Err(problem) => self.say(&format!("读取失败：{problem}")),
        }
        Ok(())
    }

    fn fill_session(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let sessions = self.sessions.borrow();
        let Some(session) = sessions.get(at) else { return Ok(()) };
        let grid = parts.GetAt(0)?.cast::<IFrameworkElement>()?.Parent()?.cast::<IPanel>()?;
        let picked = self.picked.get() == Some(at);
        grid.SetBackground(&SolidColorBrush::CreateInstanceWithColor(if picked { PICKED } else { CLEAR })?)?;
        let left = parts.GetAt(0)?.cast::<IPanel>()?.Children()?;
        let name = left.GetAt(0)?.cast::<ITextBlock>()?;
        name.SetText(&session.name)?;
        name.SetFontWeight(crate::xaml::FontWeight { weight: if session.unread > 0 { 600 } else { 300 } })?;
        let last = format!("{}{}", if session.last_from_me { "我：" } else { "" }, session.last_text);
        left.GetAt(1)?.cast::<ITextBlock>()?.SetText(&last)?;
        let right = parts.GetAt(1)?.cast::<IPanel>()?.Children()?;
        right.GetAt(0)?.cast::<ITextBlock>()?.SetText(&age(session.last_at, now()))?;
        let badge = right.GetAt(1)?;
        badge.cast::<IBorder>()?.Child()?.cast::<ITextBlock>()?.SetText(&session.unread.to_string())?;
        badge.cast::<IUIElement>()?.SetVisibility(shown(session.unread > 0))
    }

    /// Opens the chat with session `at`.
    fn pick(&self, at: usize) -> Result<()> {
        let Some(session) = self.sessions.borrow().get(at).cloned() else { return Ok(()) };
        let before = self.picked.replace(Some(at));
        // Rows redraw to show which is picked.
        let rows = self.list("Sessions")?;
        for row in [before, Some(at)].into_iter().flatten() {
            if (row as u32) < rows.Size()? {
                rows.SetAt(row as u32, &crate::xaml::PropertyValue::CreateInt32(row as i32)?)?;
            }
        }
        set_text(&self.root, "Talker", &session.name)?;
        self.messages.borrow_mut().clear();
        self.list("Messages")?.Clear()?;
        self.note("");
        self.fit()?;
        self.load_chat(session.talker_mid);
        Ok(())
    }

    fn load_chat(&self, mid: u64) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        self.turn.set(self.turn.get() + 1);
        let (turn, dispatcher, core) = (self.turn.get(), shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let found = core.services.messages.chat(mid).await.map_err(|e| e.to_string());
            if let Ok(Some(last)) = found.as_ref().map(|m| m.last()) {
                let _ = core.services.messages.mark_read(mid, last.seqno).await;
            }
            post(&dispatcher, move || {
                if let Some(page) = held().filter(|p| p.turn.get() == turn) {
                    let _ = page.took_chat(mid, found);
                }
            });
        });
    }

    fn took_chat(&self, mid: u64, found: std::result::Result<Vec<ChatMessage>, String>) -> Result<()> {
        match found {
            Ok(messages) => {
                let count = messages.len();
                *self.messages.borrow_mut() = messages;
                let list = self.list("Messages")?;
                list.Clear()?;
                add_rows(&named(&self.root, "Messages")?, 0, count)?;
                // Read now: the count goes from the session and from the 消息 icon.
                let read = {
                    let mut sessions = self.sessions.borrow_mut();
                    sessions.iter_mut().find(|s| s.talker_mid == mid).map(|s| std::mem::take(&mut s.unread))
                };
                if let Some(read) = read.filter(|r| *r > 0) {
                    let mut counts = UNREAD.with_borrow(Clone::clone);
                    counts.chat = counts.chat.saturating_sub(read);
                    took_unread(counts);
                    if let Some(at) = self.picked.get() {
                        self.list("Sessions")?.SetAt(at as u32, &crate::xaml::PropertyValue::CreateInt32(at as i32)?)?;
                    }
                }
            }
            Err(problem) => self.note(&format!("读取失败：{problem}")),
        }
        Ok(())
    }

    fn talker(&self) -> Option<u64> {
        self.picked.get().and_then(|at| self.sessions.borrow().get(at).map(|s| s.talker_mid))
    }

    fn fill_message(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let messages = self.messages.borrow();
        let Some(message) = messages.get(at) else { return Ok(()) };
        let mine = self.talker().is_some_and(|t| message.sender_mid != t);
        let side = if mine { HorizontalAlignment::Right } else { HorizontalAlignment::Left };
        let card = message.video.as_ref().filter(|v| !v.title.is_empty());
        let (bubble, shared, link) = (parts.GetAt(0)?, parts.GetAt(1)?, parts.GetAt(2)?);
        for part in [&bubble, &shared, &link] {
            part.cast::<IFrameworkElement>()?.SetHorizontalAlignment(side)?;
        }
        bubble.cast::<IUIElement>()?.SetVisibility(shown(card.is_none()))?;
        shared.cast::<IUIElement>()?.SetVisibility(shown(card.is_some()))?;
        link.cast::<IUIElement>()?.SetVisibility(shown(card.is_none() && message.video.is_some()))?;
        match card {
            Some(video) => {
                let inside = shared.cast::<IPanel>()?.Children()?;
                if !video.cover.is_empty() {
                    self.covers.show(&inside.GetAt(0)?.cast::<IImage>()?, &video.cover, COVER_DECODE_WIDTH)?;
                }
                let words = inside.GetAt(1)?.cast::<IPanel>()?.Children()?;
                words.GetAt(0)?.cast::<ITextBlock>()?.SetText(&video.title)?;
                let by = if video.author.is_empty() { "▷ 视频".to_owned() } else { format!("▷ {}", video.author) };
                words.GetAt(1)?.cast::<ITextBlock>()?.SetText(&by)?;
            }
            None => {
                let border = bubble.cast::<IBorder>()?;
                let fill = if mine {
                    SolidColorBrush::CreateInstanceWithColor(COBALT)?
                } else {
                    SolidColorBrush::CreateInstanceWithColor(Color { a: 0x1F, r: 0x80, g: 0x80, b: 0x80 })?
                };
                border.SetBackground(&fill)?;
                let text = border.Child()?.cast::<IRichTextBlock>()?;
                if mine {
                    text.SetForeground(&SolidColorBrush::CreateInstanceWithColor(WHITE)?)?;
                } else {
                    text.cast::<crate::xaml::IDependencyObject>()?
                        .ClearValue(&crate::xaml::RichTextBlock::ForegroundProperty()?)?;
                }
                crate::ui::emote_text::show(&text, &message.text, &BTreeMap::new(), EMOTE_SIZE)?;
            }
        }
        Ok(())
    }

    fn note(&self, text: &str) {
        let _ = set_text(&self.root, "ChatNote", text);
        let _ = named::<IUIElement>(&self.root, "ChatNote").and_then(|n| n.SetVisibility(shown(!text.is_empty())));
    }

    fn send(&self) -> Result<()> {
        let Some(mid) = self.talker() else { return Ok(()) };
        let box_ = named::<ITextBox>(&self.root, "ChatText")?;
        let text = box_.Text()?.to_string();
        if text.trim().is_empty() || self.sending.replace(true) {
            return Ok(());
        }
        let Some(shell) = shell() else { return Ok(()) };
        let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let sent = core.services.messages.send(mid, text.trim()).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                let Some(page) = held() else { return };
                page.sending.set(false);
                match sent {
                    Ok(()) => {
                        if let Ok(text) = named::<ITextBox>(&page.root, "ChatText") {
                            let _ = text.SetText("");
                        }
                        page.note("");
                        if page.talker() == Some(mid) {
                            page.load_chat(mid);
                        }
                    }
                    Err(problem) => page.note(&format!("发送失败：{problem}")),
                }
            });
        });
        Ok(())
    }

    /// Lays the chats out: side by side, or on a narrow window the list or the chat alone.
    fn fit(&self) -> Result<()> {
        let narrow = shell().is_some_and(|s| s.class() == WindowClass::Narrow);
        let chatting = self.picked.get().is_some();
        let star = |value| GridLength { value, grid_unit_type: GridUnitType::Star };
        let (list, chat) = match (narrow, chatting) {
            (false, _) => (38.0, 62.0),
            (true, false) => (1.0, 0.0),
            (true, true) => (0.0, 1.0),
        };
        named::<IColumnDefinition>(&self.root, "SessionsColumn")?.SetWidth(star(list))?;
        named::<IColumnDefinition>(&self.root, "ChatColumn")?.SetWidth(star(chat))?;
        named::<IUIElement>(&self.root, "Sessions")?.SetVisibility(shown(list > 0.0))?;
        named::<IUIElement>(&self.root, "ChatPane")?.SetVisibility(shown(chat > 0.0 && (chatting || !narrow)))?;
        named::<IUIElement>(&self.root, "ChatBack")?.SetVisibility(shown(narrow && chatting))
    }
}

/// Opens the video `video` points at, at `focus` when a notice names a comment.
fn open_video(video: VideoRef, title: &str, focus: Option<rubi_model::CommentRef>) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core, title) = (shell.dispatcher.clone(), backend.clone(), title.to_owned());
    backend.spawn(async move {
        let bvid = core.services.videos.resolve(video).await.map_err(|e| e.to_string());
        post(&dispatcher, move || {
            let opened = match (bvid, focus) {
                (Ok(bvid), Some(focus)) => player::show_comment(&bvid, &title, focus).map_err(|e| e.message()),
                (Ok(bvid), None) => player::show(&bvid, &title, "").map_err(|e| e.message()),
                (Err(problem), _) => Err(problem),
            };
            if let Err(problem) = opened {
                toast(&format!("无法打开视频：{problem}"));
            }
        });
    });
}
