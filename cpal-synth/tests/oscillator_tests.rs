use cpal_synth::{initialize_wave_banks, AudioContext, AudioNode, Oscillator, OscillatorType};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Arc<AudioContext> {
        let context = Arc::new(AudioContext::new(44100.0));
        initialize_wave_banks(&context).unwrap();
        context
    }

    #[test]
    fn test_linear_ramp() {
        let context = setup();
        let mut osc = Oscillator::new(OscillatorType::Sine);
        osc.frequency().set_value(440.0);

        println!("Initial frequency: {}", osc.frequency().get_value(0));

        // Ramp from 440Hz to 880Hz over 4410 samples (0.1s)
        let current_sample = 0;
        osc.frequency().linear_ramp_to_value_at_time(
            880.0,
            0.1,
            current_sample,
            context.sample_rate(),
        );

        let check_points = [
            (0, 440.0),
            (1102, 550.0),
            (2205, 660.0),
            (3307, 770.0),
            (4410, 880.0),
        ];

        // Process samples and check at specific points
        for (sample_index, expected_freq) in check_points {
            // Process up to this point
            let freq = osc.frequency().get_value(sample_index);
            println!(
                "Sample {}: freq = {:.1}, expected = {:.1}",
                sample_index, freq, expected_freq
            );

            let tolerance = expected_freq * 0.01;
            assert!(
                (freq - expected_freq).abs() < tolerance,
                "Sample {}: Expected {}, got {}",
                sample_index,
                expected_freq,
                freq
            );
        }
    }

    #[test]
    fn test_gain_ramp() {
        let context = setup();
        let mut osc = Oscillator::new(OscillatorType::Sine);
        osc.gain().set_value(0.0);
        println!("Initial gain: {}", osc.gain().get_value(0));

        // Ramp from 0.0 to 1.0 over 4410 samples
        osc.gain()
            .linear_ramp_to_value_at_time(1.0, 0.1, 0, context.sample_rate());

        let check_points = [
            (0, 0.0),
            (1102, 0.25),
            (2205, 0.5),
            (3307, 0.75),
            (4410, 1.0),
        ];

        for (sample_index, expected_gain) in check_points {
            let gain = osc.gain().get_value(sample_index);
            println!(
                "Sample {}: gain = {:.3}, expected = {:.3}",
                sample_index, gain, expected_gain
            );

            assert!(
                (gain - expected_gain).abs() < 0.01,
                "Sample {}: Expected {}, got {}",
                sample_index,
                expected_gain,
                gain
            );
        }
    }

    #[test]
    fn test_smooth_transitions() {
        let context = setup();
        let mut osc = Oscillator::new(OscillatorType::Sine);
        osc.frequency().set_value(440.0);
        osc.frequency()
            .linear_ramp_to_value_at_time(880.0, 0.1, 0, context.sample_rate());

        let mut last_value = 440.0;

        // Check first 100 samples for smooth transitions
        for i in 0..100 {
            let value = osc.frequency().get_value(i);
            let change = (value - last_value).abs();

            assert!(
                change < 1.0,
                "Sample {}: Too large change {} -> {} (delta: {})",
                i,
                last_value,
                value,
                change
            );

            if i % 10 == 0 {
                println!("Sample {}: value = {:.3}, change = {:.3}", i, value, change);
            }

            last_value = value;
        }
    }

    #[test]
    fn test_exponential_ramp() {
        let context = setup();
        let mut osc = Oscillator::new(OscillatorType::Sine);
        osc.frequency().set_value(440.0);
        osc.frequency()
            .exponential_ramp_to_value_at_time(880.0, 0.1, 0, context.sample_rate());

        // Test at several points
        let check_points = [
            (0, 440.0),
            (2205, 440.0 * (2.0f32).powf(0.5)), // Geometric midpoint
            (4410, 880.0),
        ];

        for (sample_index, expected_freq) in check_points {
            let freq = osc.frequency().get_value(sample_index);
            println!(
                "Sample {}: freq = {:.1}, expected = {:.1}",
                sample_index, freq, expected_freq
            );

            let tolerance = expected_freq * 0.01;
            assert!(
                (freq - expected_freq).abs() < tolerance,
                "Sample {}: Expected {}, got {}",
                sample_index,
                expected_freq,
                freq
            );
        }
    }

    #[test]
    fn test_oscillator_output() {
        let context = setup();
        let mut osc = Oscillator::new(OscillatorType::Sine);
        osc.frequency().set_value(440.0);
        osc.gain().set_value(1.0);

        // Test several samples
        let mut outputs = Vec::new();
        for i in 0..100 {
            let sample = osc.process(&context, i);
            outputs.push(sample);
        }

        // Basic sanity checks
        assert!(
            outputs.iter().any(|&x| x > 0.0),
            "No positive samples found"
        );
        assert!(
            outputs.iter().any(|&x| x < 0.0),
            "No negative samples found"
        );
        assert!(outputs.iter().all(|&x| x <= 1.0), "Samples exceed maximum");
        assert!(outputs.iter().all(|&x| x >= -1.0), "Samples below minimum");
    }
}
