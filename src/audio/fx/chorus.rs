//! Stereo Multi-Voice Chorus.

use std::f32::consts::PI;

const MAX_CHORUS_DELAY_SAMPLES: usize = 8192;

#[derive(Clone, Debug)]
pub struct Chorus {
    pub enabled: bool,
    pub rate_hz: f32,  // 0.1 to 10.0 Hz
    pub depth_ms: f32, // 0.1 to 20.0 ms
    pub delay_ms: f32, // 0.1 to 30.0 ms
    pub width: f32,    // 0.0 to 1.0 (phase spread)
    pub mix: f32,      // 0.0 to 1.0

    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_idx: usize,
    lfo_phase: f32,
    smoothed_mix: f32,
    sample_rate: f32,
}

impl Default for Chorus {
    fn default() -> Self {
        Self {
            enabled: false,
            rate_hz: 1.5,
            depth_ms: 3.0,
            delay_ms: 12.0,
            width: 0.8,
            mix: 0.5,
            buffer_l: vec![0.0; MAX_CHORUS_DELAY_SAMPLES],
            buffer_r: vec![0.0; MAX_CHORUS_DELAY_SAMPLES],
            write_idx: 0,
            lfo_phase: 0.0,
            smoothed_mix: 0.0,
            sample_rate: 44100.0,
        }
    }
}

impl Chorus {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1000.0);
    }

    pub fn reset(&mut self) {
        self.buffer_l.fill(0.0);
        self.buffer_r.fill(0.0);
        self.write_idx = 0;
        self.lfo_phase = 0.0;
        self.smoothed_mix = 0.0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let target_mix = if self.enabled {
            self.mix.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let coeff = 1.0 - (-1.0 / (0.004 * self.sample_rate)).exp();
        self.smoothed_mix += (target_mix - self.smoothed_mix) * coeff;
        if self.smoothed_mix <= 0.0001 && target_mix <= 0.0 {
            self.smoothed_mix = 0.0;
            return (in_l, in_r);
        }

        let sr = self.sample_rate;
        let buf_len = self.buffer_l.len();

        // Write input
        self.buffer_l[self.write_idx] = in_l;
        self.buffer_r[self.write_idx] = in_r;

        // Advance LFO
        let phase_inc = self.rate_hz.clamp(0.05, 10.0) / sr;
        self.lfo_phase = (self.lfo_phase + phase_inc) - (self.lfo_phase + phase_inc).floor();

        let lfo_l = (self.lfo_phase * 2.0 * PI).sin();
        let lfo_r = ((self.lfo_phase + self.width * 0.25) * 2.0 * PI).sin();

        // Keep the full modulated read range inside the delay line at every
        // supported sample rate. The old 2048-sample line could wrap at 48 kHz
        // with the documented delay/depth settings, which produced crackle.
        let max_delay = (buf_len - 4) as f32;
        let base_delay_samples = (self.delay_ms * 0.001 * sr).clamp(1.0, max_delay * 0.5);
        let mod_depth_samples = (self.depth_ms * 0.001 * sr)
            .min(base_delay_samples * 0.8)
            .min((max_delay - base_delay_samples - 2.0).max(0.0));

        let read_time_l = base_delay_samples + lfo_l * mod_depth_samples;
        let read_time_r = base_delay_samples + lfo_r * mod_depth_samples;

        let wet_l = read_interpolated(&self.buffer_l, self.write_idx, read_time_l);
        let wet_r = read_interpolated(&self.buffer_r, self.write_idx, read_time_r);

        self.write_idx = (self.write_idx + 1) % buf_len;

        let wet_gain = self.smoothed_mix;
        let dry_gain = 1.0 - wet_gain * 0.5;

        (
            in_l * dry_gain + wet_l * wet_gain,
            in_r * dry_gain + wet_r * wet_gain,
        )
    }
}

#[inline(always)]
fn read_interpolated(buffer: &[f32], write_pos: usize, delay_samples: f32) -> f32 {
    let len = buffer.len();
    let read_pos = (write_pos as f32 + len as f32 - delay_samples) % len as f32;
    let idx0 = read_pos.floor() as usize % len;
    let idx1 = (idx0 + 1) % len;
    let frac = read_pos - read_pos.floor();
    buffer[idx0] + frac * (buffer[idx1] - buffer[idx0])
}
