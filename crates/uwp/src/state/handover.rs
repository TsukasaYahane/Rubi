//! Changing quality without a stop. The new stream waits, paused and buffered, a few seconds ahead
//! of the one playing; when that reaches it, the new one takes over.

/// How far ahead the new stream waits.
const LEAD_MS: u64 = 4_000;
/// How near the new stream must be to where it was sent.
const NEAR_MS: u64 = 1_000;
/// How long before the point the old stream counts as there.
const EARLY_MS: u64 = 100;
/// Samples after which the new stream takes over however things stand.
const GIVE_UP_AFTER: u32 = 240;

/// One look at both players.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Sample {
    /// The new stream has opened.
    pub opened: bool,
    /// The new stream holds still, ready to play.
    pub settled: bool,
    pub new_ms: u64,
    pub old_ms: u64,
    pub old_playing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Wait,
    /// Send the new stream here.
    SeekTo(u64),
    /// The new stream takes over, playing on if the old one was.
    Swap,
}

#[derive(Debug, Default)]
pub struct Handover {
    target_ms: Option<u64>,
    samples: u32,
}

impl Handover {
    pub fn step(&mut self, now: Sample) -> Step {
        self.samples += 1;
        if self.samples >= GIVE_UP_AFTER {
            return Step::Swap;
        }
        if !now.opened {
            return Step::Wait;
        }
        let Some(target) = self.target_ms else {
            return self.aim(now.old_ms);
        };
        let ready = now.settled && now.new_ms.abs_diff(target) < NEAR_MS;
        if !now.old_playing {
            return if ready { Step::Swap } else { Step::Wait };
        }
        if now.old_ms + EARLY_MS < target {
            return Step::Wait;
        }
        if ready { Step::Swap } else { self.aim(now.old_ms) }
    }

    fn aim(&mut self, old_ms: u64) -> Step {
        let target = old_ms + LEAD_MS;
        self.target_ms = Some(target);
        Step::SeekTo(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready_at(new_ms: u64, old_ms: u64) -> Sample {
        Sample { opened: true, settled: true, new_ms, old_ms, old_playing: true }
    }

    #[test]
    fn waits_for_the_stream_to_open_then_aims_ahead() {
        let mut handover = Handover::default();
        assert_eq!(handover.step(Sample::default()), Step::Wait);
        assert_eq!(handover.step(ready_at(0, 10_000)), Step::SeekTo(14_000));
    }

    #[test]
    fn takes_over_when_the_old_stream_arrives() {
        let mut handover = Handover::default();
        handover.step(ready_at(0, 10_000));
        assert_eq!(handover.step(ready_at(14_000, 12_000)), Step::Wait);
        assert_eq!(handover.step(ready_at(14_000, 13_950)), Step::Swap);
    }

    #[test]
    fn aims_again_when_the_new_stream_was_too_slow() {
        let mut handover = Handover::default();
        handover.step(ready_at(0, 10_000));
        let late = Sample { settled: false, ..ready_at(0, 14_000) };
        assert_eq!(handover.step(late), Step::SeekTo(18_000));
    }

    #[test]
    fn a_paused_video_takes_over_as_soon_as_it_can() {
        let mut handover = Handover::default();
        handover.step(ready_at(0, 10_000));
        let paused = Sample { old_playing: false, ..ready_at(14_000, 10_000) };
        assert_eq!(handover.step(paused), Step::Swap);
    }

    #[test]
    fn gives_up_waiting_in_the_end() {
        let mut handover = Handover::default();
        let steps = (0..GIVE_UP_AFTER).map(|_| handover.step(Sample::default())).last();
        assert_eq!(steps, Some(Step::Swap));
    }
}
