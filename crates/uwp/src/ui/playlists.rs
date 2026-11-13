//! 播放列表: lists of one's own, the order they play in, and playing one whole. Rows are reordered
//! and taken out from their menu, so the row template stays plain text.

use std::cell::RefCell;
use std::rc::Rc;

use rubi_core::{MAX_PLAYLIST_NAME, PlaylistMode};
use rubi_model::VideoCard;
use rubi_store::PlaylistInfo;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::ui::card_menu;
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::{shell, toast};
use crate::ui::{keep, load, named, on_click, popup, post, set_text};
use crate::xaml::{
    FrameworkElement, IButtonBase, IContentControl, IFrameworkElement, IImage, IItemsControl, IPanel, ITextBlock,
    ITextBox, PropertyValue, RoutedEventArgs, UIElement,
};

const LAYOUT: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions>
    <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
    <RowDefinition Height="Auto" />
  </Grid.RowDefinitions>
  <Grid Margin="16,12,16,6">
    <Grid.ColumnDefinitions>
      <ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" />
      <ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" />
    </Grid.ColumnDefinitions>
    <Button x:Name="Pick" Background="Transparent" Padding="10,4"
            FontSize="14" />
    <Button x:Name="NewList" BorderThickness="0" Grid.Column="1" Content="＋ 新建" Margin="8,0,0,0" Background="Transparent"
            Foreground="#0050EF" Padding="10,4" />
    <Button x:Name="RenameList" BorderThickness="0" Grid.Column="2" Content="重命名" Margin="8,0,0,0" Background="Transparent"
            Padding="10,4" />
    <Button x:Name="DeleteList" BorderThickness="0" Grid.Column="3" Content="删除" Margin="8,0,0,0" Background="Transparent"
            Foreground="#C04A4A" Padding="10,4" />
  </Grid>
  <StackPanel x:Name="Modes" Grid.Row="1" Orientation="Horizontal" Margin="16,0,16,8" />
  <Grid Grid.Row="2" Margin="16,0,16,0">
    <ListView x:Name="Items" SelectionMode="None" IsItemClickEnabled="True">
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
          <Grid.ColumnDefinitions><ColumnDefinition Width="28" /><ColumnDefinition Width="Auto" />
            <ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
          <TextBlock FontSize="12" VerticalAlignment="Center"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          <Image Grid.Column="1" Width="88" Height="50" Stretch="UniformToFill" />
          <StackPanel Grid.Column="2" Margin="10,0,0,0" VerticalAlignment="Center">
            <TextBlock FontSize="13" TextTrimming="CharacterEllipsis" />
            <TextBlock FontSize="11" Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          </StackPanel>
        </Grid>
      </DataTemplate></ListView.ItemTemplate>
    </ListView>
    <TextBlock x:Name="Note" FontSize="12" Margin="0,10" VerticalAlignment="Top" IsHitTestVisible="False"
               TextWrapping="Wrap" Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </Grid>
  <StackPanel Grid.Row="3" Orientation="Horizontal" Margin="16,10,16,14">
    <Button x:Name="PlayAll" Content="▶ 播放全部" Background="#0050EF" Foreground="White" BorderThickness="0"
            Padding="14,6" />
    <TextBlock x:Name="Count" Margin="12,0,0,0" VerticalAlignment="Center" FontSize="12"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  </StackPanel>
</Grid>"##;

const MODES: [(PlaylistMode, &str); 4] = [
    (PlaylistMode::Sequential, "顺序"),
    (PlaylistMode::LoopAll, "列表循环"),
    (PlaylistMode::LoopOne, "单曲循环"),
    (PlaylistMode::Shuffle, "随机"),
];

#[derive(Debug, Clone)]
struct Row {
    bvid: String,
    title: String,
    line: String,
    cover: String,
}

struct Page {
    root: IFrameworkElement,
    lists: RefCell<Vec<PlaylistInfo>>,
    list: RefCell<i64>,
    rows: RefCell<Vec<Row>>,
    mode: RefCell<PlaylistMode>,
    covers: Covers,
}

thread_local! {
    static PAGE: RefCell<Option<Rc<Page>>> = const { RefCell::new(None) };
}

fn open_page() -> Option<Rc<Page>> {
    PAGE.with_borrow(Clone::clone)
}

