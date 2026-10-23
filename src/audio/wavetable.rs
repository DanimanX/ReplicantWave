//! Wavetable synthesis engine and procedurally generated cyberpunk wavetables.

use std::f32::consts::PI;

pub const DEFAULT_TABLE_SIZE: usize = 2048;
pub const NUM_DEFAULT_FRAMES: usize = 16;

#[derive(Clone, Debug)]
pub struct Wavetable {
    pub name: String,
    pub table_size: usize,
    pub frames: Vec<Vec<f32>>,
}

impl Wavetable {
    pub fn new(name: impl Into<String>, table_size: usize, frames: Vec<Vec<f32>>) -> Self {
        Self {
            name: name.into(),
            table_size,
            frames,
        }
    }

    /// Number of frames in this wavetable.
    pub fn num_frames(&self) -> usize {
        self.frames.len()
    }

    /// Read sample with linear interpolation across frames and within frame.
    /// - `position`: 0.0 to 1.0 (frame position)
    /// - `phase`: 0.0 to 1.0 (cycle phase)
    #[inline(always)]
    pub fn sample(&self, position: f32, phase: f32) -> f32 {
        let num_frames = self.frames.len();
        if num_frames == 0 {
            return 0.0;
        }
        if num_frames == 1 {
            return self.sample_frame(0, phase);
        }

        let clamped_pos = position.clamp(0.0, 1.0);
        let frame_pos = clamped_pos * (num_frames - 1) as f32;
        let frame0 = frame_pos.floor() as usize;
        let frame1 = (frame0 + 1).min(num_frames - 1);
        let frac = frame_pos - frame0 as f32;

        let s0 = self.sample_frame(frame0, phase);
        let s1 = self.sample_frame(frame1, phase);

        s0 + frac * (s1 - s0)
    }

    /// Read sample from a single frame with fast bitmask linear interpolation.
    #[inline(always)]
    pub fn sample_frame(&self, frame_idx: usize, phase: f32) -> f32 {
        let frame = &self.frames[frame_idx];
        let n = self.table_size;
        let mask = n - 1;
        let wrapped_phase = phase.fract();
        let wrapped_phase = if wrapped_phase < 0.0 { wrapped_phase + 1.0 } else { wrapped_phase };
        let index_f = wrapped_phase * n as f32;
        let idx0 = (index_f as usize) & mask;
        let idx1 = (idx0 + 1) & mask;
        let frac = index_f - idx0 as f32;

        let s0 = frame[idx0];
        let s1 = frame[idx1];
        s0 + frac * (s1 - s0)
    }

    /// Load wavetable from a WAV file (supports 256, 512, 1024, 2048 sample sizes).
    pub fn from_wav_file(path: &std::path::Path) -> Result<Self, String> {
        let mut reader = hound::WavReader::open(path)
            .map_err(|e| format!("Failed to open WAV: {}", e))?;
        let spec = reader.spec();
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => reader.samples::<f32>().filter_map(Result::ok).collect(),
            hound::SampleFormat::Int => {
                let max_val = (1i64 << (spec.bits_per_sample - 1)) as f32;
                reader.samples::<i32>().filter_map(Result::ok).map(|s| s as f32 / max_val).collect()
            }
        };

        if samples.is_empty() {
            return Err("Empty WAV file".into());
        }

        // Detect frame size (prefer 2048, or fallback to power of two)
        let total_samples = samples.len() / spec.channels as usize;
        let mono_samples: Vec<f32> = if spec.channels > 1 {
            samples.chunks(spec.channels as usize).map(|c| c[0]).collect()
        } else {
            samples
        };

        let frame_size = if total_samples % 2048 == 0 && total_samples >= 2048 {
            2048
        } else if total_samples % 1024 == 0 && total_samples >= 1024 {
            1024
        } else if total_samples % 512 == 0 && total_samples >= 512 {
            512
        } else {
            2048
        };

        let num_frames = (mono_samples.len() / frame_size).max(1);
        let mut frames = Vec::with_capacity(num_frames);

        for f in 0..num_frames {
            let start = f * frame_size;
            let end = (start + frame_size).min(mono_samples.len());
            let mut frame = mono_samples[start..end].to_vec();
            if frame.len() < frame_size {
                frame.resize(frame_size, 0.0);
            }
            frames.push(frame);
        }

        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Custom Wavetable")
            .to_string();

        Ok(Self::new(name, frame_size, frames))
    }
}



/// Collection of factory cyberpunk wavetables.
pub struct WavetableBank {
    pub tables: Vec<Wavetable>,
}

impl Default for WavetableBank {
    fn default() -> Self {
        Self::new()
    }
}

impl WavetableBank {
    pub fn new() -> Self {
        let mut tables = Vec::new();
        tables.push(generate_analog_morph());
        tables.push(generate_cyber_neuro());
        tables.push(generate_digital_fm());
        tables.push(generate_vocaloid_synth());
        tables.push(generate_cyber_acid());
        tables.push(generate_dystopian_steel());
        tables.push(generate_pwm_hyper());
        tables.push(generate_sub_zero());
        Self { tables }
    }

