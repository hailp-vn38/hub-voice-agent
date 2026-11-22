use hub_core::Clock;
use jiff::{Span, Zoned};
use tokio::time::Instant;

/// Clock that starts at a fixed instant and advances with tokio's (paused) time.
pub struct TestClock {
    start: Zoned,
    origin: Instant,
}

impl TestClock {
    pub fn at(start: Zoned) -> Self {
        Self {
            start,
            origin: Instant::now(),
        }
    }
}

impl Clock for TestClock {
    fn now(&self) -> Zoned {
        let ms = self.origin.elapsed().as_millis() as i64;
        self.start
            .checked_add(Span::new().milliseconds(ms))
            .unwrap()
    }
}
