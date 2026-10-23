//! High performance wavetable oscillator, sub-oscillator, and noise generator.

use super::wavetable::Wavetable;
use std::f32::consts::PI;

pub const MAX_UNISON: usize = 8;

#[derive(Clone, Debug)]
pub struct WavetableOscillator {
    pub phases: [f32; MAX_UNISON],
    pub enabled: bool,
    pub retrigger: bool,
    pub unison: usize,
    pub tune_semitones: f32,
    pub fine_cents: f32,
    pub level_gain: f32,
    pub position: f32,
    pub detune: f32,
    pub spread: f32,
    pub pan: f32,
    pub formant: f32,
    pub bend: f32,
}

impl Default for WavetableOscillator {
    fn default() -> Self {
        Self {
            phases: [0.0; MAX_UNISON],
            enabled: true,
            retrigger: true,
            unison: 1,
            tune_semitones: 0.0,
            fine_cents: 0.0,
            level_gain: 1.0,
            position: 0.0,
            detune: 0.0,
            spread: 0.0,
            pan: 0.0,
            formant: 0.0,
            bend: 0.0,
        }
    }
}

impl WavetableOscillator {
    pub fn reset_phase(&mut self, prng_seed: Option<f32>) {
        if self.retrigger {
            for (i, p) in self.phases.iter_mut().enumerate() {
                if let Some(seed) = prng_seed {
                    // Spread phases deterministically to avoid sudden phase bursts
                    *p = (seed + i as f32 * 0.173) % 1.0;
                } else {
                    *p = 0.0;
                }
            }
        }
    }

    /// Process one sample block or sample.
    /// Returns stereo output `(left, right)`.
    #[inline(always)]
    pub fn process_sample(&mut self, wt: &Wavetable, base_freq: f32, sample_rate: f32) -> (f32, f32) {
        if !self.enabled || self.level_gain <= 0.0001 {
            return (0.0, 0.0);
        }

        let unison = self.unison.clamp(1, MAX_UNISON);
        let semitones = self.tune_semitones + (self.fine_cents / 100.0);
        let center_freq = base_freq * 2.0f32.powf(semitones / 12.0);

        let mut out_l = 0.0;
        let mut out_r = 0.0;

        let gain_norm = 1.0 / (unison as f32).sqrt();

        for i in 0..unison {
            // Unison detune factor (-0.5 to +0.5 spread across voices)
            let detune_ratio = if unison > 1 {
                let frac = (i as f32 / (unison - 1) as f32) - 0.5;
                2.0f32.powf((frac * self.detune * 1.5) / 12.0)
            } else {
                1.0
            };

            let voice_freq = center_freq * detune_ratio;
            let phase_inc = (voice_freq / sample_rate).min(0.499);

            // Phase with Bend & Formant
            let current_phase = self.phases[i];
            let modified_phase = apply_phase_modifiers(current_phase, self.formant, self.bend);

            let s = wt.sample(self.position, modified_phase);

            // Advance phase
            self.phases[i] = (current_phase + phase_inc) - (current_phase + phase_inc).floor();

            // Stereo pan per unison voice
            let pan_voice = if unison > 1 {
                let frac = (i as f32 / (unison - 1) as f32) * 2.0 - 1.0;
                (self.pan + frac * self.spread).clamp(-1.0, 1.0)
            } else {
                self.pan.clamp(-1.0, 1.0)
            };

            // Equal power panning
            let pan_rad = (pan_voice + 1.0) * (PI / 4.0);
            let gl = pan_rad.cos();
            let gr = pan_rad.sin();

            out_l += s * gl * gain_norm;
            out_r += s * gr * gain_norm;
        }

        (out_l * self.level_gain, out_r * self.level_gain)
    }
}

