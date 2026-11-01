//! The Kotlin-facing API. One [`RubiCore`] object wraps [`rubi_core::App`]; every call
//! returns flat records from [`types`]. Generated bindings live in the Android module.

mod types;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rubi_api::{ApiError, CommentSort, SearchOrder, SendDanmaku, Session, UpOrder};
use rubi_core::{App, CoreError, DanmakuSession, ExportOptions, export_bundle, import_bundle, load_danmaku};
use rubi_filter::danmaku::layout::{Lane, LayoutConfig, MeasuredWidths, layout};
use rubi_model::{
    Comment, Danmaku, DislikeReason, FeedbackEvent, ItemRef, NoticeKind, Signal, ThreadRow, VideoCard, VideoRef,
    flatten_thread, now_secs,
};

pub use types::*;

uniffi::setup_scaffolding!();

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum RubiError {
    #[error("网络错误：{msg}")]
    Network { msg: String },
    #[error("请求被风控拦截：{msg}")]
    RiskControl { msg: String },
    #[error("需要登录")]
    NotLoggedIn,
    #[error("{msg}")]
    Invalid { msg: String },
    #[error("{msg}")]
    Internal { msg: String },
}

impl From<ApiError> for RubiError {
    fn from(e: ApiError) -> Self {
        match e {
            ApiError::Http(e) => RubiError::Network { msg: e.to_string() },
            ApiError::RiskControl { .. } => RubiError::RiskControl { msg: e.to_string() },
            ApiError::NotLoggedIn => RubiError::NotLoggedIn,
            other => RubiError::Internal { msg: other.to_string() },
        }
    }
}

impl From<CoreError> for RubiError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Api(a) => a.into(),
            CoreError::Rules(_) | CoreError::Json(_) | CoreError::Invalid(_) => {
                RubiError::Invalid { msg: e.to_string() }
            }
            other => RubiError::Internal { msg: other.to_string() },
        }
    }
}

impl From<serde_json::Error> for RubiError {
    fn from(e: serde_json::Error) -> Self {
        RubiError::Invalid { msg: e.to_string() }
    }
}

impl From<std::io::Error> for RubiError {
    fn from(e: std::io::Error) -> Self {
        RubiError::Internal { msg: e.to_string() }
    }
}

type Res<T> = Result<T, RubiError>;

/// Replies loaded per page when a thread is expanded.
const THREAD_MAX_PAGES: u32 = 5;

#[derive(uniffi::Object)]
pub struct RubiCore {
    app: App,
    /// Cards handed to the UI, so signals and actions can carry item features.
    cards: Mutex<HashMap<String, VideoCard>>,
    /// Root comments by rpid, so threads can be expanded later.
    roots: Mutex<HashMap<u64, Comment>>,
}

impl RubiCore {
    fn remember(&self, cards: &[VideoCard]) {
        let mut map = self.cards.lock().unwrap_or_else(|e| e.into_inner());
        if map.len() > 5_000 {
            map.clear();
        }
        for c in cards {
            map.insert(c.bvid.clone(), c.clone());
        }
    }

