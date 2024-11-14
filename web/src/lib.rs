use cpal_synth::{
    initialize_wave_banks, AudioGraph, AudioNode, AudioProcessor, BandlimitedWavetableOscillator,
    Oscillator, OscillatorType,
};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    initialize_wave_banks();
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

        let master_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));
        graph.add_node("master_gain", master_gain.clone());
        graph.set_output("master_gain");

        if let Ok(mut gain) = master_gain.try_lock() {
            gain.set_parameter("gain", 1.0);
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
    pub fn sweep_wavetable(&mut self, osc_type: String) -> Result<(), JsValue> {
        let osc_type = match osc_type.as_str() {
            "sine" => OscillatorType::Sine,
            "square" => OscillatorType::Square,
            "sawtooth" => OscillatorType::Sawtooth,
            "triangle" => OscillatorType::Triangle,
            _ => return Err(JsValue::from_str("Invalid oscillator type")),
        };

        let wavetable_osc = Arc::new(Mutex::new(BandlimitedWavetableOscillator::new(osc_type)));
        let wavetable_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));

        if let Ok(mut osc) = wavetable_osc.try_lock() {
            osc.frequency().set_value(20.0);
            osc.gain().set_value(1.0);
        }

        if let Ok(mut gain) = wavetable_gain.try_lock() {
            gain.set_parameter("gain", 0.5);
        }

        self.graph.add_node("wavetable_osc", wavetable_osc.clone());
        self.graph
            .add_node("wavetable_gain", wavetable_gain.clone());

        self.graph
            .connect("wavetable_osc", "wavetable_gain", "input");
        self.graph
            .connect("wavetable_gain", "master_gain", "input1");

        self.wavetable_gain = Some(wavetable_gain);
        self.wavetable_osc = Some(wavetable_osc);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn set_wavetable_frequency(&mut self, freq: f32) {
        if let Some(osc) = &self.wavetable_osc {
            if let Ok(mut osc) = osc.try_lock() {
                osc.frequency().set_value(freq);
            }
        }
    }

    #[wasm_bindgen]
    pub fn set_regular_frequency(&mut self, freq: f32) {
        if let Some(osc) = &self.regular_osc {
            if let Ok(mut osc) = osc.try_lock() {
                osc.frequency().set_value(freq);
            }
        }
    }

    #[wasm_bindgen]
    pub fn sweep_regular(&mut self, osc_type: String) -> Result<(), JsValue> {
        let osc_type = match osc_type.as_str() {
            "sine" => OscillatorType::Sine,
            "square" => OscillatorType::Square,
            "sawtooth" => OscillatorType::Sawtooth,
            "triangle" => OscillatorType::Triangle,
            _ => return Err(JsValue::from_str("Invalid oscillator type")),
        };

        let regular_osc = Arc::new(Mutex::new(Oscillator::new(osc_type)));
        let regular_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));

        if let Ok(mut osc) = regular_osc.try_lock() {
            osc.frequency().set_value(20.0);
            osc.gain().set_value(1.0);
        }

        if let Ok(mut gain) = regular_gain.try_lock() {
            gain.set_parameter("gain", 0.5);
        }

        self.graph.add_node("regular_osc", regular_osc.clone());
        self.graph.add_node("regular_gain", regular_gain.clone());

        self.graph.connect("regular_osc", "regular_gain", "input");
        self.graph.connect("regular_gain", "master_gain", "input2");

        self.regular_gain = Some(regular_gain);
        self.regular_osc = Some(regular_osc);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn silence_wavetable(&mut self) {
        if let Some(gain) = &self.wavetable_gain {
            if let Ok(mut gain) = gain.try_lock() {
                gain.set_parameter("gain", 0.0);
            }
        }
    }

    #[wasm_bindgen]
    pub fn silence_regular(&mut self) {
        if let Some(gain) = &self.regular_gain {
            if let Ok(mut gain) = gain.try_lock() {
                gain.set_parameter("gain", 0.0);
            }
        }
    }
}
