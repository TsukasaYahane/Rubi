//! 关注: followed UPs beside their posts, or above them on a narrow window. The page lives on while
//! a video plays, so coming back finds the same place.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rubi_core::CommentArea;
use rubi_model::{FollowedUp, FollowingUp};
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Result};

use crate::state::layout::WindowClass;
use crate::ui::comment_list::{self, CommentList};
use crate::ui::post_list::PostList;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::{shell, toast};
use crate::ui::{load, named, on_click, post, set_text, up};
use crate::xaml::{
    GridLength, GridUnitType, IBorder, IColumnDefinition, IFrameworkElement, IImage, IItemsControl, IPanel,
    IRowDefinition, ISelector, ITextBlock, IUIElement, UIElement, Visibility,
};

/// The row that lists the rest of the followed UPs.
const MORE: u64 = u64::MAX;
const FACE_DECODE_WIDTH: i32 = 64;
/// Width of the UP list on a wide and a medium window.
const UPS_WIDE: f64 = 230.0;
const UPS_MEDIUM: f64 = 190.0;
/// Width of the comment pane beside the posts.
const COMMENTS_WIDE: f64 = 380.0;

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.ColumnDefinitions>
    <ColumnDefinition x:Name="UpsColumn" Width="230" /><ColumnDefinition Width="*" />
    <ColumnDefinition x:Name="CommentsColumn" Width="0" />
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
             IsHitTestVisible="False"
             Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  <Button x:Name="UpLink" Grid.Column="1" Grid.Row="1" Content="进入主页" Visibility="Collapsed"
          HorizontalAlignment="Right" VerticalAlignment="Top" Margin="0,8,16,0" Background="#59000000"
          Foreground="White" BorderThickness="0" Padding="10,4" FontSize="12" Canvas.ZIndex="1" />
  <Grid x:Name="PostsHost" Grid.Column="1" Grid.Row="1" />
  <Border x:Name="CommentsPane" Grid.Column="2" Grid.RowSpan="2" Visibility="Collapsed" BorderThickness="1,0,0,0"
          BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}" />
</Grid>"##;

fn shown(element: windows_core::Result<UIElement>, on: bool) -> Result<()> {
    element?.cast::<IUIElement>()?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })
}

pub struct FollowPage {
    root: IFrameworkElement,
    ups: RefCell<Vec<FollowedUp>>,
    posts: Rc<PostList>,
    comments: Rc<CommentList>,
    /// The comment pane is open on a wide window.
    comments_open: Cell<bool>,
    /// The UP whose posts show; 0 for everyone.
    mid: Cell<u64>,
    started: Cell<bool>,
    covers: Covers,
}

