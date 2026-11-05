//! Offline timing of the ranking against the budget in docs/design.md §10. Run with:
//! `cargo test -p rubi-recommend --test stress --release -- --ignored --nocapture`

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use rubi_model::{Owner, SourceKind, Stats, VideoCard};
use rubi_recommend::{Candidate, FeedEngine, InterestProfile, ScoringContext, SourceWeights, Weights, default_factors};

const NOW: i64 = 1_800_000_000;

fn candidates(n: u64) -> Vec<Candidate> {
    (0..n)
        .map(|i| Candidate {
            card: VideoCard {
                bvid: format!("BV{i}"),
                owner: Owner { mid: i % 50, name: format!("up{}", i % 50) },
                tags: vec![format!("tag{}", i % 40), format!("tag{}", i % 7)],
                pubdate: NOW - (i as i64 % 72) * 3_600,
                stats: Stats { view: 10_000 + i, like: 400, coin: 100, favorite: 150, danmaku: 50, reply: 50 },
                sources: vec![SourceKind::Popular],
                ..Default::default()
            },
            penalty: 1.0,
        })
        .collect()
}

#[test]
#[ignore = "stress"]
fn ranking_keeps_to_its_budget() {
    let engine = FeedEngine::new(default_factors(), Weights::default(), SourceWeights::default());
    let profile = InterestProfile::default();
    let followed: HashSet<u64> = (0..10).collect();
    let exposures: HashMap<String, u32> = (0..250).map(|i| (format!("BV{i}"), 2)).collect();
    let ctx = ScoringContext { now: NOW, profile: &profile, followed: &followed, exposures: &exposures };
    for n in [500, 5_000] {
        let mut worst = Duration::ZERO;
        for _ in 0..20 {
            let batch = candidates(n);
            let started = Instant::now();
            let ranked = engine.rank(batch, &ctx, n as usize);
            worst = worst.max(started.elapsed());
            assert_eq!(ranked.len(), n as usize);
        }
        eprintln!("rank {n}: worst of 20 runs {worst:?}");
        if n == 500 {
            assert!(worst <= Duration::from_millis(20), "budget is 20 ms, took {worst:?}");
        }
    }
}
