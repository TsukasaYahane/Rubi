//! A season of 影视: poster, what it is, and its episodes. An episode opens on the video page only
//! when this account may watch all of it; a trial or a DRM stream gets the reason instead.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rubi_core::{PgcEpisode, PgcSeasonDetail};
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::layout::WindowClass;
use crate::state::pgc::{episode_columns, episode_label, poster_url};
use crate::ui::rows::{add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{keep, load, named, on_click, player, post, set_text};
use crate::xaml::{
    BitmapImage, FrameworkElement, Grid, GridLength, GridUnitType, IBorder, IColumnDefinition, IFrameworkElement,
    IImage, IItemsControl2, IItemsWrapGrid, IRowDefinition, ITextBlock, IUIElement, SizeChangedEventArgs, UIElement,
    Uri, Visibility,
};

const EPISODE_HEIGHT: f64 = 44.0;
/// The poster beside the season is 130 wide: twice that keeps it sharp.
const POSTER_WIDTH: u32 = 260;
/// The quality asked for when checking whether an episode may be watched.
const CHECK_QN: u32 = 16;

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <Button x:Name="Back" Background="Transparent" BorderThickness="0" Padding="12,8">
    <StackPanel Orientation="Horizontal">
      <FontIcon Glyph="&#xE72B;" FontSize="16" /><TextBlock Text="返回" Margin="10,0,0,0" />
    </StackPanel>
  </Button>
  <Grid x:Name="Body" Grid.Row="1">
    <Grid.ColumnDefinitions>
      <ColumnDefinition x:Name="InfoColumn" Width="*" /><ColumnDefinition x:Name="ListColumn" Width="1.1*" />
    </Grid.ColumnDefinitions>
    <Grid.RowDefinitions>
      <RowDefinition x:Name="InfoRow" Height="*" /><RowDefinition x:Name="ListRow" Height="0" />
    </Grid.RowDefinitions>
    <ScrollViewer x:Name="InfoPane" Padding="16,0,16,12">
      <StackPanel>
        <Grid>
          <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
          <Grid x:Name="PosterBox" Width="130" Height="173" Background="{ThemeResource SystemControlBackgroundBaseLowBrush}">
            <Image x:Name="Poster" Stretch="UniformToFill" />
            <Border x:Name="Badge" HorizontalAlignment="Right" VerticalAlignment="Top" Padding="5,1"
                    Visibility="Collapsed"><TextBlock x:Name="BadgeText" FontSize="10" Foreground="White" /></Border>
          </Grid>
          <StackPanel Grid.Column="1" Margin="12,0,0,0">
            <TextBlock x:Name="SeasonTitle" FontSize="22" FontWeight="Light" TextWrapping="Wrap" MaxLines="3" />
            <TextBlock x:Name="Score" FontSize="16" Foreground="#E0A000" Margin="0,4,0,0" />
            <TextBlock x:Name="Kinds" FontSize="12" Margin="0,4,0,0" TextWrapping="Wrap"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
            <TextBlock x:Name="Aired" FontSize="12" TextWrapping="Wrap"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          </StackPanel>
        </Grid>
        <TextBlock x:Name="Evaluate" FontSize="13" Margin="0,10,0,0" TextWrapping="Wrap" LineHeight="20"
                   IsTextSelectionEnabled="True" Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        <TextBlock x:Name="Status" FontSize="13" Margin="0,10,0,0" TextWrapping="Wrap"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      </StackPanel>
    </ScrollViewer>
    <Grid x:Name="ListPane" Grid.Column="1" Padding="16,0,16,0">
      <Grid.RowDefinitions>
        <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
      </Grid.RowDefinitions>
      <TextBlock x:Name="EpisodeCount" FontSize="18" FontWeight="Light" Margin="0,0,0,6" />
      <Border x:Name="Denied" Grid.Row="1" Visibility="Collapsed" BorderThickness="3,0,0,0" BorderBrush="#E81123"
              Padding="10,6" Margin="0,0,0,8">
        <TextBlock x:Name="DeniedText" FontSize="12" TextWrapping="Wrap" />
      </Border>
      <GridView x:Name="Episodes" Grid.Row="2" SelectionMode="None" IsItemClickEnabled="True">
        <GridView.ItemContainerTransitions><TransitionCollection /></GridView.ItemContainerTransitions>
        <GridView.ItemContainerStyle>
          <Style TargetType="GridViewItem">
            <Setter Property="Margin" Value="0" />
            <Setter Property="Padding" Value="0,0,6,6" />
            <Setter Property="HorizontalContentAlignment" Value="Stretch" />
            <Setter Property="VerticalContentAlignment" Value="Stretch" />
          </Style>
        </GridView.ItemContainerStyle>
        <GridView.ItemsPanel><ItemsPanelTemplate><ItemsWrapGrid Orientation="Horizontal" /></ItemsPanelTemplate></GridView.ItemsPanel>
        <GridView.ItemTemplate><DataTemplate>
          <Grid>
            <Border BorderThickness="1" Padding="8,0" BorderBrush="{ThemeResource SystemControlForegroundBaseMediumLowBrush}">
              <TextBlock FontSize="12" VerticalAlignment="Center" TextTrimming="CharacterEllipsis" />
            </Border>
          </Grid>
        </DataTemplate></GridView.ItemTemplate>
      </GridView>
    </Grid>
  </Grid>
</Grid>"##;

pub struct SeasonPage {
    root: IFrameworkElement,
    season_id: u64,
    title: RefCell<String>,
    episodes: RefCell<Vec<PgcEpisode>>,
    /// The episode being checked, so a second press waits.
    checking: Cell<Option<u64>>,
}

thread_local! {
    static PAGE: RefCell<Option<Rc<SeasonPage>>> = const { RefCell::new(None) };
}

fn held() -> Option<Rc<SeasonPage>> {
    PAGE.with_borrow(Clone::clone)
}

fn show(page: &Rc<SeasonPage>) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    PAGE.with_borrow_mut(|p| *p = Some(page.clone()));
    page.arrange()?;
    shell.set_page(&page.root)
}

