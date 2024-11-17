use crate::synth::audio_context::AudioContext;
use crate::synth::audio_node::AudioNode;
use crate::synth::audio_param::AudioParam;
use crate::synth::oscillator::OscillatorType;
use lazy_static::lazy_static;
use rustfft::{num_complex::Complex, FftPlanner};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Constants for wavetable generation
const OVERSAMPLE: usize = 2;
const BASE_FREQ: f32 = 20.0;
const MIN_TABLE_SIZE: usize = 64;

#[derive(Debug)]
struct WaveTable {
    wave_table: Arc<Vec<f32>>,
    top_freq: f32,
    table_mask: usize, // For power-of-2 size tables
    table_size: usize,
}

#[derive(Debug)]
struct WaveTableBank {
    tables: Vec<WaveTable>,
    sample_rate: f32,
    frequency_bounds: Vec<f32>, // Pre-computed frequency boundaries
}

lazy_static! {
    static ref WAVETABLE_BANKS: Mutex<HashMap<(OscillatorType, u32), Arc<WaveTableBank>>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

impl WaveTableBank {
    fn new(waveform: OscillatorType, sample_rate: f32) -> Self {
        let max_harmonics = (sample_rate / (3.0 * BASE_FREQ)) as usize;

        // Find next power of 2
        let mut table_len = MIN_TABLE_SIZE;
        while table_len < max_harmonics * 2 * OVERSAMPLE {
            table_len *= 2;
        }

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(table_len);

        let mut tables = Vec::new();
        let mut frequency_bounds = Vec::new();
        let mut harmonics = max_harmonics;
        let mut top_freq = BASE_FREQ * 2.0 / sample_rate;

        while harmonics >= 1 {
            let table = Self::create_wavetable(table_len, harmonics, waveform, top_freq, &fft);
            frequency_bounds.push(top_freq * sample_rate);
            tables.push(table);
            harmonics >>= 1;
            top_freq *= 2.0;
        }

        // Normalize all tables
        let global_max = tables
            .iter()
            .flat_map(|table| table.wave_table.iter())
            .fold(0.0f32, |max, &x| max.max(x.abs()));

        if global_max > 0.0 {
            for table in &mut tables {
                let normalized: Vec<f32> = table
                    .wave_table
                    .iter()
                    .map(|&sample| sample / global_max)
                    .collect();
                table.wave_table = Arc::new(normalized);
            }
        }

        Self {
            tables,
            sample_rate,
            frequency_bounds,
        }
    }

    fn create_wavetable(
        len: usize,
        num_harmonics: usize,
        waveform: OscillatorType,
        top_freq: f32,
        fft: &Arc<dyn rustfft::Fft<f32>>,
    ) -> WaveTable {
        let mut spectrum = vec![Complex::new(0.0f32, 0.0f32); len];

        match waveform {
            OscillatorType::Sawtooth => {
                for idx in 1..=num_harmonics {
                    let temp = -1.0 / idx as f32;
                    spectrum[idx] = Complex::new(-temp, 0.0);
                    spectrum[len - idx] = Complex::new(temp, 0.0);
                }
            }
            OscillatorType::Square => {
                for idx in (1..=num_harmonics).step_by(2) {
                    let temp = 1.0 / idx as f32;
                    spectrum[idx] = Complex::new(temp, 0.0);
                    spectrum[len - idx] = Complex::new(-temp, 0.0);
                }
            }
            OscillatorType::Triangle => {
                let mut sign = 1.0f32;
                for idx in (1..=num_harmonics).step_by(2) {
                    let temp = sign / (idx * idx) as f32;
                    spectrum[idx] = Complex::new(temp, 0.0);
                    spectrum[len - idx] = Complex::new(-temp, 0.0);
                    sign = -sign;
                }
            }
            OscillatorType::Sine => {
                spectrum[1] = Complex::new(1.0, 0.0);
                spectrum[len - 1] = Complex::new(-1.0, 0.0);
            }
        }

        fft.process(&mut spectrum);

        // Create table with padding for interpolation
        let mut wave_table: Vec<f32> = spectrum.iter().map(|c| c.im).collect();
        wave_table.push(wave_table[0]); // Add padding for interpolation

        WaveTable {
            wave_table: Arc::new(wave_table),
            top_freq,
            table_mask: len - 1,
            table_size: len,
        }
    }

    #[inline]
    fn find_table_index(&self, freq: f32) -> usize {
        match self
            .frequency_bounds
            .binary_search_by(|&bound| bound.partial_cmp(&freq).unwrap())
        {
            Ok(index) => index,
            Err(index) => index.min(self.tables.len() - 1),
        }
    }
}

pub fn initialize_wave_banks(context: &AudioContext) -> anyhow::Result<()> {
    let sample_rate = context.sample_rate();
    let sample_rate_key = sample_rate as u32;

    let oscillator_types = [
        OscillatorType::Sine,
        OscillatorType::Square,
        OscillatorType::Sawtooth,
        OscillatorType::Triangle,
    ];

    // Single lock acquisition for the entire initialization
    let mut banks = WAVETABLE_BANKS
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to acquire wavetable banks lock"))?;

    // Pre-allocate with capacity
    if banks.is_empty() {
        banks.reserve(oscillator_types.len());
    }

    for &osc_type in &oscillator_types {
        let key = (osc_type, sample_rate_key);
        if !banks.contains_key(&key) {
            println!(
                "Initializing wave bank for {:?} at {}Hz",
                osc_type, sample_rate_key
            );
            let bank = Arc::new(WaveTableBank::new(osc_type, sample_rate));
            banks.insert(key, bank);
        } else {
            println!(
                "Wave bank for {:?} at {}Hz already initialized",
                osc_type, sample_rate_key
            );
        }
    }

    Ok(())
}

// New helper function to check if banks are initialized
pub fn are_wave_banks_initialized(sample_rate: f32) -> bool {
    if let Ok(banks) = WAVETABLE_BANKS.lock() {
        let sample_rate_key = sample_rate as u32;
        [
            OscillatorType::Sine,
            OscillatorType::Square,
            OscillatorType::Sawtooth,
            OscillatorType::Triangle,
        ]
        .iter()
        .all(|&osc_type| banks.contains_key(&(osc_type, sample_rate_key)))
    } else {
        false
    }
}

pub struct BandlimitedWavetableOscillator {
    bank: Arc<WaveTableBank>,
    frequency: AudioParam,
    gain: AudioParam,
    phase: f32,
    phase_increment: f32,
    current_table: usize,
    last_freq: f32,
    interpolation_mode: InterpolationType,
}

#[derive(Clone, Copy, Debug)]
pub enum InterpolationType {
    Linear,
    Cubic,
    #[cfg(target_arch = "x86_64")]
    Simd,
}

impl BandlimitedWavetableOscillator {
    pub fn new(waveform: OscillatorType, context: &AudioContext) -> anyhow::Result<Self> {
        let sample_rate = context.sample_rate();
        let sample_rate_key = sample_rate as u32;

        let bank = {
            let mut banks = WAVETABLE_BANKS
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire wavetable banks lock"))?;

            let key = (waveform, sample_rate_key);
            if let Some(bank) = banks.get(&key) {
                bank.clone()
            } else {
                let bank = Arc::new(WaveTableBank::new(waveform, sample_rate));
                banks.insert(key, bank.clone());
                bank
            }
        };

        Ok(Self {
            bank,
            frequency: AudioParam::new(440.0, 0.01, 22050.0),
            gain: AudioParam::new(1.0, 0.0, 1.0),
            phase: 0.0,
            phase_increment: 0.0,
            current_table: 0,
            last_freq: 0.0,
            interpolation_mode: InterpolationType::Linear,
        })
    }

    pub fn frequency(&self) -> &AudioParam {
        &self.frequency
    }

    pub fn gain(&self) -> &AudioParam {
        &self.gain
    }

    pub fn set_interpolation_mode(&mut self, mode: InterpolationType) {
        self.interpolation_mode = mode;
    }

    #[inline(always)]
    fn linear_interpolate(&self, table: &[f32], idx: usize, frac: f32) -> f32 {
        let sample0 = table[idx];
        let sample1 = table[idx + 1];
        sample0 + (sample1 - sample0) * frac
    }

    #[inline(always)]
    fn cubic_interpolate(&self, table: &[f32], idx: usize, frac: f32) -> f32 {
        let y0 = table[idx.wrapping_sub(1) & self.bank.tables[self.current_table].table_mask];
        let y1 = table[idx];
        let y2 = table[idx + 1];
        let y3 = table[idx + 2];

        let mu2 = frac * frac;
        let a0 = y3 - y2 - y0 + y1;
        let a1 = y0 - y1 - a0;
        let a2 = y2 - y0;
        let a3 = y1;

        a0 * frac * mu2 + a1 * mu2 + a2 * frac + a3
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    unsafe fn simd_interpolate(&self, table: &[f32], idx: usize, frac: f32) -> f32 {
        let samples = _mm_set_ps(0.0, 0.0, table[idx + 1], table[idx]);
        let factors = _mm_set_ps(0.0, 0.0, frac, 1.0 - frac);
        let result = _mm_dp_ps(samples, factors, 0x31);
        _mm_cvtss_f32(result)
    }
}

impl AudioNode for BandlimitedWavetableOscillator {
    fn process(&mut self, context: &AudioContext, current_sample: u64) -> f32 {
        let freq = self.frequency.get_value(current_sample);

        // Update phase increment and table selection only if frequency changed
        if freq != self.last_freq {
            self.phase_increment = freq / context.sample_rate();
            self.last_freq = freq;
            self.current_table = self.bank.find_table_index(freq);
        }

        let table = &self.bank.tables[self.current_table].wave_table;
        let table_size = self.bank.tables[self.current_table].table_size as f32;
        let table_mask = self.bank.tables[self.current_table].table_mask;

        // Calculate table indices
        let temp = self.phase * table_size;
        let int_part = temp as usize;
        let frac_part = temp - int_part as f32;
        let idx = int_part & table_mask;

        // Interpolate based on selected mode
        let output = match self.interpolation_mode {
            InterpolationType::Linear => self.linear_interpolate(table, idx, frac_part),
            InterpolationType::Cubic => self.cubic_interpolate(table, idx, frac_part),
            #[cfg(target_arch = "x86_64")]
            InterpolationType::Simd => unsafe { self.simd_interpolate(table, idx, frac_part) },
        };

        // Update phase
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output * self.gain.get_value(current_sample)
    }

    fn set_parameter(&self, name: &str, value: f32) {
        match name {
            "frequency" => self.frequency.set_value(value),
            "gain" => self.gain.set_value(value),
            _ => {}
        }
    }

    fn connect_input(&mut self, _name: &str, _node: Box<dyn AudioNode + Send>) {
        // Oscillators don't have inputs
    }

    fn clear_input(&mut self, _input_name: &str) {
        // No-op for oscillators
    }

    fn clone_box(&self) -> Box<dyn AudioNode + Send> {
        Box::new(self.clone())
    }
}

impl Clone for BandlimitedWavetableOscillator {
    fn clone(&self) -> Self {
        Self {
            bank: self.bank.clone(),
            frequency: self.frequency.clone(),
            gain: self.gain.clone(),
            phase: self.phase,
            phase_increment: self.phase_increment,
            current_table: self.current_table,
            last_freq: self.last_freq,
            interpolation_mode: self.interpolation_mode,
        }
    }
}
