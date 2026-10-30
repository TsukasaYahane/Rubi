//! The user's interest profile: time-decayed weights over tags, zones, UPs and
//! title keywords, plus the matching "not interested" weights.

use std::collections::HashMap;
use std::hash::Hash;

use rubi_model::{DislikeReason, FeedbackEvent, ItemRef, Signal};
use serde::{Deserialize, Serialize};

use crate::text::tokens;

const SECS_PER_DAY: f64 = 86_400.0;
const MAX_KEYWORDS: usize = 2_000;

/// A value that halves every `half_life_days`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Decayed {
    pub value: f64,
    /// Unix seconds of the last update.
    pub at: i64,
}

impl Decayed {
    pub fn get(&self, now: i64, half_life_days: f64) -> f64 {
        let days = (now - self.at).max(0) as f64 / SECS_PER_DAY;
        self.value * 0.5f64.powf(days / half_life_days)
    }

    fn add(&mut self, delta: f64, now: i64, half_life_days: f64) {
        self.value = self.get(now, half_life_days) + delta;
        self.at = now;
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Weighted {
    pub tags: HashMap<String, Decayed>,
    pub zones: HashMap<u32, Decayed>,
    pub ups: HashMap<u64, Decayed>,
    pub keywords: HashMap<String, Decayed>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InterestProfile {
    pub half_life_days: f64,
    pub likes: Weighted,
    pub dislikes: Weighted,
}

impl Default for InterestProfile {
    fn default() -> Self {
        Self { half_life_days: 30.0, likes: Weighted::default(), dislikes: Weighted::default() }
    }
}

/// How strongly a signal moves the profile.
fn signal_weight(signal: &Signal) -> f64 {
    match signal {
        Signal::Impression => 0.0,
        Signal::Click => 0.5,
        Signal::QuickExit => -0.6,
        Signal::Completion { ratio } if *ratio >= 0.6 => f64::from(*ratio),
        Signal::Completion { .. } => 0.1,
        Signal::Rewatch => 1.0,
        Signal::Like => 1.5,
        Signal::Favorite => 2.0,
        Signal::Coin => 2.5,
        Signal::Follow => 3.0,
        Signal::Dislike { .. } | Signal::Search { .. } => 0.0,
    }
}

impl InterestProfile {
    pub fn apply(&mut self, ev: &FeedbackEvent) {
        let (now, hl) = (ev.at, self.half_life_days);
        match (&ev.signal, &ev.item) {
            (Signal::Search { term }, _) => {
                for t in tokens(term) {
                    bump(&mut self.likes.keywords, t, 1.0, now, hl);
                }
            }
            (Signal::Dislike { reason }, Some(item)) => self.apply_dislike(reason, item, now),
            (signal, Some(item)) => {
                let w = signal_weight(signal);
                if w == 0.0 {
                    return;
                }
                let target = if w > 0.0 { &mut self.likes } else { &mut self.dislikes };
                let w = w.abs();
                for tag in &item.tags {
                    bump(&mut target.tags, tag.clone(), w, now, hl);
                }
                if let Some(z) = item.zone_tid {
                    bump(&mut target.zones, z, w * 0.5, now, hl);
                }
                let up_w = if matches!(signal, Signal::Follow) { w } else { w * 0.6 };
                bump(&mut target.ups, item.up_mid, up_w, now, hl);
                if w >= 1.0 {
                    for t in tokens(&item.title) {
                        bump(&mut target.keywords, t, w * 0.2, now, hl);
                    }
                }
            }
            (_, None) => {}
        }
        self.prune(now);
    }

    fn apply_dislike(&mut self, reason: &DislikeReason, item: &ItemRef, now: i64) {
        let (d, hl) = (&mut self.dislikes, self.half_life_days);
        match reason {
            DislikeReason::Up => bump(&mut d.ups, item.up_mid, 3.0, now, hl),
            DislikeReason::Zone => {
                if let Some(z) = item.zone_tid {
                    bump(&mut d.zones, z, 3.0, now, hl);
                }
            }
            DislikeReason::Tag(tag) => bump(&mut d.tags, tag.clone(), 3.0, now, hl),
            DislikeReason::Title => {
                for t in tokens(&item.title) {
                    bump(&mut d.keywords, t, 1.0, now, hl);
                }
            }
        }
    }

    fn prune(&mut self, now: i64) {
        let hl = self.half_life_days;
        for map in [&mut self.likes.keywords, &mut self.dislikes.keywords] {
            if map.len() > MAX_KEYWORDS {
                let mut v: Vec<(String, f64)> = map.iter().map(|(k, d)| (k.clone(), d.get(now, hl))).collect();
                v.sort_by(|a, b| b.1.total_cmp(&a.1));
                let keep: std::collections::HashSet<String> =
                    v.into_iter().take(MAX_KEYWORDS).map(|(k, _)| k).collect();
                map.retain(|k, _| keep.contains(k));
            }
        }
    }
}

fn bump<K: Eq + Hash>(map: &mut HashMap<K, Decayed>, key: K, delta: f64, now: i64, hl: f64) {
    map.entry(key).or_default().add(delta, now, hl);
}

/// Read helpers used by the factors.
impl Weighted {
    pub fn tag(&self, t: &str, now: i64, hl: f64) -> f64 {
        self.tags.get(t).map_or(0.0, |d| d.get(now, hl))
    }

    pub fn zone(&self, z: u32, now: i64, hl: f64) -> f64 {
        self.zones.get(&z).map_or(0.0, |d| d.get(now, hl))
    }

    pub fn up(&self, m: u64, now: i64, hl: f64) -> f64 {
        self.ups.get(&m).map_or(0.0, |d| d.get(now, hl))
    }

    pub fn keyword(&self, k: &str, now: i64, hl: f64) -> f64 {
        self.keywords.get(k).map_or(0.0, |d| d.get(now, hl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item() -> ItemRef {
        ItemRef {
            bvid: "BV1".into(),
            up_mid: 7,
            zone_tid: Some(36),
            tags: vec!["rust".into()],
            title: "Rust 异步".into(),
        }
    }

    #[test]
    fn decays_by_half_life() {
        let d = Decayed { value: 8.0, at: 0 };
        assert!((d.get(30 * 86_400, 30.0) - 4.0).abs() < 1e-9);
    }

    #[test]
    fn positive_and_negative_signals_go_to_their_side() {
        let mut p = InterestProfile::default();
        p.apply(&FeedbackEvent { signal: Signal::Coin, item: Some(item()), at: 0 });
        assert!(p.likes.tag("rust", 0, 30.0) > 2.0);
        assert!(p.likes.keyword("rust", 0, 30.0) > 0.0);
        p.apply(&FeedbackEvent { signal: Signal::QuickExit, item: Some(item()), at: 0 });
        assert!(p.dislikes.tag("rust", 0, 30.0) > 0.0);
        p.apply(&FeedbackEvent { signal: Signal::Dislike { reason: DislikeReason::Up }, item: Some(item()), at: 0 });
        assert!(p.dislikes.up(7, 0, 30.0) >= 3.0);
    }
}
