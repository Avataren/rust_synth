use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub enum RampType {
    Linear,
    Exponential,
}

#[derive(Debug, Clone)]
pub struct RampEvent {
    start_value: f32,
    end_value: f32,
    start_time: Instant,
    duration: f32,
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
        self.events.clear(); // Cancel any scheduled ramps
    }

    pub fn set_value_at_time(&mut self, value: f32, time: Instant) {
        let value = self.clamp_value(value);
        self.events.push(RampEvent {
            start_value: self.current_value,
            end_value: value,
            start_time: time,
            duration: 0.0,
            ramp_type: RampType::Linear,
        });
    }

    pub fn reset(&mut self) {
        self.current_value = self.default_value;
        self.events.clear();
    }

    pub fn default_value(&self) -> f32 {
        self.default_value
    }

    pub fn linear_ramp_to_value_at_time(&mut self, value: f32, duration: f32) {
        let value = self.clamp_value(value);
        self.events.push(RampEvent {
            start_value: self.current_value,
            end_value: value,
            start_time: Instant::now(),
            duration,
            ramp_type: RampType::Linear,
        });
    }

    pub fn exponential_ramp_to_value_at_time(&mut self, value: f32, duration: f32) {
        let value = self.clamp_value(value.max(0.00001)); // Prevent zero for exp ramp
        self.events.push(RampEvent {
            start_value: self.current_value.max(0.00001),
            end_value: value,
            start_time: Instant::now(),
            duration,
            ramp_type: RampType::Exponential,
        });
    }

    pub fn cancel_scheduled_values(&mut self) {
        self.events.clear();
    }

    pub fn get_value(&mut self) -> f32 {
        if let Some(event) = self.events.first() {
            let elapsed = event.start_time.elapsed().as_secs_f32();

            if elapsed >= event.duration {
                // Event is complete
                self.current_value = event.end_value;
                self.events.remove(0);
            } else if event.duration > 0.0 {
                // Event is in progress
                let t = elapsed / event.duration;
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

    fn clamp_value(&self, value: f32) -> f32 {
        value.clamp(self.min_value, self.max_value)
    }
}
