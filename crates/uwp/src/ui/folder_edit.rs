//! The sheets that make or edit a 收藏夹, and the one that deletes it.

use std::cell::Cell;
use std::rc::Rc;

use rubi_model::FavFolder;
use windows_core::{Interface, Result};

use crate::backend::Backend;
use crate::ui::shell::shell;
use crate::ui::{load, named, on_click, post, set_text};
use crate::xaml::{IContentControl, IFrameworkElement, ITextBox, IToggleButton, IUIElement, UIElement, Visibility};

/// The frame both sheets share; `<!--brush-->` is its border, `<!--body-->` what it holds.
const FRAME: &str = r##"
<Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
        BorderBrush="<!--brush-->" BorderThickness="1" Padding="16,12" Margin="16,8" MinWidth="320" MaxWidth="440"
        HorizontalAlignment="Left" VerticalAlignment="Top"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
  <StackPanel>
    <StackPanel.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></StackPanel.Resources>
    <!--body-->
  </StackPanel>
</Border>"##;

const EDIT: &str = r##"
    <TextBlock x:Name="Heading" FontSize="20" FontWeight="Light" />
    <TextBlock Text="名称" FontSize="12" Margin="0,10,0,4"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    <TextBox x:Name="FolderName" MaxLength="20" PlaceholderText="1 到 20 个字" />
    <TextBlock Text="简介" FontSize="12" Margin="0,10,0,4"
               Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    <TextBox x:Name="FolderIntro" MaxLength="200" PlaceholderText="可不填" TextWrapping="Wrap" AcceptsReturn="True"
             MaxHeight="96" />
    <CheckBox x:Name="FolderPublic" Content="公开" Margin="0,10,0,0" />
    <TextBlock x:Name="SheetStatus" FontSize="12" Margin="0,6,0,0" TextWrapping="Wrap" Foreground="#E81123" />
    <StackPanel Orientation="Horizontal" Margin="0,12,0,0">
      <Button x:Name="SaveFolder" Content="保存" Background="#0050EF" Foreground="White" BorderThickness="0"
              Padding="18,5" />
      <Button x:Name="CancelFolder" Content="取消" Margin="8,0,0,0" Padding="14,5" />
    </StackPanel>"##;

const DELETE: &str = r##"
    <TextBlock Text="删除收藏夹" FontSize="20" FontWeight="Light" />
    <TextBlock x:Name="DeleteNote" FontSize="13" Margin="0,10,0,0" TextWrapping="Wrap" />
    <TextBlock x:Name="SheetStatus" FontSize="12" Margin="0,6,0,0" TextWrapping="Wrap" Foreground="#E81123" />
    <StackPanel Orientation="Horizontal" Margin="0,12,0,0">
      <Button x:Name="DeleteFolder" Content="删除" Background="#E81123" Foreground="White" BorderThickness="0"
              Padding="18,5" />
      <Button x:Name="CancelFolder" Content="取消" Margin="8,0,0,0" Padding="14,5" />
    </StackPanel>"##;

/// What a sheet asks B站 to do.
enum Change {
    Save { id: Option<u64>, title: String, intro: String, private: bool },
    Delete(u64),
}

impl Change {
    async fn apply(self, core: &Backend) -> rubi_core::Result<()> {
        let favorites = &core.services.favorites;
        match self {
            Self::Save { id: Some(id), title, intro, private } => {
                favorites.edit_folder(id, &title, &intro, private).await
            }
            Self::Save { id: None, title, intro, private } => {
                favorites.create_folder(&title, &intro, private).await.map(|_| ())
            }
            Self::Delete(id) => favorites.delete_folder(id).await,
        }
    }
}

/// How a sheet was left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Saved,
    Deleted,
    Cancelled,
}

struct Sheet {
    root: IFrameworkElement,
    /// The folder at hand; None for a new one.
    id: Option<u64>,
    busy: Cell<bool>,
    done: Box<dyn Fn(Outcome)>,
}

thread_local! {
    static SHOWN: std::cell::RefCell<Option<Rc<Sheet>>> = const { std::cell::RefCell::new(None) };
}

