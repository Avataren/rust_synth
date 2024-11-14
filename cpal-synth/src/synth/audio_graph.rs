use super::audio_node::AudioNode;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct AudioGraph {
    nodes: HashMap<String, Arc<Mutex<dyn AudioNode>>>,
    output_node: Arc<Mutex<dyn AudioNode>>,
    playing: Arc<AtomicBool>,
    stream: Option<cpal::Stream>,
}

impl AudioGraph {
    pub fn new() -> anyhow::Result<Self> {
        use super::processor::AudioProcessor;
        // Create a default gain node as the output
        let output_node = Arc::new(Mutex::new(AudioProcessor::new("gain")));

        Ok(Self {
            nodes: HashMap::new(),
            output_node,
            playing: Arc::new(AtomicBool::new(false)),
            stream: None,
        })
    }

    pub fn add_node(&mut self, name: &str, node: Arc<Mutex<dyn AudioNode>>) {
        self.nodes.insert(name.to_string(), node);
    }

    pub fn connect(&mut self, from: &str, to: &str, input_name: &str) {
        if let Some(from_node) = self.nodes.get(from).cloned() {
            if let Some(to_node) = self.nodes.get_mut(to) {
                if let Ok(mut node) = to_node.lock() {
                    node.connect_input(input_name, from_node);
                }
            }
        }
    }

    pub fn disconnect(&mut self, from: &str, to: &str) {
        if let Some(to_node) = self.nodes.get_mut(to) {
            if let Ok(mut node) = to_node.lock() {
                node.clear_input(from);
            }
        }
    }

    pub fn set_output(&mut self, node_name: &str) {
        if let Some(node) = self.nodes.get(node_name) {
            self.output_node = node.clone();
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

        let config = device.default_output_config()?;

        let playing = self.playing.clone();
        let output_node = self.output_node.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                self.build_stream::<f32>(&device, &config.into(), playing, output_node)?
            }
            cpal::SampleFormat::I16 => {
                self.build_stream::<i16>(&device, &config.into(), playing, output_node)?
            }
            cpal::SampleFormat::U16 => {
                self.build_stream::<u16>(&device, &config.into(), playing, output_node)?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported sample format: {:?}",
                    config.sample_format()
                ));
            }
        };

        stream.play()?;
        self.stream = Some(stream);
        self.playing.store(true, Ordering::SeqCst);

        Ok(())
    }

    fn build_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        playing: Arc<AtomicBool>,
        output_node: Arc<Mutex<dyn AudioNode>>,
    ) -> anyhow::Result<cpal::Stream>
    where
        T: Sample + FromSample<f32> + cpal::SizedSample,
    {
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                Self::write_data(data, channels, &playing, &output_node, sample_rate);
            },
            move |err| {
                eprintln!("Audio stream error: {}", err);
            },
            None,
        )?;

        Ok(stream)
    }

    fn write_data<T>(
        output: &mut [T],
        channels: usize,
        playing: &Arc<AtomicBool>,
        output_node: &Arc<Mutex<dyn AudioNode>>,
        sample_rate: f32,
    ) where
        T: Sample + FromSample<f32>,
    {
        if !playing.load(Ordering::SeqCst) {
            for sample in output.iter_mut() {
                *sample = T::EQUILIBRIUM;
            }
            return;
        }

        for frame in output.chunks_mut(channels) {
            let value = output_node
                .lock()
                .map(|mut node| node.process(sample_rate))
                .unwrap_or(0.0);

            let sample_value = T::from_sample(value);
            for sample in frame.iter_mut() {
                *sample = sample_value;
            }
        }
    }

    pub fn stop(&mut self) {
        self.playing.store(false, Ordering::SeqCst);
        self.stream = None;
    }
}
