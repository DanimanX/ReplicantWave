//! Cytomic State-Variable Filter (SVF) with zero-delay feedback (ZDF).
//! Modes: LP 12dB, LP 24dB (two-stage), HP 12dB, HP 24dB, BP 12dB, BP 24dB, Notch.

use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterType {
    LowPass12,
    LowPass24,
    HighPass12,
    HighPass24,
    BandPass12,
    BandPass24,
    Notch,
}

impl FilterType {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::LowPass12,
            1 => Self::LowPass24,
            2 => Self::HighPass12,
            3 => Self::HighPass24,
            4 => Self::BandPass12,
            5 => Self::BandPass24,
            _ => Self::Notch,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StereoFilter {
    pub enabled: bool,
    pub filter_type: FilterType,
    pub cutoff_hz: f32,
    pub resonance: f32, // 0.0 to 1.0 (maps to Q 0.5 to 25.0)
    pub drive: f32,     // 1.0 to 4.0
    pub key_tracking: f32, // 0.0 to 1.0
    pub vel_tracking: f32, // 0.0 to 1.0
    pub env_amount: f32,   // -1.0 to +1.0
    pub route_osc1: bool,
    pub route_osc2: bool,
    pub route_sub: bool,
    pub route_noise: bool,

    // Two SVF stages per channel for up to 24dB/oct slope
    s1_l: [f32; 2],
    s2_l: [f32; 2],
    s1_r: [f32; 2],
    s2_r: [f32; 2],
}

impl Default for StereoFilter {
    fn default() -> Self {
        Self {
            enabled: true,
            filter_type: FilterType::LowPass24,
            cutoff_hz: 2500.0,
            resonance: 0.2,
            drive: 1.0,
            key_tracking: 0.0,
            vel_tracking: 0.0,
            env_amount: 0.0,
            route_osc1: true,
            route_osc2: true,
            route_sub: true,
            route_noise: true,
            s1_l: [0.0; 2],
            s2_l: [0.0; 2],
            s1_r: [0.0; 2],
            s2_r: [0.0; 2],
        }
    }
}

impl StereoFilter {
    pub fn reset(&mut self) {
        self.s1_l = [0.0; 2];
        self.s2_l = [0.0; 2];
        self.s1_r = [0.0; 2];
        self.s2_r = [0.0; 2];
    }

    /// Process one stereo sample.
    #[inline(always)]
    pub fn process_sample(
        &mut self,
        in_l: f32,
        in_r: f32,
        mod_cutoff_hz: f32,
        sample_rate: f32,
    ) -> (f32, f32) {
        if !self.enabled {
            return (in_l, in_r);
        }

        // Clamp cutoff to safe audible range below Nyquist
        let target_cutoff = mod_cutoff_hz.clamp(20.0, sample_rate * 0.48);

        // Prewarp frequency for bilinear transform
        let wd = 2.0 * PI * target_cutoff;
        let t = 1.0 / sample_rate;
        let wa = (2.0 / t) * (wd * t / 2.0).tan();
        let g = wa * t / 2.0;

        // Q from resonance (0.0 to 1.0 -> 0.707 to 20.0)
        let q = 0.707 + self.resonance.clamp(0.0, 1.0) * 19.3;
        let k = 1.0 / q;

        // Drive saturation factor
        let drive = self.drive.clamp(1.0, 5.0);

        let out_l = self.process_channel(in_l * drive, g, k, 0) / drive.sqrt();
        let out_r = self.process_channel(in_r * drive, g, k, 1) / drive.sqrt();

        (out_l, out_r)
    }

    #[inline(always)]
    fn process_channel(&mut self, input: f32, g: f32, k: f32, ch: usize) -> f32 {
        let (s1, s2) = if ch == 0 {
            (&mut self.s1_l, &mut self.s2_l)
        } else {
            (&mut self.s1_r, &mut self.s2_r)
        };

        // First SVF stage
        let out1 = svf_stage(input, g, k, &mut s1[0], &mut s2[0], self.filter_type, false);

        match self.filter_type {
            FilterType::LowPass12 | FilterType::HighPass12 | FilterType::BandPass12 | FilterType::Notch => {
                out1
            }
            FilterType::LowPass24 | FilterType::HighPass24 | FilterType::BandPass24 => {
                // Second stage cascaded for 24 dB/oct slope
                svf_stage(out1, g, k, &mut s1[1], &mut s2[1], self.filter_type, true)
            }
        }
    }
}

#[inline(always)]
fn svf_stage(
    input: f32,
    g: f32,
    k: f32,
    s1: &mut f32,
    s2: &mut f32,
    filter_type: FilterType,
    is_second_stage: bool,
) -> f32 {
    let a1 = 1.0 / (1.0 + g * (g + k));
    let a2 = g * a1;
    let a3 = g * a2;

    let v0 = input;
    let v1 = a1 * *s1 + a2 * (v0 - *s2);
    let v2 = *s2 + a3 * (v0 - *s2) + a2 * *s1;

    let v3 = v0 - k * v1 - v2;

    *s1 = 2.0 * v1 - *s1;
    *s2 = 2.0 * v2 - *s2;

    match filter_type {
        FilterType::LowPass12 | FilterType::LowPass24 => v2,
        FilterType::HighPass12 | FilterType::HighPass24 => v3,
        FilterType::BandPass12 | FilterType::BandPass24 => v1,
        FilterType::Notch => {
            if is_second_stage {
                v2 // Fallback
            } else {
                v0 - k * v1
            }
        }
    }
}
