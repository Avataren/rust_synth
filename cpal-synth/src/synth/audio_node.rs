use std::sync::{Arc, Mutex};

pub trait AudioNode: Send + Sync {
    fn process(&mut self, sample_rate: f32) -> f32;
    fn set_parameter(&mut self, name: &str, value: f32);
    fn connect_input(&mut self, name: &str, node: Arc<Mutex<dyn AudioNode>>);
    fn clear_input(&mut self, input_name: &str);
}
