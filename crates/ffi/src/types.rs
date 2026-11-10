//! Flat, FFI-friendly records. Each mirrors a core type with only what the UI draws.

use rubi_core::ScoreBreakdown;
use rubi_core::Stream;
use rubi_model::{
    ChatMessage, ChatSession, Comment, Danmaku, JumpMark, Notice, ThreadRow, UnreadCounts, VideoCard, VideoRef,
    without_reply_prefix,
};

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
    /// A portrait (taller than wide) video, where the list said; false when it did not.
    pub portrait: bool,
}

impl FfiVideoCard {
    pub fn of(c: &VideoCard, score: Option<&ScoreBreakdown>) -> Self {
        Self {
            bvid: c.bvid.clone(),
            aid: c.aid,
            title: rubi_core::plain_letters(&c.title),
            cover: rubi_core::https_image(&c.cover),
            up_name: c.owner.name.clone(),
            up_mid: c.owner.mid,
            zone: c.zone.as_ref().map(|z| z.name.clone()),
            duration_s: c.duration_s,
            views: c.stats.view,
            danmaku: c.stats.danmaku,
            pubdate: c.pubdate,
            reason: score.map(reason_label).unwrap_or_default(),
            score: score.map_or(0.0, |s| s.total),
            portrait: rubi_core::portrait_of(&c.raw).unwrap_or(false),
        }
    }
}

