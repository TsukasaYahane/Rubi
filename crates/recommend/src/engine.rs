use std::collections::{BTreeMap, HashMap, HashSet};

use rubi_model::{SourceKind, VideoCard};
use serde::{Deserialize, Serialize};

use crate::factors::{Factor, FactorId, default_factors};
use crate::profile::InterestProfile;

/// How much a source's weight above / below 1.0 moves the total score.
const SOURCE_BIAS: f32 = 0.2;
/// How much a filter penalty (e.g. 商业合作 ×0.85) lowers the total score.
const PENALTY_SCALE: f32 = 0.3;
/// The same UP may not appear more than once in this many consecutive slots, when avoidable.
const DIVERSITY_WINDOW: usize = 2;

pub struct ScoringContext<'a> {
    pub now: i64,
    pub profile: &'a InterestProfile,
    pub followed: &'a HashSet<u64>,
    /// Impressions since the last click, per bvid.
    pub exposures: &'a HashMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Weights(pub BTreeMap<FactorId, f32>);

impl Default for Weights {
    fn default() -> Self {
        use FactorId::*;
        Self(BTreeMap::from([
            (Interest, 0.30),
            (Follow, 0.25),
            (Freshness, 0.15),
            (Topic, 0.10),
            (Quality, 0.20),
            (Explore, 0.10),
            (Dislike, 0.40),
            (RepeatExposure, 0.15),
        ]))
    }
}

