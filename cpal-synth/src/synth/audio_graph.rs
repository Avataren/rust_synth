use crate::synth::audio_context::AudioContext;
use crate::synth::audio_node::AudioNode;
use crate::synth::processor::AudioProcessor;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[cfg(feature = "cpal-output")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::BufferSize;
#[cfg(feature = "cpal-output")]
use cpal::{FromSample, Sample};

pub struct AudioGraph {
    nodes: HashMap<String, Box<dyn AudioNode + Send>>,
    output_node: Box<dyn AudioNode + Send>,
    playing: Arc<AtomicBool>,
    #[cfg(feature = "cpal-output")]
    stream: Option<cpal::Stream>,
    pub context: Arc<AudioContext>,
}

impl AudioGraph {
    pub fn new() -> anyhow::Result<Self> {
        println!("Creating new AudioGraph");
        #[cfg(feature = "cpal-output")]
        {
            let host = cpal::default_host();
            println!("Using audio host: {}", host.id().name());

            let device = host
                .default_output_device()
                .ok_or_else(|| anyhow::anyhow!("No output device available"))?;
            println!("Using output device: {}", device.name()?);

            let config_format = device.default_output_config()?;
            println!("Default config format: {:?}", config_format);

            let sample_rate = config_format.sample_rate().0 as f32;
            println!("Sample rate: {}", sample_rate);

            let context = Arc::new(AudioContext::new(sample_rate));

            let output_node = Box::new(AudioProcessor::new("gain"));

            Ok(Self {
                nodes: HashMap::new(),
                output_node,
                playing: Arc::new(AtomicBool::new(false)),
                stream: None,
                context,
            })
        }

        #[cfg(not(feature = "cpal-output"))]
        {
            let sample_rate = 44100.0;
            let context = Arc::new(AudioContext::new(sample_rate));

            let output_node = Box::new(AudioProcessor::new("gain"));

            Ok(Self {
                nodes: HashMap::new(),
                output_node,
                playing: Arc::new(AtomicBool::new(false)),
                stream: None,
                context,
            })
        }
    }

    pub fn add_node(&mut self, name: &str, node: Box<dyn AudioNode + Send>) {
        println!("Adding node: {}", name);
        self.nodes.insert(name.to_string(), node);
    }

    pub fn connect(&mut self, from: &str, to: &str, input_name: &str) {
        println!("Connecting {} to {} at input {}", from, to, input_name);
        if let Some(from_node) = self.nodes.get(from) {
            let from_node_clone = from_node.clone_box();
            if let Some(to_node) = self.nodes.get_mut(to) {
                to_node.connect_input(input_name, from_node_clone);
                println!("Connection successful");
            } else {
                println!("Destination node '{}' not found", to);
            }
        } else {
            println!("Source node '{}' not found", from);
        }
    }

    pub fn disconnect(&mut self, from: &str, to: &str) {
        println!("Disconnecting {} from {}", from, to);
        if let Some(to_node) = self.nodes.get_mut(to) {
            to_node.clear_input(from);
        }
    }

    pub fn set_output(&mut self, node_name: &str) {
        println!("Setting output to node: {}", node_name);
        if let Some(node) = self.nodes.get(node_name) {
            self.output_node = node.clone_box();
            println!("Output node set successfully");
        } else {
            println!("Output node '{}' not found", node_name);
        }
    }

    #[cfg(feature = "cpal-output")]
    fn write_data<T>(
        output: &mut [T],
        channels: usize,
        playing: &Arc<AtomicBool>,
        output_node: &mut dyn AudioNode,
        context: Arc<AudioContext>,
    ) where
        T: Sample + FromSample<f32> + Send,
    {
        let num_frames = output.len() / channels;
        println!(
            "Received buffer size: {} ({} frames)",
            output.len(),
            num_frames
        );

        if !playing.load(Ordering::SeqCst) {
            for sample in output.iter_mut() {
                *sample = T::EQUILIBRIUM;
            }
            return;
        }

        let base_sample = context.current_sample();

        for (frame_index, frame) in output.chunks_mut(channels).enumerate() {
            let current_sample = base_sample + frame_index as u64;

            let sample_value = output_node.process(&*context, current_sample);

            let sample_value = T::from_sample(sample_value);
            for sample in frame.iter_mut() {
                *sample = sample_value;
            }
        }

        context.increment_samples(num_frames as u64);
    }

    // #[cfg(feature = "cpal-output")]
    // fn write_data<T>(
    //     output: &mut [T],
    //     channels: usize,
    //     playing: &Arc<AtomicBool>,
    //     output_node: &mut dyn AudioNode,
    //     context: Arc<AudioContext>,
    // ) where
    //     T: Sample + FromSample<f32> + Send,
    // {
    //     use std::time::Instant;

    //     if !playing.load(Ordering::SeqCst) {
    //         for sample in output.iter_mut() {
    //             *sample = T::EQUILIBRIUM;
    //         }
    //         return;
    //     }

    //     let num_frames = output.len() / channels;
    //     let sample_rate = context.sample_rate(); // Ensure this method provides sample rate in Hz
    //     let buffer_duration = num_frames as f32 / sample_rate;

    //     // Start timing
    //     let start_time = Instant::now();

    //     let base_sample = context.current_sample();

    //     // Process audio data
    //     for (frame_index, frame) in output.chunks_mut(channels).enumerate() {
    //         let current_sample = base_sample + frame_index as u64;

    //         let sample_value = output_node.process(&*context, current_sample);

    //         let sample_value = T::from_sample(sample_value);
    //         for sample in frame.iter_mut() {
    //             *sample = sample_value;
    //         }
    //     }

    //     context.increment_samples(num_frames as u64);

    //     // Stop timing
    //     let processing_time = start_time.elapsed();
    //     let processing_time_secs = processing_time.as_secs_f32();

    //     // Compute CPU usage
    //     let cpu_usage = (processing_time_secs / buffer_duration) * 100.0;

    //     // Log CPU usage
    //     println!(
    //         "Processed buffer of {} frames in {:.3} ms (CPU Usage: {:.2}%)",
    //         num_frames,
    //         processing_time_secs * 1000.0,
    //         cpu_usage
    //     );

    //     // Optional: Warn if close to underrun
    //     if cpu_usage > 80.0 {
    //         eprintln!(
    //             "Warning: High CPU usage detected ({:.2}%). Risk of buffer underrun!",
    //             cpu_usage
    //         );
    //     }
    // }

    pub fn start(&mut self, buffer_size: Option<usize>) -> anyhow::Result<()> {
        println!("Starting audio graph");
        #[cfg(feature = "cpal-output")]
        {
            let host = cpal::default_host();
            let device = host
                .default_output_device()
                .ok_or_else(|| anyhow::anyhow!("No output device available"))?;

            let supported_configs = device.supported_output_configs()?;
            for config in supported_configs {
                println!("Supported config: {:?}", config);
            }

            let config_format = device.default_output_config()?;
            println!("Using config format: {:?}", config_format);

            // Start with default config and apply buffer size if specified
            let mut config: cpal::StreamConfig = config_format.clone().into();
            if let Some(size) = buffer_size {
                println!("Setting buffer size to {} frames", size);
                config.buffer_size = BufferSize::Fixed(size as u32);
            } else {
                println!("Using default buffer size");
            }

            let sample_rate = config.sample_rate.0 as f32;
            self.context = Arc::new(AudioContext::new(sample_rate));

            let playing = self.playing.clone();
            let output_node = self.output_node.clone_box();
            let context = self.context.clone();

            let stream = match config_format.sample_format() {
                cpal::SampleFormat::F32 => {
                    println!("Using F32 sample format");
                    Self::build_stream::<f32>(&device, &config, playing, output_node, context)?
                }
                cpal::SampleFormat::I16 => {
                    println!("Using I16 sample format");
                    Self::build_stream::<i16>(&device, &config, playing, output_node, context)?
                }
                cpal::SampleFormat::U16 => {
                    println!("Using U16 sample format");
                    Self::build_stream::<u16>(&device, &config, playing, output_node, context)?
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported sample format: {:?}",
                        config_format.sample_format()
                    ));
                }
            };
            println!("Effective buffer size: {:?}", config.buffer_size);

            println!("Starting audio stream");
            stream.play()?;
            println!("Audio stream started successfully");
            self.stream = Some(stream);
        }

        self.playing.store(true, Ordering::SeqCst);
        Ok(())
    }

    #[cfg(feature = "cpal-output")]
    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        playing: Arc<AtomicBool>,
        mut output_node: Box<dyn AudioNode + Send>,
        context: Arc<AudioContext>,
    ) -> anyhow::Result<cpal::Stream>
    where
        T: Sample + FromSample<f32> + cpal::SizedSample + Send + 'static,
    {
        let channels = config.channels as usize;
        println!("Building stream with {} channels", channels);

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                Self::write_data(data, channels, &playing, &mut *output_node, context.clone());
            },
            move |err| {
                eprintln!("Audio stream error: {}", err);
            },
            None,
        )?;

        Ok(stream)
    }

    pub fn stop(&mut self) {
        println!("Stopping audio graph");
        self.playing.store(false, Ordering::SeqCst);
        #[cfg(feature = "cpal-output")]
        {
            self.stream = None;
        }
    }
}
