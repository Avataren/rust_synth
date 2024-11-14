// First, make the synth module public
pub mod synth {
    // Re-export public types from each module
    pub use self::audio_graph::AudioGraph;
    pub use self::audio_node::AudioNode; // Make the trait public
    pub use self::audio_param::AudioParam;
    pub use self::bandlimited_wavetableoscillator::{
        initialize_wave_banks, BandlimitedWavetableOscillator,
    };
    pub use self::oscillator::{Oscillator, OscillatorType};
    pub use self::processor::AudioProcessor;
    // Declare the modules
    mod audio_graph;
    pub mod audio_node; // Make this public
    mod audio_param;
    mod bandlimited_wavetableoscillator;
    mod oscillator;
    mod processor;
}

// Re-export everything at the crate root level
pub use synth::{
    initialize_wave_banks, AudioGraph, AudioNode, AudioProcessor, BandlimitedWavetableOscillator,
    Oscillator, OscillatorType,
};
