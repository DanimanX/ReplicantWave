//! High performance wavetable oscillator, sub-oscillator, and noise generator.
//! Unison detune and stereo panning are precomputed to eliminate transcendental math in the hot loop.

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

    // Precalculated unison tables
    cached_unison: usize,
    cached_detune: f32,
    cached_spread: f32,
    cached_pan: f32,
    cached_detune_ratios: [f32; MAX_UNISON],
    cached_unison_gl: [f32; MAX_UNISON],
    cached_unison_gr: [f32; MAX_UNISON],

    cached_semitones: f32,
    cached_pitch_mult: f32,
}

impl Default for WavetableOscillator {
    fn default() -> Self {
        let mut osc = Self {
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
            cached_unison: 0,
            cached_detune: -1.0,
            cached_spread: -1.0,
            cached_pan: -99.0,
            cached_detune_ratios: [1.0; MAX_UNISON],
            cached_unison_gl: [1.0; MAX_UNISON],
            cached_unison_gr: [1.0; MAX_UNISON],
            cached_semitones: -999.0,
            cached_pitch_mult: 1.0,
        };
        osc.update_unison_cache();
        osc
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

    #[inline(always)]
    fn update_unison_cache(&mut self) {
        let unison = self.unison.clamp(1, MAX_UNISON);
        self.cached_unison = unison;
        self.cached_detune = self.detune;
        self.cached_spread = self.spread;
        self.cached_pan = self.pan;

        let gain_norm = 1.0 / (unison as f32).sqrt();
        let inv_unison_m1 = if unison > 1 {
            1.0 / (unison - 1) as f32
        } else {
            1.0
        };
        let detune_scaled = self.detune * (1.5 / 12.0);

        for i in 0..unison {
            let frac = if unison > 1 {
                (i as f32 * inv_unison_m1) - 0.5
            } else {
                0.0
            };
            self.cached_detune_ratios[i] = (frac * detune_scaled).exp2();
            let pan_voice = (self.pan + (frac * 2.0) * self.spread).clamp(-1.0, 1.0);
            let pan_rad = (pan_voice + 1.0) * (PI * 0.25);
            self.cached_unison_gl[i] = pan_rad.cos() * gain_norm;
            self.cached_unison_gr[i] = pan_rad.sin() * gain_norm;
        }
    }

    /// Process one sample block or sample.
    /// Returns stereo output `(left, right)`.
    #[inline(always)]
    pub fn process_sample(
        &mut self,
        wt: &Wavetable,
        base_freq: f32,
        sample_rate: f32,
    ) -> (f32, f32) {
        if !self.enabled || self.level_gain <= 0.0001 {
            return (0.0, 0.0);
        }

        let unison = self.unison.clamp(1, MAX_UNISON);

        // Update unison cache only when parameters change
        if unison != self.cached_unison
            || (self.detune - self.cached_detune).abs() > 0.001
            || (self.spread - self.cached_spread).abs() > 0.001
            || (self.pan - self.cached_pan).abs() > 0.001
        {
            self.update_unison_cache();
        }

        // Cache pitch multiplier
        let semitones = self.tune_semitones + (self.fine_cents * 0.01);
        if (semitones - self.cached_semitones).abs() > 0.001 {
            self.cached_semitones = semitones;
            self.cached_pitch_mult = (semitones * (1.0 / 12.0)).exp2();
        }

        let center_freq = base_freq * self.cached_pitch_mult;
        let base_phase_inc = (center_freq / sample_rate).min(0.499);

        let has_bend = self.bend.abs() > 0.001;
        let has_formant = self.formant.abs() > 0.001;
        let formant_factor = if has_formant {
            (self.formant * 1.5).exp2()
        } else {
            1.0
        };

        // Ultra-fast path for 1-voice unison (avoids loop overhead)
        if unison == 1 {
            let current_phase = self.phases[0];
            let mut p = current_phase;

            if has_bend {
                let b = self.bend;
                p = if b > 0.0 {
                    p * (1.0 - b * (1.0 - p))
                } else {
                    p * (1.0 + (-b) * (1.0 - p))
                };
            }

            if has_formant {
                p = (p * formant_factor).fract();
            }

            let s = wt.sample(self.position, p);
            let next_phase = current_phase + base_phase_inc;
            self.phases[0] = next_phase - next_phase.floor();

            let gl = self.cached_unison_gl[0] * self.level_gain;
            let gr = self.cached_unison_gr[0] * self.level_gain;

            return (s * gl, s * gr);
        }

        let mut out_l = 0.0;
        let mut out_r = 0.0;

        for i in 0..unison {
            let phase_inc = (base_phase_inc * self.cached_detune_ratios[i]).min(0.499);
            let current_phase = self.phases[i];
            let mut p = current_phase;

            if has_bend {
                let b = self.bend;
                p = if b > 0.0 {
                    p * (1.0 - b * (1.0 - p))
                } else {
                    p * (1.0 + (-b) * (1.0 - p))
                };
            }

            if has_formant {
                p = (p * formant_factor).fract();
            }

            let s = wt.sample(self.position, p);
            let next_phase = current_phase + phase_inc;
            self.phases[i] = next_phase - next_phase.floor();

            out_l += s * self.cached_unison_gl[i];
            out_r += s * self.cached_unison_gr[i];
        }

        (out_l * self.level_gain, out_r * self.level_gain)
    }
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

        let mult = match self.octave_offset {
            -2 => 0.25,
            -1 => 0.5,
            _ => 1.0,
        };
        let freq = base_freq * mult;
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
            SubWave::Pulse50 => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            SubWave::Pulse25 => {
                if self.phase < 0.25 {
                    1.0
                } else {
                    -1.0
                }
            }
            SubWave::Pulse12 => {
                if self.phase < 0.125 {
                    1.0
                } else {
                    -1.0
                }
            }
        };

        self.phase = (self.phase + phase_inc) - (self.phase + phase_inc).floor();

        let pan_rad = (self.pan.clamp(-1.0, 1.0) + 1.0) * (PI * 0.25);
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
                (self.b0
                    + self.b1
                    + self.b2
                    + self.b3
                    + self.b4
                    + self.b5
                    + self.b6
                    + white * 0.5362)
                    * 0.11
            }
            NoiseType::CyberBit => {
                // Quantized harsh digital noise
                (white * 4.0).round() * 0.25
            }
        };

        let pan_rad = (self.pan.clamp(-1.0, 1.0) + 1.0) * (PI * 0.25);
        let gl = pan_rad.cos() * self.level_gain;
        let gr = pan_rad.sin() * self.level_gain;

        (s * gl, s * gr)
    }
}
