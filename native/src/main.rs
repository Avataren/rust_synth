use cpal_synth::{
    initialize_wave_banks,
    AudioGraph,
    AudioNode, // Added AudioNode
    AudioProcessor,
    BandlimitedWavetableOscillator,
    Oscillator,
    OscillatorType,
};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    println!("Initializing audio system...");
    let mut graph = AudioGraph::new()?;
    let context = graph.context.clone();
    initialize_wave_banks(&context)?;
    println!("Wave banks initialized");

    // Create master gain node and set it as the output
    let master_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));
    graph.add_node("master_gain", Box::new(master_gain.clone()));
    graph.set_output("master_gain");
    println!("Master gain node created and set as output");

    graph.start(Some(256))?;
    println!("Audio graph started");

    // Set master gain to maximum
    {
        let master = master_gain.lock().unwrap();
        master.set_parameter("gain", 1.0);
        println!("Set master gain to 1.0");
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
            "\nSweeping BandlimitedWavetableOscillator with {:?} waveform...",
            osc_type
        );

        // Bandlimited Wavetable Oscillator
        {
            println!("Creating wavetable oscillator...");
            let wavetable_osc = Arc::new(Mutex::new(BandlimitedWavetableOscillator::new(
                osc_type, &context,
            )?));
            let wavetable_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));

            graph.add_node("wavetable_osc", Box::new(wavetable_osc.clone()));
            println!("Added wavetable oscillator to graph");
            graph.add_node("wavetable_gain", Box::new(wavetable_gain.clone()));
            println!("Added wavetable gain to graph");

            graph.connect("wavetable_osc", "wavetable_gain", "input");
            println!("Connected oscillator to gain node");
            graph.connect("wavetable_gain", "master_gain", "input1");
            println!("Connected gain to master gain");

            // Set up and perform frequency sweep
            {
                let osc_node = wavetable_osc.lock().unwrap();
                osc_node.frequency().set_value(20.0);
                println!("Set initial frequency to 20.0 Hz");
                osc_node.gain().set_value(1.0);
                println!("Set oscillator gain to 1.0");

                let current_sample = graph.context.current_sample();
                let sample_rate = graph.context.sample_rate();
                osc_node.frequency().exponential_ramp_to_value_at_time(
                    10000.0,
                    5.0,
                    current_sample,
                    sample_rate,
                );
                println!("Set frequency ramp 20 Hz -> 10kHz over 5 seconds");
            }
            {
                let gain_node = wavetable_gain.lock().unwrap();
                gain_node.set_parameter("gain", 0.5);
                println!("Set wavetable gain to 0.5");
            }

            sleep(Duration::from_secs(5));
            println!("Finished wavetable sweep");

            // Silence the wavetable oscillator by setting its gain to 0.0
            {
                let gain_node = wavetable_gain.lock().unwrap();
                gain_node.set_parameter("gain", 0.0);
                println!("Set wavetable gain to 0.0");
            }
        }

        println!("\nSweeping Oscillator with {:?} waveform...", osc_type);

        // Regular Oscillator
        {
            println!("Creating regular oscillator...");
            let regular_osc = Arc::new(Mutex::new(Oscillator::new(osc_type)));
            let regular_gain = Arc::new(Mutex::new(AudioProcessor::new("gain")));

            graph.add_node("regular_osc", Box::new(regular_osc.clone()));
            println!("Added regular oscillator to graph");
            graph.add_node("regular_gain", Box::new(regular_gain.clone()));
            println!("Added regular gain to graph");

            graph.connect("regular_osc", "regular_gain", "input");
            println!("Connected oscillator to gain node");
            graph.connect("regular_gain", "master_gain", "input2");
            println!("Connected gain to master gain");

            // Set up and perform frequency sweep
            {
                let osc_node = regular_osc.lock().unwrap();
                osc_node.frequency().set_value(20.0);
                println!("Set initial frequency to 20.0 Hz");
                osc_node.gain().set_value(1.0);
                println!("Set oscillator gain to 1.0");

                let current_sample = graph.context.current_sample();
                let sample_rate = graph.context.sample_rate();
                osc_node.frequency().exponential_ramp_to_value_at_time(
                    10000.0,
                    5.0,
                    current_sample,
                    sample_rate,
                );
                println!("Set frequency ramp 20 Hz -> 10kHz over 5 seconds");
            }
            {
                let gain_node = regular_gain.lock().unwrap();
                gain_node.set_parameter("gain", 0.5);
                println!("Set regular gain to 0.5");
            }

            sleep(Duration::from_secs(5));
            println!("Finished regular oscillator sweep");

            // Silence the regular oscillator by setting its gain to 0.0
            {
                let gain_node = regular_gain.lock().unwrap();
                gain_node.set_parameter("gain", 0.0);
                println!("Set regular gain to 0.0");
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