/// Apply Formant (stretch/compress) and Bend (phase warp)
#[inline(always)]
fn apply_phase_modifiers(phase: f32, formant: f32, bend: f32) -> f32 {
    let mut p = phase;

    // Bend warp: bends phase curve nonlinearly
    if bend.abs() > 0.001 {
        if bend > 0.0 {
            // Squeeze towards start
            let k = 1.0 + bend * 3.0;
            p = p.powf(k);
        } else {
            // Squeeze towards end
            let k = 1.0 + (-bend) * 3.0;
            p = 1.0 - (1.0 - p).powf(k);
        }
    }

    // Formant warp: scales phase frequency window
    if formant.abs() > 0.001 {
        let factor = 2.0f32.powf(formant * 1.5);
        p = (p * factor).fract();
    }

    p
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SubWave {
    Sine,
    Triangle,
    Saw,
    Pulse50,
    Pulse25,
    Pulse12,
}

impl SubWave {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Sine,
            1 => Self::Triangle,
            2 => Self::Saw,
            3 => Self::Pulse50,
            4 => Self::Pulse25,
            _ => Self::Pulse12,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubOscillator {
    pub phase: f32,
    pub enabled: bool,
    pub wave: SubWave,
    pub octave_offset: i8, // -2, -1, 0
    pub level_gain: f32,
    pub pan: f32,
}

impl Default for SubOscillator {
    fn default() -> Self {
        Self {
            phase: 0.0,
            enabled: false,
            wave: SubWave::Sine,
            octave_offset: -1,
            level_gain: 0.5,
            pan: 0.0,
        }
    }
}

impl SubOscillator {
    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, base_freq: f32, sample_rate: f32) -> (f32, f32) {
        if !self.enabled || self.level_gain <= 0.0001 {
            return (0.0, 0.0);
        }

        let freq = base_freq * 2.0f32.powi(self.octave_offset as i32);
        let phase_inc = (freq / sample_rate).min(0.499);

        let s = match self.wave {
            SubWave::Sine => (self.phase * 2.0 * PI).sin(),
            SubWave::Triangle => {
                if self.phase < 0.25 {
                    4.0 * self.phase
                } else if self.phase < 0.75 {
                    2.0 - 4.0 * self.phase
                } else {
                    -4.0 + 4.0 * self.phase
                }
            }
            SubWave::Saw => 2.0 * (self.phase - 0.5),
            SubWave::Pulse50 => if self.phase < 0.5 { 1.0 } else { -1.0 },
            SubWave::Pulse25 => if self.phase < 0.25 { 1.0 } else { -1.0 },
            SubWave::Pulse12 => if self.phase < 0.125 { 1.0 } else { -1.0 },
        };

        self.phase = (self.phase + phase_inc) - (self.phase + phase_inc).floor();

        let pan_rad = (self.pan.clamp(-1.0, 1.0) + 1.0) * (PI / 4.0);
        let gl = pan_rad.cos() * self.level_gain;
        let gr = pan_rad.sin() * self.level_gain;

        (s * gl, s * gr)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoiseType {
    White,
    Pink,
    CyberBit,
}

impl NoiseType {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::White,
            1 => Self::Pink,
            _ => Self::CyberBit,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NoiseGenerator {
    pub enabled: bool,
    pub noise_type: NoiseType,
    pub level_gain: f32,
    pub pan: f32,
    // Pink noise filter state (Paul Kellet filter)
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
    seed: u32,
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self {
            enabled: false,
            noise_type: NoiseType::White,
            level_gain: 0.2,
            pan: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
            seed: 0x12345678,
        }
    }
}

impl NoiseGenerator {
    #[inline(always)]
    fn next_random(&mut self) -> f32 {
        // Simple fast xorshift32 PRNG for low CPU overhead
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    #[inline(always)]
    pub fn process_sample(&mut self) -> (f32, f32) {
        if !self.enabled || self.level_gain <= 0.0001 {
            return (0.0, 0.0);
        }

        let white = self.next_random();

        let s = match self.noise_type {
            NoiseType::White => white,
            NoiseType::Pink => {
                // Paul Kellet filter
                self.b0 = 0.99886 * self.b0 + white * 0.0555179;
                self.b1 = 0.99332 * self.b1 + white * 0.0750759;
                self.b2 = 0.96900 * self.b2 + white * 0.1538520;
                self.b3 = 0.86650 * self.b3 + white * 0.3104856;
                self.b4 = 0.55000 * self.b4 + white * 0.5329522;
                self.b5 = -0.7616 * self.b5 - white * 0.0168980;
                (self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6 + white * 0.5362) * 0.11
            }
            NoiseType::CyberBit => {
                // Quantized harsh digital noise
                (white * 4.0).round() * 0.25
            }
        };

        let pan_rad = (self.pan.clamp(-1.0, 1.0) + 1.0) * (PI / 4.0);
        let gl = pan_rad.cos() * self.level_gain;
        let gr = pan_rad.sin() * self.level_gain;

        (s * gl, s * gr)
    }
}
