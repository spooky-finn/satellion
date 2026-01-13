use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Throttler that only allows action every `interval`.
pub struct Throttler {
    last_emit: Instant,
    interval: Duration,
}

impl Throttler {
    pub fn new(interval: Duration) -> Self {
        Self {
            last_emit: Instant::now() - interval, // allow immediate first emit
            interval,
        }
    }

    /// Returns true if enough time has passed and updates the last_emit.
    pub fn should_emit(&mut self) -> bool {
        if self.last_emit.elapsed() >= self.interval {
            self.last_emit = Instant::now();
            true
        } else {
            false
        }
    }
}
