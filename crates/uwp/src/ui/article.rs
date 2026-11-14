//! A 专栏 or 图文 to read, in the app.

use rubi_model::{Article, ArticleBlock};
use std::cell::RefCell;
use std::rc::Rc;

use rubi_core::CommentArea;
use windows_core::{IInspectable, Interface, Result};

use crate::state::library::age;
use crate::state::settings::escape;
use crate::ui::comment_list::CommentList;
use crate::ui::shell::shell;
use crate::ui::{load, named, on_click, post, set_text};
use crate::xaml::{IContentControl, IFrameworkElement, UIElement};
use std::time::{SystemTime, UNIX_EPOCH};

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.RowDefinitions><RowDefinition Height="Auto" /><RowDefinition Height="*" /></Grid.RowDefinitions>
  <Button x:Name="Back" Background="Transparent" BorderThickness="0" Padding="12,8">
    <StackPanel Orientation="Horizontal">
      <FontIcon Glyph="&#xE72B;" FontSize="16" /><TextBlock Text="返回" Margin="10,0,0,0" />
    </StackPanel>
  </Button>
  <TextBlock x:Name="Status" Grid.Row="1" Margin="16,8" FontSize="13" IsHitTestVisible="False"
             Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
  <ContentControl x:Name="Body" Grid.Row="1" IsTabStop="False" MaxWidth="760" HorizontalAlignment="Left"
                  HorizontalContentAlignment="Stretch" VerticalContentAlignment="Stretch" />
</Grid>"##;

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |t| t.as_secs() as i64)
}

fn text(body: &str, size: u32, weight: &str, margin: &str) -> String {
    let none = std::collections::BTreeMap::new();
    let inlines = crate::ui::emote_text::inlines(body, crate::ui::emote_text::lookup(&none), f64::from(size) * 1.4);
    format!(
        r##"<RichTextBlock FontSize="{size}" FontWeight="{weight}" Margin="{margin}" TextWrapping="Wrap"
             IsTextSelectionEnabled="True" LineHeight="{}"><Paragraph>{inlines}</Paragraph></RichTextBlock>"##,
        size * 3 / 2
    )
}

fn block_xaml(block: &ArticleBlock) -> String {
    match block {
        ArticleBlock::Heading(line) => text(line, 18, "Normal", "0,16,0,6"),
        ArticleBlock::Text(line) => text(line, 15, "Normal", "0,4"),
        ArticleBlock::Quote(line) => format!(
            r##"<Border BorderThickness="3,0,0,0" Padding="12,4" Margin="0,6"
                 BorderBrush="{{ThemeResource SystemControlForegroundBaseLowBrush}}">{}</Border>"##,
            text(line, 14, "Normal", "0")
        ),
        ArticleBlock::Pictures(pictures) => pictures
            .iter()
            .map(|p| {
                format!(
                    r##"<Image Source="{}" Stretch="Uniform" HorizontalAlignment="Left" Margin="0,6"
                         MaxWidth="{}" />"##,
                    escape(&p.url),
                    if p.width > 0 { p.width.min(720) } else { 720 }
                )
            })
            .collect(),
        ArticleBlock::Line => r##"<Border Height="1" Margin="0,12"
             Background="{ThemeResource SystemControlForegroundBaseLowBrush}" />"##
            .to_owned(),
    }
}

