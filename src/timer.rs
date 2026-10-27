use std::time::{Duration, Instant};

/// Elapsed time is calulated by using segments.
/// Segment is an `Instant` when the timer was started of resumed.
/// Once the timer is stopped, the segment completes and its elapsed time is accumulated.
#[derive(Default)]
pub struct Timer {
    /// Beginning of current segment
    started_at: Option<Instant>,
    /// Time accumulated by previously completed segments
    accumulated: Duration,
}

impl Timer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }
    }

    pub fn stop(&mut self) {
        if let Some(started_at) = self.started_at.take() {
            self.accumulated += started_at.elapsed();
        }
    }

    pub fn reset(&mut self) {
        self.started_at = None;
        self.accumulated = Duration::ZERO;
    }

    pub fn elapsed(&self) -> Duration {
        self.accumulated + self.started_at.map_or(Duration::ZERO, |t| t.elapsed())
    }

    pub fn set_accumulated(&mut self, value: Duration) {
        self.accumulated = value;
        if self.started_at.is_some() {
            // `self.elapsed` should be equal to `value`
            self.started_at = Some(Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        let timer = Timer::new();
        assert_eq!(timer.elapsed(), Duration::ZERO);
    }

    #[test]
    fn starts_segment() {
        let mut timer = Timer::new();
        timer.start();
        assert!(timer.started_at.is_some());
        assert!(timer.elapsed() > Duration::ZERO);
    }

    #[test]
    fn stops_segment() {
        let mut timer = Timer::new();
        timer.start();
        assert!(timer.started_at.is_some());
        timer.stop();
        assert!(timer.started_at.is_none());
        assert!(timer.elapsed() > Duration::ZERO);
    }

    #[test]
    fn resets() {
        let mut timer = Timer::new();
        timer.start();
        assert!(timer.started_at.is_some());
        assert!(timer.elapsed() > Duration::ZERO);
        timer.reset();
        assert!(timer.started_at.is_none());
        assert_eq!(timer.accumulated, Duration::ZERO);
        assert_eq!(timer.elapsed(), Duration::ZERO);
    }

    #[test]
    fn returns_elapsed() {
        let mut timer = Timer::new();
        timer.start();
        std::thread::sleep(Duration::from_millis(100));
        assert!(timer.elapsed() > Duration::from_millis(100));
    }

    #[test]
    fn sets_accumulated() {
        let mut timer = Timer::new();
        timer.set_accumulated(Duration::from_secs(10));
        assert_eq!(timer.elapsed(), Duration::from_secs(10));
    }
}
