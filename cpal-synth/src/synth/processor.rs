use crate::synth::audio_context::AudioContext;
use crate::synth::audio_node::AudioNode;
use crate::synth::audio_param::AudioParam;
use std::collections::HashMap;

pub struct AudioProcessor {
    gain: AudioParam,
    inputs: HashMap<String, Box<dyn AudioNode + Send>>,
}

impl AudioProcessor {
    pub fn new(_type: &str) -> Self {
        println!("Creating new AudioProcessor");
        Self {
            gain: AudioParam::new(1.0, 0.0, 1.0),
            inputs: HashMap::new(),
        }
    }

    pub fn gain(&self) -> &AudioParam {
        &self.gain
    }

    // Public method that delegates to the trait method
    pub fn set_parameter(&self, name: &str, value: f32) {
        println!("Setting parameter {} to {}", name, value);
        AudioNode::set_parameter(self, name, value)
    }
}

impl AudioNode for AudioProcessor {
    fn process(&mut self, context: &AudioContext, current_sample: u64) -> f32 {
        // Sum all inputs
        let input_signal: f32 = self
            .inputs
            .values_mut()
            .map(|node| node.process(context, current_sample))
            .sum();

        let gain = self.gain.get_value(current_sample);
        let output = input_signal * gain;

        // Debug output every second (assuming 44.1kHz sample rate)
        // if current_sample % 44100 == 0 {
        //     println!(
        //         "AudioProcessor: inputs={}, input_signal={:.3}, gain={:.3}, output={:.3}",
        //         self.inputs.len(),
        //         input_signal,
        //         gain,
        //         output
        //     );
        // }

        output
    }

    fn set_parameter(&self, name: &str, value: f32) {
        match name {
            "gain" => self.gain.set_value(value),
            _ => println!("Unknown parameter: {}", name),
        }
    }

    fn connect_input(&mut self, name: &str, node: Box<dyn AudioNode + Send>) {
        println!(
            "AudioProcessor: Connecting input '{}' (total inputs: {})",
            name,
            self.inputs.len() + 1
        );
        self.inputs.insert(name.to_string(), node);
    }

    fn clear_input(&mut self, input_name: &str) {
        println!("AudioProcessor: Clearing input '{}'", input_name);
        self.inputs.remove(input_name);
    }

    fn clone_box(&self) -> Box<dyn AudioNode + Send> {
        Box::new(self.clone())
    }
}

impl Clone for AudioProcessor {
    fn clone(&self) -> Self {
        Self {
            gain: self.gain.clone(),
            inputs: self.inputs.clone(),
        }
    }
}
