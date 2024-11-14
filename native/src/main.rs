use cpal_synth::{
    initialize_wave_banks, AudioGraph, AudioNode, AudioProcessor, BandlimitedWavetableOscillator,
    Oscillator, OscillatorType,
};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let mut graph = AudioGraph::new()?;
    initialize_wave_banks();
    // Create master gain node and set it as the output
    let master_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));
    graph.add_node("master_gain", master_gain.clone());
    graph.set_output("master_gain");
    graph.start()?;
    // Set master gain to maximum
    {
        let mut master = master_gain.lock().unwrap();
        master.set_parameter("gain", 1.0);
    }

    // Define the sequence of oscillator types for sweeping
    let oscillator_types = [
        OscillatorType::Sine,
        OscillatorType::Square,
        OscillatorType::Sawtooth,
        OscillatorType::Triangle,
    ];

    // Iterate through each oscillator type and play both implementations in sequence
    for &osc_type in &oscillator_types {
        println!(
            "Sweeping BandlimitedWavetableOscillator with {:?} waveform...",
            osc_type
        );

        // Bandlimited Wavetable Oscillator
        {
            let wavetable_osc = Arc::new(Mutex::new(BandlimitedWavetableOscillator::new(osc_type)));
            let wavetable_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));

            graph.add_node("wavetable_osc", wavetable_osc.clone());
            graph.add_node("wavetable_gain", wavetable_gain.clone());

            graph.connect("wavetable_osc", "wavetable_gain", "input");
            graph.connect("wavetable_gain", "master_gain", "input1");

            // Set up and perform frequency sweep
            {
                let mut osc_node = wavetable_osc.lock().unwrap();
                osc_node.frequency().set_value(20.0);
                osc_node.gain().set_value(1.0);
            }
            {
                let mut gain_node = wavetable_gain.lock().unwrap();
                gain_node.set_parameter("gain", 0.5);
            }
            {
                let mut osc_node = wavetable_osc.lock().unwrap();
                osc_node
                    .frequency()
                    .exponential_ramp_to_value_at_time(10000.0, 5.0);
            }

            sleep(Duration::from_secs(5));

            // Silence the wavetable oscillator by setting its gain to 0.0
            {
                let mut gain_node = wavetable_gain.lock().unwrap();
                gain_node.set_parameter("gain", 0.0);
            }
        }

        println!("Sweeping Oscillator with {:?} waveform...", osc_type);

        // Regular Oscillator
        {
            let regular_osc = Arc::new(Mutex::new(Oscillator::new(osc_type)));
            let regular_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));

            graph.add_node("regular_osc", regular_osc.clone());
            graph.add_node("regular_gain", regular_gain.clone());

            graph.connect("regular_osc", "regular_gain", "input");
            graph.connect("regular_gain", "master_gain", "input2");

            // Set up and perform frequency sweep
            {
                let mut osc_node = regular_osc.lock().unwrap();
                osc_node.frequency().set_value(20.0);
                osc_node.gain().set_value(1.0);
            }
            {
                let mut gain_node = regular_gain.lock().unwrap();
                gain_node.set_parameter("gain", 0.5);
            }
            {
                let mut osc_node = regular_osc.lock().unwrap();
                osc_node
                    .frequency()
                    .exponential_ramp_to_value_at_time(10000.0, 5.0);
            }

            sleep(Duration::from_secs(5));

            // Silence the regular oscillator by setting its gain to 0.0
            {
                let mut gain_node = regular_gain.lock().unwrap();
                gain_node.set_parameter("gain", 0.0);
            }
        }

        println!(
            "Finished sweeping {:?} waveform for both oscillators.\n",
            osc_type
        );
    }

    println!("All sweeps completed!");
    Ok(())
}
