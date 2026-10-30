use std::time::{Duration, Instant};

use tokio::sync::Mutex;

/// A token bucket shared by every request, so bursts stay polite to risk control.
pub struct RateLimiter {
    capacity: f64,
    per_sec: f64,
    state: Mutex<(f64, Instant)>,
}

impl RateLimiter {
    pub fn new(capacity: u32, per_sec: f64) -> Self {
        Self { capacity: f64::from(capacity), per_sec, state: Mutex::new((f64::from(capacity), Instant::now())) }
    }

    /// Waits until a token is available and takes it.
    pub async fn acquire(&self) {
        loop {
            let wait = {
                let mut s = self.state.lock().await;
                let now = Instant::now();
                s.0 = (s.0 + now.duration_since(s.1).as_secs_f64() * self.per_sec).min(self.capacity);
                s.1 = now;
                if s.0 >= 1.0 {
                    s.0 -= 1.0;
                    return;
                }
                Duration::from_secs_f64((1.0 - s.0) / self.per_sec)
            };
            tokio::time::sleep(wait).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bursts_up_to_capacity_then_waits() {
        let l = RateLimiter::new(2, 20.0);
        let t = Instant::now();
        l.acquire().await;
        l.acquire().await;
        assert!(t.elapsed() < Duration::from_millis(20));
        l.acquire().await;
        assert!(t.elapsed() >= Duration::from_millis(40));
    }
}
