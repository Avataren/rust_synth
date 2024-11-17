use cpal_synth::{
    initialize_wave_banks, AudioContext, AudioNode, BandlimitedWavetableOscillator, Oscillator,
    OscillatorType,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;

fn setup() -> Arc<AudioContext> {
    let context = Arc::new(AudioContext::new(44100.0));
    initialize_wave_banks(&context).unwrap();
    context
}

pub fn oscillator_benchmark(c: &mut Criterion) {
    let context = setup();
    let mut group = c.benchmark_group("Oscillators");

    // Test frequencies
    let frequencies = [20.0, 440.0, 10000.0];

    // Benchmark basic oscillator
    for &osc_type in &[
        OscillatorType::Sine,
        OscillatorType::Square,
        OscillatorType::Sawtooth,
        OscillatorType::Triangle,
    ] {
        for &freq in &frequencies {
            group.bench_function(format!("basic_{:?}_{:.0}Hz", osc_type, freq), |b| {
                let mut osc = Oscillator::new(osc_type);
                osc.frequency().set_value(freq);
                let mut sample: u64 = 0;
                b.iter(|| {
                    let out = black_box(osc.process(&context, black_box(sample)));
                    sample = sample.wrapping_add(1);
                    out
                });
            });
        }
    }

    // Benchmark wavetable oscillator
    for &osc_type in &[
        OscillatorType::Sine,
        OscillatorType::Square,
        OscillatorType::Sawtooth,
        OscillatorType::Triangle,
    ] {
        for &freq in &frequencies {
            group.bench_function(format!("wavetable_{:?}_{:.0}Hz", osc_type, freq), |b| {
                let mut osc = BandlimitedWavetableOscillator::new(osc_type, &context).unwrap();
                osc.frequency().set_value(freq);
                let mut sample: u64 = 0;
                b.iter(|| {
                    let out = black_box(osc.process(&context, black_box(sample)));
                    sample = sample.wrapping_add(1);
                    out
                });
            });
        }
    }

    group.finish();
}

criterion_group!(benches, oscillator_benchmark);
criterion_main!(benches);
