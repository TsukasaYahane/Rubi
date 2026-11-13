//! Scoring factors. Each returns a normalised value in [-1, 1]; the sign is the
//! factor's direction (dislike and repeat exposure are negative).

use rubi_model::VideoCard;
use serde::{Deserialize, Serialize};

use crate::engine::ScoringContext;
use crate::text::tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactorId {
    Interest,
    Follow,
    Freshness,
    Topic,
    Quality,
    Explore,
    Dislike,
    RepeatExposure,
}

pub trait Factor: Send + Sync {
    fn id(&self) -> FactorId;
    fn score(&self, item: &VideoCard, ctx: &ScoringContext) -> f32;
}

/// Maps [0, ∞) onto [0, 1); `k` is the input that maps to 0.5.
fn sat(x: f64, k: f64) -> f64 {
    if x <= 0.0 { 0.0 } else { x / (x + k) }
}

fn age_hours(item: &VideoCard, now: i64) -> f64 {
    ((now - item.pubdate).max(0) as f64) / 3600.0
}

pub struct InterestFactor;

impl Factor for InterestFactor {
    fn id(&self) -> FactorId {
        FactorId::Interest
    }

    fn score(&self, item: &VideoCard, ctx: &ScoringContext) -> f32 {
        let (p, now) = (&ctx.profile.likes, ctx.now);
        let hl = ctx.profile.half_life_days;
        let tags: f64 = item.tags.iter().map(|t| p.tag(t, now, hl)).sum();
        let zone = item.zone.as_ref().map_or(0.0, |z| p.zone(z.tid, now, hl));
        let up = p.up(item.owner.mid, now, hl);
        let kw: f64 = tokens(&item.title).iter().map(|t| p.keyword(t, now, hl)).sum();
        sat(tags + zone + up + kw * 0.5, 3.0) as f32
    }
}

pub struct FollowFactor;

impl Factor for FollowFactor {
    fn id(&self) -> FactorId {
        FactorId::Follow
    }

    fn score(&self, item: &VideoCard, ctx: &ScoringContext) -> f32 {
        if ctx.followed.contains(&item.owner.mid) { 1.0 } else { 0.0 }
    }
}

pub struct FreshnessFactor {
    pub half_life_hours: f64,
}

impl Factor for FreshnessFactor {
    fn id(&self) -> FactorId {
        FactorId::Freshness
    }

    fn score(&self, item: &VideoCard, ctx: &ScoringContext) -> f32 {
        if item.pubdate <= 0 {
            return 0.0;
        }
        0.5f64.powf(age_hours(item, ctx.now) / self.half_life_hours) as f32
    }
}

/// View velocity, plus a bonus when several sources surface the same video.
pub struct TopicFactor;

impl Factor for TopicFactor {
    fn id(&self) -> FactorId {
        FactorId::Topic
    }

    fn score(&self, item: &VideoCard, ctx: &ScoringContext) -> f32 {
        let per_hour = item.stats.view as f64 / age_hours(item, ctx.now).max(1.0);
        let velocity = ((1.0 + per_hour).ln() / (1.0 + 20_000f64).ln()).min(1.0);
        let multi = ((item.sources.len().max(1) - 1) as f64 * 0.25).min(1.0);
        (0.75 * velocity + 0.25 * multi) as f32
    }
}

/// Engagement ratios against per-view baselines. Low-view videos are shrunk towards
/// the neutral 0.5 so new uploads are not punished for small samples.
pub struct QualityFactor {
    pub like: f64,
    pub coin: f64,
    pub favorite: f64,
    pub interact: f64,
}

impl Default for QualityFactor {
    fn default() -> Self {
        Self { like: 0.04, coin: 0.01, favorite: 0.015, interact: 0.01 }
    }
}

impl Factor for QualityFactor {
    fn id(&self) -> FactorId {
        FactorId::Quality
    }

    fn score(&self, item: &VideoCard, _: &ScoringContext) -> f32 {
        let s = &item.stats;
        if s.view == 0 {
            return 0.5;
        }
        let v = s.view as f64;
        let ratio = |n: u64, base: f64| sat(n as f64 / v / base, 1.0);
        let raw = (ratio(s.like, self.like)
            + ratio(s.coin, self.coin)
            + ratio(s.favorite, self.favorite)
            + ratio(s.danmaku + s.reply, self.interact))
            / 4.0;
        let confidence = v / (v + 2_000.0);
        (confidence * raw + (1.0 - confidence) * 0.5) as f32
    }
}

/// Rewards zones and UPs the profile does not know yet.
pub struct ExploreFactor;

impl Factor for ExploreFactor {
    fn id(&self) -> FactorId {
        FactorId::Explore
    }

    fn score(&self, item: &VideoCard, ctx: &ScoringContext) -> f32 {
        let (p, now, hl) = (ctx.profile, ctx.now, ctx.profile.half_life_days);
        let zone = item.zone.as_ref().map(|z| z.tid);
        let disliked =
            p.dislikes.up(item.owner.mid, now, hl) > 0.5 || zone.is_some_and(|z| p.dislikes.zone(z, now, hl) > 0.5);
        if disliked {
            return 0.0;
        }
        let known_zone = zone.is_none_or(|z| p.likes.zone(z, now, hl) > 0.5);
        let known_up = p.likes.up(item.owner.mid, now, hl) > 0.5;
        match (known_zone, known_up) {
            (false, false) => 1.0,
            (true, false) => 0.4,
            _ => 0.0,
        }
    }
}

pub struct DislikeFactor;

impl Factor for DislikeFactor {
    fn id(&self) -> FactorId {
        FactorId::Dislike
    }

    fn score(&self, item: &VideoCard, ctx: &ScoringContext) -> f32 {
        let (d, now, hl) = (&ctx.profile.dislikes, ctx.now, ctx.profile.half_life_days);
        let up = d.up(item.owner.mid, now, hl);
        let zone = item.zone.as_ref().map_or(0.0, |z| d.zone(z.tid, now, hl));
        let tags: f64 = item.tags.iter().map(|t| d.tag(t, now, hl)).sum();
        let kw: f64 = tokens(&item.title).iter().map(|t| d.keyword(t, now, hl)).sum();
        -sat(up + zone * 0.7 + tags * 0.5 + kw * 0.3, 2.0) as f32
    }
}

/// Penalises videos shown again and again without a click.
pub struct RepeatExposureFactor;

impl Factor for RepeatExposureFactor {
    fn id(&self) -> FactorId {
        FactorId::RepeatExposure
    }

    fn score(&self, item: &VideoCard, ctx: &ScoringContext) -> f32 {
        let n = ctx.exposures.get(&item.bvid).copied().unwrap_or(0);
        // One showing costs as much as a whole factor: a card the user passed over gives way to
        // ones they have not seen yet, which is what a fresh page should offer.
        -(n as f32 * 0.85).min(1.6)
    }
}

pub fn default_factors() -> Vec<Box<dyn Factor>> {
    vec![
        Box::new(InterestFactor),
        Box::new(FollowFactor),
        Box::new(FreshnessFactor { half_life_hours: 48.0 }),
        Box::new(TopicFactor),
        Box::new(QualityFactor::default()),
        Box::new(ExploreFactor),
        Box::new(DislikeFactor),
        Box::new(RepeatExposureFactor),
    ]
}
