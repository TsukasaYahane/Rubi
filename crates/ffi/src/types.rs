//! Flat, FFI-friendly records. Each mirrors a core type with only what the UI draws.

use rubi_api::{PlayUrl, Stream};
use rubi_model::{
    ChatMessage, ChatSession, Comment, Danmaku, JumpMark, Notice, ThreadRow, UnreadCounts, VideoCard, VideoRef,
};
use rubi_recommend::ScoreBreakdown;

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiVideoCard {
    pub bvid: String,
    pub aid: u64,
    pub title: String,
    pub cover: String,
    pub up_name: String,
    pub up_mid: u64,
    pub zone: Option<String>,
    pub duration_s: u32,
    pub views: u64,
    pub danmaku: u64,
    pub pubdate: i64,
    /// Short label of the strongest positive factor ("兴趣匹配", "关注 UP"…), empty for non-feed lists.
    pub reason: String,
    pub score: f32,
}

impl FfiVideoCard {
    pub fn of(c: &VideoCard, score: Option<&ScoreBreakdown>) -> Self {
        Self {
            bvid: c.bvid.clone(),
            aid: c.aid,
            title: rubi_api::plain_letters(&c.title),
            cover: rubi_api::https_image(&c.cover),
            up_name: c.owner.name.clone(),
            up_mid: c.owner.mid,
            zone: c.zone.as_ref().map(|z| z.name.clone()),
            duration_s: c.duration_s,
            views: c.stats.view,
            danmaku: c.stats.danmaku,
            pubdate: c.pubdate,
            reason: score.map(reason_label).unwrap_or_default(),
            score: score.map_or(0.0, |s| s.total),
        }
    }
}

pub fn factor_label(f: rubi_recommend::FactorId) -> &'static str {
    use rubi_recommend::FactorId::*;
    match f {
        Interest => "兴趣匹配",
        Follow => "关注 UP",
        Freshness => "新鲜",
        Topic => "话题上升",
        Quality => "质量高",
        Explore => "探索",
        Dislike => "不感兴趣",
        RepeatExposure => "重复曝光",
    }
}

fn reason_label(s: &ScoreBreakdown) -> String {
    s.parts
        .iter()
        .filter(|p| p.contribution > 0.0)
        .max_by(|a, b| a.contribution.total_cmp(&b.contribution))
        .map(|p| factor_label(p.factor).to_owned())
        .unwrap_or_default()
}

/// Everything the player page's 视频详情 block shows.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiVideoDetail {
    pub bvid: String,
    pub aid: u64,
    pub title: String,
    pub desc: String,
    pub up_name: String,
    pub up_mid: u64,
    pub zone: Option<String>,
    pub tags: Vec<String>,
    pub pubdate: i64,
    pub duration_s: u32,
    pub pages: u32,
    pub views: u64,
    pub danmaku: u64,
    pub replies: u64,
    pub likes: u64,
    pub coins: u64,
    pub favorites: u64,
}