pub fn factor_label(f: rubi_core::FactorId) -> &'static str {
    use rubi_core::FactorId::*;
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
    s.main_factor().map(factor_label).unwrap_or_default().to_owned()
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
    /// 分P, in order; one entry for a single-part video.
    pub parts: Vec<FfiPart>,
    pub season: Option<FfiSeason>,
    /// Taller than wide: opens in the 竖屏 feed.
    pub portrait: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPart {
    pub cid: u64,
    pub title: String,
    pub duration_s: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiEpisode {
    pub bvid: String,
    pub cid: u64,
    pub title: String,
    pub cover: String,
    pub duration_s: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSeasonSection {
    pub title: String,
    pub episodes: Vec<FfiEpisode>,
}

/// The 合集 a video is in.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSeason {
    pub id: u64,
    pub title: String,
    pub sections: Vec<FfiSeasonSection>,
}

impl FfiVideoDetail {
    pub fn of(info: &rubi_core::VideoInfo) -> Self {
        let c = &info.card;
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
            pages: info.parts.len().max(1) as u32,
            views: c.stats.view,
            danmaku: c.stats.danmaku,
            replies: c.stats.reply,
            likes: c.stats.like,
            coins: c.stats.coin,
            favorites: c.stats.favorite,
            parts: info
                .parts
                .iter()
                .map(|p| FfiPart { cid: p.cid, title: p.part.clone(), duration_s: p.duration_s })
                .collect(),
            season: info.season.as_ref().map(|s| FfiSeason {
                id: s.id,
                title: s.title.clone(),
                sections: s
                    .sections
                    .iter()
                    .map(|sec| FfiSeasonSection {
                        title: sec.title.clone(),
                        episodes: sec
                            .episodes
                            .iter()
                            .map(|e| FfiEpisode {
                                bvid: e.bvid.clone(),
                                cid: e.cid,
                                title: e.title.clone(),
                                cover: e.cover.clone(),
                                duration_s: e.duration_s,
                            })
                            .collect(),
                    })
                    .collect(),
            }),
            portrait: rubi_core::portrait_of(&info.card.raw).unwrap_or(false),
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

impl From<rubi_core::Playback> for FfiPlayback {
    fn from(p: rubi_core::Playback) -> Self {
        Self {
            cid: p.cid,
            duration_ms: p.url.duration_ms,
            quality: p.url.quality,
            accept: p.url.accept.into_iter().map(|(qn, label)| FfiQuality { qn, label }).collect(),
            video: p.url.video.into_iter().map(Into::into).collect(),
            audio: p.url.audio.into_iter().map(Into::into).collect(),
            headers: p.headers.into_iter().map(|(name, value)| FfiHeader { name, value }).collect(),
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
    pub pictures: Vec<FfiPicture>,
    /// Video links in `text`; draw `token` as the title and open the video on tap.
    pub video_links: Vec<FfiVideoLink>,
    /// 专栏 id of the note (笔记) this comment carries; 0 for ordinary comments.
    pub note_cvid: u64,
    /// The signed-in user has liked it.
    pub liked: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPicture {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiVideoLink {
    pub token: String,
    pub title: String,
    pub video: FfiVideoRef,
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
            text: if row.reply_to.is_some() { without_reply_prefix(&c.text).to_owned() } else { c.text.clone() },
            likes: c.likes,
            is_up: c.is_up,
            ctime: c.ctime,
            emotes: c.emotes.iter().map(|(text, url)| FfiEmote { text: text.clone(), url: url.clone() }).collect(),
            pictures: c
                .pictures
                .iter()
                .map(|p| FfiPicture { url: p.url.clone(), width: p.width, height: p.height })
                .collect(),
            video_links: c
                .video_links
                .iter()
                .map(|l| FfiVideoLink { token: l.token.clone(), title: l.title.clone(), video: l.video.clone().into() })
                .collect(),
            note_cvid: c.note_cvid,
            liked: c.liked,
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
    pub private: bool,
    /// 默认收藏夹: cannot be deleted.
    pub default: bool,
}

impl From<rubi_model::FavFolder> for FfiFavFolder {
    fn from(f: rubi_model::FavFolder) -> Self {
        Self { id: f.id, title: f.title, count: f.count, contains: f.contains, private: f.private, default: f.default }
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

impl From<rubi_core::Triple> for FfiTriple {
    fn from(t: rubi_core::Triple) -> Self {
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiLoginCaptcha {
    pub token: String,
    pub gt: String,
    pub challenge: String,
}

/// What the Geetest widget returned (its challenge may differ from the one it was given).
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSolvedCaptcha {
    pub token: String,
    pub challenge: String,
    pub validate: String,
    pub seccode: String,
}

impl From<FfiSolvedCaptcha> for rubi_core::SolvedCaptcha {
    fn from(c: FfiSolvedCaptcha) -> Self {
        Self { token: c.token, challenge: c.challenge, validate: c.validate, seccode: c.seccode }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiLoginKey {
    pub hash: String,
    pub public_key_pem: String,
}

/// A node of an interactive video and the question asked near the end of its part.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSteinNode {
    pub edge_id: u64,
    pub title: String,
    pub is_leaf: bool,
    pub question: Option<FfiSteinQuestion>,
    /// Variables the video shows (name, value), e.g. a score.
    pub vars: Vec<FfiSteinVar>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSteinQuestion {
    /// 0: no options shown, jump on at the end; otherwise show the options.
    pub kind: u8,
    pub before_end_ms: u32,
    pub time_limit_ms: Option<u32>,
    pub pause: bool,
    pub title: String,
    /// Only the choices whose conditions hold.
    pub choices: Vec<FfiSteinChoice>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSteinChoice {
    pub edge_id: u64,
    pub cid: u64,
    pub option: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiSteinVar {
    pub name: String,
    pub value: f64,
}

impl FfiSteinNode {
    pub fn of(n: &rubi_core::SteinNode, vars: Vec<(String, f64)>) -> Self {
        // The first question that has choices; trailing entries are the ending module.
        let question = n.questions.iter().find(|q| !q.choices.is_empty()).map(|q| FfiSteinQuestion {
            kind: q.kind,
            before_end_ms: q.before_end_ms,
            time_limit_ms: q.time_limit_ms,
            pause: q.pause,
            title: q.title.clone(),
            choices: q
                .choices
                .iter()
                .map(|c| FfiSteinChoice {
                    edge_id: c.edge_id,
                    cid: c.cid,
                    option: c.option.clone(),
                    is_default: c.is_default,
                })
                .collect(),
        });
        Self {
            edge_id: n.edge_id,
            title: n.title.clone(),
            is_leaf: n.is_leaf,
            question,
            vars: vars.into_iter().map(|(name, value)| FfiSteinVar { name, value }).collect(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiFollowPage {
    pub cards: Vec<FfiVideoCard>,
    /// Offset of the next page; `None` at the end.
    pub next: Option<String>,
    /// Cards this page's filters removed.
    pub hidden: u32,
}

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum FfiPlaylistMode {
    Sequential,
    LoopAll,
    LoopOne,
    Shuffle,
}

impl From<rubi_core::PlaylistMode> for FfiPlaylistMode {
    fn from(m: rubi_core::PlaylistMode) -> Self {
        use rubi_core::PlaylistMode::*;
        match m {
            Sequential => Self::Sequential,
            LoopAll => Self::LoopAll,
            LoopOne => Self::LoopOne,
            Shuffle => Self::Shuffle,
        }
    }
}

impl From<FfiPlaylistMode> for rubi_core::PlaylistMode {
    fn from(m: FfiPlaylistMode) -> Self {
        match m {
            FfiPlaylistMode::Sequential => Self::Sequential,
            FfiPlaylistMode::LoopAll => Self::LoopAll,
            FfiPlaylistMode::LoopOne => Self::LoopOne,
            FfiPlaylistMode::Shuffle => Self::Shuffle,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiFollowedUp {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub has_update: bool,
}

impl From<rubi_model::FollowedUp> for FfiFollowedUp {
    fn from(u: rubi_model::FollowedUp) -> Self {
        Self { mid: u.mid, name: u.name, face: u.face, has_update: u.has_update }
    }
}

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum FfiDynamicKind {
    Video,
    Word,
    Draw,
    Article,
    Opus,
    Forward,
    Other,
}

/// What a post shows; a repost's original is one level of this (records cannot nest).
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiDynamicBody {
    pub kind: FfiDynamicKind,
    pub author_name: String,
    pub text: String,
    pub title: String,
    pub pictures: Vec<String>,
    pub video: Option<FfiVideoCard>,
    pub link: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiDynamic {
    pub id: String,
    pub author_mid: u64,
    pub author_face: String,
    pub published_at: i64,
    pub body: FfiDynamicBody,
    pub orig: Option<FfiDynamicBody>,
    /// Where its comments are; `None` when it has none.
    pub comments: Option<FfiCommentArea>,
    pub comment_count: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiDynamicPage {
    pub items: Vec<FfiDynamic>,
    pub next: Option<String>,
}

impl FfiDynamicBody {
    fn of(d: &rubi_model::Dynamic) -> Self {
        use rubi_model::DynamicKind as K;
        Self {
            kind: match d.kind {
                K::Video => FfiDynamicKind::Video,
                K::Word => FfiDynamicKind::Word,
                K::Draw => FfiDynamicKind::Draw,
                K::Article => FfiDynamicKind::Article,
                K::Opus => FfiDynamicKind::Opus,
                K::Forward => FfiDynamicKind::Forward,
                K::Other => FfiDynamicKind::Other,
            },
            author_name: d.author_name.clone(),
            text: d.text.clone(),
            title: d.title.clone(),
            pictures: d.pictures.clone(),
            video: d.video.as_ref().map(|v| FfiVideoCard::of(v, None)),
            link: d.link.clone(),
        }
    }
}

impl FfiDynamic {
    pub fn of(d: &rubi_model::Dynamic) -> Self {
        Self {
            id: d.id.clone(),
            author_mid: d.author_mid,
            author_face: d.author_face.clone(),
            published_at: d.published_at,
            body: FfiDynamicBody::of(d),
            orig: d.orig.as_deref().map(FfiDynamicBody::of),
            comments: (d.comment_kind > 0 && d.comment_oid > 0)
                .then_some(FfiCommentArea { kind: d.comment_kind, oid: d.comment_oid }),
            comment_count: d.comment_count,
        }
    }
}

impl From<&rubi_core::ThreadView> for FfiCommentThread {
    fn from(t: &rubi_core::ThreadView) -> Self {
        Self {
            root_rpid: t.root_rpid,
            rows: t.rows.iter().map(|(row, c)| FfiCommentRow::of(row, c)).collect(),
            hidden: t.hidden,
            total_replies: t.total_replies,
        }
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum FfiArticleBlock {
    Heading {
        text: String,
    },
    Text {
        text: String,
    },
    /// A quote or list item.
    Quote {
        text: String,
    },
    Pictures {
        pictures: Vec<FfiPicture>,
    },
    Line,
}

/// A 专栏 / 图文 ready to read.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiArticle {
    pub id: String,
    pub title: String,
    pub author_name: String,
    pub author_mid: u64,
    pub author_face: String,
    pub published_at: i64,
    pub cover: String,
    pub blocks: Vec<FfiArticleBlock>,
    /// Where its comments are; `None` when it has none.
    pub comments: Option<FfiCommentArea>,
    /// Promotion cards taken out of the body.
    pub hidden: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiDownloadState {
    Queued,
    Running,
    Paused,
    Done,
    Failed,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiDownload {
    pub card: FfiVideoCard,
    pub quality: u32,
    pub state: FfiDownloadState,
    /// Why it failed.
    pub error: Option<String>,
    pub done_bytes: u64,
    /// An estimate until both streams have answered.
    pub total_bytes: u64,
    pub created_at: i64,
}

/// A finished download's files.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiLocalCopy {
    pub video_path: String,
    pub audio_path: Option<String>,
}

/// A comment area: a video's, an article's… (`kind` is B站's `type`).
#[derive(Debug, Clone, Copy, uniffi::Record)]
pub struct FfiCommentArea {
    pub kind: u32,
    pub oid: u64,
}

/// Where watching a video can pick up again.
#[derive(Debug, Clone, Copy, uniffi::Record)]
pub struct FfiResumePoint {
    pub cid: u64,
    pub position_ms: u64,
}

/// A followed UP in the 全部关注 list.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiFollowing {
    pub mid: u64,
    pub name: String,
    pub face: String,
    /// Their latest post seen (Unix seconds); 0 when none since `checked_since`.
    pub last_post_at: i64,
    pub has_update: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiFollowings {
    pub ups: Vec<FfiFollowing>,
    /// How far back posts were read (Unix seconds).
    pub checked_since: i64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiLiveRoom {
    pub room_id: u64,
    pub title: String,
    pub cover: String,
    pub uid: u64,
    pub uname: String,
    pub face: String,
    pub online: u64,
    pub area: String,
    pub live: bool,
}

impl From<rubi_model::LiveRoom> for FfiLiveRoom {
    fn from(r: rubi_model::LiveRoom) -> Self {
        Self {
            room_id: r.room_id,
            title: r.title,
            cover: r.cover,
            uid: r.uid,
            uname: r.uname,
            face: r.face,
            online: r.online,
            area: r.area,
            live: r.live,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiLiveRoomPage {
    pub rooms: Vec<FfiLiveRoom>,
    pub has_more: bool,
}

/// A parent 直播 area (网游, 手游…).
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiLiveArea {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiLiveQuality {
    pub qn: u32,
    pub name: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiLiveStream {
    pub url: String,
    /// HLS (`.m3u8`); otherwise FLV.
    pub hls: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiLivePlay {
    pub live: bool,
    pub qn: u32,
    pub qualities: Vec<FfiLiveQuality>,
    /// Most suitable first; try the next when one fails.
    pub streams: Vec<FfiLiveStream>,
    pub headers: Vec<FfiHeader>,
}

/// What the room page shows of each kind of extra; `true` holds it back.
#[derive(Debug, Clone, Copy, uniffi::Record)]
pub struct FfiLiveFilter {
    pub gifts: bool,
    pub popups: bool,
    pub effects: bool,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum FfiLiveEvent {
    Danmaku { uname: String, text: String, color: u32 },
    Gift { text: String },
    Popup { text: String },
    Effect { text: String },
    Online { count: u64 },
    Status { live: bool },
}

impl From<rubi_model::LiveEvent> for FfiLiveEvent {
    fn from(e: rubi_model::LiveEvent) -> Self {
        use rubi_model::LiveEvent as E;
        match e {
            E::Danmaku { uname, text, color } => Self::Danmaku { uname, text, color },
            E::Gift { text } => Self::Gift { text },
            E::Popup { text } => Self::Popup { text },
            E::Effect { text } => Self::Effect { text },
            E::Online { count } => Self::Online { count },
            E::Status { live } => Self::Status { live },
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiLiveBatch {
    pub events: Vec<FfiLiveEvent>,
    pub blocked_gifts: u32,
    pub blocked_popups: u32,
    pub blocked_effects: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiMatchState {
    Upcoming,
    Live,
    Done,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiTeam {
    pub name: String,
    pub logo: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiMatch {
    pub id: u64,
    pub season: String,
    pub stage: String,
    pub start_at: i64,
    pub home: FfiTeam,
    pub away: FfiTeam,
    pub home_score: u32,
    pub away_score: u32,
    pub state: FfiMatchState,
    pub live_room: u64,
    pub replay_bvid: String,
    pub subscribed: bool,
}

impl From<rubi_model::EsportsMatch> for FfiMatch {
    fn from(m: rubi_model::EsportsMatch) -> Self {
        use rubi_model::MatchState as S;
        let team = |t: rubi_model::Team| FfiTeam { name: t.name, logo: t.logo };
        Self {
            id: m.id,
            season: m.season,
            stage: m.stage,
            start_at: m.start_at,
            home: team(m.home),
            away: team(m.away),
            home_score: m.home_score,
            away_score: m.away_score,
            state: match m.state {
                S::Upcoming => FfiMatchState::Upcoming,
                S::Live => FfiMatchState::Live,
                S::Done => FfiMatchState::Done,
            },
            live_room: m.live_room,
            replay_bvid: m.replay_bvid,
            subscribed: m.subscribed,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiMatchPage {
    pub matches: Vec<FfiMatch>,
    pub has_more: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiEsportsGame {
    pub id: u32,
    pub title: String,
}

/// Portrait videos to play next.
#[derive(Debug, Clone, uniffi::Record)]
pub struct FfiPortraitPick {
    pub cards: Vec<FfiVideoCard>,
    /// Risk control holds the sorting back; these are what was ready.
    pub limited: bool,
}
