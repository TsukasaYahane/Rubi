//! 播放: one video through the system player. The manifest comes from `state`; the stream
//! requests carry the headers B站 asks for through the player's own HTTP client.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use rubi_core::{Playback, SendDanmaku, pick_streams};
use windows_core::{Error, HRESULT, Interface, Ref, Result};

use crate::state::layout::WindowClass;
use crate::state::manifest::dash_manifest;
use crate::ui::danmaku::Overlay;
use crate::ui::player_actions::Actions;
use crate::ui::player_side::Side;
use crate::ui::shell::shell;
use crate::ui::{load, named, on_click, post, set_text};
use crate::xaml::{
    AdaptiveMediaSource, AdaptiveMediaSourceCreationStatus, DataWriter, FrameworkElement, Grid, GridLength,
    GridUnitType, HttpClient, IColumnDefinition, IDataWriter, IFrameworkElement, IInputStream, IMediaPlaybackSession,
    IMediaPlaybackSource, IMediaPlayer, IMediaPlayer3, IMediaPlayerElement, IMediaPlayerFailedEventArgs,
    IMediaPlayerSource2, IOutputStream, IRandomAccessStream, IRowDefinition, ITextBox, InMemoryRandomAccessStream,
    MediaPlaybackSession, MediaPlayer, MediaPlayerFailedEventArgs, MediaSource, SizeChangedEventArgs, Uri,
};

/// Quality asked for; the core caps it to what the account may play.
const QUALITY: u32 = 80;
const DASH_TYPE: &str = "application/dash+xml";

