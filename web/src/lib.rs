// src/lib.rs

use cpal_synth::{
    initialize_wave_banks, AudioGraph, AudioProcessor, BandlimitedWavetableOscillator, Oscillator,
    OscillatorType,
};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    // Initialize the audio graph
    let graph = AudioGraph::new().map_err(|e| JsValue::from_str(&e.to_string()))?;
    let context = graph.context.clone();

    // Initialize wave banks with the audio context
    initialize_wave_banks(&context).map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(())
}

#[wasm_bindgen]
pub struct Handle {
    graph: AudioGraph,
    master_gain: Arc<Mutex<AudioProcessor>>,
    wavetable_gain: Option<Arc<Mutex<AudioProcessor>>>,
    regular_gain: Option<Arc<Mutex<AudioProcessor>>>,
    wavetable_osc: Option<Arc<Mutex<BandlimitedWavetableOscillator>>>,
    regular_osc: Option<Arc<Mutex<Oscillator>>>,
}

#[wasm_bindgen]
impl Handle {
    #[wasm_bindgen]
    pub fn new() -> Result<Handle, JsValue> {
        let mut graph = AudioGraph::new().map_err(|e| JsValue::from_str(&e.to_string()))?;
        let context = graph.context.clone();

        // Initialize wave banks
        initialize_wave_banks(&context).map_err(|e| JsValue::from_str(&e.to_string()))?;

        let master_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));
        graph.add_node("master_gain", Box::new(master_gain.clone()));
        graph.set_output("master_gain");

        if let Ok(gain_node) = master_gain.try_lock() {
            gain_node.set_parameter("gain", 1.0);
        }

        Ok(Handle {
            graph,
            master_gain,
            wavetable_gain: None,
            regular_gain: None,
            wavetable_osc: None,
            regular_osc: None,
        })
    }

    #[wasm_bindgen]
    pub fn start(&mut self) -> Result<(), JsValue> {
        self.graph
            .start()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn sweep_wavetable(
        &mut self,
        osc_type: String,
        start_freq: f32,
        end_freq: f32,
        duration: f32,
    ) -> Result<(), JsValue> {
        let osc_type = match osc_type.as_str() {
            "sine" => OscillatorType::Sine,
            "square" => OscillatorType::Square,
            "sawtooth" => OscillatorType::Sawtooth,
            "triangle" => OscillatorType::Triangle,
            _ => return Err(JsValue::from_str("Invalid oscillator type")),
        };

        let context = self.graph.context.clone();
        let wavetable_osc = Arc::new(Mutex::new(
            BandlimitedWavetableOscillator::new(osc_type, &context)
                .map_err(|e| JsValue::from_str(&e.to_string()))?,
        ));
        let wavetable_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));

        // Set initial parameters and start the frequency sweep
        if let Ok(mut osc) = wavetable_osc.try_lock() {
            osc.frequency().set_value(start_freq);
            osc.gain().set_value(1.0);

            let current_sample = self.graph.context.current_sample();
            let sample_rate = self.graph.context.sample_rate();
            osc.frequency().exponential_ramp_to_value_at_time(
                end_freq,
                duration,
                current_sample,
                sample_rate,
            );
        }

        if let Ok(gain_node) = wavetable_gain.try_lock() {
            gain_node.set_parameter("gain", 0.5);
        }

        self.graph
            .add_node("wavetable_osc", Box::new(wavetable_osc.clone()));
        self.graph
            .add_node("wavetable_gain", Box::new(wavetable_gain.clone()));

        self.graph
            .connect("wavetable_osc", "wavetable_gain", "input");
        self.graph
            .connect("wavetable_gain", "master_gain", "input1");

        self.wavetable_gain = Some(wavetable_gain);
        self.wavetable_osc = Some(wavetable_osc);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn sweep_regular(
        &mut self,
        osc_type: String,
        start_freq: f32,
        end_freq: f32,
        duration: f32,
    ) -> Result<(), JsValue> {
        let osc_type = match osc_type.as_str() {
            "sine" => OscillatorType::Sine,
            "square" => OscillatorType::Square,
            "sawtooth" => OscillatorType::Sawtooth,
            "triangle" => OscillatorType::Triangle,
            _ => return Err(JsValue::from_str("Invalid oscillator type")),
        };

        let regular_osc = Arc::new(Mutex::new(Oscillator::new(osc_type)));
        let regular_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));

        // Set initial parameters and start the frequency sweep
        if let Ok(mut osc) = regular_osc.try_lock() {
            osc.frequency().set_value(start_freq);
            osc.gain().set_value(1.0);

            let current_sample = self.graph.context.current_sample();
            let sample_rate = self.graph.context.sample_rate();
            osc.frequency().exponential_ramp_to_value_at_time(
                end_freq,
                duration,
                current_sample,
                sample_rate,
            );
        }

        if let Ok(gain_node) = regular_gain.try_lock() {
            gain_node.set_parameter("gain", 0.5);
        }

        self.graph
            .add_node("regular_osc", Box::new(regular_osc.clone()));
        self.graph
            .add_node("regular_gain", Box::new(regular_gain.clone()));

        self.graph.connect("regular_osc", "regular_gain", "input");
        self.graph.connect("regular_gain", "master_gain", "input2");

        self.regular_gain = Some(regular_gain);
        self.regular_osc = Some(regular_osc);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_wavetable_gain(&mut self, value: f32, duration: Option<f32>) {
        if let Some(gain) = &self.wavetable_gain {
            if let Ok(gain_node) = gain.try_lock() {
                if let Some(duration) = duration {
                    let current_sample = self.graph.context.current_sample();
                    let sample_rate = self.graph.context.sample_rate();
                    gain_node.gain().linear_ramp_to_value_at_time(
                        value,
                        duration,
                        current_sample,
                        sample_rate,
                    );
                } else {
                    gain_node.set_parameter("gain", value);
                }
            }
        }
    }

    #[wasm_bindgen]
    pub fn set_regular_gain(&mut self, value: f32, duration: Option<f32>) {
        if let Some(gain) = &self.regular_gain {
            if let Ok(gain_node) = gain.try_lock() {
                if let Some(duration) = duration {
                    let current_sample = self.graph.context.current_sample();
                    let sample_rate = self.graph.context.sample_rate();
                    gain_node.gain().linear_ramp_to_value_at_time(
                        value,
                        duration,
                        current_sample,
                        sample_rate,
                    );
                } else {
                    gain_node.set_parameter("gain", value);
                }
            }
        }
    }

    #[wasm_bindgen]
    pub fn silence_wavetable(&mut self) {
        self.set_wavetable_gain(0.0, Some(0.1)); // 100ms fade out
    }

    #[wasm_bindgen]
    pub fn silence_regular(&mut self) {
        self.set_regular_gain(0.0, Some(0.1)); // 100ms fade out
    }
}
