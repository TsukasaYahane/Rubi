//! Guest-identity contract tests against the live API: they catch Bilibili changing a response
//! shape. Read-only, ignored by default; CI runs them nightly with `-- --ignored`.

use std::sync::Arc;

use rubi_api::{
    AuthApi, CommentApi, CommentSort, DanmakuApi, FeedApi, HttpClient, SearchApi, Session, SpaceApi, SteinApi, UpOrder,
    VideoApi, WebAuth, WebComment, WebDanmaku, WebFeed, WebSearch, WebSpace, WebVideo,
};
use rubi_model::VideoCard;

async fn guest() -> Arc<HttpClient> {
    let http = Arc::new(HttpClient::new(Session::default()).expect("client"));
    WebAuth::new(http.clone()).ensure_device_ids().await.expect("buvid");
    http
}

/// A current, popular video to run the per-video checks on.
async fn sample(http: &Arc<HttpClient>) -> VideoCard {
    let cards = WebFeed::new(http.clone()).popular(1, 5).await.expect("popular");
    cards.into_iter().find(|c| !c.bvid.is_empty()).expect("popular has a video")
}

#[tokio::test]
#[ignore = "live API"]
async fn feed_lists() {
    let http = guest().await;
    let feed = WebFeed::new(http.clone());
    assert!(!feed.home_rcmd(1, 10).await.expect("rcmd").is_empty());
    assert!(!feed.ranking(0).await.expect("ranking").is_empty());
    let card = sample(&http).await;
    assert!(card.aid > 0 && card.owner.mid > 0 && !card.title.is_empty());
    assert!(!feed.related(&card.bvid).await.expect("related").is_empty());
}

#[tokio::test]
#[ignore = "live API"]
async fn video_playback_and_danmaku() {
    let http = guest().await;
    let card = sample(&http).await;
    let video = WebVideo::new(http.clone());
    let detail = video.view(&card.bvid).await.expect("view");
    assert!(detail.cid > 0 && !detail.pages.is_empty());
    let play = video.playurl(&card.bvid, detail.cid, 32).await.expect("playurl");
    assert!(!play.video.is_empty() && !play.audio.is_empty() && play.duration_ms > 0);
    let danmaku = WebDanmaku::new(http.clone());
    danmaku.view(detail.cid).await.expect("danmaku view");
    danmaku.segment(detail.cid, 1).await.expect("danmaku segment");
    // b23.tv answers any BV id with a redirect, like a real share code: checks the hop only.
    let target = video.expand_short(&format!("https://b23.tv/{}", card.bvid)).await.expect("short link");
    assert!(target.is_some_and(|t| t.contains(&card.bvid)));
}

#[tokio::test]
#[ignore = "live API"]
async fn comments_and_search() {
    let http = guest().await;
    let card = sample(&http).await;
    let comments = WebComment::new(http.clone());
    let page = comments.page(card.aid, CommentSort::Hot, None).await.expect("comments");
    assert!(!page.threads.is_empty());
    let first = page.threads[0].root.rpid;
    assert_eq!(comments.root(card.aid, first).await.expect("thread root").rpid, first);
    // The cursor leads to a second page of other threads.
    if let Some(next) = page.next_offset.as_deref() {
        let more = comments.page(card.aid, CommentSort::Hot, Some(next)).await.expect("comments page 2");
        assert!(more.threads.iter().all(|t| t.root.rpid != first));
    }
    let search = WebSearch::new(http.clone());
    let words = search.hot_words(5).await.expect("hot words");
    // From s.search hotword (not the square fallback): it carries the commercial flag.
    assert!(words.first().is_some_and(|w| w.raw["stat_datas"]["is_commercial"].is_string()));
    search.videos("bilibili", 1, rubi_api::SearchOrder::Relevance).await.expect("search");
}

#[tokio::test]
#[ignore = "live API"]
async fn up_space() {
    let http = guest().await;
    let card = sample(&http).await;
    let space = WebSpace::new(http.clone());
    let profile = space.profile(card.owner.mid).await.expect("card");
    assert_eq!(profile.mid, card.owner.mid);
    // arc/search is often rejected by risk control from datacenter IPs; only the shape matters.
    match space.videos(card.owner.mid, 1, UpOrder::Pubdate).await {
        Ok(page) => assert!(page.total > 0),
        Err(rubi_api::ApiError::RiskControl { .. }) => eprintln!("arc/search: risk control, skipped"),
        Err(e) => panic!("arc/search: {e}"),
    }
}

#[tokio::test]
#[ignore = "live API"]
async fn login_handshake() {
    // Only the read-only steps: nothing is submitted.
    let http = guest().await;
    let auth = WebAuth::new(http.clone());
    let c = auth.login_captcha().await.expect("captcha");
    assert!(!c.token.is_empty() && !c.gt.is_empty() && !c.challenge.is_empty());
    let k = auth.login_key().await.expect("login key");
    assert_eq!(k.hash.len(), 16);
    assert!(k.key.contains("BEGIN PUBLIC KEY"));
}

#[tokio::test]
#[ignore = "live API"]
async fn interactive_video() {
    // A long-standing interactive video (2021); skipped if it ever disappears.
    let http = guest().await;
    let stein = rubi_api::WebStein::new(http.clone());
    let Ok(Some(gv)) = stein.graph_version("BV1vb4y1r7cg", 381841269).await else {
        eprintln!("interactive sample gone, skipped");
        return;
    };
    let start = stein.node("BV1vb4y1r7cg", gv, None).await.expect("start node");
    let q = start.questions.iter().find(|q| !q.choices.is_empty()).expect("a question");
    let first = &q.choices[0];
    assert!(first.edge_id > 0 && first.cid > 0);
    let next = stein.node("BV1vb4y1r7cg", gv, Some(first.edge_id)).await.expect("next node");
    assert_eq!(next.edge_id, first.edge_id);
}

#[tokio::test]
#[ignore = "live API"]
async fn region_feed() {
    // v2 main region 1008 (游戏): cards carry their uploader and play count.
    let http = guest().await;
    let region = WebFeed::new(http.clone()).region_feed(1008, 1, 5).await.expect("region feed");
    assert!(!region.is_empty());
    assert!(region.iter().all(|c| !c.bvid.is_empty() && c.owner.mid > 0 && c.stats.view > 0));
}
