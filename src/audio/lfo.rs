//! Multi-shape LFO with tempo sync, fade-in, and retrigger options.

use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LfoShape {
    Sine,
    Triangle,
    SawUp,
    SawDown,
    Square,
    SampleAndHold,
    Noise,
}

impl LfoShape {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Sine,
            1 => Self::Triangle,
            2 => Self::SawUp,
            3 => Self::SawDown,
            4 => Self::Square,
            5 => Self::SampleAndHold,
            _ => Self::Noise,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Lfo {
    pub enabled: bool,
    pub shape: LfoShape,
    pub rate_hz: f32,       // 0.01 to 50.0 Hz
    pub sync: bool,
    pub sync_beat_division: f32, // In beats, e.g. 0.25 = 1/16th, 1.0 = 1/4th, 4.0 = 1 bar
    pub depth: f32,         // -1.0 to 1.0
    pub retrigger: bool,
    pub fade_s: f32,

    phase: f32,
    current_value: f32,
    hold_value: f32,
    fade_timer: f32,
    sample_rate: f32,
    seed: u32,
}

impl Default for Lfo {
    fn default() -> Self {
        Self {
            enabled: true,
            shape: LfoShape::Sine,
            rate_hz: 2.0,
            sync: false,
            sync_beat_division: 1.0,
            depth: 1.0,
            retrigger: true,
            fade_s: 0.0,
            phase: 0.0,
            current_value: 0.0,
            hold_value: 0.0,
            fade_timer: 0.0,
            sample_rate: 44100.0,
            seed: 0x87654321,
        }
    }
}

impl Lfo {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1000.0);
    }

    pub fn reset_phase(&mut self) {
        if self.retrigger {
            self.phase = 0.0;
            self.fade_timer = 0.0;
            self.hold_value = self.next_random();
        }
    }

    #[inline(always)]
    fn next_random(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Process one sample. Returns modulation output between -1.0 and +1.0.
    #[inline(always)]
    pub fn process_sample(&mut self, host_bpm: f32) -> f32 {
        if !self.enabled {
            return 0.0;
        }

        let freq = if self.sync && host_bpm > 20.0 {
            let beats_per_sec = host_bpm / 60.0;
            (beats_per_sec / self.sync_beat_division.max(0.0625)).clamp(0.01, 100.0)
        } else {
            self.rate_hz.clamp(0.01, 50.0)
        };

        let phase_inc = freq / self.sample_rate;
        let old_phase = self.phase;
        self.phase = (self.phase + phase_inc) - (self.phase + phase_inc).floor();

        // Sample and hold triggers when phase wraps around
        if self.phase < old_phase {
            self.hold_value = self.next_random();
        }

        let raw = match self.shape {
            LfoShape::Sine => (self.phase * 2.0 * PI).sin(),
            LfoShape::Triangle => {
                if self.phase < 0.25 {
                    4.0 * self.phase
                } else if self.phase < 0.75 {
                    2.0 - 4.0 * self.phase
                } else {
                    -4.0 + 4.0 * self.phase
                }
            }
            LfoShape::SawUp => 2.0 * (self.phase - 0.5),
            LfoShape::SawDown => 2.0 * (0.5 - self.phase),
            LfoShape::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },
            LfoShape::SampleAndHold => self.hold_value,
            LfoShape::Noise => self.next_random(),
        };

        // Fade in
        let fade_mult = if self.fade_s > 0.001 {
            self.fade_timer += 1.0 / self.sample_rate;
            (self.fade_timer / self.fade_s).min(1.0)
        } else {
            1.0
        };

        self.current_value = raw * self.depth * fade_mult;
        self.current_value
    }

    #[inline(always)]
    pub fn current_value(&self) -> f32 {
        self.current_value
    }
}
