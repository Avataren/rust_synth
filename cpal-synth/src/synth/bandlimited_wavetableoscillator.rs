use super::audio_context::AUDIO_CONTEXT;
use super::audio_node::AudioNode;
use super::audio_param::AudioParam;
use super::oscillator::OscillatorType;
use lazy_static::lazy_static;
use rustfft::{num_complex::Complex, FftPlanner};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Move the lazy_static definition outside any struct/impl blocks
lazy_static! {
    static ref WAVETABLE_BANKS: Mutex<HashMap<(OscillatorType, u32), Arc<WaveTableBank>>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

pub fn initialize_wave_banks() -> anyhow::Result<()> {
    let sample_rate = AUDIO_CONTEXT
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to acquire audio context lock"))?
        .sample_rate();

    let sample_rate_key = sample_rate as u32;

    let oscillator_types = [
        OscillatorType::Sine,
        OscillatorType::Square,
        OscillatorType::Sawtooth,
        OscillatorType::Triangle,
    ];

    let mut banks = WAVETABLE_BANKS
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to acquire wavetable banks lock"))?;

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

#[derive(Debug)]
struct WaveTable {
    wave_table: Arc<Vec<f32>>,
    top_freq: f32,
}

#[derive(Debug)]
struct WaveTableBank {
    tables: Vec<WaveTable>,
    sample_rate: f32,
}

impl WaveTableBank {
    fn new(waveform: OscillatorType, sample_rate: f32) -> Self {
        const BASE_FREQ: f32 = 20.0;
        const OVERSAMP: usize = 2;

        let max_harmonics = (sample_rate / (3.0 * BASE_FREQ)) as usize;

        let mut v = max_harmonics;
        v = v.saturating_sub(1);
        v |= v >> 1;
        v |= v >> 2;
        v |= v >> 4;
        v |= v >> 8;
        v |= v >> 16;
        v += 1;

        let table_len = v * 2 * OVERSAMP;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(table_len);

        let mut tables = Vec::new();
        let mut harmonics = max_harmonics;
        let mut top_freq = BASE_FREQ * 2.0 / sample_rate;

        while harmonics >= 1 {
            let table = Self::create_wavetable(table_len, harmonics, waveform, top_freq, &fft);
            tables.push(table);
            harmonics >>= 1;
            top_freq *= 2.0;
        }

        // Normalize all tables...
        let global_max = tables
            .iter()
            .flat_map(|table| table.wave_table.iter())
            .fold(0.0f32, |max, &x| max.max(x.abs()));

        if global_max > 0.0 {
            for table in &mut tables {
                let mut normalized = Vec::with_capacity(table.wave_table.len());
                for &sample in table.wave_table.iter() {
                    normalized.push(sample / global_max);
                }
                table.wave_table = Arc::new(normalized);
            }
        }

        Self {
            tables,
            sample_rate,
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

        WaveTable {
            wave_table: Arc::new(spectrum.iter().map(|c| c.im).collect()),
            top_freq,
        }
    }
}

pub struct BandlimitedWavetableOscillator {
    bank: Arc<WaveTableBank>,
    frequency: AudioParam,
    gain: AudioParam,
    phase: f32,
    phase_increment: f32,
    current_table: usize,
}

impl BandlimitedWavetableOscillator {
    pub fn new(waveform: OscillatorType) -> anyhow::Result<Self> {
        let sample_rate = AUDIO_CONTEXT
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to acquire audio context lock"))?
            .sample_rate();

        let sample_rate_key = sample_rate as u32;

        let bank = {
            let mut banks = WAVETABLE_BANKS
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to acquire wavetable banks lock"))?;

            let key = (waveform, sample_rate_key);
            if let Some(bank) = banks.get(&key) {
                println!(
                    "Reusing existing wavetables for {:?} at {}Hz",
                    waveform, sample_rate_key
                );
                bank.clone()
            } else {
                println!(
                    "Creating new wavetables for {:?} at {}Hz",
                    waveform, sample_rate_key
                );
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
        })
    }
    pub fn frequency(&mut self) -> &mut AudioParam {
        &mut self.frequency
    }

    pub fn gain(&mut self) -> &mut AudioParam {
        &mut self.gain
    }
}

impl AudioNode for BandlimitedWavetableOscillator {
    fn process(&mut self, sample_rate: f32) -> f32 {
        let freq = self.frequency.get_value();
        self.phase_increment = freq / sample_rate;

        self.current_table = 0;
        while (self.phase_increment >= self.bank.tables[self.current_table].top_freq)
            && (self.current_table < (self.bank.tables.len() - 1))
        {
            self.current_table += 1;
        }

        let table = &self.bank.tables[self.current_table].wave_table;
        let table_size = table.len() as f32;

        let temp = self.phase * table_size;
        let int_part = temp as usize;
        let frac_part = temp - int_part as f32;

        let sample0 = table[int_part % table.len()];
        let sample1 = table[(int_part + 1) % table.len()];

        let output = sample0 + (sample1 - sample0) * frac_part;

        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output * self.gain.get_value()
    }

    fn set_parameter(&mut self, name: &str, value: f32) {
        match name {
            "frequency" => self.frequency.set_value(value),
            "gain" => self.gain.set_value(value),
            _ => {}
        }
    }

    fn connect_input(&mut self, _name: &str, _node: Arc<Mutex<dyn AudioNode>>) {}
    fn clear_input(&mut self, _input_name: &str) {
        // No-op implementation for BandlimitedWavetableOscillator as it does not store inputs
    }
}
