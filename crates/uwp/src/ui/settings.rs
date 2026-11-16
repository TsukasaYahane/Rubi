//! The settings of every category but 账号: rows built from what is set now, saved as they change.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use rubi_core::MAX_FREE_QN;
use serde_json::Value;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::backend::Backend;
use crate::state::settings::{
    Category, DANMAKU, FACTORS, FILTERS, Prefs, QUALITIES, Rule, WEIGHT_MAX, escape, keyword, keywords_patch,
    quality_patch, rule_patch, weight_patch,
};
use crate::ui::shell::shell;
use crate::ui::{keep, load, me, named, on_click, post, set_text};
use crate::xaml::{
    Color, IButtonBase, IControl, IFrameworkElement, IRangeBase, IRangeBaseValueChangedEventArgs, ITextBox,
    IToggleButton, RangeBaseValueChangedEventArgs, RoutedEventArgs, SolidColorBrush,
};

/// How long a slider must rest before its value is saved.
const SLIDER_SETTLE: Duration = Duration::from_millis(300);
const COBALT: Color = Color { a: 255, r: 0, g: 0x50, b: 0xEF };
const DIM: Color = Color { a: 255, r: 0x76, g: 0x76, b: 0x76 };

/// Square on purpose: the system switch is a pill.
const STYLES: &str = r##"
<StackPanel.Resources>
  <Style x:Key="Square" TargetType="ToggleButton">
    <Setter Property="UseSystemFocusVisuals" Value="True" />
    <Setter Property="Template"><Setter.Value><ControlTemplate TargetType="ToggleButton">
      <Grid x:Name="Track" Width="42" Height="20" BorderThickness="2" Background="Transparent"
            BorderBrush="{ThemeResource SystemControlForegroundBaseHighBrush}">
        <VisualStateManager.VisualStateGroups>
          <VisualStateGroup x:Name="CommonStates">
            <VisualState x:Name="Normal" /><VisualState x:Name="PointerOver" /><VisualState x:Name="Pressed" />
            <VisualState x:Name="Disabled" />
            <VisualState x:Name="Checked"><VisualState.Setters><!--on--></VisualState.Setters></VisualState>
            <VisualState x:Name="CheckedPointerOver"><VisualState.Setters><!--on--></VisualState.Setters></VisualState>
            <VisualState x:Name="CheckedPressed"><VisualState.Setters><!--on--></VisualState.Setters></VisualState>
          </VisualStateGroup>
        </VisualStateManager.VisualStateGroups>
        <Border x:Name="Knob" Width="10" Height="10" Margin="3,0" HorizontalAlignment="Left" VerticalAlignment="Center"
                Background="{ThemeResource SystemControlForegroundBaseHighBrush}" />
      </Grid>
    </ControlTemplate></Setter.Value></Setter>
  </Style>
  <Style x:Key="Pick" TargetType="Button">
    <Setter Property="Background" Value="Transparent" />
    <Setter Property="BorderThickness" Value="0" />
    <Setter Property="Padding" Value="8,4" />
  </Style>
</StackPanel.Resources>"##;

const TURNED_ON: &str = r##"<Setter Target="Track.Background" Value="#0050EF" />
<Setter Target="Track.BorderBrush" Value="#0050EF" /><Setter Target="Knob.Background" Value="White" />
<Setter Target="Knob.HorizontalAlignment" Value="Right" />"##;

fn page(category: Category, rows: &str) -> String {
    format!(
        r##"<StackPanel xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                 xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">{styles}
          <TextBlock Text="{title}" FontSize="20" FontWeight="Light" Margin="16,12,16,4" />
          <TextBlock x:Name="Note" FontSize="12" Margin="16,0" TextWrapping="Wrap" Foreground="#C42B1C" />
          {rows}</StackPanel>"##,
        styles = STYLES.replace("<!--on-->", TURNED_ON),
        title = category.label(),
    )
}