const LAYOUT: &str = r##"
<Grid xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
      xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <Grid.ColumnDefinitions>
    <ColumnDefinition x:Name="MainColumn" Width="62*" />
    <ColumnDefinition x:Name="SideColumn" Width="38*" />
  </Grid.ColumnDefinitions>
  <Grid.RowDefinitions>
    <RowDefinition x:Name="MainRow" Height="*" />
    <RowDefinition x:Name="SideRow" Height="0" />
  </Grid.RowDefinitions>
  <Grid x:Name="Main">
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" />
      <RowDefinition x:Name="VideoRow" Height="*" />
      <RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" />
      <RowDefinition Height="Auto" />
    </Grid.RowDefinitions>
    <Button x:Name="Back" Background="Transparent" BorderThickness="0" Padding="12,8">
      <StackPanel Orientation="Horizontal">
        <FontIcon Glyph="&#xE72B;" FontSize="16" /><TextBlock Text="返回" Margin="10,0,0,0" />
      </StackPanel>
    </Button>
    <MediaPlayerElement x:Name="Video" Grid.Row="1" AreTransportControlsEnabled="True" Stretch="Uniform" />
    <Canvas x:Name="Danmaku" Grid.Row="1" IsHitTestVisible="False" />
    <Grid Grid.Row="2" Background="#111111" Padding="12,6">
      <Grid.ColumnDefinitions>
        <ColumnDefinition Width="*" />
        <ColumnDefinition Width="Auto" />
      </Grid.ColumnDefinitions>
      <TextBox x:Name="SendText" PlaceholderText="发个友善的弹幕…" MaxLength="100" BorderThickness="0,0,0,1"
               Background="Transparent" Foreground="White" />
      <Button x:Name="SendButton" Grid.Column="1" Background="Transparent" BorderThickness="0" Margin="8,0,0,0"
              Foreground="White">
        <FontIcon Glyph="&#xE724;" FontSize="18" />
      </Button>
    </Grid>
    <StackPanel Grid.Row="3" Margin="16,10,16,6">
      <TextBlock x:Name="Title" FontSize="20" FontWeight="Light" TextWrapping="Wrap" MaxLines="2" />
      <TextBlock x:Name="Meta" FontSize="12" Margin="0,4,0,0" TextWrapping="Wrap" MaxLines="3"
                 Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
    </StackPanel>
    <ScrollViewer Grid.Row="4" HorizontalScrollMode="Auto" VerticalScrollMode="Disabled"
                  HorizontalScrollBarVisibility="Hidden" VerticalScrollBarVisibility="Disabled">
      <StackPanel Orientation="Horizontal" Margin="6,0,6,8">
        <StackPanel.Resources>
          <Style TargetType="Button">
            <Setter Property="Background" Value="Transparent" />
            <Setter Property="BorderThickness" Value="0" />
            <Setter Property="Padding" Value="10,6" />
            <Setter Property="MinWidth" Value="64" />
          </Style>
        </StackPanel.Resources>
        <Button x:Name="ActLike"><StackPanel>
          <FontIcon Glyph="&#xE8E1;" FontSize="20" /><TextBlock x:Name="LikeText" Text="赞" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActCoin"><StackPanel>
          <FontIcon Glyph="&#xE8C7;" FontSize="20" /><TextBlock x:Name="CoinText" Text="币" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel>
          <Button.Flyout><MenuFlyout>
            <MenuFlyoutItem x:Name="Coin1" Text="投 1 枚" /><MenuFlyoutItem x:Name="Coin2" Text="投 2 枚" />
          </MenuFlyout></Button.Flyout></Button>
        <Button x:Name="ActFav"><StackPanel>
          <FontIcon Glyph="&#xE734;" FontSize="20" /><TextBlock x:Name="FavText" Text="藏" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActLater"><StackPanel>
          <FontIcon Glyph="&#xE823;" FontSize="20" /><TextBlock x:Name="LaterText" Text="稍后" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActDanmaku"><StackPanel>
          <FontIcon Glyph="&#xE8BD;" FontSize="20" /><TextBlock x:Name="DanmakuText" Text="弹幕 开" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
        <Button x:Name="ActFollow"><StackPanel>
          <FontIcon Glyph="&#xE8FA;" FontSize="20" /><TextBlock x:Name="FollowText" Text="关注" FontSize="12" Margin="0,3,0,0" HorizontalAlignment="Center" /></StackPanel></Button>
      </StackPanel>
    </ScrollViewer>
  </Grid>
  <Grid x:Name="Side" Grid.Column="1" BorderThickness="1,0,0,0"
        BorderBrush="{ThemeResource SystemControlForegroundBaseLowBrush}">
    <Grid.RowDefinitions>
      <RowDefinition Height="Auto" />
      <RowDefinition Height="*" />
    </Grid.RowDefinitions>
    <StackPanel Orientation="Horizontal" Margin="4,6,4,2">
      <Button x:Name="TabRelated" Content="推荐" Background="Transparent" BorderThickness="0" FontSize="20"
              FontWeight="Light" Padding="12,2" />
      <Button x:Name="TabComments" Content="评论" Background="Transparent" BorderThickness="0" FontSize="20"
              FontWeight="Light" Padding="12,2" />
    </StackPanel>
    <ListView x:Name="Related" Grid.Row="1" SelectionMode="None" IsItemClickEnabled="True">
      <ListView.ItemContainerStyle>
        <Style TargetType="ListViewItem">
          <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          <Setter Property="Padding" Value="12,6" />
        </Style>
      </ListView.ItemContainerStyle>
      <ListView.ItemTemplate>
        <DataTemplate>
          <Grid>
            <Grid.ColumnDefinitions>
              <ColumnDefinition Width="128" />
              <ColumnDefinition Width="*" />
            </Grid.ColumnDefinitions>
            <Grid.RowDefinitions>
              <RowDefinition Height="*" />
              <RowDefinition Height="Auto" />
            </Grid.RowDefinitions>
            <Image Grid.RowSpan="2" Stretch="UniformToFill" Height="72" />
            <TextBlock Grid.Column="1" Margin="10,0,0,0" FontSize="13" TextWrapping="Wrap" MaxLines="2"
                       TextTrimming="CharacterEllipsis" />
            <TextBlock Grid.Column="1" Grid.Row="1" Margin="10,0,0,0" FontSize="12" TextTrimming="CharacterEllipsis"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          </Grid>
        </DataTemplate>
      </ListView.ItemTemplate>
    </ListView>
    <ListView x:Name="Comments" Grid.Row="1" SelectionMode="None" Visibility="Collapsed">
      <ListView.ItemContainerStyle>
        <Style TargetType="ListViewItem">
          <Setter Property="HorizontalContentAlignment" Value="Stretch" />
          <Setter Property="Padding" Value="16,8" />
        </Style>
      </ListView.ItemContainerStyle>
      <ListView.ItemTemplate>
        <DataTemplate>
          <StackPanel>
            <TextBlock FontSize="12" Foreground="#0050EF" />
            <TextBlock FontSize="14" TextWrapping="Wrap" Margin="0,3,0,0" />
            <TextBlock FontSize="11" Margin="0,3,0,0"
                       Foreground="{ThemeResource SystemControlForegroundBaseMediumBrush}" />
          </StackPanel>
        </DataTemplate>
      </ListView.ItemTemplate>
    </ListView>
  </Grid>