    async fn card(&self, bvid: &str) -> Res<VideoCard> {
        if let Some(c) = self.cards.lock().unwrap_or_else(|e| e.into_inner()).get(bvid) {
            return Ok(c.clone());
        }
        let mut card = self.app.video.view(bvid).await?.card;
        card.tags = self.app.video.tags(bvid).await.unwrap_or_default();
        self.remember(std::slice::from_ref(&card));
        Ok(card)
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl RubiCore {
    /// `data_dir` is the app's private files directory; `session_json` is what a
    /// previous login returned (decrypted by the caller), or `None` for a guest.
    #[uniffi::constructor]
    pub fn new(data_dir: String, session_json: Option<String>) -> Res<Arc<Self>> {
        let session: Session = session_json.as_deref().map(serde_json::from_str).transpose()?.unwrap_or_default();
        let db = PathBuf::from(data_dir).join("rubi.db");
        let app = App::open(db, session)?;
        Ok(Arc::new(Self { app, cards: Mutex::default(), roots: Mutex::default() }))
    }

    /// The current session, to be stored encrypted by the caller.
    pub fn session_json(&self) -> Res<String> {
        Ok(serde_json::to_string(&self.app.http.session())?)
    }

    pub fn logout(&self) {
        self.app.http.update_session(|s| {
            *s = Session { buvid3: s.buvid3.take(), buvid4: s.buvid4.take(), ..Session::default() };
        });
    }

    pub async fn ensure_device_ids(&self) -> Res<()> {
        Ok(self.app.auth.ensure_device_ids().await?)
    }

    pub async fn account(&self) -> Res<FfiAccount> {
        let nav = self.app.auth.nav().await?;
        Ok(FfiAccount { logged_in: nav.is_login, mid: nav.mid, name: nav.uname })
    }

    pub async fn qr_generate(&self) -> Res<FfiQrTicket> {
        let t = self.app.auth.qr_generate().await?;
        Ok(FfiQrTicket { url: t.url, key: t.key })
    }

    pub async fn qr_poll(&self, key: String) -> Res<FfiQrState> {
        use rubi_api::QrState::*;
        Ok(match self.app.auth.qr_poll(&key).await? {
            Waiting => FfiQrState::Waiting,
            Scanned => FfiQrState::Scanned,
            Expired => FfiQrState::Expired,
            Success(s) => FfiQrState::Success { session_json: serde_json::to_string(&s)? },
        })
    }

    // ----- password / SMS login (Geetest solved by the user) -----

    pub async fn login_captcha(&self) -> Res<FfiLoginCaptcha> {
        let c = self.app.auth.login_captcha().await?;
        Ok(FfiLoginCaptcha { token: c.token, gt: c.gt, challenge: c.challenge })
    }

    /// RSA key (PEM) and salt for the password; the app encrypts `hash + password` itself.
    pub async fn login_key(&self) -> Res<FfiLoginKey> {
        let k = self.app.auth.login_key().await?;
        Ok(FfiLoginKey { hash: k.hash, public_key_pem: k.key })
    }

    /// `encrypted_password` is `base64(RSA-PKCS1(hash + password))`. Returns the session JSON
    /// to persist, like a finished QR login.
    pub async fn password_login(
        &self,
        username: String,
        encrypted_password: String,
        captcha: FfiSolvedCaptcha,
    ) -> Res<String> {
        let s = self.app.auth.password_login(&username, &encrypted_password, &captcha.into()).await?;
        Ok(serde_json::to_string(&s)?)
    }

    /// Sends the SMS code; keep the returned key for [`sms_login`](Self::sms_login).
    pub async fn sms_send(&self, cid: u32, tel: String, captcha: FfiSolvedCaptcha) -> Res<String> {
        Ok(self.app.auth.sms_send(cid, &tel, &captcha.into()).await?)
    }

    pub async fn sms_login(&self, cid: u32, tel: String, code: String, captcha_key: String) -> Res<String> {
        let s = self.app.auth.sms_login(cid, &tel, &code, &captcha_key).await?;
        Ok(serde_json::to_string(&s)?)
    }

    // ----- home feed -----

    pub async fn refresh_feed(&self, per_source: u32) -> Res<FfiRefreshReport> {
        let r = self.app.feed.refresh(per_source as usize).await?;
        Ok(FfiRefreshReport { kept: r.kept as u32, blocked: r.blocked as u32, errors: r.errors })
    }

    /// Call before rebuilding the home list (pull to refresh) so earlier cards may come back.
    pub fn new_feed_session(&self) {
        self.app.feed.new_session();
    }

    /// The next `n` cards not yet shown since `new_feed_session`; empty when the pool is used up.
    pub fn next_page(&self, n: u32) -> Res<Vec<FfiVideoCard>> {
        let ranked = self.app.feed.next_page(n as usize)?;
        let cards: Vec<VideoCard> = ranked.iter().map(|r| r.card.clone()).collect();
        self.remember(&cards);
        Ok(ranked.iter().map(|r| FfiVideoCard::of(&r.card, Some(&r.score))).collect())
    }

    pub fn explain(&self, bvid: String) -> Res<Option<FfiBreakdown>> {
        Ok(self.app.feed.explain(&bvid)?.map(Into::into))
    }

    pub async fn record(&self, bvid: String, signal: FfiSignal) -> Res<()> {
        let card = self.card(&bvid).await?;
        let first_tag = card.tags.first().cloned().unwrap_or_default();
        let signal = match signal {
            FfiSignal::Click => Signal::Click,
            FfiSignal::QuickExit => Signal::QuickExit,
            FfiSignal::Completion { ratio } => Signal::Completion { ratio },
            FfiSignal::Rewatch => Signal::Rewatch,
            FfiSignal::DislikeUp => Signal::Dislike { reason: DislikeReason::Up },
            FfiSignal::DislikeZone => Signal::Dislike { reason: DislikeReason::Zone },
            FfiSignal::DislikeTag { tag } => {
                Signal::Dislike { reason: DislikeReason::Tag(if tag.is_empty() { first_tag } else { tag }) }
            }
            FfiSignal::DislikeTitle => Signal::Dislike { reason: DislikeReason::Title },
        };
        Ok(self.app.feed.record(FeedbackEvent { signal, item: Some(ItemRef::of(&card)), at: now_secs() })?)
    }

    pub fn filter_log(&self, limit: u32) -> Res<Vec<FfiFilterLogEntry>> {
        Ok(self
            .app
            .feed
            .filter_log(limit as usize)?
            .into_iter()
            .map(|e| FfiFilterLogEntry {
                at: e.at,
                rule_id: e.rule_id,
                category: e.category,
                title: e.title,
                source: e.source,
                reason: e.reason,
            })
            .collect())
    }

    // ----- video page -----

    pub async fn video(&self, bvid: String) -> Res<FfiVideoCard> {
        Ok(FfiVideoCard::of(&self.card(&bvid).await?, None))
    }

    /// Full details for the player page (always fresh from `view`, plus tags).
    pub async fn video_detail(&self, bvid: String) -> Res<FfiVideoDetail> {
        let detail = self.app.video.view(&bvid).await?;
        let mut card = detail.card;
        card.tags = self.app.video.tags(&bvid).await.unwrap_or_default();
        card.cid = card.cid.or(Some(detail.cid)).filter(|&c| c > 0);
        self.remember(std::slice::from_ref(&card));
        Ok(FfiVideoDetail::of(&card, detail.pages.len().max(1) as u32))
    }

    /// The BV id to open for a video pointer (av numbers are looked up, short links followed).
    pub async fn resolve_video(&self, video: FfiVideoRef) -> Res<String> {
        let video = match video {
            FfiVideoRef::Bvid { bvid } => VideoRef::Bvid(bvid),
            FfiVideoRef::Aid { aid } => VideoRef::Aid(aid),
            FfiVideoRef::Short { url } => VideoRef::Short(url),
        };
        Ok(self.app.resolve_video(video).await?)
    }

    /// Stream URLs of one part (`cid`) of a video, e.g. a branch of an interactive video.
    pub async fn playback_part(&self, bvid: String, cid: u64, qn: u32) -> Res<FfiPlayback> {
        let p = self.app.video.playurl(&bvid, cid, qn).await?;
        Ok(FfiPlayback::of(cid, p, self.app.video.stream_headers()))
    }

    /// The story of an interactive video starting at part `cid`; `None` for ordinary videos.
    pub async fn stein_open(&self, bvid: String, cid: u64) -> Res<Option<Arc<FfiStein>>> {
        let session = rubi_core::SteinSession::open(self.app.stein.clone(), &bvid, cid).await?;
        Ok(session.map(|s| Arc::new(FfiStein(s))))
    }

    /// Stream URLs for `qn` (the server caps it to what the account may play).
    pub async fn playback(&self, bvid: String, qn: u32) -> Res<FfiPlayback> {
        // Feed cards usually carry the cid already; only look it up when they do not.
        let cached = self.cards.lock().unwrap_or_else(|e| e.into_inner()).get(&bvid).and_then(|c| c.cid);
        let cid = match cached {
            Some(cid) => cid,
            None => self.app.video.view(&bvid).await?.cid,
        };
        let p = self.app.video.playurl(&bvid, cid, qn).await?;
        Ok(FfiPlayback::of(cid, p, self.app.video.stream_headers()))
    }

    /// Posts a danmaku at `progress_ms` and returns it, ready to add to the shown set.
    /// `mode`: 1 scroll, 4 bottom, 5 top; `color` is 0xRRGGBB. Login required.
    pub async fn send_danmaku(
        &self,
        bvid: String,
        cid: u64,
        progress_ms: u32,
        text: String,
        mode: u8,
        color: u32,
    ) -> Res<FfiDanmaku> {
        let text = text.trim().to_owned();
        if text.is_empty() {
            return Err(RubiError::Invalid { msg: "弹幕不能为空".into() });
        }
        let req = SendDanmaku { bvid, cid, progress_ms, text, mode, color };
        let id = self.app.danmaku.send(&req).await?;
        Ok(FfiDanmaku { id, progress_ms, mode, font_size: 25, color, content: req.text })
    }

    /// All danmaku at once. Prefer [`danmaku_session`](Self::danmaku_session) for playback.
    pub async fn danmaku(&self, cid: u64, duration_ms: u32) -> Res<FfiDanmakuSet> {
        let opts = self.app.feed.settings().rules.danmaku_options().map_err(CoreError::from)?;
        Ok(load_danmaku(self.app.danmaku.clone(), cid, duration_ms, &opts).await?.into())
    }

    /// A segment-by-segment loader: load the segment being watched first, then the rest.
    pub fn danmaku_session(&self, cid: u64, duration_ms: u32) -> Res<Arc<FfiDanmakuSession>> {
        let opts = self.app.feed.settings().rules.danmaku_options().map_err(CoreError::from)?;
        Ok(Arc::new(FfiDanmakuSession(DanmakuSession::open(self.app.danmaku.clone(), cid, duration_ms, opts))))
    }

    /// Track layout for a batch of danmaku (font-free width estimate for now).
    /// `scroll_ms` is how long a scrolling danmaku takes to cross the screen; the
    /// renderer must use the same value.
    pub fn layout_danmaku(
        &self,
        items: Vec<FfiDanmaku>,
        widths: Vec<f32>,
        viewport_w: f32,
        font_size: f32,
        tracks: u32,
        scroll_ms: u32,
    ) -> Vec<FfiPlacement> {
        let dms: Vec<Danmaku> = items
            .into_iter()
            .map(|d| Danmaku {
                id: d.id,
                progress_ms: d.progress_ms,
                mode: d.mode,
                content: d.content,
                ..Default::default()
            })
            .collect();
        let cfg = LayoutConfig {
            viewport_w,
            font_size,
            tracks: tracks.max(1) as usize,
            scroll_ms: scroll_ms.max(1000),
            ..LayoutConfig::default()
        };
        // `widths[i]` is the rendered width of `items[i]` (same unit as `viewport_w`); empty or
        // shorter lists fall back to the estimate for the rest.
        let measured = MeasuredWidths(dms.iter().map(|d| d.content.as_str()).zip(widths).collect());
        layout(&dms, &cfg, &measured)
            .into_iter()
            .map(|p| FfiPlacement {
                id: p.id,
                lane: match p.lane {
                    Lane::Scroll => FfiLane::Scroll,
                    Lane::Top => FfiLane::Top,
                    Lane::Bottom => FfiLane::Bottom,
                },
                track: p.track as u32,
                start_ms: p.start_ms,
                width: p.width,
            })
            .collect()
    }

    // ----- comments -----

    /// Root comments with their inline previews drawn as fishbone rows.
    pub async fn comments(&self, aid: u64, by_time: bool, offset: Option<String>) -> Res<FfiCommentPage> {
        let sort = if by_time { CommentSort::Time } else { CommentSort::Hot };
        let page = self.app.comments(aid, sort, offset.as_deref()).await?;
        let mut roots = self.roots.lock().unwrap_or_else(|e| e.into_inner());
        let threads = page
            .page
            .threads
            .iter()
            .map(|t| {
                roots.insert(t.root.rpid, t.root.clone());
                thread_of(&t.root, &t.preview, usize::MAX, t.total_replies)
            })
            .collect();
        let pinned = page
            .page
            .pinned
            .iter()
            .map(|c| {
                let row = ThreadRow { rpid: c.rpid, depth: 0, has_later_sibling: false, reply_to: None };
                FfiCommentRow::of(&row, c)
            })
            .collect();
        Ok(FfiCommentPage {
            pinned,
            threads,
            hidden_pinned: page.hidden_pinned as u32,
            next_offset: page.page.next_offset,
        })
    }

    /// Loads a thread fully and returns its first `limit` replies as fishbone rows.
    pub async fn comment_thread(&self, aid: u64, root_rpid: u64, limit: u32) -> Res<FfiCommentThread> {
        let root = self
            .roots
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&root_rpid)
            .cloned()
            .ok_or_else(|| RubiError::Invalid { msg: "unknown root comment; load the page first".into() })?;
        let replies = self.app.thread(aid, root_rpid, THREAD_MAX_PAGES).await?;
        let total = root.reply_count.max(replies.len() as u32);
        Ok(thread_of(&root, &replies, limit as usize, total))
    }

    /// The whole thread holding a comment from a notice, root first, even when it is not on
    /// any loaded page. "再看 5 条" then works on it like on any other thread.
    pub async fn focus_thread(&self, aid: u64, comment: FfiCommentRef) -> Res<FfiCommentThread> {
        let root = self.app.comments.root(aid, comment.root).await?;
        self.roots.lock().unwrap_or_else(|e| e.into_inner()).insert(root.rpid, root.clone());
        let replies = self.app.thread(aid, root.rpid, THREAD_MAX_PAGES).await?;
        let total = root.reply_count.max(replies.len() as u32);
        Ok(thread_of(&root, &replies, usize::MAX, total))
    }

    // ----- search -----

    pub async fn search(&self, keyword: String, page: u32) -> Res<FfiSearchResult> {
        let (cards, hidden) = self.app.search_videos(&keyword, page.max(1), SearchOrder::Relevance).await?;
        self.remember(&cards);
        Ok(FfiSearchResult { cards: cards.iter().map(|c| FfiVideoCard::of(c, None)).collect(), hidden: hidden as u32 })
    }

    pub async fn hot_words(&self) -> Res<FfiHotWords> {
        let (words, hidden) = self.app.hot_words(10).await?;
        Ok(FfiHotWords { words: words.into_iter().map(|w| w.keyword).collect(), hidden: hidden as u32 })
    }

    /// The default emote set, for rendering `[doge]`-style tokens as images.
    pub async fn emotes(&self) -> Res<Vec<FfiEmote>> {
        Ok(self.app.emotes().await?.iter().map(|e| FfiEmote { text: e.text.clone(), url: e.url.clone() }).collect())
    }

    // ----- home lists -----

    pub async fn popular(&self, page: u32) -> Res<Vec<FfiVideoCard>> {
        let cards = self.app.popular(page).await?;
        self.remember(&cards);
        Ok(cards.iter().map(|c| FfiVideoCard::of(c, None)).collect())
    }

    pub async fn related(&self, bvid: String) -> Res<Vec<FfiVideoCard>> {
        let cards = self.app.related(&bvid).await?;
        self.remember(&cards);
        Ok(cards.iter().map(|c| FfiVideoCard::of(c, None)).collect())
    }

    pub async fn ranking(&self) -> Res<Vec<FfiVideoCard>> {
        let cards = self.app.ranking().await?;
        self.remember(&cards);
        Ok(cards.iter().map(|c| FfiVideoCard::of(c, None)).collect())
    }

    // ----- play list (also 听视频) -----

    pub fn playlist_items(&self) -> Res<Vec<FfiVideoCard>> {
        let items = self.app.playlist.items()?;
        self.remember(&items);
        Ok(items.iter().map(|c| FfiVideoCard::of(c, None)).collect())
    }

    pub fn playlist_contains(&self, bvid: String) -> Res<bool> {
        Ok(self.app.playlist.contains(&bvid)?)
    }

    /// Adds a video (looked up if it is not cached); false when it was already listed.
    pub async fn playlist_add(&self, bvid: String) -> Res<bool> {
        let card = self.card(&bvid).await?;
        Ok(self.app.playlist.add(card)?)
    }

    pub fn playlist_remove(&self, bvid: String) -> Res<()> {
        Ok(self.app.playlist.remove(&bvid)?)
    }

    pub fn playlist_move(&self, from: u32, to: u32) -> Res<()> {
        Ok(self.app.playlist.move_item(from as usize, to as usize)?)
    }

    pub fn playlist_clear(&self) -> Res<()> {
        Ok(self.app.playlist.clear()?)
    }

    pub fn playlist_mode(&self) -> Res<FfiPlaylistMode> {
        Ok(self.app.playlist.mode()?.into())
    }

    pub fn set_playlist_mode(&self, mode: FfiPlaylistMode) -> Res<()> {
        Ok(self.app.playlist.set_mode(mode.into())?)
    }

    /// Where 播放全部 / 听全部 starts.
    pub fn playlist_first(&self) -> Res<Option<FfiVideoCard>> {
        Ok(self.app.playlist.first()?.map(|c| FfiVideoCard::of(&c, None)))
    }

    /// What plays after `current` by the list's mode; `None` stops.
    pub fn playlist_next(&self, current: String) -> Res<Option<FfiVideoCard>> {
        Ok(self.app.playlist.next_after(&current)?.map(|c| FfiVideoCard::of(&c, None)))
    }

    // ----- 关注动态 -----

    /// One page of followed UPs' videos; pass `next` back for the following page.
    pub async fn followed_feed(&self, offset: Option<String>) -> Res<FfiFollowPage> {
        let (cards, next, hidden) = self.app.followed_feed(offset.as_deref()).await?;
        self.remember(&cards);
        Ok(FfiFollowPage {
            cards: cards.iter().map(|c| FfiVideoCard::of(c, None)).collect(),
            next,
            hidden: hidden as u32,
        })
    }

    /// Followed UPs with recent posts, for the bar above the 关注 feed.
    pub async fn followed_ups(&self) -> Res<Vec<FfiFollowedUp>> {
        Ok(self.app.dynamics.followed_ups().await?.into_iter().map(Into::into).collect())
    }

    /// One UP's posts of every kind; pass `next` back for more.
    pub async fn up_dynamics(&self, mid: u64, offset: Option<String>) -> Res<FfiDynamicPage> {
        let (items, next) = self.app.up_dynamics(mid, offset.as_deref()).await?;
        let videos: Vec<VideoCard> = items
            .iter()
            .flat_map(|d| d.video.iter().chain(d.orig.iter().flat_map(|o| o.video.iter())))
            .cloned()
            .collect();
        self.remember(&videos);
        Ok(FfiDynamicPage { items: items.iter().map(FfiDynamic::of).collect(), next })
    }

    // ----- message center -----

    pub async fn unread(&self) -> Res<FfiUnread> {
        Ok(self.app.notices.unread().await?.into())
    }

    pub async fn notices(&self, kind: FfiNoticeKind, cursor: Option<String>) -> Res<FfiNoticePage> {
        let kind = match kind {
            FfiNoticeKind::Reply => NoticeKind::Reply,
            FfiNoticeKind::At => NoticeKind::At,
            FfiNoticeKind::Like => NoticeKind::Like,
        };
        let (items, cursor) = self.app.notices.notices(kind, cursor.as_deref()).await?;
        Ok(FfiNoticePage { items: items.into_iter().map(Into::into).collect(), cursor })
    }

    pub async fn chat_sessions(&self) -> Res<FfiChatSessions> {
        let (sessions, hidden) = self.app.chat_sessions().await?;
        Ok(FfiChatSessions { sessions: sessions.into_iter().map(Into::into).collect(), hidden: hidden as u32 })
    }

    pub async fn chat_messages(&self, talker_mid: u64) -> Res<Vec<FfiChatMessage>> {
        let msgs = self.app.chat.messages(talker_mid, 30).await?;
        Ok(msgs.into_iter().map(|m| FfiChatMessage::of(m, talker_mid)).collect())
    }

    pub async fn send_message(&self, talker_mid: u64, text: String) -> Res<()> {
        Ok(self.app.chat.send_text(talker_mid, &text).await?)
    }

    pub async fn mark_read(&self, talker_mid: u64, ack_seqno: u64) -> Res<()> {
        Ok(self.app.chat.mark_read(talker_mid, ack_seqno).await?)
    }

    // ----- account actions -----

    pub async fn like(&self, bvid: String, on: bool) -> Res<()> {
        let card = self.card(&bvid).await?;
        Ok(self.app.like(&card, on).await?)
    }

    /// Whether the user has liked / coined / favourited the video and follows its UP.
    pub async fn relation(&self, bvid: String) -> Res<FfiVideoRelation> {
        Ok(self.app.actions.relation(&bvid).await?.into())
    }

    /// `count` is 1 or 2; `also_like` likes the video in the same request.
    pub async fn coin(&self, bvid: String, count: u32, also_like: bool) -> Res<()> {
        let card = self.card(&bvid).await?;
        Ok(self.app.coin(&card, count, also_like).await?)
    }

    /// 一键三连: like, coins (none if already coined) and the default favourite folder.
    pub async fn triple(&self, bvid: String) -> Res<FfiTriple> {
        let card = self.card(&bvid).await?;
        Ok(self.app.triple(&card).await?.into())
    }

    /// The user's favourite folders, each marked if it already holds the video.
    pub async fn fav_folders(&self, bvid: String) -> Res<Vec<FfiFavFolder>> {
        let card = self.card(&bvid).await?;
        Ok(self.app.actions.fav_folders(card.aid).await?.into_iter().map(Into::into).collect())
    }

    /// Adds the video to the `add` folders and removes it from the `remove` ones.
    pub async fn set_favorite(&self, bvid: String, add: Vec<u64>, remove: Vec<u64>) -> Res<()> {
        let card = self.card(&bvid).await?;
        Ok(self.app.favorite(&card, &add, &remove).await?)
    }

    pub async fn watch_later(&self, bvid: String) -> Res<()> {
        let card = self.card(&bvid).await?;
        Ok(self.app.watch_later(&card).await?)
    }

    pub async fn follow_up(&self, bvid: String, on: bool) -> Res<()> {
        let card = self.card(&bvid).await?;
        Ok(self.app.follow(&card, on).await?)
    }

    /// Follow / unfollow from the UP's page.
    pub async fn follow_mid(&self, mid: u64, name: String, on: bool) -> Res<()> {
        Ok(self.app.follow_mid(mid, &name, on).await?)
    }

    // ----- UP space -----

    pub async fn up_profile(&self, mid: u64) -> Res<FfiUpProfile> {
        Ok(self.app.up_profile(mid).await?.into())
    }

    /// Page `pn` (from 1) of an UP's uploads, 充电专属 videos removed.
    pub async fn up_videos(&self, mid: u64, pn: u32, order: FfiUpOrder) -> Res<FfiUpUploads> {
        let order = match order {
            FfiUpOrder::Newest => UpOrder::Pubdate,
            FfiUpOrder::MostViewed => UpOrder::Click,
            FfiUpOrder::MostFavorited => UpOrder::Stow,
        };
        let page = self.app.up_videos(mid, pn, order).await?;
        self.remember(&page.items);
        Ok(FfiUpUploads {
            items: page.items.iter().map(|c| FfiVideoCard::of(c, None)).collect(),
            total: page.total,
            hidden_charging: page.hidden_charging as u32,
            has_more: page.has_more,
        })
    }

    // ----- library -----

    /// Watch history; pass the returned cursor back for the next page.
    pub async fn history(&self, cursor: Option<String>) -> Res<FfiHistoryPage> {
        let (items, cursor) = self.app.library.history(cursor.as_deref()).await?;
        self.remember(&items.iter().map(|h| h.card.clone()).collect::<Vec<_>>());
        let items = items
            .iter()
            .map(|h| FfiHistoryItem {
                card: FfiVideoCard::of(&h.card, None),
                progress_s: h.progress_s,
                viewed_at: h.viewed_at,
            })
            .collect();
        Ok(FfiHistoryPage { items, cursor })
    }

    pub async fn watch_later_list(&self) -> Res<Vec<FfiVideoCard>> {
        let cards = self.app.library.watch_later_list().await?;
        self.remember(&cards);
        Ok(cards.iter().map(|c| FfiVideoCard::of(c, None)).collect())
    }

    /// The user's own favourite folders.
    pub async fn my_fav_folders(&self) -> Res<Vec<FfiFavFolder>> {
        Ok(self.app.actions.fav_folders(0).await?.into_iter().map(Into::into).collect())
    }

    /// Page `pn` (from 1) of a favourite folder.
    pub async fn fav_items(&self, folder_id: u64, pn: u32) -> Res<FfiFavPage> {
        let (cards, has_more) = self.app.library.fav_items(folder_id, pn.max(1)).await?;
        self.remember(&cards);
        Ok(FfiFavPage { items: cards.iter().map(|c| FfiVideoCard::of(c, None)).collect(), has_more })
    }

    /// Reports playback so the video lands in 历史记录. `play_type`: 1 start, 0 playing,
    /// 2 pause / leaving. Guests are skipped silently.
    pub async fn report_watch(&self, bvid: String, cid: u64, played_s: u64, realtime_s: u64, play_type: u8) -> Res<()> {
        if !self.app.http.session().is_logged_in() {
            return Ok(());
        }
        let card = self.card(&bvid).await?;
        Ok(self.app.library.heartbeat(card.aid, cid, played_s, realtime_s, play_type).await?)
    }

    pub async fn sync_followings(&self) -> Res<u32> {
        Ok(self.app.sync_followings().await? as u32)
    }

    // ----- settings, export -----

    /// The whole settings object (rules, weights, intervals) as JSON.
    pub fn settings_json(&self) -> Res<String> {
        Ok(serde_json::to_string(&self.app.feed.settings())?)
    }

    pub fn set_settings_json(&self, json: String) -> Res<()> {
        Ok(self.app.feed.set_settings(serde_json::from_str(&json)?)?)
    }

    /// Writes a zip with settings, rules, weights and profile (never credentials).
    pub fn export_to(&self, path: String, include_history: bool) -> Res<()> {
        let history = if include_history { self.app.feed.history()? } else { Vec::new() };
        let file = std::fs::File::create(path)?;
        let opts = ExportOptions { include_history };
        let playlist = self.app.playlist.items()?;
        Ok(export_bundle(file, &self.app.feed.settings(), &self.app.feed.profile(), &history, &playlist, opts)?)
    }

    pub fn import_from(&self, path: String) -> Res<()> {
        let imported = import_bundle(std::fs::File::open(path)?)?;
        self.app.feed.set_settings(imported.settings)?;
        self.app.feed.set_profile(imported.profile)?;
        for ev in imported.history {
            self.app.feed.record(ev)?;
        }
        if !imported.playlist.is_empty() {
            self.app.playlist.replace(&imported.playlist)?;
        }
        Ok(())
    }
}

/// An interactive video being played: its current node and hidden variables.
#[derive(uniffi::Object)]
pub struct FfiStein(rubi_core::SteinSession);

#[uniffi::export(async_runtime = "tokio")]
impl FfiStein {
    pub fn node(&self) -> FfiSteinNode {
        FfiSteinNode::of(&self.0.current(), self.0.shown_vars())
    }

