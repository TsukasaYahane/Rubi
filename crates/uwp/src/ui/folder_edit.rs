//! The sheet that makes, edits or deletes a 收藏夹.

use std::cell::Cell;
use std::rc::Rc;

use rubi_model::FavFolder;
use windows_core::{Interface, Result};

use crate::backend::Backend;
use crate::ui::shell::shell;
use crate::ui::{load, named, on_click, post, set_text};
use crate::xaml::{
    IContentControl, IFrameworkElement, ITextBox, IToggleButton, IUIElement, PropertyValue, UIElement, Visibility,
};

const SHEET: &str = r##"
<Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
        BorderBrush="#0050EF" BorderThickness="1" Padding="16,12" Margin="16,8" MinWidth="320" MaxWidth="440"
        HorizontalAlignment="Left" VerticalAlignment="Top"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
  <StackPanel>
    <StackPanel.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></StackPanel.Resources>
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
    <Grid Margin="0,12,0,0">
      <Grid.ColumnDefinitions>
        <ColumnDefinition Width="Auto" /><ColumnDefinition Width="Auto" /><ColumnDefinition Width="*" />
        <ColumnDefinition Width="Auto" />
      </Grid.ColumnDefinitions>
      <Button x:Name="SaveFolder" Content="保存" Background="#0050EF" Foreground="White" BorderThickness="0"
              Padding="18,5" />
      <Button x:Name="CancelFolder" Grid.Column="1" Content="取消" Margin="8,0,0,0" Padding="14,5" />
      <Button x:Name="DeleteFolder" Grid.Column="3" Content="删除收藏夹" Visibility="Collapsed" Background="Transparent"
              BorderThickness="0" Padding="8,5" Foreground="#E81123" />
    </Grid>
  </StackPanel>
</Border>"##;

/// What the sheet asks B站 to do.
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

/// How the sheet was left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Saved,
    Deleted,
    Cancelled,
}

struct Sheet {
    root: IFrameworkElement,
    /// The folder edited; None for a new one.
    id: Option<u64>,
    busy: Cell<bool>,
    /// 删除 was pressed once.
    confirming: Cell<bool>,
    done: Box<dyn Fn(Outcome)>,
}

/// Opens the sheet in `host`: a new folder for None. `deleting` opens it ready to delete.
/// `done` hears how it ended, after the sheet has gone.
pub fn open(
    host: &IContentControl,
    folder: Option<&FavFolder>,
    deleting: bool,
    done: impl Fn(Outcome) + 'static,
) -> Result<()> {
    let root = load(SHEET)?;
    set_text(&root, "Heading", if folder.is_some() { "编辑收藏夹" } else { "新建收藏夹" })?;
    let public = folder.is_none_or(|f| !f.private);
    named::<IToggleButton>(&root, "FolderPublic")?.SetIsChecked(Some(public))?;
    if let Some(folder) = folder {
        named::<ITextBox>(&root, "FolderName")?.SetText(&folder.title)?;
        if !folder.default {
            named::<IUIElement>(&root, "DeleteFolder")?.SetVisibility(Visibility::Visible)?;
        }
    }
    let host_done = host.clone();
    let sheet = Rc::new(Sheet {
        root: root.clone(),
        id: folder.map(|f| f.id),
        busy: Cell::new(false),
        confirming: Cell::new(false),
        done: Box::new(move |outcome| {
            let _ = close(&host_done);
            done(outcome);
        }),
    });
    let saving = Rc::downgrade(&sheet);
    on_click(&root, "SaveFolder", move || {
        if let Some(sheet) = saving.upgrade() {
            let _ = sheet.save();
        }
    })?;
    let leaving = Rc::downgrade(&sheet);
    on_click(&root, "CancelFolder", move || {
        if let Some(sheet) = leaving.upgrade() {
            (sheet.done)(Outcome::Cancelled);
        }
    })?;
    let deleting_now = Rc::downgrade(&sheet);
    on_click(&root, "DeleteFolder", move || {
        if let Some(sheet) = deleting_now.upgrade() {
            let _ = sheet.delete();
        }
    })?;
    host.SetContent(&root.cast::<UIElement>()?)?;
    host.cast::<IUIElement>()?.SetVisibility(Visibility::Visible)?;
    // The sheet lives while it is on show: its buttons hold it through this cell.
    SHOWN.with_borrow_mut(|s| *s = Some(sheet.clone()));
    if let Some(id) = sheet.id {
        sheet.load_intro(id);
    }
    if deleting && folder.is_some_and(|f| !f.default) {
        sheet.delete()?;
    }
    Ok(())
}

thread_local! {
    static SHOWN: std::cell::RefCell<Option<Rc<Sheet>>> = const { std::cell::RefCell::new(None) };
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

    /// The first press asks again; the second deletes.
    fn delete(&self) -> Result<()> {
        let Some(id) = self.id else { return Ok(()) };
        if !self.confirming.replace(true) {
            named::<IContentControl>(&self.root, "DeleteFolder")?
                .SetContent(&PropertyValue::CreateString("再按一次确认删除")?)?;
            self.say("删除后收藏夹里的视频一并移除，不能恢复");
            return Ok(());
        }
        self.run("正在删除…", Outcome::Deleted, Change::Delete(id));
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
