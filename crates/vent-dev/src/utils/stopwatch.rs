use std::time::{Duration, Instant};

pub struct Stopwatch {
    start: Option<Instant>,
    elapsed: Duration,
}

impl Stopwatch {
    pub fn new() -> Self {
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
            Some(t1) => {
                return t1.elapsed() + self.elapsed;
            }
            None => {
                return self.elapsed;
            }
        }
    }

    pub fn elapsed_ms(&self) -> i64 {
        let dur = self.elapsed();
        return (dur.as_secs() * 1000 + (dur.subsec_nanos() / 1000000) as u64) as i64;
    }
}
