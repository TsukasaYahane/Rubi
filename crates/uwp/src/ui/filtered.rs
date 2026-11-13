//! 被过滤内容: what the rules turned away lately, with the rule that did it. A record only: 误杀
//! marks a line, and the rule table itself is still edited in 设置.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use rubi_store::FilterLogEntry;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Result};

use crate::ui::card_menu;
use crate::ui::popup;
use crate::ui::rows::{add_rows, on_row};
use crate::ui::shell::{shell, toast};
use crate::ui::{load, named, on_click, post, set_text};
use crate::xaml::{IBorder, IFrameworkElement, IItemsControl, IPanel, ITextBlock, UIElement};

const LIMIT: usize = 200;

const LAYOUT: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="720" MaxHeight="620" Padding="18"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
      <RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <TextBlock Text="被过滤内容" FontSize="20" FontWeight="Light" />
    <TextBlock x:Name="DumpCount" Grid.Row="1" FontSize="12" Margin="0,4,0,10"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    <Grid Grid.Row="2" MinHeight="220">
      <ListView x:Name="Dump" SelectionMode="None" IsItemClickEnabled="False">
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
            <Grid.ColumnDefinitions><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" />
              <ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
            <Border Padding="6,1" BorderThickness="1" BorderBrush="#3A3A3A" VerticalAlignment="Top">
              <TextBlock FontSize="10" TextTrimming="CharacterEllipsis" />
            </Border>
            <StackPanel Grid.Column="1" Margin="10,0,10,0">
              <TextBlock FontSize="13" TextTrimming="CharacterEllipsis" />
              <TextBlock FontSize="11" Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
            </StackPanel>
            <TextBlock Grid.Column="2" FontSize="11" VerticalAlignment="Top"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          </Grid>
        </DataTemplate></ListView.ItemTemplate>
      </ListView>
      <TextBlock x:Name="DumpStatus" FontSize="12" Margin="0,8" VerticalAlignment="Top" IsHitTestVisible="False"
                 TextWrapping="Wrap" Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    </Grid>
    <StackPanel Grid.Row="3" Orientation="Horizontal" Margin="0,12,0,0" HorizontalAlignment="Right">
      <TextBlock FontSize="11" VerticalAlignment="Center" Margin="0,0,12,0" MaxWidth="420" TextWrapping="Wrap"
                 Text="右键一条可以标记「误杀」，只记一笔，不改规则。"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      <Button x:Name="DumpClose" Content="关闭" Background="#0050EF" Foreground="White" BorderThickness="0" />
    </StackPanel>
  </Grid>
</Grid>"##;

struct Dump {
    root: IFrameworkElement,
    entries: RefCell<Vec<FilterLogEntry>>,
    marked: RefCell<Vec<bool>>,
}

thread_local! {
    static SHEET: RefCell<Option<Rc<Dump>>> = const { RefCell::new(None) };
}

fn sheet() -> Option<Rc<Dump>> {
    SHEET.with_borrow(Clone::clone)
}

/// Opens 被过滤内容: what the rules turned away lately.
pub fn open() {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let root = match load(LAYOUT) {
        Ok(root) => root,
        Err(problem) => {
            toast(&format!("打不开：{problem}"));
            return;
        }
    };
    let _ = set_text(&root, "DumpStatus", "正在读取…");
    let opened = Rc::new(Dump { root: root.clone(), entries: RefCell::default(), marked: RefCell::default() });
    SHEET.with_borrow_mut(|s| *s = Some(opened.clone()));
    if let Err(problem) = wire(&opened) {
        toast(&format!("打不开：{problem}"));
        return;
    }
    if let Err(problem) = popup::show(&root) {
        toast(&format!("打不开：{problem}"));
        return;
    }
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        // Reading the log is a local database read: no waiting, no thread hop of its own.
        let found = core.services.home.filter_log(LIMIT).map_err(|e| e.to_string());
        post(&dispatcher, move || {
            if let Some(dump) = sheet() {
                let _ = dump.answered(found);
            }
        });
    });
}

fn wire(dump: &Rc<Dump>) -> Result<()> {
    let root = &dump.root;
    let list = named::<IFrameworkElement>(root, "Dump")?;
    on_row(&list, |at, parts| {
        let Some(dump) = sheet() else { return Ok(()) };
        let entries = dump.entries.borrow();
        let Some(entry) = entries.get(at) else { return Ok(()) };
        let marked = dump.marked.borrow().get(at).copied().unwrap_or(false);
        fill(&parts, entry, marked)
    })?;
    card_menu::attach_choices(&list, |at| {
        let dump = sheet()?;
        dump.entries.borrow().get(at)?;
        let marking: card_menu::Choice = (
            "标记为误杀".to_owned(),
            Rc::new(move || {
                if let Some(dump) = sheet() {
                    dump.mark(at);
                }
            }),
        );
        Some(vec![marking])
    })?;
    on_click(root, "DumpClose", || {
        let _ = popup::hide();
    })
}

fn fill(parts: &IVector<UIElement>, entry: &FilterLogEntry, marked: bool) -> Result<()> {
    let tag = parts.GetAt(0)?.cast::<IBorder>()?.Child()?.cast::<ITextBlock>()?;
    tag.SetText(&entry.category)?;
    let stack = parts.GetAt(1)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
    stack.GetAt(0)?.cast::<ITextBlock>()?.SetText(&entry.title)?;
    let rule = format!("规则 {} · {}", entry.rule_id, entry.reason);
    stack.GetAt(1)?.cast::<ITextBlock>()?.SetText(&rule)?;
    let when = if marked { "已记误杀".to_owned() } else { ago(entry.at) };
    parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&when)?;
    Ok(())
}

fn ago(at: i64) -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(at, |d| d.as_secs() as i64);
    let passed = (now - at).max(0);
    match passed {
        0..=59 => "刚刚".to_owned(),
        60..=3599 => format!("{} 分钟前", passed / 60),
        3600..=86_399 => format!("{} 小时前", passed / 3600),
        _ => format!("{} 天前", passed / 86_400),
    }
}

impl Dump {
    fn answered(&self, found: std::result::Result<Vec<FilterLogEntry>, String>) -> Result<()> {
        match found {
            Ok(entries) => {
                let count = entries.len();
                let videos = entries.iter().filter(|e| e.category.contains("视频")).count();
                *self.entries.borrow_mut() = entries;
                *self.marked.borrow_mut() = vec![false; count];
                let summary = format!("最近 {count} 条 · 其中视频 {videos} 条");
                set_text(&self.root, "DumpCount", &summary)?;
                set_text(&self.root, "DumpStatus", if count == 0 { "最近没有拦下什么" } else { "" })?;
                let list = named::<IFrameworkElement>(&self.root, "Dump")?;
                list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
                add_rows(&list, 0, count)
            }
            Err(problem) => set_text(&self.root, "DumpStatus", &format!("读取失败：{problem}")),
        }
    }

    /// Marks line `at` as 误杀 and says so.
    fn mark(&self, at: usize) {
        if let Some(slot) = self.marked.borrow_mut().get_mut(at) {
            *slot = true;
        }
        toast("已记为误杀，规则没有改");
    }
}