fn body_xaml(article: &Article) -> String {
    let cover = if article.cover.is_empty() {
        String::new()
    } else {
        format!(r##"<Image Source="{}" Stretch="Uniform" Margin="0,0,0,12" />"##, escape(&article.cover))
    };
    let blocks: String = article.blocks.iter().map(block_xaml).collect();
    let hidden = if article.hidden > 0 { format!(" · 已隐藏 {} 处推广", article.hidden) } else { String::new() };
    format!(
        r##"<StackPanel xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" Padding="16,0,16,32">
          {cover}{}
          <TextBlock Text="{} · {}{hidden}" FontSize="12" Margin="0,0,0,12"
                     Foreground="{{ThemeResource SystemControlForegroundBaseMediumBrush}}" />
          {blocks}</StackPanel>"##,
        text(&article.title, 24, "Light", "0,0,0,4"),
        escape(&article.author_name),
        age(article.published_at, now()),
    )
}

/// The article, then its comments under it in one scroll (none: the article alone).
fn show_article(
    root: &IFrameworkElement,
    article: &Article,
    link: &str,
    fallback: Option<(u32, u64)>,
    total: u64,
) -> Result<()> {
    let body = load(&body_xaml(article))?;
    // TEMP-PROBE: what the article itself carried, and what the dynamic handed over.
    let probe = format!(
        "article {}/{} · 传入 {} · {total} 条",
        article.comment_kind,
        article.comment_oid,
        match fallback {
            Some((k, o)) => format!("{k}/{o}"),
            None => "无".to_owned(),
        }
    );
    set_text(root, "Status", &probe)?;
    let host = named::<IContentControl>(root, "Body")?;
    let area = (article.comment_kind > 0 && article.comment_oid > 0)
        .then_some((article.comment_kind, article.comment_oid))
        .or(fallback);
    let Some((kind, oid)) = area else {
        let scroll = load(
            r#"<ScrollViewer xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" />"#,
        )?;
        scroll.cast::<IContentControl>()?.SetContent(&body.cast::<IInspectable>()?)?;
        return host.SetContent(&scroll.cast::<IInspectable>()?);
    };
    let comments = CommentList::new()?;
    comments.put_above(&body.cast::<UIElement>()?)?;
    let back = link.to_owned();
    comments.before_leaving(move || {
        if let Some(shell) = shell() {
            let back = back.clone();
            shell.remember(move || {
                let _ = open(&back);
            });
        }
    });
    host.SetContent(&comments.element().cast::<IInspectable>()?)?;
    // A failure here used to be swallowed, leaving the page without comments and no reason.
    if let Err(problem) = comments.show(CommentArea { kind, oid }, total) {
        set_text(root, "Status", &format!("评论加载失败：{problem}"))?;
    }
    COMMENTS.with_borrow_mut(|c| *c = Some(comments));
    Ok(())
}

thread_local! {
    /// The comments of the article on show.
    static COMMENTS: RefCell<Option<Rc<CommentList>>> = const { RefCell::new(None) };
}

fn go_back() {
    if let Some(shell) = shell() {
        shell.back();
    }
}

/// Opens the article at `link`: an `opus/ID` or `read/cvN` link, or `cvN`.
pub fn open(link: &str) -> Result<()> {
    open_with(link, None, 0)
}

/// Opens `link`, using `area` for the comments when the article itself names none: a 图文
/// dynamic carries its own comment area, the 专栏 it points at may not.
pub fn open_with(link: &str, area: Option<(u32, u64)>, total: u64) -> Result<()> {
    let Some(host) = shell() else { return Ok(()) };
    let Some(backend) = host.backend.get().cloned() else { return Ok(()) };
    let root = load(LAYOUT)?;
    set_text(&root, "Status", "正在读取…")?;
    on_click(&root, "Back", go_back)?;
    host.set_page(&root)?;
    let turn = crate::ui::shell::page_turn();
    let (dispatcher, core, link) = (host.dispatcher.clone(), backend.clone(), link.to_owned());
    backend.spawn(async move {
        let article = core.services.reading.open(&link).await.map_err(|e| e.to_string());
        post(&dispatcher, move || {
            let Some(shell) = shell() else { return };
            if crate::ui::shell::page_turn() != turn {
                return;
            }
            let Ok(root) = shell.current_page() else { return };
            let _ = match &article {
                Ok(article) => show_article(&root, article, &link, area, total),
                Err(problem) => set_text(&root, "Status", &format!("打不开：{problem}")),
            };
        });
    });
    Ok(())
}