/// Builds the 播放列表 page.
pub fn build() -> Result<IFrameworkElement> {
    let root = load(LAYOUT)?;
    let opened = Rc::new(Page {
        root: root.clone(),
        lists: RefCell::default(),
        list: RefCell::new(0),
        rows: RefCell::default(),
        mode: RefCell::new(PlaylistMode::default()),
        covers: Covers::default(),
    });
    PAGE.with_borrow_mut(|slot| *slot = Some(opened.clone()));
    // A page whose wiring failed still shows, saying why, instead of taking 我的 down with it.
    if let Err(problem) = wire(&opened) {
        toast(&format!("播放列表：{problem}"));
    }
    opened.load();
    Ok(root)
}

fn wire(page: &Rc<Page>) -> Result<()> {
    let root = &page.root;
    on_click(root, "Pick", move || {
        if let Some(page) = open_page() {
            let _ = page.offer_lists();
        }
    })?;
    on_click(root, "NewList", || make_list())?;
    on_click(root, "RenameList", || rename_list())?;
    on_click(root, "DeleteList", || delete_list())?;
    on_click(root, "PlayAll", || play_all())?;
    let list = named::<IFrameworkElement>(root, "Items")?;
    crate::ui::rows::on_row_click(&list, |at| {
        let Some(page) = open_page() else { return };
        let Some(row) = page.rows.borrow().get(at).cloned() else { return };
        let owner = row.line.split(" · ").next().unwrap_or_default().to_owned();
        let _ = crate::ui::player::open(false, &row.bvid, &row.title, &owner);
    })?;
    on_row(&list, |at, parts| {
        let Some(page) = open_page() else { return Ok(()) };
        let rows = page.rows.borrow();
        let Some(row) = rows.get(at) else { return Ok(()) };
        let number = format!("{}", at + 1);
        parts.GetAt(0)?.cast::<ITextBlock>()?.SetText(&number)?;
        page.covers.show(&parts.GetAt(1)?.cast::<IImage>()?, &row.cover, 240)?;
        let stack = parts.GetAt(2)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
        stack.GetAt(0)?.cast::<ITextBlock>()?.SetText(&row.title)?;
        stack.GetAt(1)?.cast::<ITextBlock>()?.SetText(&row.line)?;
        Ok(())
    })?;
    card_menu::attach_choices(&list, |at| {
        open_page()?.rows.borrow().get(at)?;
        let up: card_menu::Choice = (
            "上移".to_owned(),
            Rc::new(move || {
                if let Some(page) = open_page() {
                    page.move_row(at, at.saturating_sub(1));
                }
            }),
        );
        let down: card_menu::Choice = (
            "下移".to_owned(),
            Rc::new(move || {
                if let Some(page) = open_page() {
                    page.move_row(at, at + 1);
                }
            }),
        );
        let out: card_menu::Choice = (
            "移出列表".to_owned(),
            Rc::new(move || {
                if let Some(page) = open_page() {
                    page.drop_row(at);
                }
            }),
        );
        Some(vec![up, down, out])
    })?;
    Ok(())
}

