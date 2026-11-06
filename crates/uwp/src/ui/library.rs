//! 历史记录, 稍后再看 and 收藏夹. Each page lives on while 我的 comes and goes, so a video opened
//! from a list comes back to the same place in it.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rubi_model::FavFolder;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Result};

use crate::state::layout::WindowClass;
use crate::state::library::{Shelf, folder_line};
use crate::ui::rows::{add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::video_list::{Source, VideoList};
use crate::ui::{load, me, named, on_click, post, set_text};
use crate::xaml::{
    GridLength, GridUnitType, IColumnDefinition, IContentControl, IFrameworkElement, IItemsControl, ISelector,
    ITextBlock, IUIElement, UIElement, Visibility,
};

/// Width of the folder list beside a folder's videos.
const FOLDERS_WIDE: f64 = 230.0;
const FOLDERS_MEDIUM: f64 = 190.0;

const FAVORITES: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.ColumnDefinitions>
    <ColumnDefinition x:Name="FoldersColumn" Width="230" /><ColumnDefinition x:Name="VideosColumn" Width="*" />
  </Grid.ColumnDefinitions>
  <Border x:Name="FolderPane" BorderThickness="0,0,1,0"
          BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <Grid>
      <Grid.RowDefinitions>
        <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
      </Grid.RowDefinitions>
      <Button x:Name="BackToMe" Content="&#x2190; 我的" Visibility="Collapsed" Background="Transparent"
              BorderThickness="0" Padding="16,6" Foreground="#0050EF" />
      <TextBlock Grid.Row="1" Text="收藏夹" FontSize="20" FontWeight="Light" Margin="16,12,16,6" />
      <ListView x:Name="Folders" Grid.Row="2" SelectionMode="Single" IsItemClickEnabled="True">
        <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
        <ListView.ItemContainerStyle>
          <Style TargetType="ListViewItem">
            <Setter Property="Padding" Value="16,8" />
            <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          </Style>
        </ListView.ItemContainerStyle>
        <ListView.ItemTemplate><DataTemplate>
          <StackPanel>
            <TextBlock FontSize="14" TextTrimming="CharacterEllipsis" />
            <TextBlock FontSize="11" TextTrimming="CharacterEllipsis"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          </StackPanel>
        </DataTemplate></ListView.ItemTemplate>
      </ListView>
      <TextBlock x:Name="Status" Grid.Row="2" FontSize="12" Margin="16,0" TextWrapping="Wrap"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    </Grid>
  </Border>
  <ContentControl x:Name="VideoHost" Grid.Column="1" IsTabStop="False" HorizontalContentAlignment="Stretch"
                  VerticalContentAlignment="Stretch" />
</Grid>"##;

/// 收藏夹: the folders, and the videos of the one open: beside each other, or by turns on a
/// narrow window.
pub struct Favorites {
    root: IFrameworkElement,
    videos: Rc<VideoList>,
    folders: RefCell<Vec<FavFolder>>,
    class: Cell<WindowClass>,
    /// On a narrow window: showing a folder, not the list of them.
    inside: Cell<bool>,
    /// The videos' way back has been set up.
    back_set: Cell<bool>,
}

impl Favorites {
    fn new() -> Result<Rc<Self>> {
        let root = load(FAVORITES)?;
        let videos = VideoList::new(Shelf::Favorites.label(), Source::Folder(0))?;
        named::<IContentControl>(&root, "VideoHost")?.SetContent(&videos.root().cast::<UIElement>()?)?;
        let page = Rc::new(Self {
            root: root.clone(),
            videos,
            folders: RefCell::default(),
            class: Cell::new(WindowClass::Wide),
            inside: Cell::new(false),
            back_set: Cell::new(false),
        });
        let list = named::<IFrameworkElement>(&root, "Folders")?;
        let filling = Rc::downgrade(&page);
        on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |p| p.fill(at, parts)))?;
        let opening = Rc::downgrade(&page);
        on_row_click(&list, move |at| {
            if let Some(page) = opening.upgrade() {
                let _ = page.open(at);
            }
        })?;
        on_click(&root, "BackToMe", me::back_from_shelf)?;
        Ok(page)
    }

    fn fill(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let folders = self.folders.borrow();
        let Some(folder) = folders.get(at) else { return Ok(()) };
        parts.GetAt(0)?.cast::<ITextBlock>()?.SetText(&folder.title)?;
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&folder_line(folder))
    }

    fn open(self: &Rc<Self>, at: usize) -> Result<()> {
        let Some(folder) = self.folders.borrow().get(at).cloned() else { return Ok(()) };
        named::<ISelector>(&self.root, "Folders")?.SetSelectedIndex(at as i32)?;
        self.videos.open(Source::Folder(folder.id), &folder.title, &folder_line(&folder))?;
        self.inside.set(true);
        self.arrange()
    }

    fn reload(self: &Rc<Self>) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let _ = set_text(&self.root, "Status", "正在读取…");
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let folders = core.services.favorites.folders().await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(page) = FAVORITE.with_borrow(Clone::clone) {
                    let _ = page.took(folders);
                }
            });
        });
    }

    fn took(self: &Rc<Self>, folders: std::result::Result<Vec<FavFolder>, String>) -> Result<()> {
        let folders = match folders {
            Ok(folders) => folders,
            Err(problem) => return set_text(&self.root, "Status", &format!("读取失败：{problem}")),
        };
        set_text(&self.root, "Status", if folders.is_empty() { "还没有收藏夹" } else { "" })?;
        let count = folders.len();
        let list = named::<IFrameworkElement>(&self.root, "Folders")?;
        list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        *self.folders.borrow_mut() = folders;
        add_rows(&list, 0, count)?;
        if count > 0 && self.class.get() != WindowClass::Narrow {
            let inside = self.inside.get();
            self.open(0)?;
            self.inside.set(inside);
        }
        self.arrange()
    }

    fn set_class(self: &Rc<Self>, class: WindowClass) -> Result<()> {
        let was_narrow = self.class.replace(class) == WindowClass::Narrow;
        let narrow = class == WindowClass::Narrow;
        if was_narrow == narrow && self.back_set.replace(true) {
            return self.arrange();
        }
        let weak = Rc::downgrade(self);
        self.videos.set_back(narrow.then_some("收藏夹"), move || {
            if let Some(page) = weak.upgrade() {
                page.inside.set(false);
                let _ = page.arrange();
            }
        })?;
        if !narrow && self.folders.borrow().first().is_some() {
            let at = named::<ISelector>(&self.root, "Folders")?.SelectedIndex()?;
            if at < 0 {
                self.open(0)?;
            }
        }
        self.arrange()
    }

    fn arrange(&self) -> Result<()> {
        let shown = |on| if on { Visibility::Visible } else { Visibility::Collapsed };
        let (narrow, inside) = (self.class.get() == WindowClass::Narrow, self.inside.get());
        let folders_on = !narrow || !inside;
        let videos_on = !narrow || inside;
        let fill = GridLength { value: 1.0, grid_unit_type: GridUnitType::Star };
        let none = GridLength { value: 0.0, grid_unit_type: GridUnitType::Pixel };
        let fixed = if self.class.get() == WindowClass::Wide { FOLDERS_WIDE } else { FOLDERS_MEDIUM };
        let beside = GridLength { value: fixed, grid_unit_type: GridUnitType::Pixel };
        let (folders, videos) = match (narrow, inside) {
            (true, false) => (fill, none),
            (true, true) => (none, fill),
            (false, _) => (beside, fill),
        };
        named::<IColumnDefinition>(&self.root, "FoldersColumn")?.SetWidth(folders)?;
        named::<IColumnDefinition>(&self.root, "VideosColumn")?.SetWidth(videos)?;
        named::<IUIElement>(&self.root, "FolderPane")?.SetVisibility(shown(folders_on))?;
        named::<IUIElement>(&self.root, "VideoHost")?.SetVisibility(shown(videos_on))?;
        named::<IUIElement>(&self.root, "BackToMe")?.SetVisibility(shown(narrow))
    }
}