</Grid>"##;

/// Bumped when a video opens or the page is left: an answer for an earlier one is dropped.
static RUN: AtomicU64 = AtomicU64::new(0);

thread_local! {
    /// The player of the page on show; on the UI thread.
    static PLAYING: RefCell<Option<MediaPlayer>> = const { RefCell::new(None) };
    /// The page on show, while there is one.
    static PAGE: RefCell<Option<Page>> = const { RefCell::new(None) };
}

/// The page on show and its parts.
#[derive(Clone)]
struct Page {
    root: IFrameworkElement,
    side: Rc<Side>,
    actions: Rc<Actions>,
    overlay: Rc<RefCell<Option<Rc<Overlay>>>>,
    /// What 发弹幕 needs: the video, and its part once known.
    bvid: String,
    cid: Rc<Cell<u64>>,
}

fn page() -> Option<Page> {
    PAGE.with(|p| p.borrow().clone())
}

/// Which opening of the page is current; answers for another are dropped.
pub fn run() -> u64 {
    RUN.load(Ordering::Relaxed)
}

/// The side pane, if opening `run` is still on show.
pub fn side(run: u64) -> Option<Rc<Side>> {
    if run != self::run() {
        return None;
    }
    page().map(|p| p.side)
}

/// The action row, if opening `run` is still on show.
pub fn actions(run: u64) -> Option<Rc<Actions>> {
    if run != self::run() {
        return None;
    }
    page().map(|p| p.actions)
}

/// The danmaku layer, if opening `run` is still on show.
pub fn overlay(run: u64) -> Option<Rc<Overlay>> {
    if run != self::run() {
        return None;
    }
    page()?.overlay.borrow().clone()
}

/// Shows a line under the title.
pub fn say(text: &str) {
    if let Some(page) = page() {
        let _ = set_text(&page.root, "Meta", text);
    }
}

/// Follows a new window class, if the page is on show.
pub fn resized() {
    if let Some(page) = page() {
        let _ = apply_layout(&page.root);
    }
}

/// Beside the video on a wide window; under it otherwise, the video keeping 16:9.
fn apply_layout(root: &IFrameworkElement) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    let wide = shell.class() == WindowClass::Wide;
    let star = |value| GridLength { value, grid_unit_type: GridUnitType::Star };
    let pixels = |value| GridLength { value, grid_unit_type: GridUnitType::Pixel };
    let auto = GridLength { value: 1.0, grid_unit_type: GridUnitType::Auto };
    named::<IColumnDefinition>(root, "MainColumn")?.SetWidth(if wide { star(62.0) } else { star(1.0) })?;
    named::<IColumnDefinition>(root, "SideColumn")?.SetWidth(if wide { star(38.0) } else { pixels(0.0) })?;
    named::<IRowDefinition>(root, "MainRow")?.SetHeight(if wide { star(1.0) } else { auto })?;
    named::<IRowDefinition>(root, "SideRow")?.SetHeight(if wide { pixels(0.0) } else { star(1.0) })?;
    let side = named::<FrameworkElement>(root, "Side")?;
    Grid::SetColumn(&side, if wide { 1 } else { 0 })?;
    Grid::SetRow(&side, if wide { 0 } else { 1 })?;
    let video = if wide { star(1.0) } else { pixels((root.ActualWidth()? * 9.0 / 16.0).floor().max(1.0)) };
    named::<IRowDefinition>(root, "VideoRow")?.SetHeight(video)
}

/// Stops the video and lets its player go.
pub fn leave() {
    RUN.fetch_add(1, Ordering::Relaxed);
    PAGE.with(|p| *p.borrow_mut() = None);
    if let Some(player) = PLAYING.with(|p| p.borrow_mut().take()) {
        let _ = player.cast::<IMediaPlayer>().and_then(|p| p.Pause());
        let _ = player.cast::<IMediaPlayerSource2>().and_then(|p| p.SetSource(None::<&IMediaPlaybackSource>));
    }
}

