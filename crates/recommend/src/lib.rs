//! On-device recommendation. The engine only knows the [`Source`] and [`Factor`]
//! abstractions; concrete sources that call the API are wired up in `rubi-core`.

mod engine;
mod factors;
mod profile;
mod source;
mod text;

pub use engine::{Candidate, FeedEngine, Part, Ranked, ScoreBreakdown, ScoringContext, SourceWeights, Weights};
pub use factors::{
    DislikeFactor, ExploreFactor, Factor, FactorId, FollowFactor, FreshnessFactor, InterestFactor, QualityFactor,
    RepeatExposureFactor, TopicFactor, default_factors,
};
pub use profile::{Decayed, InterestProfile};
pub use source::{Source, SourceError};