thread_local! {
    static HISTORY: RefCell<Option<Rc<VideoList>>> = const { RefCell::new(None) };
    static LATER: RefCell<Option<Rc<VideoList>>> = const { RefCell::new(None) };
    static FAVORITE: RefCell<Option<Rc<Favorites>>> = const { RefCell::new(None) };
}

fn list_page(
    place: &'static std::thread::LocalKey<RefCell<Option<Rc<VideoList>>>>,
    shelf: Shelf,
    source: Source,
    fresh: bool,
    class: WindowClass,
) -> Result<IFrameworkElement> {
    let held = place.with_borrow(Clone::clone);
    let list = match held {
        Some(list) => {
            if fresh {
                list.reload()?;
            }
            list
        }
        None => {
            let list = VideoList::new(shelf.label(), source)?;
            place.with_borrow_mut(|p| *p = Some(list.clone()));
            list.reload()?;
            list
        }
    };
    list.set_back((class == WindowClass::Narrow).then_some("我的"), me::back_from_shelf)?;
    Ok(list.root().clone())
}

/// The page of `shelf`, read again when `fresh`.
pub fn page(shelf: Shelf, fresh: bool, class: WindowClass) -> Result<IFrameworkElement> {
    match shelf {
        Shelf::History => list_page(&HISTORY, shelf, Source::History, fresh, class),
        Shelf::Later => list_page(&LATER, shelf, Source::Later, fresh, class),
        Shelf::Favorites => {
            let held = FAVORITE.with_borrow(Clone::clone);
            let page = match held {
                Some(page) => {
                    if fresh {
                        page.reload();
                    }
                    page
                }
                None => {
                    let page = Favorites::new()?;
                    FAVORITE.with_borrow_mut(|p| *p = Some(page.clone()));
                    page.reload();
                    page
                }
            };
            page.set_class(class)?;
            Ok(page.root.clone())
        }
    }
}

/// Drops what the lists hold: the account changed.
pub fn forget() {
    HISTORY.with_borrow_mut(|p| *p = None);
    LATER.with_borrow_mut(|p| *p = None);
    FAVORITE.with_borrow_mut(|p| *p = None);
}