    /// Takes the choice leading to `edge_id`; play the returned node's part next.
    pub async fn choose(&self, edge_id: u64) -> Res<FfiSteinNode> {
        let n = self.0.choose(edge_id).await?;
        Ok(FfiSteinNode::of(&n, self.0.shown_vars()))
    }
}

#[derive(uniffi::Object)]
pub struct FfiDanmakuSession(DanmakuSession);

#[uniffi::export(async_runtime = "tokio")]
impl FfiDanmakuSession {
    pub fn segment_count(&self) -> u32 {
        self.0.segment_count()
    }

    /// The 1-based segment containing `position_ms`.
    pub fn segment_at(&self, position_ms: u32) -> u32 {
        self.0.segment_at(position_ms)
    }

    /// Loads `segments` (concurrently, skipping loaded ones) and returns the filtered
    /// danmaku over everything loaded so far.
    pub async fn load(&self, segments: Vec<u32>) -> Res<FfiDanmakuSet> {
        Ok(self.0.load(&segments).await?.into())
    }
}

impl From<rubi_filter::danmaku::DanmakuOutput> for FfiDanmakuSet {
    fn from(out: rubi_filter::danmaku::DanmakuOutput) -> Self {
        let mut removed: Vec<String> = out.removed.iter().map(|(c, n)| format!("{} {n}", c.label())).collect();
        removed.sort();
        Self {
            items: out.kept.into_iter().map(Into::into).collect(),
            jump_marks: out.jump_marks.into_iter().map(Into::into).collect(),
            removed,
        }
    }
}

fn thread_of(root: &Comment, replies: &[Comment], limit: usize, total: u32) -> FfiCommentThread {
    let (rows, hidden) = flatten_thread(root, replies, limit);
    let rows = rows
        .iter()
        .filter_map(|r| {
            let c = if r.rpid == root.rpid { Some(root) } else { replies.iter().find(|c| c.rpid == r.rpid) };
            c.map(|c| FfiCommentRow::of(r, c))
        })
        .collect::<Vec<_>>();
    let shown = rows.len().saturating_sub(1) as u32;
    let hidden = (hidden as u32).max(total.saturating_sub(shown));
    FfiCommentThread { root_rpid: root.rpid, rows, hidden, total_replies: total }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_counts_unloaded_replies_as_hidden() {
        let root = Comment { rpid: 1, reply_count: 10, ..Default::default() };
        let preview = vec![Comment { rpid: 2, root: 1, parent: 1, ..Default::default() }];
        let t = thread_of(&root, &preview, usize::MAX, 10);
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.hidden, 9);
    }

    #[test]
    fn guest_core_opens_and_ranks_an_empty_pool() {
        let dir = std::env::temp_dir().join(format!("rubi-ffi-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let core = RubiCore::new(dir.to_string_lossy().into_owned(), None).unwrap();
        assert!(core.next_page(10).unwrap().is_empty());
        assert!(core.settings_json().unwrap().contains("rules"));
        drop(core);
        let _ = std::fs::remove_dir_all(dir);
    }
}
