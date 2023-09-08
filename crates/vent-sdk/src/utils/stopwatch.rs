use std::time::{Duration, Instant};

pub struct Stopwatch {
    start: Option<Instant>,
    elapsed: Duration,
}

impl Stopwatch {
    pub const fn new() -> Self {
        Self {
            start: None,
            elapsed: Duration::ZERO,
        }
    }

    pub fn new_and_start() -> Self {
        let mut sw = Self::new();
        sw.start();
        sw
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        self.start = None;
    }

    pub fn elapsed(&self) -> Duration {
        match self.start {
            Some(t1) => t1.elapsed() + self.elapsed,
            None => self.elapsed,
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        let dur = self.elapsed();
        dur.as_secs() * 1000 + dur.subsec_millis() as u64
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}