/// Opens the sheet that edits `folder`, or makes a new one for None. `done` hears how it ended,
/// after the sheet has gone.
pub fn edit(host: &IContentControl, folder: Option<&FavFolder>, done: impl Fn(Outcome) + 'static) -> Result<()> {
    let root = load(&FRAME.replace("<!--brush-->", "#0050EF").replace("<!--body-->", EDIT))?;
    set_text(&root, "Heading", if folder.is_some() { "编辑收藏夹" } else { "新建收藏夹" })?;
    named::<IToggleButton>(&root, "FolderPublic")?.SetIsChecked(Some(folder.is_none_or(|f| !f.private)))?;
    if let Some(folder) = folder {
        named::<ITextBox>(&root, "FolderName")?.SetText(&folder.title)?;
    }
    let sheet = show(host, root, folder.map(|f| f.id), done)?;
    let saving = Rc::downgrade(&sheet);
    on_click(&sheet.root, "SaveFolder", move || {
        if let Some(sheet) = saving.upgrade() {
            let _ = sheet.save();
        }
    })?;
    if let Some(id) = sheet.id {
        sheet.load_intro(id);
    }
    Ok(())
}

/// Opens the sheet that deletes `folder`, saying what goes with it.
pub fn delete(host: &IContentControl, folder: &FavFolder, done: impl Fn(Outcome) + 'static) -> Result<()> {
    let root = load(&FRAME.replace("<!--brush-->", "#E81123").replace("<!--body-->", DELETE))?;
    let note = format!("「{}」和其中的 {} 个视频会一并删除，不能恢复。", folder.title, folder.count);
    set_text(&root, "DeleteNote", &note)?;
    let sheet = show(host, root, Some(folder.id), done)?;
    let (deleting, id) = (Rc::downgrade(&sheet), folder.id);
    on_click(&sheet.root, "DeleteFolder", move || {
        if let Some(sheet) = deleting.upgrade() {
            sheet.run("正在删除…", Outcome::Deleted, Change::Delete(id));
        }
    })?;
    Ok(())
}

/// Puts a sheet up in `host` with its 取消 wired; it lives while it is on show.
fn show(
    host: &IContentControl,
    root: IFrameworkElement,
    id: Option<u64>,
    done: impl Fn(Outcome) + 'static,
) -> Result<Rc<Sheet>> {
    let host_done = host.clone();
    let sheet = Rc::new(Sheet {
        root: root.clone(),
        id,
        busy: Cell::new(false),
        done: Box::new(move |outcome| {
            let _ = close(&host_done);
            done(outcome);
        }),
    });
    let leaving = Rc::downgrade(&sheet);
    on_click(&root, "CancelFolder", move || {
        if let Some(sheet) = leaving.upgrade() {
            (sheet.done)(Outcome::Cancelled);
        }
    })?;
    host.SetContent(&root.cast::<UIElement>()?)?;
    host.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)?;
    SHOWN.with_borrow_mut(|s| *s = Some(sheet.clone()));
    Ok(sheet)
}

fn close(host: &IContentControl) -> Result<()> {
    SHOWN.with_borrow_mut(|s| *s = None);
    host.SetContent(None)?;
    host.cast::<IUIElement>()?.SetVisibility(Visibility::Collapsed)
}

impl Sheet {
    fn load_intro(&self, id: u64) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let Ok(intro) = core.services.favorites.folder_intro(id).await else { return };
            post(&dispatcher, move || {
                let Some(sheet) = SHOWN.with_borrow(Clone::clone).filter(|s| s.id == Some(id)) else { return };
                if let Ok(box_) = named::<ITextBox>(&sheet.root, "FolderIntro")
                    && box_.Text().is_ok_and(|t| t.is_empty())
                {
                    let _ = box_.SetText(&intro);
                }
            });
        });
    }

    fn say(&self, text: &str) {
        let _ = set_text(&self.root, "SheetStatus", text);
    }

    fn save(&self) -> Result<()> {
        let title = named::<ITextBox>(&self.root, "FolderName")?.Text()?.to_string();
        if title.trim().is_empty() {
            self.say("先写个名称");
            return Ok(());
        }
        let intro = named::<ITextBox>(&self.root, "FolderIntro")?.Text()?.to_string();
        let private = !named::<IToggleButton>(&self.root, "FolderPublic")?.IsChecked()?;
        self.run("正在保存…", Outcome::Saved, Change::Save { id: self.id, title, intro, private });
        Ok(())
    }

    /// Runs one change on B站; the sheet goes once it is done, and stays saying why if not.
    fn run(&self, working: &str, outcome: Outcome, change: Change) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.busy.replace(true) {
            return;
        }
        self.say(working);
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let done = change.apply(&core).await.map_err(|e| e.to_string());
            post(&dispatcher, move || {
                let Some(sheet) = SHOWN.with_borrow(Clone::clone) else { return };
                sheet.busy.set(false);
                match done {
                    Ok(()) => (sheet.done)(outcome),
                    Err(problem) => sheet.say(&format!("没有成功：{problem}")),
                }
            });
        });
    }
}
