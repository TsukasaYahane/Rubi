//! Offline timing of the filters over large inputs. Run with:
//! `cargo test -p rubi-filter --test stress --release -- --ignored --nocapture`

use std::time::Instant;

use rubi_filter::RuleSet;
use rubi_filter::danmaku::process;
use rubi_model::{Danmaku, Owner, VideoCard};

const NOW: i64 = 1_800_000_000;

#[test]
#[ignore = "stress"]
fn a_hundred_thousand_danmaku() {
    let opts = RuleSet::builtin().danmaku_options().unwrap();
    let dms: Vec<Danmaku> = (0..100_000u32)
        .map(|i| Danmaku {
            id: i as u64 + 1,
            progress_ms: i % 1_200_000,
            mode: 1,
            font_size: 25,
            color: 0xFFFFFF,
            mid_hash: format!("{:x}", i % 5_000),
            content: format!("第{}条弹幕，哈哈哈哈", i % 300),
            ctime: NOW,
            ..Default::default()
        })
        .collect();
    let before = dms.len();
    let started = Instant::now();
    let out = process(dms, Vec::new(), 1_200_000, &opts);
    let took = started.elapsed();
    eprintln!("danmaku: {before} in, {} kept, {took:?}", out.kept.len());
    assert!(out.kept.len() <= before);
    assert!(took.as_secs() < 2, "took {took:?}");
}

#[test]
#[ignore = "stress"]
fn a_hundred_thousand_cards() {
    let pipeline = RuleSet::builtin().card_pipeline(None, || NOW).unwrap();
    let cards: Vec<VideoCard> = (0..100_000u64)
        .map(|i| VideoCard {
            bvid: format!("BV{i}"),
            aid: i + 1,
            title: format!("普通视频标题 {i}"),
            desc: "简介".repeat(20),
            owner: Owner { mid: i % 1_000, name: format!("up{}", i % 1_000) },
            pubdate: NOW - 3_600,
            ..Default::default()
        })
        .collect();
    let started = Instant::now();
    let blocked = cards.iter().filter(|c| pipeline.judge(c).is_blocked()).count();
    let took = started.elapsed();
    eprintln!("cards: {} judged, {blocked} blocked, {took:?}", cards.len());
    assert!(took.as_secs() < 2, "took {took:?}");
}
