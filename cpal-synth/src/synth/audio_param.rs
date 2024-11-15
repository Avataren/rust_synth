use super::audio_context::AUDIO_CONTEXT;

#[derive(Debug, Clone, Copy)]
pub enum RampType {
    Linear,
    Exponential,
}

#[derive(Debug, Clone)]
struct RampEvent {
    start_value: f32,
    end_value: f32,
    start_sample: u64,
    duration_samples: u64,
    ramp_type: RampType,
}

pub struct AudioParam {
    current_value: f32,
    default_value: f32,
    min_value: f32,
    max_value: f32,
    events: Vec<RampEvent>,
}

impl AudioParam {
    pub fn new(default_value: f32, min_value: f32, max_value: f32) -> Self {
        Self {
            current_value: default_value,
            default_value,
            min_value,
            max_value,
            events: Vec::new(),
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.current_value = self.clamp_value(value);
        self.events.clear();
    }

    pub fn set_value_at_time(&mut self, value: f32, time: f64) {
        let value = self.clamp_value(value);
        if let Ok(context) = AUDIO_CONTEXT.lock() {
            let sample_offset = (time * context.sample_rate() as f64) as u64;
            let current_sample = context.current_sample();
            self.events.push(RampEvent {
                start_value: self.current_value,
                end_value: value,
                start_sample: current_sample + sample_offset,
                duration_samples: 0,
                ramp_type: RampType::Linear,
            });
        }
    }

    pub fn linear_ramp_to_value_at_time(&mut self, value: f32, duration_seconds: f32) {
        let value = self.clamp_value(value);
        if let Ok(context) = AUDIO_CONTEXT.lock() {
            let duration_samples = (duration_seconds * context.sample_rate()) as u64;
            let current_sample = context.current_sample();
            self.events.push(RampEvent {
                start_value: self.current_value,
                end_value: value,
                start_sample: current_sample,
                duration_samples,
                ramp_type: RampType::Linear,
            });
        }
    }

    pub fn exponential_ramp_to_value_at_time(&mut self, value: f32, duration_seconds: f32) {
        let value = self.clamp_value(value.max(0.00001));
        if let Ok(context) = AUDIO_CONTEXT.lock() {
            let duration_samples = (duration_seconds * context.sample_rate()) as u64;
            let current_sample = context.current_sample();
            self.events.push(RampEvent {
                start_value: self.current_value.max(0.00001),
                end_value: value,
                start_sample: current_sample,
                duration_samples,
                ramp_type: RampType::Exponential,
            });
        }
    }

    pub fn get_value(&mut self) -> f32 {
        let current_sample = if let Ok(context) = AUDIO_CONTEXT.lock() {
            context.current_sample()
        } else {
            return self.current_value;
        };

        if let Some(event) = self.events.first() {
            let samples_elapsed = current_sample.saturating_sub(event.start_sample);

            if samples_elapsed >= event.duration_samples {
                // Event is complete
                self.current_value = event.end_value;
                self.events.remove(0);
            } else if event.duration_samples > 0 {
                // Event is in progress
                let t = samples_elapsed as f32 / event.duration_samples as f32;
                self.current_value = match event.ramp_type {
                    RampType::Linear => {
                        event.start_value + (event.end_value - event.start_value) * t
                    }
                    RampType::Exponential => {
                        event.start_value * (event.end_value / event.start_value).powf(t)
                    }
                };
            } else {
                // Immediate value change
                self.current_value = event.end_value;
                self.events.remove(0);
            }
        }

        self.current_value
    }

    pub fn reset(&mut self) {
        self.current_value = self.default_value;
        self.events.clear();
    }

    pub fn default_value(&self) -> f32 {
        self.default_value
    }

    pub fn cancel_scheduled_values(&mut self) {
        self.events.clear();
    }

    fn clamp_value(&self, value: f32) -> f32 {
        value.clamp(self.min_value, self.max_value)
    }
}
