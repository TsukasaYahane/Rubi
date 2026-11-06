//! 关注: followed UPs beside their posts, or above them on a narrow window. The page lives on while
//! a video plays, so coming back finds the same place.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use rubi_model::FollowedUp;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Result};

use crate::state::follow::{PICTURES, Post};
use crate::state::layout::WindowClass;
use crate::ui::feed;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{load, named, post, set_text};
use crate::xaml::{
    GridLength, GridUnitType, IColumnDefinition, IFrameworkElement, IImage, IItemsControl, IPanel, IRowDefinition,
    ISelector, ITextBlock, IUIElement, Launcher, UIElement, Uri, Visibility,
};

const FACE_DECODE_WIDTH: i32 = 64;
const COVER_DECODE_WIDTH: i32 = 320;
const PICTURE_DECODE_WIDTH: i32 = 240;
/// Rows from the end at which more load.
const LOAD_AHEAD: usize = 6;
/// Width of the UP list on a wide and a medium window.
const UPS_WIDE: f64 = 230.0;
const UPS_MEDIUM: f64 = 190.0;

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.ColumnDefinitions>
    <ColumnDefinition x:Name="UpsColumn" Width="230" /><ColumnDefinition Width="*" />
  </Grid.ColumnDefinitions>
  <Grid.RowDefinitions><RowDefinition x:Name="BarRow" Height="0" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <Border x:Name="UpsPane" Grid.RowSpan="2" BorderThickness="0,0,1,0"
          BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <Grid>
      <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
      <TextBlock Text="关注" FontSize="20" FontWeight="Light" Margin="16,8,16,6" />
      <ListView x:Name="Ups" Grid.Row="1" SelectionMode="Single" IsItemClickEnabled="True">
        <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
        <ListView.ItemContainerStyle>
          <Style TargetType="ListViewItem">
            <Setter Property="Padding" Value="16,4" />
            <Setter Property="MinHeight" Value="0" />
            <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          </Style>
        </ListView.ItemContainerStyle>
        <ListView.ItemTemplate><DataTemplate>
          <StackPanel Orientation="Horizontal">
            <Image Width="32" Height="32" Stretch="UniformToFill" />
            <TextBlock Margin="10,0,0,0" FontSize="13" VerticalAlignment="Center" TextTrimming="CharacterEllipsis" />
          </StackPanel>
        </DataTemplate></ListView.ItemTemplate>
      </ListView>
    </Grid>
  </Border>
  <ListView x:Name="Bar" Grid.ColumnSpan="2" SelectionMode="Single" IsItemClickEnabled="True" Visibility="Collapsed"
            ScrollViewer.HorizontalScrollBarVisibility="Hidden" ScrollViewer.HorizontalScrollMode="Enabled"
            ScrollViewer.VerticalScrollMode="Disabled" ScrollViewer.VerticalScrollBarVisibility="Disabled"
            BorderThickness="0,0,0,1" BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <ListView.ItemsPanel><ItemsPanelTemplate><ItemsStackPanel Orientation="Horizontal" /></ItemsPanelTemplate></ListView.ItemsPanel>
    <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
    <ListView.ItemContainerStyle>
      <Style TargetType="ListViewItem">
        <Setter Property="Padding" Value="6,4" />
        <Setter Property="MinWidth" Value="0" />
        <Setter Property="Width" Value="62" />
      </Style>
    </ListView.ItemContainerStyle>
    <ListView.ItemTemplate><DataTemplate>
      <StackPanel>
        <Image Width="36" Height="36" Stretch="UniformToFill" HorizontalAlignment="Center" />
        <TextBlock FontSize="11" TextTrimming="CharacterEllipsis" TextAlignment="Center" Margin="0,3,0,0" />
      </StackPanel>
    </DataTemplate></ListView.ItemTemplate>
  </ListView>
  <TextBlock x:Name="Status" Grid.Column="1" Grid.Row="1" FontSize="13" Margin="16,12" TextWrapping="Wrap"
             Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  <ListView x:Name="Posts" Grid.Column="1" Grid.Row="1" SelectionMode="None" IsItemClickEnabled="True">
    <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
    <ListView.ItemContainerStyle>
      <Style TargetType="ListViewItem">
        <Setter Property="Padding" Value="16,12" />
        <Setter Property="HorizontalContentAlignment" Value="Stretch" />
        <Setter Property="VerticalContentAlignment" Value="Top" />
      </Style>
    </ListView.ItemContainerStyle>
    <ListView.ItemTemplate><DataTemplate>
      <Grid>
        <Grid.ColumnDefinitions>
          <ColumnDefinition Width="46" /><ColumnDefinition MaxWidth="602" /><ColumnDefinition Width="Auto" />
        </Grid.ColumnDefinitions>
        <Grid.RowDefinitions>
          <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
          <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
        </Grid.RowDefinitions>
        <Image Grid.RowSpan="2" Width="36" Height="36" Stretch="UniformToFill" VerticalAlignment="Top" />
        <TextBlock Grid.Column="1" FontSize="13" TextTrimming="CharacterEllipsis" />
        <TextBlock Grid.Column="1" Grid.Row="1" FontSize="11"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        <TextBlock Grid.Column="1" Grid.Row="2" FontSize="13" Margin="0,8,0,0" TextWrapping="Wrap" MaxLines="6"
                   TextTrimming="CharacterEllipsis" />
        <Grid Grid.Column="1" Grid.Row="3" Margin="0,8,0,0" Height="90"
              Background="{ThemeResource SystemControlBackgroundBaseLowBrush}">
          <Grid.ColumnDefinitions><ColumnDefinition Width="170" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
          <Grid.RowDefinitions><RowDefinition Height="*" /><RowDefinition Height="Auto" /></Grid.RowDefinitions>
          <Image Grid.RowSpan="2" Margin="0,0,10,0" Stretch="UniformToFill" />
          <TextBlock Grid.Column="1" Margin="0,8,8,0" FontSize="13" TextWrapping="Wrap" MaxLines="2"
                     TextTrimming="CharacterEllipsis" />
          <TextBlock Grid.Column="1" Grid.Row="1" Margin="0,0,8,8" FontSize="11"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </Grid>
        <StackPanel Grid.Column="1" Grid.Row="4" Margin="0,8,0,0" Orientation="Horizontal">
          <Image Width="130" Height="130" Margin="0,0,4,0" Stretch="UniformToFill" />
          <Image Width="130" Height="130" Margin="0,0,4,0" Stretch="UniformToFill" />
          <Image Width="130" Height="130" Margin="0,0,4,0" Stretch="UniformToFill" />
          <TextBlock FontSize="12" VerticalAlignment="Bottom" Margin="4,0,0,0"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </StackPanel>
        <Border Grid.Column="1" Grid.Row="5" Margin="0,6,0,0" Padding="10,4,0,4" BorderThickness="3,0,0,0"
                BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
          <TextBlock FontSize="12" TextWrapping="Wrap" MaxLines="3" TextTrimming="CharacterEllipsis"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </Border>
      </Grid>
    </DataTemplate></ListView.ItemTemplate>
  </ListView>
