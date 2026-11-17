//! 全部关注: every followed UP in one list, and the stream of any single one of them. A row opens
//! that UP's own posts rather than his page — from 关注 the question is always what he posted.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rubi_model::FollowingUp;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Result};

use crate::ui::post_list::PostList;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::{Tab, shell, show_tab, toast};
use crate::ui::{keep, load, named, on_click, post, set_text};
use crate::xaml::{
    IImage, IItemsControl, ITextBox, IFrameworkElement, ITextBlock, IUIElement, UIElement, Visibility,
};

const FACE_DECODE_WIDTH: i32 = 96;
const LIST_FACE: i32 = 96;

const LIST: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
  </Grid.RowDefinitions>
  <Grid>
    <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
    <Button x:Name="Back" Content="&#xE72B;" FontFamily="Segoe MDL2 Assets" FontSize="14" Background="Transparent"
            BorderThickness="0" Padding="16,12" ToolTipService.ToolTip="返回" />
    <TextBlock Grid.Column="1" Text="全部关注" FontSize="20" FontWeight="Light" VerticalAlignment="Center"
               Margin="8,0,0,0" />
  </Grid>
  <TextBox x:Name="Query" Grid.Row="1" Margin="16,2,16,8" FontSize="12" MaxLength="30"
           PlaceholderText="搜索关注的 UP" />
  <ListView x:Name="All" Grid.Row="2" SelectionMode="None" IsItemClickEnabled="True">
    <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
    <ListView.ItemContainerStyle>
      <Style TargetType="ListViewItem">
        <Setter Property="Padding" Value="16,8" />
        <Setter Property="MinHeight" Value="0" />
        <Setter Property="HorizontalContentAlignment" Value="Stretch" />
      </Style>
    </ListView.ItemContainerStyle>
    <ListView.ItemTemplate><DataTemplate>
      <Grid Background="Transparent">
        <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" />
          <ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
        <Image Width="48" Height="48" Stretch="UniformToFill" />
        <TextBlock Grid.Column="1" Margin="12,0,0,0" FontSize="15" VerticalAlignment="Center"
                   TextTrimming="CharacterEllipsis" />
        <TextBlock Grid.Column="2" Text="&#xE76C;" FontFamily="Segoe MDL2 Assets" FontSize="12"
                   VerticalAlignment="Center" Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      </Grid>
    </DataTemplate></ListView.ItemTemplate>
  </ListView>
</Grid>"##;

const STREAM: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
  </Grid.RowDefinitions>
  <Grid>
    <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
    <Button x:Name="Back" Content="&#xE72B;" FontFamily="Segoe MDL2 Assets" FontSize="14" Background="Transparent"
            BorderThickness="0" Padding="16,12" ToolTipService.ToolTip="全部关注" />
    <TextBlock Grid.Column="1" Text="动态" FontSize="20" FontWeight="Light" VerticalAlignment="Center"
               Margin="8,0,0,0" />
  </Grid>
  <Grid Grid.Row="1" Margin="16,0,16,10">
    <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
    <Image x:Name="Face" Width="48" Height="48" Stretch="UniformToFill" />
    <StackPanel Grid.Column="1" Margin="12,0,0,0" VerticalAlignment="Center">
      <TextBlock x:Name="Name" FontSize="20" FontWeight="SemiBold" TextTrimming="CharacterEllipsis" />
      <TextBlock x:Name="Note" FontSize="11" Margin="0,2,0,0"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    </StackPanel>
  </Grid>
  <Grid x:Name="Host" Grid.Row="2" />
</Grid>"##;

struct All {
    root: IFrameworkElement,
    /// Every followed UP as read, before the box narrows it.
    everyone: RefCell<Vec<FollowingUp>>,
    /// The rows on screen, in order.
    rows: RefCell<Vec<FollowingUp>>,
    covers: Covers,
    started: Cell<bool>,
}

impl All {
    fn show_ups(&self, ups: Vec<FollowingUp>) -> Result<()> {
        let count = ups.len();
        *self.rows.borrow_mut() = ups;
        let list = named::<IFrameworkElement>(&self.root, "All")?;
        list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        add_rows(&list, 0, count)
    }