impl Page {
    /// Reads the lists and the one on show.
    fn load(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let lists = core.services.playlists.lists().map_err(|e| e.to_string());
            let mode = core.services.playlists.mode().ok();
            post(&dispatcher, move || {
                if let Some(page) = open_page() {
                    let _ = page.took_lists(lists, mode);
                }
            });
        });
    }

    fn took_lists(
        &self,
        lists: std::result::Result<Vec<PlaylistInfo>, String>,
        mode: Option<PlaylistMode>,
    ) -> Result<()> {
        let lists = match lists {
            Ok(lists) => lists,
            Err(problem) => return set_text(&self.root, "Note", &format!("读取失败：{problem}")),
        };
        {
            let current = *self.list.borrow();
            if !lists.iter().any(|l| l.id == current) {
                *self.list.borrow_mut() = lists.first().map(|l| l.id).unwrap_or_default();
            }
        }
        *self.lists.borrow_mut() = lists;
        if let Some(mode) = mode {
            *self.mode.borrow_mut() = mode;
        }
        self.draw_chrome()?;
        self.load_rows();
        Ok(())
    }

    fn draw_chrome(&self) -> Result<()> {
        let current = *self.list.borrow();
        let name = {
            let lists = self.lists.borrow();
            lists
                .iter()
                .find(|l| l.id == current)
                .map(|l| format!("{} · {} ▾", l.name, l.count))
                .unwrap_or_else(|| "播放列表 ▾".to_owned())
        };
        let value = PropertyValue::CreateString(&name)?;
        named::<IContentControl>(&self.root, "Pick")?.SetContent(&value)?;

        let modes = named::<IPanel>(&self.root, "Modes")?.Children()?.cast::<IVector<UIElement>>()?;
        modes.Clear()?;
        let on = *self.mode.borrow();
        for (mode, label) in MODES {
            let tint = if mode == on {
                r##"Foreground="#0050EF" BorderBrush="#0050EF""##
            } else {
                r##"Foreground="#A6A6A6" BorderBrush="#3A3A3A""##
            };
            let button = load(&format!(
                r##"<Button xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Content="{label}" Margin="0,0,8,0"
                      Background="Transparent" BorderThickness="1" Padding="10,3" FontSize="12" {tint} />"##
            ))?;
            keep(button.cast::<IButtonBase>()?.Click(
                move |_: Ref<IInspectable>, _: Ref<RoutedEventArgs>| set_mode(mode),
            )?);
            modes.Append(&button.cast::<UIElement>()?)?;
        }
        Ok(())
    }

    fn load_rows(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let list = *self.list.borrow();
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let found = core.services.playlists.items(list).map_err(|e| e.to_string());
            post(&dispatcher, move || {
                if let Some(page) = open_page() {
                    let _ = page.took_rows(found);
                }
            });
        });
    }

    fn took_rows(&self, found: std::result::Result<Vec<VideoCard>, String>) -> Result<()> {
        let cards = match found {
            Ok(cards) => cards,
            Err(problem) => return set_text(&self.root, "Note", &format!("读取失败：{problem}")),
        };
        let rows: Vec<Row> = cards
            .into_iter()
            .map(|card| Row {
                bvid: card.bvid.clone(),
                title: card.title,
                line: format!("{} · {} 播放", card.owner.name, card.stats.view),
                cover: card.cover.clone(),
            })
            .collect();
        let count = rows.len();
        *self.rows.borrow_mut() = rows;
        let list = named::<IFrameworkElement>(&self.root, "Items")?;
        list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        add_rows(&list, 0, count)?;
        set_text(&self.root, "Count", &format!("共 {count} 个视频"))?;
        set_text(
            &self.root,
            "Note",
            if count == 0 { "这个列表还是空的：在任意视频卡片上右键可以加进来。" } else { "" },
        )
    }

    fn offer_lists(&self) -> Result<()> {
        let lists = self.lists.borrow().clone();
        let anchor = named::<IFrameworkElement>(&self.root, "Pick")?;
        let choices: Vec<card_menu::Choice> = lists
            .into_iter()
            .map(|list| {
                let label = format!("{} · {}", list.name, list.count);
                let chosen = list.id;
                (label, Rc::new(move || pick_list(chosen)) as Rc<dyn Fn()>)
            })
            .collect();
        card_menu::show_under(&anchor.cast::<FrameworkElement>()?, choices)
    }

    fn move_row(&self, from: usize, to: usize) {
        if from == to || to >= self.rows.borrow().len() {
            return;
        }
        let list = *self.list.borrow();
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let _ = core.services.playlists.move_item(list, from, to);
            post(&dispatcher, move || {
                if let Some(page) = open_page() {
                    page.load_rows();
                }
            });
        });
    }

    fn drop_row(&self, at: usize) {
        let list = *self.list.borrow();
        let Some(row) = self.rows.borrow().get(at).cloned() else { return };
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let said = match core.services.playlists.remove(list, &row.bvid) {
                Ok(()) => "已移出".to_owned(),
                Err(problem) => format!("移出失败：{problem}"),
            };
            post(&dispatcher, move || {
                toast(&said);
                if let Some(page) = open_page() {
                    page.load_rows();
                }
            });
        });
    }
}

fn pick_list(list: i64) {
    let Some(page) = open_page() else { return };
    *page.list.borrow_mut() = list;
    let _ = page.draw_chrome();
    page.load_rows();
}

