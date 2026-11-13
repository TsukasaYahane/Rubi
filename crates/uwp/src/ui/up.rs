//! An UP's page: who they are, then 投稿, 动态 and 专栏. Beside each other on a wide window, one above
//! the other otherwise.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use rubi_core::UpOrder;
use rubi_model::UpProfile;
use windows_collections::IVector;
use windows_core::{Interface, Result};

use crate::state::layout::WindowClass;
use crate::state::library::count;
use crate::state::search::{article_line, article_link, level_line};
use crate::ui::card_grid::{CardGrid, Source};
use crate::ui::post_list::PostList;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::{shell, toast};
use crate::ui::{article, load, named, on_click, post, set_text};
use crate::xaml::{
    BitmapImage, Color, FrameworkElement, Grid, GridLength, GridUnitType, IColumnDefinition, IContentControl, IControl,
    IFrameworkElement, IImage, IPanel, ITextBlock, IUIElement, PropertyValue, SolidColorBrush, UIElement, Uri,
    Visibility,
};

const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };
const HEAD_WIDTH: f64 = 300.0;
const COVER_DECODE_WIDTH: i32 = 200;
const TAB_LABELS: [&str; 3] = ["投稿", "动态", "专栏"];
const POSTS_TAB: usize = 1;
const ARTICLES_TAB: usize = 2;

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.ColumnDefinitions>
    <ColumnDefinition x:Name="HeadColumn" Width="300" /><ColumnDefinition Width="*" />
  </Grid.ColumnDefinitions>
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
  </Grid.RowDefinitions>
  <Button x:Name="Back" Grid.ColumnSpan="2" Background="Transparent" BorderThickness="0" Padding="12,8">
    <StackPanel Orientation="Horizontal">
      <FontIcon Glyph="&#xE72B;" FontSize="16" /><TextBlock Text="返回" Margin="10,0,0,0" />
    </StackPanel>
  </Button>
  <Border x:Name="HeadPane" Grid.Row="1" Grid.RowSpan="2" BorderThickness="0,0,1,0"
          BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <StackPanel Padding="16,4,16,8">
      <Grid>
        <Grid.ColumnDefinitions>
          <ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" />
        </Grid.ColumnDefinitions>
        <Grid Width="64" Height="64" Background="#0050EF"><Image x:Name="Face" Stretch="UniformToFill" /></Grid>
        <StackPanel Grid.Column="1" Margin="12,0" VerticalAlignment="Center">
          <TextBlock x:Name="Name" FontSize="20" FontWeight="Light" TextWrapping="Wrap" MaxLines="2"
                     TextTrimming="CharacterEllipsis" />
          <TextBlock x:Name="Level" FontSize="12" TextWrapping="Wrap"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </StackPanel>
        <Button x:Name="FollowUp" Grid.Column="2" Visibility="Collapsed" BorderThickness="1" Padding="14,4"
                VerticalAlignment="Center" FontSize="13" />
      </Grid>
      <TextBlock x:Name="Sign" FontSize="13" Margin="0,8,0,0" TextWrapping="Wrap" MaxLines="4"
                 TextTrimming="CharacterEllipsis"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      <StackPanel Orientation="Horizontal" Margin="0,8,0,0">
        <StackPanel Margin="0,0,24,0"><TextBlock x:Name="Fans" FontSize="18" />
          <TextBlock Text="粉丝" FontSize="11" /></StackPanel>
        <StackPanel Margin="0,0,24,0"><TextBlock x:Name="Followings" FontSize="18" />
          <TextBlock Text="关注" FontSize="11" /></StackPanel>
        <StackPanel><TextBlock x:Name="Archives" FontSize="18" /><TextBlock Text="投稿" FontSize="11" /></StackPanel>
      </StackPanel>
    </StackPanel>
  </Border>
  <Grid x:Name="BodyPane" Grid.Column="1" Grid.Row="1" Grid.RowSpan="2">
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
    </Grid.RowDefinitions>
    <Grid Margin="16,4,16,4">
      <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
      <StackPanel Orientation="Horizontal">
        <StackPanel.Resources>
          <Style TargetType="Button">
            <Setter Property="Background" Value="Transparent" />
            <Setter Property="BorderThickness" Value="0" />
            <Setter Property="Padding" Value="0" />
            <Setter Property="Margin" Value="0,0,18,0" />
            <Setter Property="FontSize" Value="20" />
            <Setter Property="FontWeight" Value="Light" />
          </Style>
        </StackPanel.Resources>
        <Button x:Name="Tab0" Content="投稿" /><Button x:Name="Tab1" Content="动态" /><Button x:Name="Tab2" Content="专栏" />
      </StackPanel>
      <TextBlock x:Name="Status" Grid.Column="1" FontSize="12" VerticalAlignment="Center" TextTrimming="CharacterEllipsis"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      <StackPanel Grid.Column="2" Orientation="Horizontal" VerticalAlignment="Center">
        <TextBox x:Name="UpQuery" Width="180" FontSize="12" PlaceholderText="在这个空间里搜索" MaxLength="40" />
        <Button x:Name="UpSearch" Content="搜索" Margin="6,0,0,0" FontSize="12" Background="#0050EF"
                Foreground="White" BorderThickness="0" Padding="12,4" />
      </StackPanel>
    </Grid>
    <StackPanel x:Name="Orders" Grid.Row="1" Orientation="Horizontal" Margin="16,0,16,6">
      <Button x:Name="Order0" Content="最新" Background="Transparent" BorderThickness="0" Padding="0" Margin="0,0,16,0"
              FontSize="13" />
      <Button x:Name="Order1" Content="最多播放" Background="Transparent" BorderThickness="0" Padding="0" FontSize="13" />
    </StackPanel>
    <Grid x:Name="Lists" Grid.Row="2">
      <ListView x:Name="Articles" SelectionMode="None" IsItemClickEnabled="True" Visibility="Collapsed">
        <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
        <ListView.ItemContainerStyle>
          <Style TargetType="ListViewItem">
            <Setter Property="Padding" Value="16,8" />
            <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          </Style>
        </ListView.ItemContainerStyle>
        <ListView.ItemTemplate><DataTemplate>
          <Grid>
            <Grid.ColumnDefinitions>
              <ColumnDefinition Width="*" /><ColumnDefinition Width="108" />
            </Grid.ColumnDefinitions>
            <Grid.RowDefinitions>
              <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
            </Grid.RowDefinitions>
            <TextBlock FontSize="14" TextTrimming="CharacterEllipsis" />
            <TextBlock Grid.Row="1" FontSize="12" TextWrapping="Wrap" MaxLines="2" TextTrimming="CharacterEllipsis"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
            <TextBlock Grid.Row="2" FontSize="11" Margin="0,2,0,0"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
            <Image Grid.Column="1" Grid.RowSpan="3" Width="96" Height="64" Stretch="UniformToFill"
                   HorizontalAlignment="Right" />
          </Grid>
        </DataTemplate></ListView.ItemTemplate>
      </ListView>
    </Grid>
  </Grid>