    fn filter(&self) -> Result<()> {
        let query = named::<ITextBox>(&self.root, "Query")
            .and_then(|b| b.Text())
            .map(|t| t.to_string().trim().to_lowercase())
            .unwrap_or_default();
        let ups: Vec<FollowingUp> = self
            .everyone
            .borrow()
            .iter()
            .filter(|up| query.is_empty() || up.name.to_lowercase().contains(&query))
            .cloned()
            .collect();
        self.show_ups(ups)
    }

    fn took(&self, ups: Vec<FollowingUp>) -> Result<()> {
        *self.everyone.borrow_mut() = ups;
        self.filter()
    }

    fn start(&self) {
        if self.started.replace(true) {
            return;
        }
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let all = core.services.following.all().await.map_err(|e| e.to_string());
            post(&dispatcher, move || match all {
                Ok(all) => {
                    if let Some(page) = held_all() {
                        let _ = page.took(all.ups);
                    }
                }
                Err(problem) => toast(&format!("读取失败：{problem}")),
            });
        });
    }

    fn fill(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let rows = self.rows.borrow();
        let Some(up) = rows.get(at) else { return Ok(()) };
        let image = parts.GetAt(0)?;
        if up.face.is_empty() {
            image.cast::<IUIElement>()?.SetVisibility(Visibility::Collapsed)?;
        } else {
            image.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)?;
            self.covers.show(&image.cast::<IImage>()?, &up.face, LIST_FACE)?;
        }
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&up.name)
    }
}

struct Stream {
    posts: Rc<PostList>,
    covers: Covers,
}

fn show_all() -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let root = load(LIST)?;
    let page = Rc::new(All {
        root: root.clone(),
        everyone: RefCell::default(),
        rows: RefCell::default(),
        covers: Covers::default(),
        started: Cell::new(false),
    });
    on_click(&root, "Back", || show_tab(Tab::Follow))?;
    let filling = Rc::downgrade(&page);
    on_row(&named::<IFrameworkElement>(&root, "All")?, move |at, parts| {
        filling.upgrade().map_or(Ok(()), |p| p.fill(at, parts))
    })?;
    let picking = Rc::downgrade(&page);
    on_row_click(&named::<IFrameworkElement>(&root, "All")?, move |at| {
        let Some(page) = picking.upgrade() else { return };
        let chosen = page.rows.borrow().get(at).cloned();
        if let Some(up) = chosen {
            let _ = show_stream(up.mid, &up.name, &up.face);
        }
    })?;
    let typing = Rc::downgrade(&page);
    keep(named::<ITextBox>(&root, "Query")?.TextChanged(
        move |_: windows_core::Ref<IInspectable>, _: windows_core::Ref<crate::xaml::TextChangedEventArgs>| {
            if let Some(page) = typing.upgrade() {
                let _ = page.filter();
            }
        },
    )?);
    ALL.with_borrow_mut(|slot| *slot = Some(page.clone()));
    shell.set_page(&root)?;
    page.start();
    Ok(())
}

fn show_stream(mid: u64, name: &str, face: &str) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let root = load(STREAM)?;
    let posts = PostList::new(|| {}, || {})?;
    named::<crate::xaml::IPanel>(&root, "Host")?
        .Children()?
        .cast::<IVector<UIElement>>()?
        .Append(&posts.element().cast::<UIElement>()?)?;
    let page = Rc::new(Stream { posts, covers: Covers::default() });
    let _ = set_text(&root, "Name", name);
    let _ = set_text(&root, "Note", "只看他的动态");
    if !face.is_empty()
        && let Ok(image) = named_val::<IImage>(&root, "Face")
    {
        let _ = page.covers.show(&image, face, FACE_DECODE_WIDTH);
    }
    on_click(&root, "Back", || {
        let _ = show_all();
    })?;
    STREAM_PAGE.with_borrow_mut(|slot| *slot = Some(page.clone()));
    shell.set_page(&root)?;
    page.posts.show_up(mid)
}

fn named_val<T: Interface>(root: &IFrameworkElement, name: &str) -> Result<T> {
    named::<T>(root, name)
}

thread_local! {
    static ALL: RefCell<Option<Rc<All>>> = const { RefCell::new(None) };
    static STREAM_PAGE: RefCell<Option<Rc<Stream>>> = const { RefCell::new(None) };
}

fn held_all() -> Option<Rc<All>> {
    ALL.with_borrow(Clone::clone)
}

/// Shows 全部关注, over the 关注 tab.
pub fn show() -> Result<()> {
    show_all()
}