/// Opens the video page.
pub fn show(bvid: &str, title: &str, uploader: &str) -> Result<()> {
    let Some(shell) = shell() else { return Ok(()) };
    leave();
    let root = load(LAYOUT)?;
    set_text(&root, "Title", title)?;
    set_text(&root, "Meta", &format!("{uploader} · 正在打开…"))?;
    on_click(&root, "Back", go_back)?;
    let pane = Side::attach(&root)?;
    let actions = Actions::attach(&root, bvid)?;
    let (shown_overlay, cid) = (Rc::new(RefCell::new(None)), Rc::new(Cell::new(0)));
    let opened = Page { root: root.clone(), side: pane, actions, overlay: shown_overlay, bvid: bvid.to_owned(), cid };
    PAGE.with(|p| *p.borrow_mut() = Some(opened));
    on_click(&root, "ActDanmaku", toggle_danmaku)?;
    on_click(&root, "SendButton", send_danmaku)?;
    let sizing = root.clone();
    let revoker = root.SizeChanged(move |_: Ref<windows_core::IInspectable>, _: Ref<SizeChangedEventArgs>| {
        let _ = apply_layout(&sizing);
    })?;
    std::mem::forget(revoker);
    shell.set_page(&root)?;
    apply_layout(&root)?;

    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    let run = RUN.load(Ordering::Relaxed);
    let dispatcher = shell.dispatcher.clone();
    let core = backend.clone();
    let (bvid, uploader) = (bvid.to_owned(), uploader.to_owned());
    let (info_dispatcher, info_core, info_bvid) = (dispatcher.clone(), core.clone(), bvid.clone());
    backend.spawn(async move {
        let services = &info_core.services;
        let (detail, related) = tokio::join!(services.videos.detail(&info_bvid), services.discover.related(&info_bvid));
        let detail = detail.ok().map(|d| d.card);
        let related = related.unwrap_or_default();
        let relation =
            if services.account.is_logged_in() { services.actions.relation(&info_bvid).await.ok() } else { None };
        post(&info_dispatcher, move || {
            let Some(page) = (run == self::run()).then(page).flatten() else { return };
            if let Some(card) = detail {
                page.side.set_aid(card.aid);
                page.actions.set_counts(card.stats.like, card.stats.coin, card.stats.favorite);
                let _ = set_text(&page.root, "Title", &card.title);
                let _ = set_text(&page.root, "Meta", &format!("{} 播放 · {}", card.stats.view, card.desc));
            }
            if let Some(relation) = relation {
                page.actions.set_relation(relation);
            }
            let _ = page.side.set_related(related);
        });
    });
    backend.spawn(async move {
        let source = match core.services.videos.playback(&bvid, QUALITY).await {
            Ok(playback) => open_source(&playback)
                .map(|source| (source, playback.cid, playback.url.duration_ms as u32))
                .map_err(|e| e.message()),
            Err(e) => Err(e.to_string()),
        };
        post(&dispatcher, move || {
            if RUN.load(Ordering::Relaxed) == run {
                let _ = shown(source, &uploader);
            }
        });
    });
    Ok(())
}

fn go_back() {
    if let Some(shell) = shell() {
        shell.back();
    }
}

fn shown(source: std::result::Result<(MediaSource, u64, u32), String>, uploader: &str) -> Result<()> {
    let Some(page) = page() else { return Ok(()) };
    match source {
        Ok((source, cid, duration_ms)) => {
            let session = play(&page.root, &source)?;
            page.cid.set(cid);
            let overlay = Overlay::attach(&page.root, session)?;
            if let Some(shell) = shell()
                && let Some(backend) = shell.backend.get().cloned()
            {
                overlay.load(backend, shell.dispatcher.clone(), cid, duration_ms);
            }
            *page.overlay.borrow_mut() = Some(overlay);
            Ok(())
        }
        Err(problem) => set_text(&page.root, "Meta", &format!("{uploader} · 无法播放：{problem}")),
    }
}

fn toggle_danmaku() {
    let Some(page) = page() else { return };
    if let Some(overlay) = page.overlay.borrow().as_ref() {
        let on = overlay.toggle();
        let _ = set_text(&page.root, "DanmakuText", if on { "弹幕 开" } else { "弹幕 关" });
    }
}

fn send_danmaku() {
    let _ = send_typed();
}

