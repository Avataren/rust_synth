// src/synth/oscillator.rs

use crate::synth::audio_context::AudioContext;
use crate::synth::audio_node::AudioNode;
use crate::synth::audio_param::AudioParam;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OscillatorType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

pub struct Oscillator {
    osc_type: OscillatorType,
    frequency: AudioParam,
    gain: AudioParam,
    phase: f32,
    triangle_state: f32,
}

impl Oscillator {
    pub fn new(osc_type: OscillatorType) -> Self {
        Self {
            osc_type,
            frequency: AudioParam::new(440.0, 0.01, 22050.0),
            gain: AudioParam::new(1.0, 0.0, 1.0),
            phase: 0.0,
            triangle_state: 0.0,
        }
    }

    pub fn frequency(&self) -> &AudioParam {
        &self.frequency
    }

    pub fn gain(&self) -> &AudioParam {
        &self.gain
    }

    fn poly_blep(&self, t: f32, dt: f32) -> f32 {
        if t < dt {
            let t = t / dt;
            2.0 * t - t * t - 1.0
        } else if t > 1.0 - dt {
            let t = (t - 1.0) / dt;
            t * t + 2.0 * t + 1.0
        } else {
            0.0
        }
    }

    fn process_bandlimited(&mut self, sample_rate: f32, current_sample: u64) -> f32 {
        let freq = self.frequency.get_value(current_sample);
        let dt = freq / sample_rate;

        let output = match self.osc_type {
            OscillatorType::Sine => (self.phase * 2.0 * PI).sin(),
            OscillatorType::Square => {
                let mut out = if self.phase < 0.5 { 1.0 } else { -1.0 };
                out += self.poly_blep(self.phase, dt);
                out -= self.poly_blep(fmod(self.phase + 0.5, 1.0), dt);
                out
            }
            OscillatorType::Sawtooth => {
                let mut out = 2.0 * self.phase - 1.0;
                out -= self.poly_blep(self.phase, dt);
                out
            }
            OscillatorType::Triangle => {
                // Generate band-limited square wave
                let mut square_bl = if self.phase < 0.5 { 1.0 } else { -1.0 };
                square_bl += self.poly_blep(self.phase, dt);
                square_bl -= self.poly_blep(fmod(self.phase + 0.5, 1.0), dt);

                // Simple integration with fixed coefficient
                let integration_scale = 2.0 * freq / sample_rate;
                self.triangle_state = 0.999 * self.triangle_state + square_bl * integration_scale;

                // Scale the output
                self.triangle_state
            }
        };

        self.phase += dt;
        self.phase = fmod(self.phase, 1.0);

        output
    }
}

fn fmod(x: f32, y: f32) -> f32 {
    x - (x / y).floor() * y
}

impl AudioNode for Oscillator {
    fn process(&mut self, context: &AudioContext, current_sample: u64) -> f32 {
        let sample_rate = context.sample_rate();
        let output = self.process_bandlimited(sample_rate, current_sample);
        let final_output = output * self.gain.get_value(current_sample);

        // Debug output every second
        // if current_sample % (sample_rate as u64) == 0 {
        //     println!(
        //         "Oscillator {:?}: freq={:.1}Hz, gain={:.2}, phase={:.3}, output={:.3}",
        //         self.osc_type,
        //         self.frequency.get_value(current_sample),
        //         self.gain.get_value(current_sample),
        //         self.phase,
        //         final_output
        //     );
        // }

        final_output
    }

    fn set_parameter(&self, name: &str, value: f32) {
        match name {
            "frequency" => self.frequency.set_value(value),
            "gain" => self.gain.set_value(value),
            _ => {}
        }
    }

    fn connect_input(&mut self, _name: &str, _node: Box<dyn AudioNode + Send>) {
        // Oscillators don't have inputs
    }

    fn clear_input(&mut self, _input_name: &str) {
        // No-op implementation for oscillators as they do not store inputs
    }

    fn clone_box(&self) -> Box<dyn AudioNode + Send> {
        Box::new(self.clone())
    }
}

// Implement Clone for Oscillator
impl Clone for Oscillator {
    fn clone(&self) -> Self {
        Self {
            osc_type: self.osc_type,
            frequency: self.frequency.clone(), // Use clone() instead of accessing private fields
            gain: self.gain.clone(),           // Use clone() instead of accessing private fields
            phase: self.phase,
            triangle_state: self.triangle_state,
        }
    }
}
