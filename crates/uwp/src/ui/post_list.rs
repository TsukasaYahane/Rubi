//! A list of posts, every followed UP's or one UP's: 关注 and the UP page both show it.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rubi_core::CommentArea;
use windows_collections::IVector;
use windows_core::{IInspectable, Interface, Ref, Result};

use crate::state::follow::{PICTURES, Post};
use crate::ui::rows::{Covers, add_rows, on_row, on_row_click};
use crate::ui::shell::shell;
use crate::ui::{article, comment_list, feed, keep, load, post};
use crate::xaml::{
    IBorder, IButtonBase, IContentControl, IFrameworkElement, IImage, IItemsControl, IPanel, IPropertyValue,
    ITextBlock, IUIElement, Launcher, PropertyValue, RoutedEventArgs, UIElement, Uri, Visibility,
};

const FACE_DECODE_WIDTH: i32 = 64;
const COVER_DECODE_WIDTH: i32 = 320;
const PICTURE_DECODE_WIDTH: i32 = 240;
/// Rows from the end at which more load.
const LOAD_AHEAD: usize = 6;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

const LIST: &str = r##"
  <ListView xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" SelectionMode="None" IsItemClickEnabled="True">
    <ListView.ItemContainerTransitions><TransitionCollection /></ListView.ItemContainerTransitions>
    <ListView.ItemContainerStyle>
      <Style TargetType="ListViewItem">
        <Setter Property="Padding" Value="16,12" />
        <Setter Property="HorizontalContentAlignment" Value="Stretch" />
        <Setter Property="VerticalContentAlignment" Value="Top" />
      </Style>
    </ListView.ItemContainerStyle>
    <ListView.ItemTemplate><DataTemplate>
      <Grid>
        <Grid.ColumnDefinitions>
          <ColumnDefinition Width="46" /><ColumnDefinition MaxWidth="602" /><ColumnDefinition Width="Auto" />
        </Grid.ColumnDefinitions>
        <Grid.RowDefinitions>
          <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
          <RowDefinition Height="Auto" /><RowDefinition Height="Auto" /><RowDefinition Height="Auto" />
          <RowDefinition Height="Auto" />
        </Grid.RowDefinitions>
        <Image Grid.RowSpan="2" Width="36" Height="36" Stretch="UniformToFill" VerticalAlignment="Top" />
        <TextBlock Grid.Column="1" FontSize="13" TextTrimming="CharacterEllipsis" />
        <TextBlock Grid.Column="1" Grid.Row="1" FontSize="11"
                   Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        <TextBlock Grid.Column="1" Grid.Row="2" FontSize="13" Margin="0,8,0,0" TextWrapping="Wrap" MaxLines="6"
                   TextTrimming="CharacterEllipsis" />
        <Grid Grid.Column="1" Grid.Row="3" Margin="0,8,0,0" Height="90"
              Background="{ThemeResource SystemControlBackgroundBaseLowBrush}">
          <Grid.ColumnDefinitions><ColumnDefinition Width="170" /><ColumnDefinition Width="*" /></Grid.ColumnDefinitions>
          <Grid.RowDefinitions><RowDefinition Height="*" /><RowDefinition Height="Auto" /></Grid.RowDefinitions>
          <Image Grid.RowSpan="2" Margin="0,0,10,0" Stretch="UniformToFill" />
          <TextBlock Grid.Column="1" Margin="0,8,8,0" FontSize="13" TextWrapping="Wrap" MaxLines="2"
                     TextTrimming="CharacterEllipsis" />
          <TextBlock Grid.Column="1" Grid.Row="1" Margin="0,0,8,8" FontSize="11"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </Grid>
        <StackPanel Grid.Column="1" Grid.Row="4" Margin="0,8,0,0" Orientation="Horizontal">
          <Image Width="130" Height="130" Margin="0,0,4,0" Stretch="UniformToFill" />
          <Image Width="130" Height="130" Margin="0,0,4,0" Stretch="UniformToFill" />
          <Image Width="130" Height="130" Margin="0,0,4,0" Stretch="UniformToFill" />
          <TextBlock FontSize="12" VerticalAlignment="Bottom" Margin="4,0,0,0"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </StackPanel>
        <Border Grid.Column="1" Grid.Row="5" Margin="0,6,0,0" Padding="10,4,0,4" BorderThickness="3,0,0,0"
                BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
          <TextBlock FontSize="12" TextWrapping="Wrap" MaxLines="3" TextTrimming="CharacterEllipsis"
                     Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
        </Border>
        <Button Grid.Column="1" Grid.Row="6" Margin="0,6,0,0" Padding="0,2" Background="Transparent"
                BorderThickness="0" FontSize="12" Foreground="#0050EF" />
      </Grid>
    </DataTemplate></ListView.ItemTemplate>
  </ListView>
"##;

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs() as i64)
}

fn shown(element: windows_core::Result<UIElement>, on: bool) -> Result<()> {
    element?.cast::<IUIElement>()?.SetVisibility(if on { Visibility::Visible } else { Visibility::Collapsed })
}

