// lib.rs

// First, make the synth module public
pub mod synth {
    // Re-export public types from each module
    pub use self::audio_context::AudioContext;
    pub use self::audio_graph::AudioGraph;
    pub use self::audio_node::AudioNode; // Make the trait public
    pub use self::audio_param::AudioParam;
    pub use self::bandlimited_wavetableoscillator::{
        initialize_wave_banks, BandlimitedWavetableOscillator,
    };
    pub use self::oscillator::{Oscillator, OscillatorType};
    pub use self::processor::AudioProcessor;

    // Declare the modules
    pub mod audio_context; // Make this public if needed
    pub mod audio_graph;
    pub mod audio_node; // Make this public
    pub mod audio_param;
    pub mod bandlimited_wavetableoscillator;
    pub mod oscillator;
    pub mod processor;
}

// Re-export everything at the crate root level
pub use synth::{
    initialize_wave_banks, AudioContext, AudioGraph, AudioNode, AudioParam, AudioProcessor,
    BandlimitedWavetableOscillator, Oscillator, OscillatorType,
};
