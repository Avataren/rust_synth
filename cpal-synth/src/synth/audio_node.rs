// src/synth/audio_node.rs

use crate::synth::audio_context::AudioContext;
use std::sync::{Arc, Mutex};

pub trait AudioNode: Send {
    fn process(&mut self, context: &AudioContext, current_sample: u64) -> f32;
    fn set_parameter(&self, name: &str, value: f32);
    fn connect_input(&mut self, name: &str, node: Box<dyn AudioNode + Send>);
    fn clear_input(&mut self, input_name: &str);

    // Optional method to clone the node
    fn clone_box(&self) -> Box<dyn AudioNode + Send>;
}

// Implement Clone for Box<dyn AudioNode + Send>
impl Clone for Box<dyn AudioNode + Send> {
    fn clone(&self) -> Box<dyn AudioNode + Send> {
        self.clone_box()
    }
}

impl<T> AudioNode for Arc<Mutex<T>>
where
    T: AudioNode + Send + 'static,
{
    fn process(&mut self, context: &AudioContext, current_sample: u64) -> f32 {
        let mut node = self.lock().unwrap();
        node.process(context, current_sample)
    }

    fn set_parameter(&self, name: &str, value: f32) {
        let node = self.lock().unwrap();
        node.set_parameter(name, value);
    }

    fn connect_input(&mut self, name: &str, input: Box<dyn AudioNode + Send>) {
        let mut node = self.lock().unwrap();
        node.connect_input(name, input);
    }

    fn clear_input(&mut self, name: &str) {
        let mut node = self.lock().unwrap();
        node.clear_input(name);
    }

    fn clone_box(&self) -> Box<dyn AudioNode + Send> {
        Box::new(self.clone())
    }
}