thread_local! {
    static LISTS: RefCell<HashMap<u64, Weak<PostList>>> = RefCell::default();
}

pub struct PostList {
    id: u64,
    list: IFrameworkElement,
    /// Told when the list has something new to say.
    spoke: Box<dyn Fn()>,
    /// Told before a post opens something: the way back is the owner's to set up.
    leaving: Box<dyn Fn()>,
    /// Shows a post's comments; a page of their own when the owner says nothing.
    comments_asked: RefCell<Option<Box<dyn Fn(CommentArea, u64)>>>,
    posts: RefCell<Vec<Post>>,
    /// The UP whose posts show; 0 for everyone.
    mid: Cell<u64>,
    next: RefCell<Option<String>>,
    ended: Cell<bool>,
    loading: Cell<bool>,
    started: Cell<bool>,
    turn: Cell<u64>,
    said: RefCell<String>,
    covers: Covers,
}

impl PostList {
    pub fn new(spoke: impl Fn() + 'static, leaving: impl Fn() + 'static) -> Result<Rc<Self>> {
        let list = load(LIST)?;
        let shown = Rc::new(Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            list: list.clone(),
            spoke: Box::new(spoke),
            leaving: Box::new(leaving),
            comments_asked: RefCell::default(),
            posts: RefCell::default(),
            mid: Cell::new(0),
            next: RefCell::default(),
            ended: Cell::new(false),
            loading: Cell::new(false),
            started: Cell::new(false),
            turn: Cell::new(0),
            said: RefCell::default(),
            covers: Covers::default(),
        });
        LISTS.with_borrow_mut(|l| l.insert(shown.id, Rc::downgrade(&shown)));
        let filling = Rc::downgrade(&shown);
        on_row(&list, move |at, parts| filling.upgrade().map_or(Ok(()), |p| p.fill_post(at, parts)))?;
        let opening = Rc::downgrade(&shown);
        on_row_click(&list, move |at| {
            if let Some(shown) = opening.upgrade() {
                let _ = shown.open(at);
            }
        })?;
        Ok(shown)
    }

    pub fn element(&self) -> &IFrameworkElement {
        &self.list
    }

    /// Says how to show a post's comments, from its area and count.
    pub fn on_comments(&self, show: impl Fn(CommentArea, u64) + 'static) {
        *self.comments_asked.borrow_mut() = Some(Box::new(show));
    }

    pub fn said(&self) -> String {
        self.said.borrow().clone()
    }

    pub fn started(&self) -> bool {
        self.started.get()
    }

    fn say(&self, text: &str) {
        *self.said.borrow_mut() = text.to_owned();
        (self.spoke)();
    }

    /// Shows `mid`'s posts, or everyone's for 0.
    pub fn show_up(&self, mid: u64) -> Result<()> {
        self.started.set(true);
        self.mid.set(mid);
        self.turn.set(self.turn.get() + 1);
        self.posts.borrow_mut().clear();
        *self.next.borrow_mut() = None;
        self.ended.set(false);
        self.loading.set(false);
        self.list.cast::<IItemsControl>()?.Items()?.cast::<IVector<IInspectable>>()?.Clear()?;
        self.say("正在读取…");
        self.load();
        Ok(())
    }

    fn load(&self) {
        let Some(shell) = shell() else { return };
        let Some(backend) = shell.backend.get().cloned() else { return };
        if self.ended.get() || self.loading.replace(true) {
            return;
        }
        let (id, mid, turn, cursor) = (self.id, self.mid.get(), self.turn.get(), self.next.borrow().clone());
        let (dispatcher, core) = (shell.dispatcher.clone(), backend.clone());
        backend.spawn(async move {
            let following = &core.services.following;
            let page = if mid == 0 {
                following.posts(cursor.as_deref()).await
            } else {
                following.up_posts(mid, cursor.as_deref()).await
            };
            let page = page.map_err(|e| e.to_string()).map(|(posts, next)| {
                let now = now();
                (posts.into_iter().map(|p| Post::of(p, now)).collect::<Vec<_>>(), next)
            });
            post(&dispatcher, move || {
                if let Some(shown) = LISTS.with_borrow(|l| l.get(&id).and_then(Weak::upgrade)) {
                    let _ = shown.took_posts(turn, page);
                }
            });
        });
    }

    fn took_posts(&self, turn: u64, page: std::result::Result<(Vec<Post>, Option<String>), String>) -> Result<()> {
        if turn != self.turn.get() {
            return Ok(());
        }
        self.loading.set(false);
        match page {
            Ok((posts, next)) => {
                self.ended.set(next.is_none());
                *self.next.borrow_mut() = next;
                let from = self.posts.borrow().len();
                self.posts.borrow_mut().extend(posts);
                let to = self.posts.borrow().len();
                add_rows(&self.list, from, to)?;
                self.say(if to == 0 { "还没有动态" } else { "" });
            }
            Err(problem) => {
                self.ended.set(true);
                self.say(&format!("读取失败：{problem}"));
            }
        }
        Ok(())
    }

    fn fill_post(&self, at: usize, parts: &IVector<UIElement>) -> Result<()> {
        let posts = self.posts.borrow();
        let Some(post) = posts.get(at) else { return Ok(()) };
        parts.GetAt(0)?.cast::<IImage>()?.SetSource(&self.covers.get(&post.face, FACE_DECODE_WIDTH)?)?;
        parts.GetAt(1)?.cast::<ITextBlock>()?.SetText(&post.name)?;
        parts.GetAt(2)?.cast::<ITextBlock>()?.SetText(&post.line)?;
        parts.GetAt(3)?.cast::<ITextBlock>()?.SetText(&post.text)?;
        shown(parts.GetAt(3), !post.text.is_empty())?;

        shown(parts.GetAt(4), post.clip.is_some())?;
        if let Some(clip) = &post.clip {
            let inside = parts.GetAt(4)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
            inside.GetAt(0)?.cast::<IImage>()?.SetSource(&self.covers.get(&clip.cover, COVER_DECODE_WIDTH)?)?;
            inside.GetAt(1)?.cast::<ITextBlock>()?.SetText(&clip.title)?;
            inside.GetAt(2)?.cast::<ITextBlock>()?.SetText(&clip.line)?;
        }

        let pictured = post.clip.is_none() && !post.pictures.is_empty();
        shown(parts.GetAt(5), pictured)?;
        if pictured {
            let inside = parts.GetAt(5)?.cast::<IPanel>()?.Children()?.cast::<IVector<UIElement>>()?;
            for slot in 0..PICTURES {
                match post.pictures.get(slot) {
                    Some(url) => {
                        shown(inside.GetAt(slot as u32), true)?;
                        let picture = self.covers.get(url, PICTURE_DECODE_WIDTH)?;
                        inside.GetAt(slot as u32)?.cast::<IImage>()?.SetSource(&picture)?;
                    }
                    None => shown(inside.GetAt(slot as u32), false)?,
                }
            }
            let more = if post.more_pictures > 0 { format!("+{}", post.more_pictures) } else { String::new() };
            inside.GetAt(PICTURES as u32)?.cast::<ITextBlock>()?.SetText(&more)?;
        }

        shown(parts.GetAt(6), !post.quote.is_empty())?;
        if !post.quote.is_empty() {
            let border = parts.GetAt(6)?.cast::<IBorder>()?;
            border.Child()?.cast::<ITextBlock>()?.SetText(&post.quote)?;
        }
        let button = parts.GetAt(7)?.cast::<IFrameworkElement>()?;
        shown(parts.GetAt(7), post.comments.is_some())?;
        button.cast::<IContentControl>()?.SetContent(&PropertyValue::CreateString(&post.comment_label)?)?;
        self.wire_comments(&button)?;
        let near_end = at + LOAD_AHEAD >= posts.len();
        drop(posts);
        if near_end {
            self.load();
        }
        Ok(())
    }

    /// Rows are reused: a card's comment button is wired once, and finds its post through the
    /// row's data.
    fn wire_comments(&self, button: &IFrameworkElement) -> Result<()> {
        if button.Tag().is_ok() {
            return Ok(());
        }
        button.SetTag(&PropertyValue::CreateString("wired")?)?;
        let id = self.id;
        keep(button.cast::<IButtonBase>()?.Click(move |sender: Ref<IInspectable>, _: Ref<RoutedEventArgs>| {
            let at = sender
                .ok()
                .and_then(|s| s.cast::<IFrameworkElement>()?.DataContext()?.cast::<IPropertyValue>()?.GetInt32());
            if let (Ok(at), Some(shown)) = (at, LISTS.with_borrow(|l| l.get(&id).and_then(Weak::upgrade))) {
                let _ = shown.show_comments(at as usize);
            }
        })?);
        Ok(())
    }

    fn show_comments(&self, at: usize) -> Result<()> {
        let posts = self.posts.borrow();
        let Some((kind, oid)) = posts.get(at).and_then(|p| p.comments) else { return Ok(()) };
        let total = posts.get(at).map_or(0, |p| p.comment_count);
        drop(posts);
        let area = CommentArea { kind, oid };
        if let Some(show) = self.comments_asked.borrow().as_ref() {
            show(area, total);
            return Ok(());
        }
        (self.leaving)();
        comment_list::open_page(area, total)
    }

    /// The card's video, its article, or what it links to in the browser.
    fn open(&self, at: usize) -> Result<()> {
        let Some(post) = self.posts.borrow().get(at).cloned() else { return Ok(()) };
        if let Some(clip) = post.clip {
            (self.leaving)();
            return feed::route(&clip.bvid, &clip.title, &clip.owner);
        }
        if post.readable && !post.link.is_empty() {
            (self.leaving)();
            return article::open(&post.link);
        }
        if post.link.starts_with("https://") {
            drop(Launcher::LaunchUriAsync(&Uri::CreateUri(&post.link)?)?);
        }
        Ok(())
    }
}