fn set_mode(mode: PlaylistMode) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let _ = core.services.playlists.set_mode(mode);
        post(&dispatcher, move || {
            if let Some(page) = open_page() {
                *page.mode.borrow_mut() = mode;
                let _ = page.draw_chrome();
            }
        });
    });
}

fn make_list() {
    ask_name("新列表", "", |name| {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let made = core.services.playlists.create(&name).map_err(|e| e.to_string());
            post(&dispatcher, move || match made {
                Ok(id) => {
                    if let Some(page) = open_page() {
                        *page.list.borrow_mut() = id;
                        page.load();
                    }
                }
                Err(problem) => toast(&format!("新建失败：{problem}")),
            });
        });
    });
}

fn rename_list() {
    let Some(page) = open_page() else { return };
    let current = *page.list.borrow();
    let Some(list) = page.lists.borrow().iter().find(|l| l.id == current).cloned() else { return };
    ask_name("改名为", &list.name, move |name| {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let said = match core.services.playlists.rename(current, &name) {
                Ok(()) => "已改名".to_owned(),
                Err(problem) => format!("改名失败：{problem}"),
            };
            post(&dispatcher, move || {
                toast(&said);
                if let Some(page) = open_page() {
                    page.load();
                }
            });
        });
    });
}

fn delete_list() {
    let Some(page) = open_page() else { return };
    let current = *page.list.borrow();
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let said = match core.services.playlists.delete(current) {
            Ok(()) => "已删除".to_owned(),
            Err(problem) => format!("删除失败：{problem}"),
        };
        post(&dispatcher, move || {
            toast(&said);
            if let Some(page) = open_page() {
                page.load();
            }
        });
    });
}

fn play_all() {
    let Some(page) = open_page() else { return };
    if page.rows.borrow().is_empty() {
        toast("这个列表还是空的");
        return;
    }
    let list = *page.list.borrow();
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        let first = core.services.play_queue.first(list).map_err(|e| e.to_string());
        post(&dispatcher, move || match first {
            Ok(Some(card)) => {
                core.services.play_queue.play(list);
                let _ = crate::ui::player::open(false, &card.bvid, &card.title, &card.owner.name);
            }
            Ok(None) => toast("这个列表还是空的"),
            Err(problem) => toast(&format!("播放失败：{problem}")),
        });
    });
}

/// Asks for a list name; `done` runs with it once confirmed.
fn ask_name(title: &str, current: &str, done: impl Fn(String) + 'static) {
    let dialog = match load(&format!(
        r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
              xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
          <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="360" Padding="18"
                Background="{{ThemeResource ApplicationPageBackgroundThemeBrush}}">
            <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
            <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
              <RowDefinition Height="Auto" /></Grid.RowDefinitions>
            <TextBlock Text="{title}" FontSize="18" FontWeight="Light" />
            <TextBox x:Name="Name" Grid.Row="1" Margin="0,10,0,0" MaxLength="{max}" />
            <StackPanel Grid.Row="2" Orientation="Horizontal" HorizontalAlignment="Right" Margin="0,12,0,0">
              <Button x:Name="Cancel" Content="取消" Margin="0,0,8,0" />
              <Button x:Name="Save" Content="确定" Background="#0050EF" Foreground="White" BorderThickness="0" />
            </StackPanel>
          </Grid>
        </Grid>"##,
        max = MAX_PLAYLIST_NAME
    )) {
        Ok(dialog) => dialog,
        Err(problem) => {
            toast(&format!("打不开：{problem}"));
            return;
        }
    };
    let _ = named::<ITextBox>(&dialog, "Name").and_then(|box_| box_.SetText(current));
    let typing = dialog.clone();
    on_click(&dialog, "Save", move || {
        let text = named::<ITextBox>(&typing, "Name").ok().and_then(|b| b.Text().ok()).map(|t| t.to_string());
        let name = text.unwrap_or_default().trim().to_owned();
        if name.is_empty() {
            toast("名字不能是空的");
            return;
        }
        let _ = popup::hide();
        done(name);
    })
    .ok();
    on_click(&dialog, "Cancel", || {
        let _ = popup::hide();
    })
    .ok();
    if let Err(problem) = popup::show(&dialog) {
        toast(&format!("打不开：{problem}"));
    }
}
