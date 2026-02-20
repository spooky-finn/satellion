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

pub mod tracing {
    use std::fmt;

    use tracing_subscriber::{EnvFilter, FmtSubscriber, fmt::time::FormatTime};

    pub fn init() {
        struct LocalTimeOnly;

        impl FormatTime for LocalTimeOnly {
            fn format_time(
                &self,
                w: &mut tracing_subscriber::fmt::format::Writer<'_>,
            ) -> fmt::Result {
                let now = chrono::Local::now();
                write!(w, "{}", now.format("%H:%M:%S"))
            }
        }

        let subscriber = FmtSubscriber::builder()
            .with_timer(LocalTimeOnly)
            .compact()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }

    pub fn init_test(level: &str) {
        let subscriber = FmtSubscriber::builder()
            .without_time()
            .compact()
            .with_env_filter(EnvFilter::new(level))
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);
    }
}