fn send_typed() -> Result<()> {
    let (Some(page), Some(shell)) = (page(), shell()) else { return Ok(()) };
    let Some(backend) = shell.backend.get().cloned() else { return Ok(()) };
    if !backend.services.account.is_logged_in() {
        crate::ui::shell::show_tab(crate::ui::shell::Tab::Me);
        return Ok(());
    }
    let field = named::<ITextBox>(&page.root, "SendText")?;
    let text = field.Text()?;
    let cid = page.cid.get();
    if text.trim().is_empty() || cid == 0 {
        return Ok(());
    }
    let position = PLAYING.with(|p| {
        p.borrow().as_ref().and_then(|player| {
            let session = player.cast::<IMediaPlayer3>().and_then(|m| m.PlaybackSession()).ok()?;
            session.cast::<IMediaPlaybackSession>().and_then(|s| s.Position()).ok()
        })
    });
    let progress_ms = position.map_or(0, |t| (t.duration / 10_000) as u32);
    field.SetText("")?;
    let request = SendDanmaku { bvid: page.bvid.clone(), cid, progress_ms, text, mode: 1, color: 0xFF_FFFF };
    let (dispatcher, run, core) = (shell.dispatcher.clone(), self::run(), backend.clone());
    backend.spawn(async move {
        let sent = core.services.danmaku.send(request).await.map_err(|e| e.to_string());
        post(&dispatcher, move || match sent {
            Ok(sent) => {
                if let Some(overlay) = overlay(run) {
                    overlay.add(sent);
                }
            }
            Err(problem) => say(&format!("发送失败：{problem}")),
        });
    });
    Ok(())
}

/// A failure in the player shows on the page.
fn report_failures(player: &MediaPlayer) -> Result<()> {
    let Some(dispatcher) = shell().map(|s| s.dispatcher.clone()) else { return Ok(()) };
    let run = RUN.load(Ordering::Relaxed);
    let revoker = player.cast::<IMediaPlayer>()?.MediaFailed(
        move |_: Ref<MediaPlayer>, args: Ref<MediaPlayerFailedEventArgs>| {
            let reason = args
                .ok()
                .and_then(|a| {
                    let a = a.cast::<IMediaPlayerFailedEventArgs>()?;
                    Ok(format!("{:?} {} {:?}", a.Error()?, a.ErrorMessage()?, a.ExtendedErrorCode()?))
                })
                .unwrap_or_default();
            post(&dispatcher, move || {
                if RUN.load(Ordering::Relaxed) == run
                    && let Some(root) = shell().and_then(|s| s.current_page().ok())
                {
                    let _ = set_text(&root, "Meta", &format!("播放失败：{reason}"));
                }
            });
        },
    )?;
    std::mem::forget(revoker);
    Ok(())
}

fn play(root: &IFrameworkElement, source: &MediaSource) -> Result<MediaPlaybackSession> {
    let player = MediaPlayer::new()?;
    player.cast::<IMediaPlayerSource2>()?.SetSource(&source.cast::<IMediaPlaybackSource>()?)?;
    named::<IMediaPlayerElement>(root, "Video")?.SetMediaPlayer(&player)?;
    report_failures(&player)?;
    player.cast::<IMediaPlayer>()?.Play()?;
    let session = player.cast::<IMediaPlayer3>()?.PlaybackSession()?;
    PLAYING.with(|p| *p.borrow_mut() = Some(player));
    Ok(session)
}

/// The media source for `playback`. Blocks on WinRT calls: never on the UI thread.
fn open_source(playback: &Playback) -> Result<MediaSource> {
    let (video, audio) = pick_streams(&playback.url);
    let manifest = video
        .and_then(|v| dash_manifest(playback.url.duration_ms, v, audio))
        .ok_or_else(|| Error::new(HRESULT(0x8000_4005_u32 as i32), "这个视频没有可用的 DASH 信息"))?;

    let client = HttpClient::new()?;
    let headers = client.DefaultRequestHeaders()?;
    for (name, value) in &playback.headers {
        headers.TryAppendWithoutValidation(name, value)?;
    }

    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream.cast::<IOutputStream>()?)?.cast::<IDataWriter>()?;
    writer.WriteBytes(manifest.as_bytes())?;
    writer.StoreAsync()?.join()?;
    writer.DetachStream()?;
    stream.cast::<IRandomAccessStream>()?.Seek(0)?;

    let base = Uri::CreateUri("https://www.bilibili.com/")?;
    let created = AdaptiveMediaSource::CreateFromStreamWithDownloaderAsync(
        &stream.cast::<IInputStream>()?,
        &base,
        DASH_TYPE,
        &client,
    )?
    .join()?;
    let status = created.Status()?;
    if status != AdaptiveMediaSourceCreationStatus::Success {
        return Err(Error::new(HRESULT(0x8000_4005_u32 as i32), format!("系统播放器拒绝了清单：{status:?}")));
    }
    MediaSource::CreateFromAdaptiveMediaSource(&created.MediaSource()?)
}
