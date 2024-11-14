use super::audio_node::AudioNode;
use super::audio_param::AudioParam;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

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

    pub fn frequency(&mut self) -> &mut AudioParam {
        &mut self.frequency
    }

    pub fn gain(&mut self) -> &mut AudioParam {
        &mut self.gain
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

    fn process_bandlimited(&mut self, sample_rate: f32) -> f32 {
        let freq = self.frequency.get_value();
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
                // Generate triangle by integrating band-limited square wave
                let square = if self.phase < 0.5 { 1.0 } else { -1.0 };
                let mut square_bl = square;

                // Add polyBLEP to square wave
                square_bl += self.poly_blep(self.phase, dt);
                square_bl -= self.poly_blep(fmod(self.phase + 0.5, 1.0), dt);

                // Leaky integrator to prevent DC offset accumulation
                let leak = 0.995;
                self.triangle_state =
                    leak * self.triangle_state + (square_bl * 8.0 * freq / sample_rate);

                // Scale the output
                4.0 * self.triangle_state
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
    fn process(&mut self, sample_rate: f32) -> f32 {
        self.process_bandlimited(sample_rate) * self.gain.get_value()
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "frequency" => self.frequency.set_value(value),
            "gain" => self.gain.set_value(value),
            _ => {}
        }
    }

    fn connect_input(&mut self, _name: &str, _node: Arc<Mutex<dyn AudioNode>>) {
        // Oscillators don't have inputs
    }
    fn clear_input(&mut self, _input_name: &str) {
        // No-op implementation for BandlimitedWavetableOscillator as it does not store inputs
    }
}