impl Weights {
    pub fn get(&self, id: FactorId) -> f32 {
        self.0.get(&id).copied().unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceWeights(pub BTreeMap<SourceKind, f32>);

impl Default for SourceWeights {
    fn default() -> Self {
        Self(SourceKind::ALL.iter().map(|k| (*k, if *k == SourceKind::BiliRecommend { 0.8 } else { 1.0 })).collect())
    }
}

impl SourceWeights {
    pub fn get(&self, k: SourceKind) -> f32 {
        self.0.get(&k).copied().unwrap_or(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub card: VideoCard,
    /// Filter penalty multiplier (1.0 = none).
    pub penalty: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Part {
    pub factor: FactorId,
    pub raw: f32,
    pub weight: f32,
    pub contribution: f32,
}

/// Everything the "为什么推荐" sheet shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub total: f32,
    pub parts: Vec<Part>,
    pub source_adjust: f32,
    pub penalty_adjust: f32,
}

#[derive(Debug, Clone)]
pub struct Ranked {
    pub card: VideoCard,
    pub score: ScoreBreakdown,
}

pub struct FeedEngine {
    factors: Vec<Box<dyn Factor>>,
    pub weights: Weights,
    pub source_weights: SourceWeights,
}

impl Default for FeedEngine {
    fn default() -> Self {
        Self::new(default_factors(), Weights::default(), SourceWeights::default())
    }
}

impl FeedEngine {
    pub fn new(factors: Vec<Box<dyn Factor>>, weights: Weights, source_weights: SourceWeights) -> Self {
        Self { factors, weights, source_weights }
    }

    pub fn score(&self, c: &Candidate, ctx: &ScoringContext) -> ScoreBreakdown {
        let parts: Vec<Part> = self
            .factors
            .iter()
            .map(|f| {
                let raw = f.score(&c.card, ctx).clamp(-1.0, 1.0);
                let weight = self.weights.get(f.id());
                Part { factor: f.id(), raw, weight, contribution: raw * weight }
            })
            .collect();
        let best_source = c
            .card
            .sources
            .iter()
            .map(|s| self.source_weights.get(*s))
            .fold(None, |a: Option<f32>, w| Some(a.map_or(w, |a| a.max(w))));
        let source_adjust = SOURCE_BIAS * (best_source.unwrap_or(1.0) - 1.0);
        let penalty_adjust = -PENALTY_SCALE * (1.0 - c.penalty.clamp(0.0, 1.0));
        let total = parts.iter().map(|p| p.contribution).sum::<f32>() + source_adjust + penalty_adjust;
        ScoreBreakdown { total, parts, source_adjust, penalty_adjust }
    }

    /// Dedupes by bvid (merging sources), scores, sorts and spreads out repeated UPs.
    pub fn rank(&self, candidates: Vec<Candidate>, ctx: &ScoringContext, limit: usize) -> Vec<Ranked> {
        let mut by_bvid: HashMap<String, Candidate> = HashMap::with_capacity(candidates.len());
        for c in candidates {
            match by_bvid.get_mut(&c.card.bvid) {
                Some(existing) => {
                    for s in &c.card.sources {
                        existing.card.add_source(*s);
                    }
                    existing.penalty = existing.penalty.min(c.penalty);
                }
                None => {
                    by_bvid.insert(c.card.bvid.clone(), c);
                }
            }
        }
        let mut scored: Vec<Ranked> = by_bvid
            .into_values()
            .map(|c| {
                let score = self.score(&c, ctx);
                Ranked { card: c.card, score }
            })
            .collect();
        scored.sort_by(|a, b| b.score.total.total_cmp(&a.score.total).then_with(|| a.card.bvid.cmp(&b.card.bvid)));
        diversify(scored, limit)
    }
}

fn diversify(mut pool: Vec<Ranked>, limit: usize) -> Vec<Ranked> {
    let mut out: Vec<Ranked> = Vec::with_capacity(limit.min(pool.len()));
    while out.len() < limit && !pool.is_empty() {
        let recent: Vec<u64> = out.iter().rev().take(DIVERSITY_WINDOW).map(|r| r.card.owner.mid).collect();
        let pick = pool.iter().position(|r| !recent.contains(&r.card.owner.mid)).unwrap_or(0);
        out.push(pool.remove(pick));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubi_model::{DislikeReason, FeedbackEvent, ItemRef, Owner, Signal, Stats};

    const NOW: i64 = 1_800_000_000;

    fn card(bvid: &str, mid: u64, tags: &[&str]) -> Candidate {
        Candidate {
            card: VideoCard {
                bvid: bvid.into(),
                owner: Owner { mid, name: format!("up{mid}") },
                tags: tags.iter().map(|t| t.to_string()).collect(),
                pubdate: NOW - 3600,
                stats: Stats { view: 10_000, like: 400, coin: 100, favorite: 150, danmaku: 50, reply: 50 },
                sources: vec![SourceKind::Popular],
                ..Default::default()
            },
            penalty: 1.0,
        }
    }

    #[test]
    fn followed_and_interesting_beat_disliked() {
        let mut profile = InterestProfile::default();
        let liked = ItemRef { up_mid: 1, tags: vec!["rust".into()], ..Default::default() };
        profile.apply(&FeedbackEvent { signal: Signal::Coin, item: Some(liked), at: NOW });
        let hated = ItemRef { up_mid: 3, ..Default::default() };
        profile.apply(&FeedbackEvent {
            signal: Signal::Dislike { reason: DislikeReason::Up },
            item: Some(hated),
            at: NOW,
        });
        let followed = HashSet::from([1]);
        let exposures = HashMap::new();
        let ctx = ScoringContext { now: NOW, profile: &profile, followed: &followed, exposures: &exposures };

        let ranked = FeedEngine::default().rank(
            vec![card("c", 3, &[]), card("b", 2, &["cooking"]), card("a", 1, &["rust"])],
            &ctx,
            10,
        );
        let order: Vec<&str> = ranked.iter().map(|r| r.card.bvid.as_str()).collect();
        assert_eq!(order, vec!["a", "b", "c"]);
        assert_eq!(ranked[0].score.parts.len(), 8);
    }

    #[test]
    fn repeat_exposure_and_penalty_lower_the_score() {
        let profile = InterestProfile::default();
        let followed = HashSet::new();
        let exposures = HashMap::from([("a".to_string(), 5)]);
        let ctx = ScoringContext { now: NOW, profile: &profile, followed: &followed, exposures: &exposures };
        let engine = FeedEngine::default();
        let a = engine.score(&card("a", 1, &[]), &ctx).total;
        let b = engine.score(&card("b", 1, &[]), &ctx).total;
        assert!(a < b);
        let mut penalized = card("b", 1, &[]);
        penalized.penalty = 0.85;
        assert!(engine.score(&penalized, &ctx).total < b);
    }

    #[test]
    fn duplicates_merge_sources_and_ups_are_spread() {
        let profile = InterestProfile::default();
        let followed = HashSet::new();
        let exposures = HashMap::new();
        let ctx = ScoringContext { now: NOW, profile: &profile, followed: &followed, exposures: &exposures };
        let mut dup = card("a", 1, &[]);
        dup.card.sources = vec![SourceKind::Ranking];
        let cands = vec![card("a", 1, &[]), dup, card("b", 1, &[]), card("c", 1, &[]), card("d", 2, &[])];
        let ranked = FeedEngine::default().rank(cands, &ctx, 10);
        assert_eq!(ranked.len(), 4);
        let a = ranked.iter().find(|r| r.card.bvid == "a").unwrap();
        assert_eq!(a.card.sources.len(), 2);
        let mids: Vec<u64> = ranked.iter().map(|r| r.card.owner.mid).collect();
        assert!(mids[..3].contains(&2), "UP 2 is pulled forward to break a run of UP 1: {mids:?}");
    }

    #[test]
    fn scores_500_candidates_quickly() {
        let profile = InterestProfile::default();
        let followed = HashSet::new();
        let exposures = HashMap::new();
        let ctx = ScoringContext { now: NOW, profile: &profile, followed: &followed, exposures: &exposures };
        let cands: Vec<Candidate> = (0..500).map(|i| card(&format!("BV{i}"), i % 37, &["rust", "游戏"])).collect();
        let t = std::time::Instant::now();
        let ranked = FeedEngine::default().rank(cands, &ctx, 500);
        let ms = t.elapsed().as_millis();
        assert_eq!(ranked.len(), 500);
        // Budget is 20 ms in release; allow headroom for unoptimised test builds.
        assert!(ms < 200, "took {ms} ms");
    }
}