/// Opens season `season_id`, called `title` so far.
pub fn open(season_id: u64, title: &str) -> Result<()> {
    let page = SeasonPage::new(season_id, title)?;
    show(&page)?;
    page.load();
    Ok(())
}

/// Follows a new window class.
pub fn resized() {
    if let Some(page) = held() {
        let _ = page.arrange();
    }
}

impl SeasonPage {
    fn new(season_id: u64, title: &str) -> Result<Rc<Self>> {
        let root = load(LAYOUT)?;
        set_text(&root, "SeasonTitle", title)?;
        set_text(&root, "Status", "正在读取…")?;
        let page = Rc::new(Self {
            root: root.clone(),
            season_id,
            title: RefCell::new(title.to_owned()),
            episodes: RefCell::default(),
            checking: Cell::new(None),
        });
        on_click(&root, "Back", || {
            if let Some(shell) = shell() {
                shell.back();
            }
        })?;
        let list = named::<IFrameworkElement>(&root, "Episodes")?;
        let filling = Rc::downgrade(&page);
        on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |p| p.fill(at, parts)))?;
        let picking = Rc::downgrade(&page);
        on_row_click(&list, move |at| {
            if let Some(page) = picking.upgrade() {
                let _ = page.pick(at);
            }
        })?;
        let sizing = Rc::downgrade(&page);
        keep(list.SizeChanged(move |_: Ref<IInspectable>, _: Ref<SizeChangedEventArgs>| {
            if let Some(page) = sizing.upgrade() {
                let _ = page.fit();
            }
        })?);
        Ok(page)
    }

    /// Side by side on a wide window, the episodes under the rest otherwise.
    fn arrange(&self) -> Result<()> {
        let Some(shell) = shell() else { return Ok(()) };
        let wide = shell.class() == WindowClass::Wide;
        let star = |value| GridLength { value, grid_unit_type: GridUnitType::Star };
        let auto = GridLength { value: 1.0, grid_unit_type: GridUnitType::Auto };
        let none = GridLength { value: 0.0, grid_unit_type: GridUnitType::Pixel };
        named::<IColumnDefinition>(&self.root, "ListColumn")?.SetWidth(if wide { star(1.1) } else { none })?;
        named::<IRowDefinition>(&self.root, "InfoRow")?.SetHeight(if wide { star(1.0) } else { auto })?;
        named::<IRowDefinition>(&self.root, "ListRow")?.SetHeight(if wide { none } else { star(1.0) })?;
        let list = named::<FrameworkElement>(&self.root, "ListPane")?;
        Grid::SetColumn(&list, if wide { 1 } else { 0 })?;
        Grid::SetRow(&list, if wide { 0 } else { 1 })?;
        self.fit()
    }

    fn fit(&self) -> Result<()> {
        let list = named::<IFrameworkElement>(&self.root, "Episodes")?;
        let width = list.ActualWidth()?;
        let Ok(panel) = list.cast::<IItemsControl2>().and_then(|l| l.ItemsPanelRoot()) else { return Ok(()) };
        let panel = panel.cast::<IItemsWrapGrid>()?;
        let columns = episode_columns(width);
        panel.SetMaximumRowsOrColumns(columns as i32)?;
        panel.SetItemWidth((width / f64::from(columns)).floor().max(1.0))?;
        panel.SetItemHeight(EPISODE_HEIGHT)
    }

    fn load(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (id, dispatcher, core) = (self.season_id, shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let detail = core.services.pgc.season(id).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(page) = held().filter(|p| p.season_id == id) {
                    let _ = page.took(detail);
                }
            });
        });
    }

    fn took(&self, detail: std::result::Result<PgcSeasonDetail, String>) -> Result<()> {
        let detail = match detail {
            Ok(detail) => detail,
            Err(problem) => return set_text(&self.root, "Status", &format!("读取失败：{problem}")),
        };
        *self.title.borrow_mut() = detail.title.clone();
        set_text(&self.root, "SeasonTitle", &detail.title)?;
        set_text(
            &self.root,
            "Score",
            &if detail.score.is_empty() { String::new() } else { format!("★ {}", detail.score) },
        )?;
        let kinds = [detail.areas.join(" / "), detail.styles.join(" / ")]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
        set_text(&self.root, "Kinds", &kinds)?;
        set_text(&self.root, "Aired", &detail.pub_time)?;
        set_text(&self.root, "Evaluate", &detail.evaluate)?;
        set_text(&self.root, "Status", "")?;
        let url = poster_url(&detail.cover, POSTER_WIDTH);
        if let Ok(poster) = Uri::CreateUri(&url).and_then(|u| BitmapImage::CreateInstanceWithUriSource(&u)) {
            named::<IImage>(&self.root, "Poster")?.SetSource(&poster)?;
        }
        set_text(&self.root, "EpisodeCount", &format!("选集 · {}", detail.episodes.len()))?;
        let count = detail.episodes.len();
        *self.episodes.borrow_mut() = detail.episodes;
        add_rows(&named(&self.root, "Episodes")?, 0, count)?;
        self.fit()
    }

    fn fill(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let episodes = self.episodes.borrow();
        let Some(episode) = episodes.get(at) else { return Ok(()) };
        let mut label = episode_label(&episode.title, &episode.long_title);
        if !episode.badge.is_empty() {
            label = format!("{} {label}", episode.badge);
        }
        if self.checking.get() == Some(episode.ep_id) {
            label = format!("检查中… {label}");
        }
        parts.GetAt(0)?.cast::<IBorder>()?.Child()?.cast::<ITextBlock>()?.SetText(&label)
    }

    /// Checks the episode may be watched in full, then opens it; says why not otherwise.
    fn pick(self: &Rc<Self>, at: usize) -> Result<()> {
        let Some(shell) = shell() else { return Ok(()) };
        let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
        let Some(episode) = self.episodes.borrow().get(at).cloned() else { return Ok(()) };
        if self.checking.get().is_some() {
            return Ok(());
        }
        self.checking.set(Some(episode.ep_id));
        named::<IUIElement>(&self.root, "Denied")?.SetVisibility(Visibility::Collapsed)?;
        set_text(&self.root, "EpisodeCount", &format!("选集 · {} · 检查中…", self.episodes.borrow().len()))?;
        let (id, dispatcher, core) = (self.season_id, shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let allowed = core.services.pgc.play(episode.ep_id, episode.cid, CHECK_QN).await.map(|_| ());
            let allowed = allowed.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(page) = held().filter(|p| p.season_id == id) {
                    let _ = page.checked(&episode, allowed);
                }
            });
        });
        Ok(())
    }

    fn checked(self: &Rc<Self>, episode: &PgcEpisode, allowed: std::result::Result<(), String>) -> Result<()> {
        self.checking.set(None);
        set_text(&self.root, "EpisodeCount", &format!("选集 · {}", self.episodes.borrow().len()))?;
        if let Err(problem) = allowed {
            set_text(&self.root, "DeniedText", &format!("暂时不能在 Rubi 观看\n{problem}"))?;
            return named::<IUIElement>(&self.root, "Denied")?.SetVisibility(Visibility::Visible);
        }
        let Some(shell) = shell() else { return Ok(()) };
        let page = self.clone();
        shell.remember(move || {
            let _ = show(&page);
        });
        crate::ui::shell::leave_pages();
        let title = format!("{} {}", self.title.borrow(), episode_label(&episode.title, &episode.long_title));
        player::show_episode(&episode.bvid, title.trim(), &self.title.borrow(), episode.ep_id, episode.cid)
    }
}
