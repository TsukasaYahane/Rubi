//! When to tell B站 how far a video has been watched, as Android does: once at the start, every
//! 15 seconds while it plays, and once on leaving.

/// Seconds between two 正在播放 reports.
const REPORT_EVERY: u32 = 15;

/// What `play_type` means to the core.
pub const STARTED: u8 = 1;
pub const PLAYING: u8 = 0;
pub const LEFT: u8 = 2;

/// Follows one video, one sample a second.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WatchClock {
    played_s: u64,
    real_s: u64,
    samples: u32,
}

impl WatchClock {
    /// Takes in a sample; true when a report is due. A paused video keeps its last position, and
    /// a report falling due during a pause is skipped.
    pub fn sample(&mut self, playing: bool, position_s: u64, real_s: u64) -> bool {
        if playing {
            self.played_s = position_s;
        }
        self.real_s = real_s;
        self.samples += 1;
        playing && self.samples.is_multiple_of(REPORT_EVERY)
    }

    /// Seconds of the video reached, and seconds since it opened.
    pub fn progress(&self) -> (u64, u64) {
        (self.played_s, self.real_s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_fifteen_samples_while_playing() {
        let mut clock = WatchClock::default();
        let due: Vec<u64> = (1..=45).filter(|&s| clock.sample(true, s, s)).collect();
        assert_eq!(due, [15, 30, 45]);
        assert_eq!(clock.progress(), (45, 45));
    }

    #[test]
    fn a_paused_video_reports_nothing_and_keeps_its_position() {
        let mut clock = WatchClock::default();
        for s in 1..=10 {
            clock.sample(true, s, s);
        }
        let due = (11..=20).filter(|&s| clock.sample(false, 99, s)).count();
        assert_eq!(due, 0);
        assert_eq!(clock.progress(), (10, 20));
    }

    #[test]
    fn the_first_report_after_a_pause_comes_on_the_beat() {
        let mut clock = WatchClock::default();
        for s in 1..=20 {
            clock.sample(s <= 3, s, s);
        }
        let next = (21..=40).find(|&s| clock.sample(true, s, s));
        assert_eq!(next, Some(30));
    }
}
