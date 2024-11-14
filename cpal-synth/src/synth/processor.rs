use super::audio_node::AudioNode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AudioProcessor {
    inputs: HashMap<String, Arc<Mutex<dyn AudioNode>>>,
    processor_type: String,
    parameters: HashMap<String, f32>,
}

impl AudioProcessor {
    pub fn new(processor_type: &str) -> Self {
        Self {
            inputs: HashMap::new(),
            processor_type: processor_type.to_string(),
            parameters: HashMap::new(),
        }
    }
}

impl AudioNode for AudioProcessor {
    fn process(&mut self, sample_rate: f32) -> f32 {
        match self.processor_type.as_str() {
            "gain" => {
                let mut sum = 0.0;

                // Process all inputs and sum them
                for input in self.inputs.values() {
                    if let Ok(mut node) = input.lock() {
                        sum += node.process(sample_rate);
                    }
                }

                let gain = self.parameters.get("gain").copied().unwrap_or(1.0);
                sum * gain
            }
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        self.parameters.insert(name.to_string(), value);
    }

    fn connect_input(&mut self, name: &str, node: Arc<Mutex<dyn AudioNode>>) {
        self.inputs.insert(name.to_string(), node);
    }

    fn clear_input(&mut self, input_name: &str) {
        self.inputs.remove(input_name);
    }
}
