// src/synth/audio_param.rs

use crossbeam::atomic::AtomicCell;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy)]
pub enum RampType {
    Linear,
    Exponential,
}

#[derive(Debug, Clone)]
pub struct RampEvent {
    pub start_value: f32,
    pub end_value: f32,
    pub start_sample: u64,
    pub duration_samples: u64,
    pub ramp_type: RampType,
}

pub struct AudioParam {
    current_value: AtomicCell<f32>,
    default_value: f32,
    min_value: f32,
    max_value: f32,
    events: Arc<RwLock<Vec<RampEvent>>>,
}

impl Clone for AudioParam {
    fn clone(&self) -> Self {
        // Deep clone events if needed
        let events_clone = {
            let events = self.events.read().unwrap();
            events.clone()
        };

        Self {
            current_value: AtomicCell::new(self.current_value.load()),
            default_value: self.default_value,
            min_value: self.min_value,
            max_value: self.max_value,
            events: Arc::new(RwLock::new(events_clone)),
        }
    }
}

impl AudioParam {
    pub fn new(default_value: f32, min_value: f32, max_value: f32) -> Self {
        Self {
            current_value: AtomicCell::new(default_value),
            default_value,
            min_value,
            max_value,
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn set_value(&self, value: f32) {
        let value = self.clamp_value(value);
        self.current_value.store(value);
    }

    pub fn exponential_ramp_to_value_at_time(
        &self,
        value: f32,
        duration_seconds: f32,
        start_sample: u64,
        sample_rate: f32,
    ) {
        let value = self.clamp_value(value);
        let duration_samples = ((duration_seconds * sample_rate) as u64).max(1);

        let start_value = self.current_value.load();

        let event = RampEvent {
            start_value,
            end_value: value,
            start_sample,
            duration_samples,
            ramp_type: RampType::Exponential,
        };

        let mut events = self.events.write().unwrap();
        events.push(event);
    }

    pub fn linear_ramp_to_value_at_time(
        &self,
        value: f32,
        duration_seconds: f32,
        start_sample: u64,
        sample_rate: f32,
    ) {
        let value = self.clamp_value(value);
        let duration_samples = ((duration_seconds * sample_rate) as u64).max(1);

        let start_value = self.current_value.load();

        let event = RampEvent {
            start_value,
            end_value: value,
            start_sample,
            duration_samples,
            ramp_type: RampType::Linear,
        };

        let mut events = self.events.write().unwrap();
        events.push(event);
    }

    pub fn get_value(&self, current_sample: u64) -> f32 {
        let mut value = self.current_value.load();

        let events = self.events.read().unwrap();

        for event in events.iter() {
            if current_sample >= event.start_sample
                && current_sample < event.start_sample + event.duration_samples
            {
                let t =
                    (current_sample - event.start_sample) as f32 / event.duration_samples as f32;

                value = match event.ramp_type {
                    RampType::Linear => {
                        let delta = event.end_value - event.start_value;
                        event.start_value + delta * t
                    }
                    RampType::Exponential => {
                        let start = event.start_value.max(0.00001);
                        let end = event.end_value.max(0.00001);
                        start * (end / start).powf(t)
                    }
                };
                break;
            } else if current_sample >= event.start_sample + event.duration_samples {
                // Event has completed; set to end_value
                value = event.end_value;
            }
        }

        value
    }

    pub fn cancel_scheduled_values(&self) {
        let mut events = self.events.write().unwrap();
        events.clear();
    }

    pub fn reset(&self) {
        self.current_value.store(self.default_value);
        let mut events = self.events.write().unwrap();
        events.clear();
    }

    fn clamp_value(&self, value: f32) -> f32 {
        value.clamp(self.min_value, self.max_value)
    }
}