</Grid>"##;

struct ArticleRow {
    cvid: u64,
    title: String,
    summary: String,
    line: String,
    cover: String,
}

pub struct UpPage {
    root: IFrameworkElement,
    mid: u64,
    name: RefCell<String>,
    uploads: Rc<CardGrid>,
    posts: Rc<PostList>,
    articles: RefCell<Vec<ArticleRow>>,
    articles_page: Cell<u32>,
    articles_ended: Cell<bool>,
    articles_loading: Cell<bool>,
    tab: Cell<usize>,
    order: Cell<UpOrder>,
    /// What 空间内搜索 found, if the box holds something.
    keyword: RefCell<Option<String>>,
    followed: Cell<bool>,
    /// 取消关注 was asked once.
    confirming: Cell<bool>,
    covers: Covers,
}

thread_local! {
    static PAGE: RefCell<Option<Rc<UpPage>>> = const { RefCell::new(None) };
}

fn held() -> Option<Rc<UpPage>> {
    PAGE.with_borrow(Clone::clone)
}

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs() as i64)
}

/// Comes back here from what this page opens.
fn return_here() {
    let (Some(page), Some(shell)) = (held(), shell()) else { return };
    shell.remember(move || {
        let _ = show(&page);
    });
}

fn show(page: &Rc<UpPage>) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    PAGE.with_borrow_mut(|p| *p = Some(page.clone()));
    page.arrange()?;
    shell.set_page(&page.root)
}