impl FfiVideoDetail {
    pub fn of(c: &VideoCard, pages: u32) -> Self {
        Self {
            bvid: c.bvid.clone(),
            aid: c.aid,
            title: c.title.clone(),
            desc: c.desc.clone(),
            up_name: c.owner.name.clone(),
            up_mid: c.owner.mid,
            zone: c.zone.as_ref().map(|z| z.name.clone()),
            tags: c.tags.clone(),
            pubdate: c.pubdate,
            duration_s: c.duration_s,
            pages,
            views: c.stats.view,
            danmaku: c.stats.danmaku,
            replies: c.stats.reply,
            likes: c.stats.like,
            coins: c.stats.coin,
            favorites: c.stats.favorite,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiScorePart {
    pub factor: String,
    pub raw: f32,
    pub weight: f32,
    pub contribution: f32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiBreakdown {
    pub total: f32,
    pub parts: Vec<FfiScorePart>,
    pub source_adjust: f32,
    pub penalty_adjust: f32,
}

impl From<ScoreBreakdown> for FfiBreakdown {
    fn from(s: ScoreBreakdown) -> Self {
        Self {
            total: s.total,
            parts: s
                .parts
                .into_iter()
                .map(|p| FfiScorePart {
                    factor: factor_label(p.factor).to_owned(),
                    raw: p.raw,
                    weight: p.weight,
                    contribution: p.contribution,
                })
                .collect(),
            source_adjust: s.source_adjust,
            penalty_adjust: s.penalty_adjust,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiRefreshReport {
    pub kept: u32,
    pub blocked: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiFilterLogEntry {
    pub at: i64,
    pub rule_id: String,
    pub category: String,
    pub title: String,
    pub source: String,
    pub reason: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiStream {
    pub id: u32,
    pub url: String,
    pub backup_urls: Vec<String>,
    pub bandwidth: u64,
    pub codecs: String,
    pub width: u32,
    pub height: u32,
}

impl From<Stream> for FfiStream {
    fn from(s: Stream) -> Self {
        Self {
            id: s.id,
            url: s.url,
            backup_urls: s.backup_urls,
            bandwidth: s.bandwidth,
            codecs: s.codecs,
            width: s.width,
            height: s.height,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiQuality {
    pub qn: u32,
    pub label: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPlayback {
    pub cid: u64,
    pub duration_ms: u64,
    pub quality: u32,
    pub accept: Vec<FfiQuality>,
    pub video: Vec<FfiStream>,
    pub audio: Vec<FfiStream>,
    /// Headers ExoPlayer must send with every stream request.
    pub headers: Vec<FfiHeader>,
}

impl FfiPlayback {
    pub fn of(cid: u64, p: PlayUrl, headers: Vec<(String, String)>) -> Self {
        Self {
            cid,
            duration_ms: p.duration_ms,
            quality: p.quality,
            accept: p.accept.into_iter().map(|(qn, label)| FfiQuality { qn, label }).collect(),
            video: p.video.into_iter().map(Into::into).collect(),
            audio: p.audio.into_iter().map(Into::into).collect(),
            headers: headers.into_iter().map(|(name, value)| FfiHeader { name, value }).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiDanmaku {
    pub id: u64,
    pub progress_ms: u32,
    pub mode: u8,
    pub font_size: u32,
    pub color: u32,
    pub content: String,
}

impl From<Danmaku> for FfiDanmaku {
    fn from(d: Danmaku) -> Self {
        Self {
            id: d.id,
            progress_ms: d.progress_ms,
            mode: d.mode,
            font_size: d.font_size,
            color: d.color,
            content: d.content,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiJumpMark {
    pub target_ms: u32,
    pub label: String,
}

impl From<JumpMark> for FfiJumpMark {
    fn from(j: JumpMark) -> Self {
        Self { target_ms: j.target_ms, label: j.label }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiDanmakuSet {
    pub items: Vec<FfiDanmaku>,
    pub jump_marks: Vec<FfiJumpMark>,
    /// "category: count" lines for the 已过滤 summary.
    pub removed: Vec<String>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum FfiLane {
    Scroll,
    Top,
    Bottom,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPlacement {
    pub id: u64,
    pub lane: FfiLane,
    pub track: u32,
    pub start_ms: u32,
    pub width: f32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiCommentRow {
    pub rpid: u64,
    pub depth: u32,
    pub has_later_sibling: bool,
    pub reply_to: Option<String>,
    pub name: String,
    pub text: String,
    pub likes: u64,
    pub is_up: bool,
    pub ctime: i64,
    /// Emotes this comment uses that may be missing from the global set.
    pub emotes: Vec<FfiEmote>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiEmote {
    /// The token as it appears in text, e.g. `[doge]`.
    pub text: String,
    pub url: String,
}

impl FfiCommentRow {
    pub fn of(row: &ThreadRow, c: &Comment) -> Self {
        Self {
            rpid: c.rpid,
            depth: row.depth,
            has_later_sibling: row.has_later_sibling,
            reply_to: row.reply_to.clone(),
            name: c.name.clone(),
            text: c.text.clone(),
            likes: c.likes,
            is_up: c.is_up,
            ctime: c.ctime,
            emotes: c.emotes.iter().map(|(text, url)| FfiEmote { text: text.clone(), url: url.clone() }).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiCommentThread {
    pub root_rpid: u64,
    /// Fishbone rows, root first.
    pub rows: Vec<FfiCommentRow>,
    /// Replies not shown yet ("再看 5 条").
    pub hidden: u32,
    pub total_replies: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiCommentPage {
    pub pinned: Vec<FfiCommentRow>,
    pub threads: Vec<FfiCommentThread>,
    pub hidden_pinned: u32,
    pub next_offset: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiUnread {
    pub reply: u32,
    pub at: u32,
    pub like: u32,
    pub chat: u32,
}

impl From<UnreadCounts> for FfiUnread {
    fn from(u: UnreadCounts) -> Self {
        Self { reply: u.reply, at: u.at, like: u.like, chat: u.chat }
    }
}

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum FfiNoticeKind {
    Reply,
    At,
    Like,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiNotice {
    pub id: u64,
    pub from: String,
    pub text: String,
    pub context: String,
    pub uri: String,
    pub at: i64,
    /// The video this notice is about; open it with `RubiCore::resolve_video`.
    pub video: Option<FfiVideoRef>,
    /// The comment to show when opening the video; load it with `RubiCore::focus_thread`.
    pub comment: Option<FfiCommentRef>,
}

#[derive(Debug, Clone, Copy, uniffi::Record)]
pub struct FfiCommentRef {
    pub rpid: u64,
    pub root: u64,
}

impl From<rubi_model::CommentRef> for FfiCommentRef {
    fn from(c: rubi_model::CommentRef) -> Self {
        Self { rpid: c.rpid, root: c.root }
    }
}

impl From<Notice> for FfiNotice {
    fn from(n: Notice) -> Self {
        Self {
            id: n.id,
            from: n.from,
            text: n.text,
            context: n.context,
            uri: n.uri,
            at: n.at,
            video: n.video.map(Into::into),
            comment: n.comment.map(Into::into),
        }
    }
}

/// A video pointer: a BV id, or an av number / short link that needs `resolve_video` first.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum FfiVideoRef {
    Bvid {
        bvid: String,
    },
    Aid {
        aid: u64,
    },
    /// A share short link; `resolve_video` follows it.
    Short {
        url: String,
    },
}

impl From<VideoRef> for FfiVideoRef {
    fn from(v: VideoRef) -> Self {
        match v {
            VideoRef::Bvid(bvid) => FfiVideoRef::Bvid { bvid },
            VideoRef::Aid(aid) => FfiVideoRef::Aid { aid },
            VideoRef::Short(url) => FfiVideoRef::Short { url },
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSharedVideo {
    pub video: FfiVideoRef,
    /// Empty when the video was only mentioned by id in a text message.
    pub title: String,
    pub cover: String,
    pub author: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiNoticePage {
    pub items: Vec<FfiNotice>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiChatSession {
    pub talker_mid: u64,
    pub name: String,
    pub unread: u32,
    pub last_text: String,
    pub last_from_me: bool,
    pub last_at: i64,
    pub ack_seqno: u64,
}

impl From<ChatSession> for FfiChatSession {
    fn from(s: ChatSession) -> Self {
        Self {
            talker_mid: s.talker_mid,
            name: s.name,
            unread: s.unread,
            last_text: s.last_text,
            last_from_me: s.last_from_me,
            last_at: s.last_at,
            ack_seqno: s.ack_seqno,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiChatSessions {
    pub sessions: Vec<FfiChatSession>,
    pub hidden: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiChatMessage {
    pub seqno: u64,
    pub from_me: bool,
    pub text: String,
    pub at: i64,
    pub video: Option<FfiSharedVideo>,
}

impl FfiChatMessage {
    pub fn of(m: ChatMessage, talker: u64) -> Self {
        Self {
            seqno: m.seqno,
            from_me: m.sender_mid != talker,
            text: m.text,
            at: m.at,
            video: m.video.map(|v| FfiSharedVideo {
                video: v.video.into(),
                title: v.title,
                cover: v.cover,
                author: v.author,
            }),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSearchResult {
    pub cards: Vec<FfiVideoCard>,
    pub hidden: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiHotWords {
    pub words: Vec<String>,
    pub hidden: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiQrTicket {
    pub url: String,
    pub key: String,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum FfiQrState {
    Waiting,
    Scanned,
    Expired,
    /// Persist `session_json` encrypted (Keystore); pass it back to `RubiCore::new`.
    Success {
        session_json: String,
    },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiAccount {
    pub logged_in: bool,
    pub mid: u64,
    pub name: String,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum FfiSignal {
    Click,
    QuickExit,
    Completion { ratio: f32 },
    Rewatch,
    DislikeUp,
    DislikeZone,
    DislikeTag { tag: String },
    DislikeTitle,
}

/// What the current user has done to a video (for the player's action row).
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiVideoRelation {
    pub liked: bool,
    pub coins: u32,
    pub favorited: bool,
    pub following_up: bool,
}

impl From<rubi_model::VideoRelation> for FfiVideoRelation {
    fn from(r: rubi_model::VideoRelation) -> Self {
        Self { liked: r.liked, coins: r.coins, favorited: r.favorited, following_up: r.following_up }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiFavFolder {
    pub id: u64,
    pub title: String,
    pub count: u32,
    /// Whether the video asked about is already in this folder.
    pub contains: bool,
}

impl From<rubi_model::FavFolder> for FfiFavFolder {
    fn from(f: rubi_model::FavFolder) -> Self {
        Self { id: f.id, title: f.title, count: f.count, contains: f.contains }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiTriple {
    pub liked: bool,
    pub coined: bool,
    pub favorited: bool,
    /// Coins given by this request.
    pub coins: u32,
}

impl From<rubi_api::Triple> for FfiTriple {
    fn from(t: rubi_api::Triple) -> Self {
        Self { liked: t.liked, coined: t.coined, favorited: t.favorited, coins: t.coins }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiUpProfile {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub sign: String,
    pub level: u32,
    pub official: String,
    pub fans: u64,
    pub followings: u64,
    pub archives: u64,
    pub followed: bool,
}

impl From<rubi_model::UpProfile> for FfiUpProfile {
    fn from(p: rubi_model::UpProfile) -> Self {
        Self {
            mid: p.mid,
            name: p.name,
            face: p.face,
            sign: p.sign,
            level: p.level,
            official: p.official,
            fans: p.fans,
            followings: p.followings,
            archives: p.archives,
            followed: p.followed,
        }
    }
}

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum FfiUpOrder {
    Newest,
    MostViewed,
    MostFavorited,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiUpUploads {
    pub items: Vec<FfiVideoCard>,
    pub total: u32,
    /// 充电专属 videos removed from this page.
    pub hidden_charging: u32,
    pub has_more: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiHistoryItem {
    pub card: FfiVideoCard,
    /// Seconds watched; -1 means finished.
    pub progress_s: i64,
    pub viewed_at: i64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiHistoryPage {
    pub items: Vec<FfiHistoryItem>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiFavPage {
    pub items: Vec<FfiVideoCard>,
    pub has_more: bool,
}
