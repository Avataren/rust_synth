use std::sync::atomic::{AtomicU64, Ordering};

pub struct AudioContext {
    sample_rate: f32,
    current_sample: AtomicU64,
}

impl AudioContext {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            current_sample: AtomicU64::new(0),
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn increment_samples(&self, n: u64) {
        self.current_sample.fetch_add(n, Ordering::Relaxed);
    }

    pub fn current_sample(&self) -> u64 {
        self.current_sample.load(Ordering::Relaxed)
    }

    pub fn current_time(&self) -> f64 {
        self.current_sample() as f64 / self.sample_rate as f64
    }
}