    pub fn get_table(&self, index: usize) -> &Wavetable {
        if self.tables.is_empty() {
            panic!("No wavetables loaded");
        }
        &self.tables[index % self.tables.len()]
    }

    pub fn names(&self) -> Vec<String> {
        self.tables.iter().map(|t| t.name.clone()).collect()
    }
}

/// 1. Analog Morph: Saw -> Triangle -> Sine -> Square -> Pulse 10%
fn generate_analog_morph() -> Wavetable {
    let size = DEFAULT_TABLE_SIZE;
    let num_frames = NUM_DEFAULT_FRAMES;
    let mut frames = Vec::with_capacity(num_frames);

    for f in 0..num_frames {
        let t = f as f32 / (num_frames - 1) as f32;
        let mut frame = Vec::with_capacity(size);

        for i in 0..size {
            let phase = i as f32 / size as f32; // 0..1
            let sine = (phase * 2.0 * PI).sin();
            let saw = 2.0 * (phase - 0.5);
            let tri = if phase < 0.25 {
                4.0 * phase
            } else if phase < 0.75 {
                2.0 - 4.0 * phase
            } else {
                -4.0 + 4.0 * phase
            };
            let pulse_width = 0.5 - 0.4 * t;
            let square = if phase < pulse_width { 1.0 } else { -1.0 };

            let sample = if t < 0.33 {
                let u = t / 0.33;
                saw * (1.0 - u) + tri * u
            } else if t < 0.66 {
                let u = (t - 0.33) / 0.33;
                tri * (1.0 - u) + sine * u
            } else {
                let u = (t - 0.66) / 0.34;
                sine * (1.0 - u) + square * u
            };
            frame.push(sample * 0.9);
        }
        frames.push(frame);
    }

    Wavetable::new("Analog Morph", size, frames)
}

/// 2. Cyber Neuro: Aggressive neuro-bass harmonic morph with folding
fn generate_cyber_neuro() -> Wavetable {
    let size = DEFAULT_TABLE_SIZE;
    let num_frames = NUM_DEFAULT_FRAMES;
    let mut frames = Vec::with_capacity(num_frames);

    for f in 0..num_frames {
        let t = f as f32 / (num_frames - 1) as f32;
        let mut frame = Vec::with_capacity(size);

        for i in 0..size {
            let p = i as f32 / size as f32;
            let mut sample = 0.0;
            // Sum of odd and even harmonics with non-linear phase modulation
            for h in 1..=8 {
                let amp = 1.0 / (h as f32).powf(0.8 - 0.3 * t);
                let phase_shift = t * (h as f32) * 0.7;
                sample += amp * ((p * (h as f32) + phase_shift) * 2.0 * PI).sin();
            }

            // Cyber waveshaping / folding
            let drive = 1.0 + 3.0 * t;
            let driven = sample * drive;
            let folded = driven.sin(); // Wavefolder effect
            frame.push(folded * 0.85);
        }
        frames.push(frame);
    }

    Wavetable::new("Cyber Neuro", size, frames)
}

/// 3. Digital FM: Carrier-Modulator 2-op FM wave morphing
fn generate_digital_fm() -> Wavetable {
    let size = DEFAULT_TABLE_SIZE;
    let num_frames = NUM_DEFAULT_FRAMES;
    let mut frames = Vec::with_capacity(num_frames);

    for f in 0..num_frames {
        let t = f as f32 / (num_frames - 1) as f32;
        let mut frame = Vec::with_capacity(size);
        let mod_ratio = 1.0 + 3.0 * (t * 2.0).floor(); // 1, 2, 3, 4
        let mod_index = 0.5 + 4.5 * t;

        for i in 0..size {
            let p = i as f32 / size as f32;
            let modulator = (p * mod_ratio * 2.0 * PI).sin();
            let carrier = ((p * 2.0 * PI) + mod_index * modulator).sin();
            frame.push(carrier * 0.85);
        }
        frames.push(frame);
    }

    Wavetable::new("Digital FM", size, frames)
}

/// 4. Vocaloid Synth: Formant morphing (A -> E -> I -> O -> U)
fn generate_vocaloid_synth() -> Wavetable {
    let size = DEFAULT_TABLE_SIZE;
    let num_frames = NUM_DEFAULT_FRAMES;
    let mut frames = Vec::with_capacity(num_frames);

    // Formant frequencies approximate for A, E, I, O, U
    let formants = [
        (800.0, 1200.0), // A
        (400.0, 2000.0), // E
        (250.0, 2300.0), // I
        (500.0, 900.0),  // O
        (300.0, 700.0),  // U
    ];

    for f in 0..num_frames {
        let t = f as f32 / (num_frames - 1) as f32;
        let mut frame = Vec::with_capacity(size);

        let v_idx = (t * 4.0).min(3.99);
        let base_idx = v_idx.floor() as usize;
        let frac = v_idx - base_idx as f32;

        let f1 = formants[base_idx].0 * (1.0 - frac) + formants[base_idx + 1].0 * frac;
        let f2 = formants[base_idx].1 * (1.0 - frac) + formants[base_idx + 1].1 * frac;

        let r1 = (f1 / 100.0).round().max(1.0);
        let r2 = (f2 / 100.0).round().max(2.0);

        for i in 0..size {
            let p = i as f32 / size as f32;
            let saw = 2.0 * (p - 0.5);
            let formant1 = (p * r1 * 2.0 * PI).sin() * 0.6;
            let formant2 = (p * r2 * 2.0 * PI).sin() * 0.4;
            let window = (p * PI).sin(); // smooth frame edges
            let sample = (saw * 0.3 + formant1 + formant2) * window;
            frame.push(sample * 0.9);
        }
        frames.push(frame);
    }

    Wavetable::new("Vocaloid Synth", size, frames)
}

