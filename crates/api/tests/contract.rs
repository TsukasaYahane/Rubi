//! Guest-identity contract tests against the live API: they catch Bilibili changing a response
//! shape. Read-only, ignored by default; CI runs them nightly with `-- --ignored`.

use std::sync::Arc;

use rubi_api::{
    ArticleApi, AuthApi, CommentApi, CommentArea, CommentSort, DanmakuApi, FeedApi, HttpClient, SearchApi, Session,
    SpaceApi, SteinApi, UpOrder, VideoApi, WebArticle, WebAuth, WebComment, WebDanmaku, WebFeed, WebSearch, WebSpace,
    WebVideo,
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
    let page = comments.page(CommentArea::video(card.aid), CommentSort::Hot, None).await.expect("comments");
    assert!(!page.threads.is_empty());
    let first = page.threads[0].root.rpid;
    assert_eq!(comments.root(CommentArea::video(card.aid), first).await.expect("thread root").rpid, first);
    // The cursor leads to a second page of other threads.
    if let Some(next) = page.next_offset.as_deref() {
        let more =
            comments.page(CommentArea::video(card.aid), CommentSort::Hot, Some(next)).await.expect("comments page 2");
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
    match space.videos(card.owner.mid, 1, UpOrder::Pubdate, "").await {
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

/// A 专栏 reads from the article view; a 图文 from its opus detail.
#[tokio::test]
#[ignore = "live API"]
async fn articles_and_opus_read() {
    let http = guest().await;
    let api = WebArticle::new(http);
    // Captured on 2026-11-01: an old article that never moved to a 图文 page, and a news 图文.
    let a = api.article(52157219).await.expect("article view");
    assert!(!a.title.is_empty() && !a.blocks.is_empty());
    let o = api.opus("1265393890559524872").await.expect("opus detail");
    assert!(!o.title.is_empty() && !o.blocks.is_empty());
}

/// 直播: lists, a room, its streams, and a few seconds of its message channel.
#[tokio::test]
#[ignore = "live API"]
async fn live_rooms_streams_and_channel() {
    use rubi_api::{ChannelJoin, ChannelOpener, LiveApi, WebChannels, WebLive};
    let http = guest().await;
    let live = WebLive::new(http);
    let rec = live.recommended().await.expect("recommended");
    assert!(!rec.is_empty() && rec[0].room_id > 0 && !rec[0].title.is_empty());
    let areas = live.areas().await.expect("areas");
    assert!(areas.len() > 5 && !areas[0].children.is_empty());
    let (rooms, _) = live.area_rooms(areas[0].id, 0, 1).await.expect("area rooms");
    assert!(!rooms.is_empty());
    let room = live.room(rec[0].room_id).await.expect("room");
    assert!(!room.uname.is_empty());
    let play = live.play(room.room_id, rubi_api::BEST_LIVE_QN).await.expect("play");
    assert!(play.live && !play.streams.is_empty() && !play.qualities.is_empty(), "{play:?}");
    let server = live.danmu_server(room.room_id).await.expect("danmu info");
    let (uid, buvid) = live.identity();
    let join = ChannelJoin { room_id: room.room_id, uid, buvid: &buvid, token: &server.token };
    let mut socket = WebChannels.open(&server.urls, &join).await.expect("channel");
    let got = tokio::time::timeout(std::time::Duration::from_secs(20), socket.next()).await;
    let msgs = got.expect("a message within 20 s").expect("channel read").expect("open");
    assert!(msgs.iter().all(|m| m["cmd"].is_string()));
    eprintln!(
        "{} rooms, {} streams, first cmds: {:?}",
        rec.len(),
        play.streams.len(),
        msgs.iter().map(|m| m["cmd"].as_str().unwrap_or_default().to_owned()).collect::<Vec<_>>()
    );
    // A busy room talks within seconds; count what the room page would show.
    let mut events = Vec::new();
    let until = tokio::time::Instant::now() + std::time::Duration::from_secs(12);
    while let Ok(Ok(Some(batch))) = tokio::time::timeout_at(until, socket.next()).await {
        events.extend(batch.iter().filter_map(rubi_api::event_of));
    }
    eprintln!("{} events, e.g. {:?}", events.len(), events.iter().take(6).collect::<Vec<_>>());
    socket.close().await;
}

/// 赛事: the schedule and its games.
#[tokio::test]
#[ignore = "live API"]
async fn esports_schedule() {
    use rubi_api::{EsportsApi, WebEsports};
    let api = WebEsports::new(guest().await);
    assert!(api.games().await.expect("games").len() > 5);
    let page = api.matches(0, "2026-11-01", "2026-11-03", 1, 10).await.expect("matches");
    assert!(page.total > 0 && !page.matches.is_empty());
}

/// UP search finds a club by its alias: 「TES」 is 滔搏电子竞技俱乐部.
#[tokio::test]
#[ignore = "live API"]
async fn user_search_knows_aliases() {
    let hits = WebSearch::new(guest().await).users("TES", 1).await.expect("user search");
    assert!(
        hits.users.iter().take(3).any(|u| u.name.contains("滔搏")),
        "{:?}",
        hits.users.iter().map(|u| &u.name).collect::<Vec<_>>()
    );
}

/// 影视专区: conditions, index, a season and its first episode's rights (read only).
#[tokio::test]
#[ignore = "live API"]
async fn pgc_zones_read() {
    use rubi_api::{PgcApi, WebPgc};
    let api = WebPgc::new(guest().await);
    assert!(!api.timeline(1).await.expect("timeline").is_empty());
    for st in [1, 2, 3, 5] {
        let c = api.conditions(st).await.expect("conditions");
        assert!(!c.orders.is_empty(), "season_type {st}");
        let (list, _) = api.index(st, &[], &c.orders[0].0, 1).await.expect("index");
        assert!(!list.is_empty(), "season_type {st}");
    }
    let d = api.season(835).await.expect("season");
    assert!(!d.episodes.is_empty());
    let p = api.play(d.episodes[0].ep_id, 80).await.expect("play");
    eprintln!(
        "guest ep {}: preview {} drm {} streams {}",
        d.episodes[0].ep_id,
        p.preview,
        p.drm,
        p.url.map(|u| u.video.len()).unwrap_or(0)
    );
}

/// 投屏: a video also comes as one MP4 (only the playurl answer is read).
#[tokio::test]
#[ignore = "live API"]
async fn single_file_for_casting() {
    let http = guest().await;
    let card = sample(&http).await;
    let video = WebVideo::new(http);
    let cid = video.view(&card.bvid).await.expect("view").cid;
    let f = video.single_file(&card.bvid, cid, 80).await.expect("single file");
    assert!(f.url.starts_with("https://") && f.quality > 0, "{f:?}");
    eprintln!("single file at qn {} of {:?}", f.quality, f.accept.iter().map(|a| a.0).collect::<Vec<_>>());
}