/// Opens the page of UP `mid`, whose name is known so far as `name`. Whoever opens it says first
/// how to come back, with [`crate::ui::shell::Shell::remember`].
pub fn open(mid: u64, name: &str) -> Result<()> {
    let page = UpPage::new(mid, name)?;
    show(&page)?;
    page.select(0)?;
    page.load_profile();
    Ok(())
}

/// Follows a new window class.
pub fn resized() {
    if let Some(page) = held() {
        let _ = page.arrange();
    }
}

impl UpPage {
    fn new(mid: u64, name: &str) -> Result<Rc<Self>> {
        let root = load(LAYOUT)?;
        let uploads = CardGrid::new(|| {
            if let Some(page) = held() {
                page.show_status();
            }
        })?;
        uploads.before_opening(return_here);
        let posts = PostList::new(
            || {
                if let Some(page) = held() {
                    page.show_status();
                }
            },
            return_here,
        )?;
        let lists = named::<IPanel>(&root, "Lists")?.Children()?.cast::<IVector<UIElement>>()?;
        lists.Append(&uploads.element().cast::<UIElement>()?)?;
        lists.Append(&posts.element().cast::<UIElement>()?)?;
        set_text(&root, "Name", name)?;
        let page = Rc::new(Self {
            root,
            mid,
            name: RefCell::new(name.to_owned()),
            uploads,
            posts,
            articles: RefCell::default(),
            articles_page: Cell::new(1),
            articles_ended: Cell::new(false),
            articles_loading: Cell::new(false),
            tab: Cell::new(0),
            order: Cell::new(UpOrder::Pubdate),
            keyword: RefCell::default(),
            followed: Cell::new(false),
            confirming: Cell::new(false),
            covers: Covers::default(),
        });
        on_click(&page.root, "Back", || {
            if let Some(shell) = shell() {
                shell.back();
            }
        })?;
        for tab in 0..TAB_LABELS.len() {
            let weak = Rc::downgrade(&page);
            on_click(&page.root, &format!("Tab{tab}"), move || {
                if let Some(page) = weak.upgrade() {
                    let _ = page.select(tab);
                }
            })?;
        }
        for (at, order) in [UpOrder::Pubdate, UpOrder::Click].into_iter().enumerate() {
            let weak = Rc::downgrade(&page);
            on_click(&page.root, &format!("Order{at}"), move || {
                if let Some(page) = weak.upgrade() {
                    page.order.set(order);
                    let _ = page.open_uploads();
                    let _ = page.mark();
                }
            })?;
        }
        let searching = Rc::downgrade(&page);
        on_click(&page.root, "UpSearch", move || {
            if let Some(page) = searching.upgrade() {
                let _ = page.search();
            }
        })?;
        let typing = Rc::downgrade(&page);
        crate::ui::on_enter(&page.root, "UpQuery", move || {
            if let Some(page) = typing.upgrade() {
                let _ = page.search();
            }
        })?;
        let weak = Rc::downgrade(&page);
        on_click(&page.root, "FollowUp", move || {
            if let Some(page) = weak.upgrade() {
                let _ = page.toggle_follow();
            }
        })?;
        let list = named::<IFrameworkElement>(&page.root, "Articles")?;
        let filling = Rc::downgrade(&page);
        on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |p| p.fill_article(at, parts)))?;
        let opening = Rc::downgrade(&page);
        on_row_click(&list, move |at| {
            if let Some(page) = opening.upgrade() {
                let cvid = page.articles.borrow().get(at).map(|a| a.cvid);
                if let Some(cvid) = cvid {
                    return_here();
                    let _ = article::open(&article_link(cvid));
                }
            }
        })?;
        Ok(page)
    }

    /// What 投稿 shows: the whole space, or what a keyword finds in it.
    fn upload_source(&self) -> Source {
        let keyword = self.keyword.borrow().clone();
        match keyword {
            Some(word) if !word.is_empty() => Source::UpSearch(self.mid, self.order.get(), word.into()),
            _ => Source::Uploads(self.mid, self.order.get()),
        }
    }

    /// Reads 投稿 from the top with what the box holds.
    fn open_uploads(&self) -> Result<()> {
        self.uploads.open(self.upload_source())
    }

    /// Takes the box as the keyword and reads 投稿 again.
    fn search(&self) -> Result<()> {
        let word = named::<crate::xaml::ITextBox>(&self.root, "UpQuery")?.Text()?.to_string();
        let word = word.trim().to_owned();
        *self.keyword.borrow_mut() = (!word.is_empty()).then_some(word);
        self.tab.set(0);
        self.arrange()?;
        self.open_uploads()
    }

    fn arrange(&self) -> Result<()> {
        let Some(shell) = shell() else { return Ok(()) };
        let wide = shell.class() == WindowClass::Wide;
        let head = named::<FrameworkElement>(&self.root, "HeadPane")?;
        let body = named::<FrameworkElement>(&self.root, "BodyPane")?;
        Grid::SetRowSpan(&head, if wide { 2 } else { 1 })?;
        Grid::SetColumnSpan(&head, if wide { 1 } else { 2 })?;
        Grid::SetColumn(&body, if wide { 1 } else { 0 })?;
        Grid::SetColumnSpan(&body, if wide { 1 } else { 2 })?;
        Grid::SetRow(&body, if wide { 1 } else { 2 })?;
        Grid::SetRowSpan(&body, if wide { 2 } else { 1 })?;
        let width = if wide {
            GridLength { value: HEAD_WIDTH, grid_unit_type: GridUnitType::Pixel }
        } else {
            GridLength { value: 0.0, grid_unit_type: GridUnitType::Pixel }
        };
        named::<IColumnDefinition>(&self.root, "HeadColumn")?.SetWidth(width)
    }

    fn select(&self, tab: usize) -> Result<()> {
        self.tab.set(tab);
        let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
        self.uploads.element().cast::<IUIElement>()?.SetVisibility(shown(tab == 0))?;
        self.posts.element().cast::<IUIElement>()?.SetVisibility(shown(tab == POSTS_TAB))?;
        named::<IUIElement>(&self.root, "Articles")?.SetVisibility(shown(tab == ARTICLES_TAB))?;
        named::<IUIElement>(&self.root, "Orders")?.SetVisibility(shown(tab == 0))?;
        match tab {
            0 if self.uploads.source().is_none() => {
                let source = self.upload_source();
                self.uploads.open(source)?;
            }
            POSTS_TAB if !self.posts.started() => self.posts.show_up(self.mid)?,
            ARTICLES_TAB if self.articles.borrow().is_empty() => self.load_articles(),
            _ => {}
        }
        self.mark()?;
        self.show_status();
        Ok(())
    }

    fn mark(&self) -> Result<()> {
        for at in 0..TAB_LABELS.len() {
            self.tint(&format!("Tab{at}"), at == self.tab.get())?;
        }
        self.tint("Order0", self.order.get() == UpOrder::Pubdate)?;
        self.tint("Order1", self.order.get() == UpOrder::Click)
    }

    fn tint(&self, name: &str, on: bool) -> Result<()> {
        let color = if on { COBALT } else { DIM };
        named::<IControl>(&self.root, name)?.SetForeground(&SolidColorBrush::CreateInstanceWithColor(color)?)
    }

    fn show_status(&self) {
        let said = match self.tab.get() {
            0 => self.uploads.said(),
            POSTS_TAB => self.posts.said(),
            _ => String::new(),
        };
        let _ = set_text(&self.root, "Status", &said);
    }

    fn load_profile(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (mid, dispatcher, core) = (self.mid, shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let Ok(profile) = core.services.space.profile(mid).await else { return };
            post(&dispatcher, move || {
                if let Some(page) = held().filter(|p| p.mid == mid) {
                    let _ = page.show_profile(&profile);
                }
            });
        });
    }

    fn show_profile(&self, profile: &UpProfile) -> Result<()> {
        *self.name.borrow_mut() = profile.name.clone();
        set_text(&self.root, "Name", &profile.name)?;
        set_text(&self.root, "Level", &level_line(profile))?;
        set_text(&self.root, "Sign", &profile.sign)?;
        set_text(&self.root, "Fans", &count(profile.fans))?;
        set_text(&self.root, "Followings", &count(profile.followings))?;
        set_text(&self.root, "Archives", &count(profile.archives))?;
        if let Ok(face) = Uri::CreateUri(&profile.face).and_then(|u| BitmapImage::CreateInstanceWithUriSource(&u)) {
            named::<IImage>(&self.root, "Face")?.SetSource(&face)?;
        }
        self.followed.set(profile.followed);
        named::<IUIElement>(&self.root, "FollowUp")?.SetVisibility(Visibility::Visible)?;
        self.show_follow()
    }

    fn show_follow(&self) -> Result<()> {
        let label = match (self.followed.get(), self.confirming.get()) {
            (false, _) => "关注",
            (true, false) => "已关注",
            (true, true) => "确认取消",
        };
        let button = named::<IContentControl>(&self.root, "FollowUp")?;
        button.SetContent(&PropertyValue::CreateString(label)?)
    }

    /// Unfollowing takes a second press.
    fn toggle_follow(&self) -> Result<()> {
        let Some(shell) = shell() else { return Ok(()) };
        let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
        if self.followed.get() && !self.confirming.replace(true) {
            return self.show_follow();
        }
        self.confirming.set(false);
        let on = !self.followed.get();
        self.followed.set(on);
        self.show_follow()?;
        let (mid, name, dispatcher, core) =
            (self.mid, self.name.borrow().clone(), shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let done = core.services.actions.follow_mid(mid, &name, on).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Err(problem) = done {
                    toast(&format!("操作失败：{problem}"));
                    if let Some(page) = held().filter(|p| p.mid == mid) {
                        page.followed.set(!on);
                        let _ = page.show_follow();
                    }
                }
            });
        });
        Ok(())
    }

    fn load_articles(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.articles_ended.get() || self.articles_loading.replace(true) {
            return;
        }
        let (mid, page, dispatcher, core) =
            (self.mid, self.articles_page.get(), shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let found = core.services.space.articles(mid, page).await.map_err(|e| e.to_string());
            post(&dispatcher, move || match found {
                Ok(found) => {
                    if let Some(page) = held().filter(|p| p.mid == mid) {
                        let now = now();
                        let rows = found
                            .items
                            .into_iter()
                            .map(|a| ArticleRow {
                                cvid: a.cvid,
                                line: article_line(a.views, a.published_at, now),
                                title: a.title,
                                summary: a.summary,
                                cover: a.cover,
                            })
                            .collect();
                        let _ = page.took_articles(rows, found.has_more);
                    }
                }
                Err(problem) => toast(&format!("读取失败：{problem}")),
            });
        });
    }

    fn took_articles(&self, rows: Vec<ArticleRow>, more: bool) -> Result<()> {
        self.articles_loading.set(false);
        self.articles_ended.set(!more);
        self.articles_page.set(self.articles_page.get() + 1);
        let from = self.articles.borrow().len();
        self.articles.borrow_mut().extend(rows);
        let to = self.articles.borrow().len();
        add_rows(&named(&self.root, "Articles")?, from, to)
    }

    fn fill_article(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let wants_more = {
            let rows = self.articles.borrow();
            let Some(row) = rows.get(at) else { return Ok(()) };
            parts.GetAt(0)?.cast::<ITextBlock>()?.SetText(&row.title)?;
            parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&row.summary)?;
            parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&row.line)?;
            let _ = self.covers.show(&parts.GetAt(3)?.cast::<IImage>()?, &row.cover, COVER_DECODE_WIDTH);
            at + 6 >= rows.len()
        };
        if wants_more {
            self.load_articles();
        }
        Ok(())
    }
}