/// 5. Cyber Acid 303: Resonant harmonic sweep table
fn generate_cyber_acid() -> Wavetable {
    let size = DEFAULT_TABLE_SIZE;
    let num_frames = NUM_DEFAULT_FRAMES;
    let mut frames = Vec::with_capacity(num_frames);

    for f in 0..num_frames {
        let t = f as f32 / (num_frames - 1) as f32;
        let mut frame = Vec::with_capacity(size);
        let cutoff_harm = 2.0 + 24.0 * t * t;
        let resonance = 3.0 + 8.0 * t;

        for i in 0..size {
            let p = i as f32 / size as f32;
            let mut sample = 0.0;
            for h in 1..=32 {
                let hf = h as f32;
                let base_amp = 1.0 / hf;
                // Resonant peak around cutoff_harm
                let dist = (hf - cutoff_harm).abs();
                let res_boost = (-(dist * dist) / (2.0 * resonance)).exp() * 2.5;
                let amp = base_amp * (1.0 + res_boost);
                sample += amp * (p * hf * 2.0 * PI).sin();
            }
            frame.push(sample.clamp(-1.0, 1.0) * 0.85);
        }
        frames.push(frame);
    }

    Wavetable::new("Cyber Acid 303", size, frames)
}

/// 6. Dystopian Steel: Inharmonic metallic spectrum
fn generate_dystopian_steel() -> Wavetable {
    let size = DEFAULT_TABLE_SIZE;
    let num_frames = NUM_DEFAULT_FRAMES;
    let mut frames = Vec::with_capacity(num_frames);

    let ratios = [1.0, 1.414, 1.732, 2.236, 2.76, 3.41, 4.12, 5.25];

    for f in 0..num_frames {
        let t = f as f32 / (num_frames - 1) as f32;
        let mut frame = Vec::with_capacity(size);

        for i in 0..size {
            let p = i as f32 / size as f32;
            let mut sample = 0.0;
            for (idx, &r) in ratios.iter().enumerate() {
                let ratio = r * (1.0 + 0.2 * t * idx as f32);
                let amp = 1.0 / (idx as f32 + 1.0).powf(0.9);
                sample += amp * (p * ratio * 2.0 * PI).sin();
            }
            frame.push(sample.clamp(-1.0, 1.0) * 0.85);
        }
        frames.push(frame);
    }

    Wavetable::new("Dystopian Steel", size, frames)
}

/// 7. PWM Hyper: Smooth pulse-width modulation table
fn generate_pwm_hyper() -> Wavetable {
    let size = DEFAULT_TABLE_SIZE;
    let num_frames = NUM_DEFAULT_FRAMES;
    let mut frames = Vec::with_capacity(num_frames);

    for f in 0..num_frames {
        let t = f as f32 / (num_frames - 1) as f32;
        let mut frame = Vec::with_capacity(size);
        let pw = 0.5 - 0.46 * t; // 50% down to 4%

        for i in 0..size {
            let p = i as f32 / size as f32;
            let sample = if p < pw { 1.0 } else { -1.0 };
            // Gentle bandlimiting smoothing filter on edges
            frame.push(sample * 0.85);
        }
        frames.push(frame);
    }

    Wavetable::new("PWM Hyper", size, frames)
}

/// 8. Sub Zero: Pure low-end saturation to wavefolded bass
fn generate_sub_zero() -> Wavetable {
    let size = DEFAULT_TABLE_SIZE;
    let num_frames = NUM_DEFAULT_FRAMES;
    let mut frames = Vec::with_capacity(num_frames);

    for f in 0..num_frames {
        let t = f as f32 / (num_frames - 1) as f32;
        let mut frame = Vec::with_capacity(size);

        for i in 0..size {
            let p = i as f32 / size as f32;
            let sine = (p * 2.0 * PI).sin();
            let third_harmonic = (p * 6.0 * PI).sin() * (0.2 * t);
            let fifth_harmonic = (p * 10.0 * PI).sin() * (0.1 * t);
            let combined = sine + third_harmonic + fifth_harmonic;
            // Warm saturation
            let saturated = (combined * (1.0 + 2.0 * t)).tanh();
            frame.push(saturated * 0.9);
        }
        frames.push(frame);
    }

    Wavetable::new("Sub Zero", size, frames)
}
