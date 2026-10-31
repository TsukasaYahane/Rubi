//! Guest-identity contract tests against the live API: they catch Bilibili changing a response
//! shape. Read-only, ignored by default; CI runs them nightly with `-- --ignored`.

use std::sync::Arc;

use rubi_api::{
    AuthApi, CommentApi, CommentSort, DanmakuApi, FeedApi, HttpClient, SearchApi, Session, SpaceApi, UpOrder, VideoApi,
    WebAuth, WebComment, WebDanmaku, WebFeed, WebSearch, WebSpace, WebVideo,
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
