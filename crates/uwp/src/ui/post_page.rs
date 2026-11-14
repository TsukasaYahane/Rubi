//! A dynamic on its own: the whole text, its pictures and its quote, with the way into its comments.
//! Drawn over the window like the other sheets.

use std::cell::RefCell;
use std::rc::Rc;

use windows_collections::IVector;
use windows_core::{Interface, Result};

use crate::state::follow::Post;
use crate::ui::emote_text;
use crate::ui::popup;
use crate::ui::rows::Covers;
use crate::ui::shell::toast;
use crate::ui::{load, named, on_click, set_text};
use crate::xaml::{IBorder, IImage, IPanel, IRichTextBlock, IUIElement, UIElement, Visibility};

const PICTURES: usize = 3;
const PICTURE_PX: i32 = 240;

const LAYOUT: &str = r##"<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Background="#99000000">
  <Grid HorizontalAlignment="Center" VerticalAlignment="Center" Width="620" MaxHeight="680" Padding="18"
        Background="{ThemeResource ApplicationPageBackgroundThemeBrush}">
    <Grid.Resources><CornerRadius x:Key="ControlCornerRadius">0</CornerRadius></Grid.Resources>
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="*" />
      <RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <StackPanel Orientation="Horizontal">
      <Border Width="36" Height="36" Background="#222"><Image x:Name="Face" Stretch="UniformToFill" /></Border>
      <StackPanel Margin="10,0,0,0" VerticalAlignment="Center">
        <TextBlock x:Name="Name" FontSize="15" />
        <TextBlock x:Name="Line" FontSize="11"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
      </StackPanel>
    </StackPanel>
    <ScrollViewer Grid.Row="2" Margin="0,12,0,0">
      <StackPanel>
        <RichTextBlock x:Name="Words" FontSize="14" TextWrapping="Wrap" IsTextSelectionEnabled="True" />
        <Border x:Name="QuoteBox" BorderThickness="3,0,0,0" BorderBrush="#3A3A3A" Padding="10,4,0,4"
                Margin="0,10,0,0" Visibility="Collapsed">
          <RichTextBlock x:Name="Quote" FontSize="13" TextWrapping="Wrap"
                         Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </Border>
        <StackPanel x:Name="Pictures" Orientation="Horizontal" Margin="0,10,0,0" Visibility="Collapsed" />
      </StackPanel>
    </ScrollViewer>
    <Border x:Name="CommentsBox" Grid.Row="3" Height="300" Margin="0,12,0,0" Visibility="Collapsed"
            BorderThickness="0,1,0,0" BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
      <ContentControl x:Name="CommentsHost" IsTabStop="False" VerticalAlignment="Stretch" />
    </Border>
    <StackPanel Grid.Row="4" Orientation="Horizontal" HorizontalAlignment="Right" Margin="0,14,0,0">
      <Button x:Name="Close" Content="关闭" Background="#0050EF" Foreground="White" BorderThickness="0" />
    </StackPanel>
  </Grid>
</Grid>"##;

thread_local! {
    /// The comment list of the post on show.
    static COMMENTS: RefCell<Option<Rc<crate::ui::comment_list::CommentList>>> = const { RefCell::new(None) };
}

struct Sheet {
    root: crate::xaml::IFrameworkElement,
    covers: Covers,
    post: Post,
}

thread_local! {
    static SHEET: RefCell<Option<Rc<Sheet>>> = const { RefCell::new(None) };
}

fn sheet() -> Option<Rc<Sheet>> {
    SHEET.with_borrow(Clone::clone)
}

/// Shows `post` on its own.
pub fn open(post: &Post) -> Result<()> {
    let root = load(LAYOUT)?;
    let opened = Rc::new(Sheet { root: root.clone(), covers: Covers::default(), post: post.clone() });
    SHEET.with_borrow_mut(|slot| *slot = Some(opened.clone()));
    opened.fill()?;
    on_click(&root, "Close", || {
        let _ = popup::hide();
    })?;
    opened.show_comments()?;
    popup::show(&root)
}

impl Sheet {
    fn fill(&self) -> Result<()> {
        let post = &self.post;
        set_text(&self.root, "Name", &post.name)?;
        set_text(&self.root, "Line", &post.line)?;
        self.covers.show(&named::<IImage>(&self.root, "Face")?, &post.face, 64)?;

        let words = named::<IRichTextBlock>(&self.root, "Words")?;
        emote_text::show(&words, &post.text, &post.emotes, 18.0)?;
        named::<IUIElement>(&self.root, "Words")?.SetVisibility(if post.text.is_empty() {
            Visibility::Collapsed
        } else {
            Visibility::Visible
        })?;

        let quoted = !post.quote.is_empty();
        named::<IUIElement>(&self.root, "QuoteBox")?.SetVisibility(if quoted {
            Visibility::Visible
        } else {
            Visibility::Collapsed
        })?;
        if quoted {
            let inner = named::<IBorder>(&self.root, "QuoteBox")?.Child()?.cast::<IRichTextBlock>()?;
            emote_text::show(&inner, &post.quote, &post.emotes, 16.0)?;
        }

        self.show_pictures()
    }

    /// Puts the post's comment area under its words.
    fn show_comments(&self) -> Result<()> {
        // TEMP-PROBE: the popup covers the toast layer, so the line itself says it.
        let said = match self.post.comments {
            Some((kind, oid)) => format!("评论区 {kind}/{oid} · {} 条", self.post.comment_count),
            None => "这条动态没有评论区".to_owned(),
        };
        let _ = set_text(&self.root, "Line", &said);
        let Some((kind, oid)) = self.post.comments else { return Ok(()) };
        let list = crate::ui::comment_list::CommentList::new()?;
        named::<crate::xaml::IContentControl>(&self.root, "CommentsHost")?
            .SetContent(&list.element().cast::<crate::xaml::UIElement>()?)?;
        named::<IUIElement>(&self.root, "CommentsBox")?.SetVisibility(Visibility::Visible)?;
        list.show(rubi_core::CommentArea { kind, oid }, self.post.comment_count)?;
        COMMENTS.with_borrow_mut(|slot| *slot = Some(list));
        Ok(())
    }

    fn show_pictures(&self) -> Result<()> {
        let shown = self.post.pictures.len().min(PICTURES);
        let host = named::<IPanel>(&self.root, "Pictures")?;
        named::<IUIElement>(&self.root, "Pictures")?.SetVisibility(if shown == 0 {
            Visibility::Collapsed
        } else {
            Visibility::Visible
        })?;
        let children = host.Children()?.cast::<IVector<UIElement>>()?;
        children.Clear()?;
        for at in 0..shown {
            let Some(url) = self.post.pictures.get(at) else { break };
            let frame = load(
                r##"<Border xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Width="180" Height="180" Margin="0,0,8,0"
                      Background="#1c1c1c"><Image Stretch="UniformToFill" /></Border>"##,
            )?;
            let image = frame.cast::<IBorder>()?.Child()?.cast::<IImage>()?;
            self.covers.show(&image, url, PICTURE_PX)?;
            children.Append(&frame.cast::<UIElement>()?)?;
        }
        if shown > 0 && self.post.pictures.len() > PICTURES {
            let more = format!("+{}", self.post.pictures.len() - PICTURES);
            let label = load(&format!(
                r##"<TextBlock xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
                      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Text="{more}" FontSize="12"
                      VerticalAlignment="Bottom" Foreground="#A6A6A6" />"##
            ))?;
            children.Append(&label.cast::<UIElement>()?)?;
        }
        Ok(())
    }
}