</Grid>"##;

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs() as i64)
}

fn shown(element: windows_core::Result<UIElement>, on: bool) -> Result<()> {
    element?.cast::<IUIElement>()?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })
}

pub struct FollowPage {
    root: IFrameworkElement,
    ups: RefCell<Vec<FollowedUp>>,
    posts: RefCell<Vec<Post>>,
    /// The UP whose posts show; 0 for everyone.
    mid: Cell<u64>,
    next: RefCell<Option<String>>,
    ended: Cell<bool>,
    loading: Cell<bool>,
    turn: Cell<u64>,
    started: Cell<bool>,
    covers: Covers,
}

impl FollowPage {
    fn new() -> Result<Rc<Self>> {
        let root = load(LAYOUT)?;
        let page = Rc::new(Self {
            root: root.clone(),
            ups: RefCell::default(),
            posts: RefCell::default(),
            mid: Cell::new(0),
            next: RefCell::default(),
            ended: Cell::new(false),
            loading: Cell::new(false),
            turn: Cell::new(0),
            started: Cell::new(false),
            covers: Covers::default(),
        });
        for name in ["Ups", "Bar"] {
            let list = named::<IFrameworkElement>(&root, name)?;
            let filling = Rc::downgrade(&page);
            on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |p| p.fill_up(at, parts)))?;
            let picking = Rc::downgrade(&page);
            on_row_click(&list, move |at| {
                if let Some(page) = picking.upgrade() {
                    let _ = page.pick(at);
                }
            })?;
        }
        let list = named::<IFrameworkElement>(&root, "Posts")?;
        let filling = Rc::downgrade(&page);
        on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |p| p.fill_post(at, parts)))?;
        let opening = Rc::downgrade(&page);
        on_row_click(&list, move |at| {
            if let Some(page) = opening.upgrade() {
                let _ = page.open(at);
            }
        })?;
        Ok(page)
    }

    fn start(&self) -> Result<()> {
        let Some(shell) = shell() else { return Ok(()) };
        let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
        if self.started.replace(true) {
            return Ok(());
        }
        if !backend.services.account.is_logged_in() {
            self.started.set(false);
            return set_text(&self.root, "Status", "登录后查看关注的动态");
        }
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let Ok(ups) = core.services.following.ups().await else { return };
            post(&dispatcher, move || {
                if let Some(page) = held() {
                    let _ = page.took_ups(ups);
                }
            });
        });
        self.show_up(0)
    }

    fn took_ups(&self, found: Vec<FollowedUp>) -> Result<()> {
        let everyone = FollowedUp { name: "全部".into(), ..FollowedUp::default() };
        let mut ups = vec![everyone];
        ups.extend(found);
        let count = ups.len();
        *self.ups.borrow_mut() = ups;
        for name in ["Ups", "Bar"] {
            add_rows(&named(&self.root, name)?, 0, count)?;
            named::<ISelector>(&self.root, name)?.SetSelectedIndex(0)?;
        }
        Ok(())
    }

    fn pick(&self, at: usize) -> Result<()> {
        let Some(mid) = self.ups.borrow().get(at).map(|u| u.mid) else { return Ok(()) };
        for name in ["Ups", "Bar"] {
            named::<ISelector>(&self.root, name)?.SetSelectedIndex(at as i32)?;
        }
        self.show_up(mid)
    }

    fn show_up(&self, mid: u64) -> Result<()> {
        self.mid.set(mid);
        self.turn.set(self.turn.get() + 1);
        self.posts.borrow_mut().clear();
        *self.next.borrow_mut() = None;
        self.ended.set(false);
        self.loading.set(false);
        let list = named::<IItemsControl>(&self.root, "Posts")?;
        list.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        set_text(&self.root, "Status", "正在读取…")?;
        self.load();
        Ok(())
    }

    fn load(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.ended.get() || self.loading.replace(true) {
            return;
        }
        let (mid, turn, cursor) = (self.mid.get(), self.turn.get(), self.next.borrow().clone());
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let following = &core.services.following;
            let page = if mid == 0 {
                following.posts(cursor.as_deref()).await
            } else {
                following.up_posts(mid, cursor.as_deref()).await
            };
            let page = page.map_err(|e| e.to_string()).map(|(posts, next)| {
                let now = now();
                (posts.into_iter().map(|p| Post::of(p, now)).collect::<Vec<_>>(), next)
            });
            post(&dispatcher, move || {
                if let Some(page_on_show) = held() {
                    let _ = page_on_show.took_posts(turn, page);
                }
            });
        });
    }

    fn took_posts(&self, turn: u64, page: std::result::Result<(Vec<Post>, Option<String>), String>) -> Result<()> {
        if turn != self.turn.get() {
            return Ok(());
        }
        self.loading.set(false);
        match page {
            Ok((posts, next)) => {
                self.ended.set(next.is_none());
                *self.next.borrow_mut() = next;
                let from = self.posts.borrow().len();
                self.posts.borrow_mut().extend(posts);
                let to = self.posts.borrow().len();
                add_rows(&named(&self.root, "Posts")?, from, to)?;
                set_text(&self.root, "Status", if to == 0 { "还没有动态" } else { "" })
            }
            Err(problem) => {
                self.ended.set(true);
                set_text(&self.root, "Status", &format!("读取失败：{problem}"))
            }
        }
    }

    fn fill_up(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let ups = self.ups.borrow();
        let Some(up) = ups.get(at) else { return Ok(()) };
        if up.face.is_empty() {
            shown(parts.GetAt(0), false)?;
        } else {
            shown(parts.GetAt(0), true)?;
            parts.GetAt(0)?.cast::<IImage>()?.SetSource(&self.covers.get(&up.face, FACE_DECODE_WIDTH)?)?;
        }
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&up.name)
    }

    fn fill_post(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let posts = self.posts.borrow();
        let Some(post) = posts.get(at) else { return Ok(()) };
        parts.GetAt(0)?.cast::<IImage>()?.SetSource(&self.covers.get(&post.face, FACE_DECODE_WIDTH)?)?;
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&post.name)?;
        parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&post.line)?;
        parts.GetAt(3)?.cast::<ITextBlock>()?.SetText(&post.text)?;
        shown(parts.GetAt(3), !post.text.is_empty())?;

        shown(parts.GetAt(4), post.clip.is_some())?;
        if let Some(clip) = &post.clip {
            let inside = parts.GetAt(4)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
            inside.GetAt(0)?.cast::<IImage>()?.SetSource(&self.covers.get(&clip.cover, COVER_DECODE_WIDTH)?)?;
            inside.GetAt(1)?.cast::<ITextBlock>()?.SetText(&clip.title)?;
            inside.GetAt(2)?.cast::<ITextBlock>()?.SetText(&clip.line)?;
        }

        let pictured = post.clip.is_none() && !post.pictures.is_empty();
        shown(parts.GetAt(5), pictured)?;
        if pictured {
            let inside = parts.GetAt(5)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
            for slot in 0..PICTURES {
                match post.pictures.get(slot) {
                    Some(url) => {
                        shown(inside.GetAt(slot as u32), true)?;
                        let picture = self.covers.get(url, PICTURE_DECODE_WIDTH)?;
                        inside.GetAt(slot as u32)?.cast::<IImage>()?.SetSource(&picture)?;
                    }
                    None => shown(inside.GetAt(slot as u32), false)?,
                }
            }
            let more = if post.more_pictures > 0 { format!("+{}", post.more_pictures) } else { String::new() };
            inside.GetAt(PICTURES as u32)?.cast::<ITextBlock>()?.SetText(&more)?;
        }

        shown(parts.GetAt(6), !post.quote.is_empty())?;
        if !post.quote.is_empty() {
            let border = parts.GetAt(6)?.cast::<crate::xaml::IBorder>()?;
            border.Child()?.cast::<ITextBlock>()?.SetText(&post.quote)?;
        }
        let near_end = at + LOAD_AHEAD >= posts.len();
        drop(posts);
        if near_end {
            self.load();
        }
        Ok(())
    }

    /// The card's video, or what it links to in the browser.
    fn open(&self, at: usize) -> Result<()> {
        let Some(post) = self.posts.borrow().get(at).cloned() else { return Ok(()) };
        if let Some(clip) = post.clip {
            return feed::route(&clip.bvid, &clip.title, &clip.owner);
        }
        if post.link.starts_with("https://") {
            drop(Launcher::LaunchUriAsync(&Uri::CreateUri(&post.link)?)?);
        }
        Ok(())
    }

    fn arrange(&self, class: WindowClass) -> Result<()> {
        let narrow = class == WindowClass::Narrow;
        let pixels = |value| GridLength { value, grid_unit_type: GridUnitType::Pixel };
        let auto = GridLength { value: 0.0, grid_unit_type: GridUnitType::Auto };
        let ups = match class {
            WindowClass::Narrow => 0.0,
            WindowClass::Medium => UPS_MEDIUM,
            WindowClass::Wide => UPS_WIDE,
        };
        named::<IColumnDefinition>(&self.root, "UpsColumn")?.SetWidth(pixels(ups))?;
        named::<IRowDefinition>(&self.root, "BarRow")?.SetHeight(if narrow { auto } else { pixels(0.0) })?;
        shown(named(&self.root, "UpsPane"), !narrow)?;
        shown(named(&self.root, "Bar"), narrow)
    }
}

thread_local! {
    static PAGE: RefCell<Option<Rc<FollowPage>>> = const { RefCell::new(None) };
}

fn held() -> Option<Rc<FollowPage>> {
    PAGE.with_borrow(Clone::clone)
}

/// Shows the 关注 tab.
pub fn show() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let page = match held() {
        Some(page) => page,
        None => {
            let page = FollowPage::new()?;
            PAGE.with_borrow_mut(|p| *p = Some(page.clone()));
            page
        }
    };
    page.arrange(shell.class())?;
    shell.set_page(&page.root)?;
    page.start()
}

/// Follows a new window class.
pub fn resized() {
    if let (Some(page), Some(shell)) = (held(), shell()) {
        let _ = page.arrange(shell.class());
    }
}

/// Drops what it holds: the account changed.
pub fn forget() {
    PAGE.with_borrow_mut(|p| *p = None);
}
