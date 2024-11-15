use lazy_static::lazy_static;
use std::sync::Mutex;

pub struct AudioContext {
    sample_rate: f32,
    current_time: f64, // in seconds
    current_sample: u64,
}

impl AudioContext {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            current_time: 0.0,
            current_sample: 0,
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    pub fn current_sample(&self) -> u64 {
        self.current_sample
    }

    pub fn increment_sample(&mut self) {
        self.current_sample = self.current_sample.wrapping_add(1);
        self.current_time = self.current_sample as f64 / self.sample_rate as f64;
    }

    pub fn reset(&mut self) {
        self.current_time = 0.0;
        self.current_sample = 0;
    }
}

lazy_static! {
    pub static ref AUDIO_CONTEXT: Mutex<AudioContext> = Mutex::new(AudioContext::new(44100.0));
}

// Helper function to initialize the audio context
pub fn initialize_audio_context(sample_rate: f32) {
    if let Ok(mut context) = AUDIO_CONTEXT.lock() {
        *context = AudioContext::new(sample_rate);
    }
}
