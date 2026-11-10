//! What a 动态 card shows, without touching XAML.

use rubi_model::{Dynamic, DynamicKind, VideoCard};

use crate::state::library::{age, count};

/// Pictures shown in a card.
pub const PICTURES: usize = 3;

/// The video a card opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clip {
    pub bvid: String,
    pub title: String,
    pub owner: String,
    pub cover: String,
    pub line: String,
}

impl Clip {
    fn of(card: &VideoCard) -> Option<Self> {
        card.is_video().then(|| Self {
            bvid: card.bvid.clone(),
            title: card.title.clone(),
            owner: card.owner.name.clone(),
            cover: card.cover.clone(),
            line: format!("{} 播放", count(card.stats.view)),
        })
    }
}

/// A post, flattened to what its card shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub id: String,
    pub face: String,
    pub name: String,
    pub line: String,
    pub text: String,
    pub clip: Option<Clip>,
    pub pictures: Vec<String>,
    /// How much further there is than the pictures shown.
    pub more_pictures: usize,
    /// The reposted post, in a line.
    pub quote: String,
    /// Where the card opens when it has no video.
    pub link: String,
    /// The link is an article the app can show.
    pub readable: bool,
    /// Its comment area (`type`, `oid`), if it has one.
    pub comments: Option<(u32, u64)>,
    /// What the comment button says.
    pub comment_label: String,
    pub comment_count: u64,
}

fn kind_label(kind: DynamicKind) -> &'static str {
    match kind {
        DynamicKind::Video => "投稿了视频",
        DynamicKind::Article | DynamicKind::Opus => "发布了图文",
        DynamicKind::Forward => "转发",
        _ => "动态",
    }
}

fn joined(title: &str, text: &str) -> String {
    match (title.is_empty(), text.is_empty()) {
        (false, false) => format!("{title}\n{text}"),
        (false, true) => title.to_owned(),
        _ => text.to_owned(),
    }
}

impl Post {
    pub fn of(post: Dynamic, now: i64) -> Self {
        let orig = post.orig.as_deref();
        let clip = post.video.as_ref().or(orig.and_then(|o| o.video.as_ref())).and_then(Clip::of);
        let quote = orig.map_or_else(String::new, |o| {
            let body = joined(&o.title, &o.text).replace('\n', " ");
            format!("@{}：{body}", o.author_name)
        });
        let mut pictures = post.pictures;
        if pictures.is_empty()
            && let Some(orig) = orig
        {
            pictures.clone_from(&orig.pictures);
        }
        let more_pictures = pictures.len().saturating_sub(PICTURES);
        pictures.truncate(PICTURES);
        let readable = matches!(post.kind, DynamicKind::Article | DynamicKind::Opus);
        let comments = (post.comment_kind > 0 && post.comment_oid > 0).then_some((post.comment_kind, post.comment_oid));
        let comment_label =
            if post.comment_count > 0 { format!("评论 {}", count(post.comment_count)) } else { "评论".to_owned() };
        let link = if post.link.is_empty() { orig.map_or_else(String::new, |o| o.link.clone()) } else { post.link };
        Self {
            id: post.id,
            face: post.author_face,
            name: post.author_name,
            line: format!("{} · {}", age(post.published_at, now), kind_label(post.kind)),
            text: joined(&post.title, &post.text),
            clip,
            pictures,
            more_pictures,
            quote,
            link,
            readable,
            comments,
            comment_label,
            comment_count: post.comment_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;

    fn post() -> Dynamic {
        Dynamic { author_name: "猫".into(), published_at: NOW - 7_200, ..Dynamic::default() }
    }

    #[test]
    fn a_card_says_who_and_when() {
        let shown = Post::of(Dynamic { kind: DynamicKind::Video, ..post() }, NOW);
        assert_eq!(shown.line, "2 小时前 · 投稿了视频");
        assert_eq!(shown.name, "猫");
    }

    #[test]
    fn title_and_text_share_the_body() {
        let shown = Post::of(Dynamic { title: "题".into(), text: "文".into(), ..post() }, NOW);
        assert_eq!(shown.text, "题\n文");
        assert_eq!(Post::of(Dynamic { text: "文".into(), ..post() }, NOW).text, "文");
    }

    #[test]
    fn a_repost_shows_what_it_reposted() {
        let orig = Dynamic {
            author_name: "狗".into(),
            text: "原文\n第二行".into(),
            video: Some(VideoCard { bvid: "BV1".into(), ..VideoCard::default() }),
            pictures: vec!["a".into()],
            link: "https://t".into(),
            ..Dynamic::default()
        };
        let shown = Post::of(Dynamic { orig: Some(Box::new(orig)), ..post() }, NOW);
        assert_eq!(shown.quote, "@狗：原文 第二行");
        assert_eq!(shown.clip.map(|c| c.bvid).as_deref(), Some("BV1"));
        assert_eq!(shown.pictures, ["a"]);
        assert_eq!(shown.link, "https://t");
    }

    #[test]
    fn pictures_past_three_are_counted() {
        let pictures = (0..5).map(|n| n.to_string()).collect();
        let shown = Post::of(Dynamic { pictures, ..post() }, NOW);
        assert_eq!(shown.pictures.len(), PICTURES);
        assert_eq!(shown.more_pictures, 2);
    }

    #[test]
    fn a_card_knows_its_comments() {
        let shown = Post::of(Dynamic { comment_kind: 17, comment_oid: 1265, comment_count: 12_400, ..post() }, NOW);
        assert_eq!(shown.comments, Some((17, 1265)));
        assert_eq!(shown.comment_label, "评论 1.2万");
        let bare = Post::of(post(), NOW);
        assert_eq!((bare.comments, bare.comment_label.as_str()), (None, "评论"));
    }

    #[test]
    fn a_card_without_a_bvid_opens_no_video() {
        let shown = Post::of(Dynamic { video: Some(VideoCard::default()), ..post() }, NOW);
        assert_eq!(shown.clip, None);
    }
}