/// A row with its label on the left and `control` on the right.
fn row(title: &str, note: &str, control: &str) -> String {
    let note = if note.is_empty() {
        String::new()
    } else {
        format!(
            r##"<TextBlock Text="{}" FontSize="12" TextWrapping="Wrap"
                 Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />"##,
            escape(note)
        )
    };
    format!(
        r##"<Border BorderThickness="0,0,0,1" Padding="16,10"
                 BorderBrush="{{ThemeResource SystemControlForegroundBaseLowBrush}}">
          <Grid><Grid.ColumnDefinitions><ColumnDefinition Width="*" /><ColumnDefinition Width="Auto" /></Grid.ColumnDefinitions>
            <StackPanel VerticalAlignment="Center" Margin="0,0,12,0">
              <TextBlock Text="{}" FontSize="14" TextWrapping="Wrap" />{note}</StackPanel>
            <Grid Grid.Column="1" VerticalAlignment="Center">{control}</Grid>
          </Grid></Border>"##,
        escape(title)
    )
}

fn toggle(name: &str, title: &str, note: &str, on: bool) -> String {
    let on = if on { "True" } else { "False" };
    row(
        title,
        note,
        &format!(r##"<ToggleButton x:Name="{name}" Style="{{StaticResource Square}}" IsChecked="{on}" />"##),
    )
}

fn section(title: &str) -> String {
    format!(r##"<TextBlock Text="{title}" FontSize="16" FontWeight="Light" Margin="16,16,16,4" />"##)
}

fn picks(prefix: &str, labels: &[&str]) -> String {
    let buttons: String = labels
        .iter()
        .enumerate()
        .map(|(at, label)| {
            format!(r##"<Button x:Name="{prefix}{at}" Content="{label}" Style="{{StaticResource Pick}}" />"##)
        })
        .collect();
    format!(r##"<StackPanel Orientation="Horizontal">{buttons}</StackPanel>"##)
}

/// Colours the picked one of the buttons `prefix0`, `prefix1`…
fn mark(root: &IFrameworkElement, prefix: &str, count: usize, picked: usize) -> Result<()> {
    for at in 0..count {
        let color = if at == picked { COBALT } else { DIM };
        named::<IControl>(root, &format!("{prefix}{at}"))?
            .SetForeground(&SolidColorBrush::CreateInstanceWithColor(color)?)?;
    }
    Ok(())
}

/// Calls `act` with the new state of the toggle named `name`.
fn on_toggle(root: &IFrameworkElement, name: &str, act: impl Fn(bool) + 'static) -> Result<()> {
    keep(named::<IButtonBase>(root, name)?.Click(move |sender: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
        let on = sender.ok().and_then(|s| s.cast::<IToggleButton>()?.IsChecked());
        if let Ok(on) = on {
            act(on);
        }
    })?);
    Ok(())
}

/// Saves `patch`, off the UI thread; a refusal shows on the page.
fn save(patch: Value) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get().cloned() else { return };
    let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
    backend.spawn(async move {
        if let Err(problem) = core.services.preferences.patch_settings(&patch.to_string()) {
            let text = format!("未保存：{problem}");
            post(&dispatcher, move || me::note(&text));
        }
    });
}

fn change_prefs(change: impl Fn(&mut Prefs)) {
    if let Some(shell) = shell() {
        let mut prefs = shell.prefs();
        change(&mut prefs);
        let _ = shell.set_prefs(prefs);
    }
}

/// The rows of `category`, as saved now.
pub fn build(category: Category, backend: &Arc<Backend>) -> Result<IFrameworkElement> {
    let settings = serde_json::to_value(backend.services.preferences.settings()).unwrap_or_default();
    match category {
        Category::Look => look(),
        Category::Play => play(&settings),
        Category::Filter => filter(&settings),
        Category::Recommend => recommend(&settings, backend),
        Category::Account | Category::About => about(),
    }
}

fn look() -> Result<IFrameworkElement> {
    const LABELS: [&str; 2] = ["图形", "文字"];
    const SKINS: [&str; 3] = ["跟随系统", "浅色", "深色"];
    let rows = [
        row("界面风格", "图形用线条图标配文字；文字只用文字", &picks("Style", &LABELS)),
        row("主题", "跟随系统时按 Windows 的设置；手动选了就一直用它", &picks("Skin", &SKINS)),
    ]
    .concat();
    let root = load(&page(Category::Look, &rows))?;
    let words = shell().is_some_and(|s| s.prefs().words);
    mark(&root, "Style", LABELS.len(), usize::from(words))?;
    for (at, _) in LABELS.iter().enumerate() {
        let shown = root.clone();
        on_click(&root, &format!("Style{at}"), move || {
            change_prefs(|p| p.words = at == 1);
            let _ = mark(&shown, "Style", LABELS.len(), at);
        })?;
    }
    let skin = shell().map(|s| s.prefs().theme).unwrap_or_default();
    mark(&root, "Skin", SKINS.len(), skin.index())?;
    for (at, _) in SKINS.iter().enumerate() {
        let shown = root.clone();
        on_click(&root, &format!("Skin{at}"), move || {
            change_prefs(|p| p.theme = crate::state::settings::Skin::at(at));
            let _ = mark(&shown, "Skin", SKINS.len(), at);
        })?;
    }
    Ok(root)
}

fn play(settings: &Value) -> Result<IFrameworkElement> {
    let prefs = shell().map(|s| s.prefs()).unwrap_or_default();
    let labels = QUALITIES.map(|(_, label)| label);
    let quality = settings["preferred_quality"].as_u64().unwrap_or(u64::from(MAX_FREE_QN));
    let picked = QUALITIES.iter().position(|(qn, _)| u64::from(*qn) == quality).unwrap_or(0);
    let rows = [
        row("默认清晰度", "打开视频时使用；片源没有时自动降低", &picks("Quality", &labels)),
        toggle("Blur", "窗口失焦时暂停", "切到其他窗口时暂停视频，切回来继续", prefs.pause_on_blur),
        toggle("Keys", "键盘快捷键", "空格暂停，左右快进，F 全屏，D 弹幕", prefs.shortcuts),
        row("缓存位置", "离线缓存下载完成后搬到这里；改完要重启一次", &place_row("CacheWhere", "PickCache")),
        row("导出位置", "缓存完成后导出到这里；没选就放在缓存位置下的视频 / 音频子目录", &place_row("ExportWhere", "PickExport")),
        row(
            "文件系统权限",
            "允许后 Rubi 才能读写你选的目录；不给就只能用应用自己的目录，缓存会留一份在这里",
            &button_row("AccessWhere", "PickAccess"),
        ),
    ]
    .concat();
    let root = load(&page(Category::Play, &rows))?;
    mark(&root, "Quality", labels.len(), picked)?;
    let _ = set_text(&root, "CacheWhere", &place_text(Place::Cache));
    let _ = set_text(&root, "ExportWhere", &place_text(Place::Exports));
    let _ = set_text(
        &root,
        "AccessWhere",
        if crate::state::access::granted() { "已允许" } else { "未允许" },
    );
    on_click(&root, "PickCache", || pick_place(Place::Cache, "CacheWhere"))?;
    on_click(&root, "PickExport", || pick_place(Place::Exports, "ExportWhere"))?;
    on_click(&root, "PickAccess", crate::state::access::open_settings)?;
    for (at, (qn, _)) in QUALITIES.iter().enumerate() {
        let (shown, qn) = (root.clone(), *qn);
        on_click(&root, &format!("Quality{at}"), move || {
            save(quality_patch(qn));
            let _ = mark(&shown, "Quality", QUALITIES.len(), at);
        })?;
    }
    on_toggle(&root, "Blur", |on| change_prefs(|p| p.pause_on_blur = on))?;
    on_toggle(&root, "Keys", |on| change_prefs(|p| p.shortcuts = on))?;
    Ok(root)
}

/// A row showing where something goes, with the button that changes it.
fn place_row(text_name: &str, button_name: &str) -> String {
    format!(
        r##"<StackPanel Orientation="Horizontal" VerticalAlignment="Center">
        <TextBlock x:Name="{text_name}" FontSize="12" VerticalAlignment="Center" MaxWidth="300"
                   TextTrimming="CharacterEllipsis"
                   Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
        <Button x:Name="{button_name}" Content="选择" Margin="10,0,0,0" />
      </StackPanel>"##
    )
}

/// A line whose note says the state of something, with a button that opens where it is set.
fn button_row(text_name: &str, button_name: &str) -> String {
    format!(
        r##"<StackPanel Orientation="Horizontal" VerticalAlignment="Center">
        <TextBlock x:Name="{text_name}" FontSize="12" VerticalAlignment="Center" MaxWidth="300"
                   TextTrimming="CharacterEllipsis"
                   Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
        <Button x:Name="{button_name}" Content="打开设置" Margin="10,0,0,0" />
      </StackPanel>"##
    )
}

/// Which of the two places is meant.
#[derive(Debug, Clone, Copy)]
enum Place {
    /// Where 离线缓存 keeps its files.
    Cache,
    /// Where a finished copy is written out.
    Exports,
}

/// Where a place is, as things stand.
fn place_text(place: Place) -> String {
    let path = match place {
        Place::Cache => crate::state::exports::download_path(),
        Place::Exports => crate::state::exports::folder_path(),
    };
    path.unwrap_or_else(|| "未选".to_owned())
}

/// Asks for a place, and keeps it for the next run. The picker needs the UI thread to keep
/// running, so the answer comes back as an event: waiting on it there would stop the message
/// pump and the window would never be answered.
fn pick_place(place: Place, text_name: &'static str) {
    let asked = (|| {
        let picker = crate::xaml::FolderPicker::new()?;
        picker.FileTypeFilter()?.Append(&windows_core::HSTRING::from("*"))?;
        picker.PickSingleFolderAsync()
    })();
    let operation = match asked {
        Ok(operation) => operation,
        Err(problem) => {
            crate::ui::shell::toast(&format!("选不了：{problem}"));
            return;
        }
    };
    // What the callback touches it reaches through the thread that is waiting for it: the text
    // and the toast belong on the UI thread whichever thread Windows answers on.
    let dispatcher = shell().map(|shell| shell.dispatcher.clone());
    let handler = windows_future::AsyncOperationCompletedHandler::new(
        move |operation: windows_core::Ref<windows_future::IAsyncOperation<crate::xaml::StorageFolder>>,
              _: windows_future::AsyncStatus| {
            let Ok(operation) = operation.ok() else { return Ok(()) };
            let Ok(folder) = operation.GetResults() else { return Ok(()) };
            let kept = match place {
                Place::Cache => crate::state::exports::keep_download_folder(&folder),
                Place::Exports => crate::state::exports::keep_folder(&folder),
            };
            let Some(dispatcher) = &dispatcher else { return Ok(()) };
            match kept {
                Ok(()) => crate::ui::post(dispatcher, move || {
                    if let Some(shell) = shell()
                        && let Ok(shown) = shell.current_page()
                    {
                        let _ = set_text(&shown, text_name, &place_text(place));
                    }
                    crate::ui::shell::toast(match place {
                        // The core takes its folder when it opens: this one waits for a restart.
                        Place::Cache => "缓存位置已改，重启 Rubi 后生效",
                        Place::Exports => "导出位置已改",
                    });
                }),
                Err(problem) => {
                    let said = problem.message().to_string();
                    crate::ui::post(dispatcher, move || crate::ui::shell::toast(&format!("记不住这个位置：{said}")));
                }
            }
            Ok(())
        },
    );
    if let Err(problem) = operation.SetCompleted(&handler) {
        crate::ui::shell::toast(&format!("选不了：{problem}"));
    }
}

fn rule_rows(rules: &[Rule], toggles: &Value) -> String {
    rules.iter().map(|r| toggle(r.key, r.title, r.note, toggles[r.key].as_bool().unwrap_or(false))).collect()
}

fn filter(settings: &Value) -> Result<IFrameworkElement> {
    let toggles = &settings["rules"]["toggles"];
    let keywords = settings["rules"]["keywords"].as_array().cloned().unwrap_or_default();
    let listed: String = keywords
        .iter()
        .enumerate()
        .map(|(at, spec)| {
            let pattern = spec["pattern"].as_str().unwrap_or_default();
            row(
                pattern,
                "",
                &format!(r##"<Button x:Name="Drop{at}" Content="移除" Style="{{StaticResource Pick}}" />"##),
            )
        })
        .collect();
    let rows = [
        section("内容过滤"),
        rule_rows(&FILTERS, toggles),
        section("弹幕"),
        rule_rows(&DANMAKU, toggles),
        section("屏蔽关键词"),
        listed,
        row(
            "添加关键词",
            "作用于标题、标签和简介",
            r##"<StackPanel Orientation="Horizontal"><TextBox x:Name="Keyword" Width="150" MaxLength="50" />
                <Button x:Name="AddKeyword" Content="添加" Style="{StaticResource Pick}" /></StackPanel>"##,
        ),
    ]
    .concat();
    let root = load(&page(Category::Filter, &rows))?;
    for rule in FILTERS.iter().chain(&DANMAKU) {
        let key = rule.key;
        on_toggle(&root, key, move |on| save(rule_patch(key, on)))?;
    }
    let keywords = Rc::new(RefCell::new(keywords));
    for at in 0..keywords.borrow().len() {
        let list = keywords.clone();
        on_click(&root, &format!("Drop{at}"), move || {
            list.borrow_mut().remove(at);
            save_keywords(&list.borrow());
        })?;
    }
    let list = keywords.clone();
    let field = named::<ITextBox>(&root, "Keyword")?;
    on_click(&root, "AddKeyword", move || {
        let typed = field.Text().map(|t| t.to_string()).unwrap_or_default();
        if !typed.trim().is_empty() {
            list.borrow_mut().push(keyword(typed.trim()));
            save_keywords(&list.borrow());
        }
    })?;
    Ok(root)
}

/// Saves the list, then shows it: which row is which has changed. The page must read what was
/// saved, so this one waits.
fn save_keywords(keywords: &[Value]) {
    let Some(shell) = shell() else { return };
    let Some(backend) = shell.backend.get() else { return };
    match backend.services.preferences.patch_settings(&keywords_patch(keywords).to_string()) {
        Ok(()) => me::refresh(),
        Err(problem) => me::note(&format!("未保存：{problem}")),
    }
}

fn recommend(settings: &Value, backend: &Arc<Backend>) -> Result<IFrameworkElement> {
    let weights = &settings["weights"];
    let value_of = |key: &str| weights[key].as_f64().unwrap_or(0.0);
    let rows: String = FACTORS
        .iter()
        .map(|(key, label)| {
            let value = value_of(key);
            format!(
                r##"<Grid Margin="16,4"><Grid.ColumnDefinitions><ColumnDefinition Width="84" /><ColumnDefinition Width="*" />
                     <ColumnDefinition Width="44" /></Grid.ColumnDefinitions>
                   <TextBlock Text="{label}" FontSize="14" VerticalAlignment="Center" />
                   <Slider x:Name="{key}" Grid.Column="1" Minimum="0" Maximum="{WEIGHT_MAX}" StepFrequency="0.01" Value="{value:.2}" />
                   <TextBlock x:Name="{key}Shown" Grid.Column="2" Text="{value:.2}" FontSize="12" VerticalAlignment="Center"
                              HorizontalAlignment="Right" /></Grid>"##
            )
        })
        .collect();
    let rows = format!(
        r##"<TextBlock Text="调整后下一次刷新首页即生效" FontSize="12" Margin="16,0,16,8"
             Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />{rows}"##
    );
    let root = load(&page(Category::Recommend, &rows))?;
    for (key, _) in FACTORS {
        on_weight(&root, key, backend)?;
    }
    Ok(root)
}

/// Shows a slider's value as it moves, and saves it once it rests.
fn on_weight(root: &IFrameworkElement, key: &'static str, backend: &Arc<Backend>) -> Result<()> {
    let shown = root.clone();
    let turns = Arc::new(AtomicU64::new(0));
    let core = Arc::downgrade(backend);
    keep(named::<IRangeBase>(root, key)?.ValueChanged(
        move |_: Ref<IInspectable>, args: Ref<RangeBaseValueChangedEventArgs>| {
            let value = args.ok().and_then(|a| a.cast::<IRangeBaseValueChangedEventArgs>()?.NewValue());
            let (Ok(value), Some(core)) = (value, core.upgrade()) else { return };
            let _ = set_text(&shown, &format!("{key}Shown"), &format!("{value:.2}"));
            let turn = turns.fetch_add(1, Ordering::Relaxed) + 1;
            let (turns, saving) = (turns.clone(), core.clone());
            core.spawn(async move {
                tokio::time::sleep(SLIDER_SETTLE).await;
                if turns.load(Ordering::Relaxed) == turn {
                    let _ = saving.services.preferences.patch_settings(&weight_patch(key, value).to_string());
                }
            });
        },
    )?);
    Ok(())
}

fn about() -> Result<IFrameworkElement> {
    let rows = [
        row("版本", env!("CARGO_PKG_VERSION"), ""),
        row("Rubi", "第三方哔哩哔哩客户端：过滤广告和推广，在本机计算推荐。与哔哩哔哩官方无关。", ""),
    ]
    .concat();
    load(&page(Category::About, &rows))
}