impl FollowPage {
    fn new() -> Result<Rc<Self>> {
        let root = load(LAYOUT)?;
        let posts = PostList::new(
            || {
                if let Some(page) = held() {
                    let _ = set_text(&page.root, "Status", &page.posts.said());
                }
            },
            || {},
        )?;
        named::<IPanel>(&root, "PostsHost")?
            .Children()?
            .cast::<IVector<UIElement>>()?
            .Append(&posts.element().cast::<UIElement>()?)?;
        let comments = CommentList::new()?;
        named::<IBorder>(&root, "CommentsPane")?.SetChild(&comments.element().cast::<UIElement>()?)?;
        let page = Rc::new(Self {
            root: root.clone(),
            ups: RefCell::default(),
            posts,
            comments,
            comments_open: Cell::new(false),
            mid: Cell::new(0),
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
        let asking = Rc::downgrade(&page);
        page.posts.on_comments(move |area, total| {
            if let Some(page) = asking.upgrade() {
                let _ = page.open_comments(area, total);
            }
        });
        let closing = Rc::downgrade(&page);
        page.comments.set_close(move || {
            if let (Some(page), Some(shell)) = (closing.upgrade(), shell()) {
                page.comments_open.set(false);
                let _ = page.arrange(shell.class());
            }
        })?;
        let weak = Rc::downgrade(&page);
        on_click(&root, "UpLink", move || {
            if let Some(page) = weak.upgrade() {
                let chosen = page.ups.borrow().iter().find(|u| u.mid == page.mid.get()).map(|u| u.name.clone());
                if let Some(name) = chosen {
                    let _ = up::open(page.mid.get(), &name);
                }
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
        self.posts.show_up(0)
    }

    fn took_ups(&self, found: Vec<FollowedUp>) -> Result<()> {
        let everyone = FollowedUp { name: "全部".into(), ..FollowedUp::default() };
        let more = FollowedUp { mid: MORE, name: "更多关注".into(), ..FollowedUp::default() };
        let mut ups = vec![everyone];
        ups.extend(found);
        ups.push(more);
        self.show_ups(ups)
    }

    fn show_ups(&self, ups: Vec<FollowedUp>) -> Result<()> {
        let selected = ups.iter().position(|u| u.mid == self.mid.get()).unwrap_or(0);
        let count = ups.len();
        *self.ups.borrow_mut() = ups;
        for name in ["Ups", "Bar"] {
            let list = named::<IFrameworkElement>(&self.root, name)?;
            list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
            add_rows(&list, 0, count)?;
            named::<ISelector>(&self.root, name)?.SetSelectedIndex(selected as i32)?;
        }
        Ok(())
    }

    /// Lists every followed UP, those already shown first.
    fn load_more_ups(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        toast("正在读取全部关注…");
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let all = core.services.following.all().await.map_err(|e| e.to_string());
            post(&dispatcher, move || match all {
                Ok(all) => {
                    if let Some(page) = held() {
                        let _ = page.took_all(all.ups);
                    }
                }
                Err(problem) => toast(&format!("读取失败：{problem}")),
            });
        });
    }

    fn took_all(&self, all: Vec<FollowingUp>) -> Result<()> {
        let mut ups = self.ups.borrow().clone();
        ups.retain(|u| u.mid != MORE);
        for up in all {
            if !ups.iter().any(|u| u.mid == up.mid) {
                ups.push(FollowedUp { mid: up.mid, name: up.name, face: up.face, has_update: up.has_update });
            }
        }
        self.show_ups(ups)
    }

    fn pick(&self, at: usize) -> Result<()> {
        let Some(mid) = self.ups.borrow().get(at).map(|u| u.mid) else { return Ok(()) };
        if mid == MORE {
            self.load_more_ups();
            return Ok(());
        }
        for name in ["Ups", "Bar"] {
            named::<ISelector>(&self.root, name)?.SetSelectedIndex(at as i32)?;
        }
        self.mid.set(mid);
        named::<IUIElement>(&self.root, "UpLink")?.SetVisibility(if mid == 0 {
            Visibility::Collapsed
        } else {
            Visibility::Visible
        })?;
        self.posts.show_up(mid)
    }

    fn fill_up(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let ups = self.ups.borrow();
        let Some(up) = ups.get(at) else { return Ok(()) };
        if up.face.is_empty() {
            shown(parts.GetAt(0), false)?;
        } else {
            shown(parts.GetAt(0), true)?;
            self.covers.show(&parts.GetAt(0)?.cast::<IImage>()?, &up.face, FACE_DECODE_WIDTH)?;
        }
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&up.name)
    }

    /// Beside the posts on a wide window; a page of their own otherwise, back to here.
    fn open_comments(&self, area: CommentArea, total: u64) -> Result<()> {
        let Some(shell) = shell() else { return Ok(()) };
        if shell.class() != WindowClass::Wide {
            return comment_list::open_page(area, total);
        }
        self.comments.show(area, total)?;
        self.comments_open.set(true);
        self.arrange(shell.class())
    }

    fn arrange(&self, class: WindowClass) -> Result<()> {
        let narrow = class == WindowClass::Narrow;
        let beside = class == WindowClass::Wide && self.comments_open.get();
        let comments =
            GridLength { value: if beside { COMMENTS_WIDE } else { 0.0 }, grid_unit_type: GridUnitType::Pixel };
        named::<IColumnDefinition>(&self.root, "CommentsColumn")?.SetWidth(comments)?;
        shown(named(&self.root, "CommentsPane"), beside)?;
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
