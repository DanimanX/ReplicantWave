//! 64-step pattern sequencer with sliding bar navigation, swing, accent, slide, and gate.

use serde::{Deserialize, Serialize};

pub const MAX_STEPS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub note_offset: i8,   // Semitone offset from base note (-24..+24)
    pub velocity: u8,      // 1..127
    pub gate: bool,        // Trigger on/off
    pub slide: bool,       // Legato glide to next step
    pub accent: bool,      // Velocity & resonance boost
    pub tie: bool,         // Tie note without retrigger
}

impl Default for Step {
    fn default() -> Self {
        Self {
            note_offset: 0,
            velocity: 100,
            gate: true,
            slide: false,
            accent: false,
            tie: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum SeqRate {
    Rate1_4,
    Rate1_8,
    Rate1_8T,
    Rate1_16,
    Rate1_16T,
    Rate1_16D,
    Rate1_32,
}

impl SeqRate {
    pub fn to_beat_fraction(&self) -> f32 {
        match self {
            Self::Rate1_4 => 1.0,
            Self::Rate1_8 => 0.5,
            Self::Rate1_8T => 0.5 * (2.0 / 3.0),
            Self::Rate1_16 => 0.25,
            Self::Rate1_16T => 0.25 * (2.0 / 3.0),
            Self::Rate1_16D => 0.25 * 1.5,
            Self::Rate1_32 => 0.125,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Rate1_4,
            1 => Self::Rate1_8,
            2 => Self::Rate1_8T,
            3 => Self::Rate1_16,
            4 => Self::Rate1_16T,
            5 => Self::Rate1_16D,
            _ => Self::Rate1_32,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Rate1_4 => "1/4",
            Self::Rate1_8 => "1/8",
            Self::Rate1_8T => "1/8 T",
            Self::Rate1_16 => "1/16",
            Self::Rate1_16T => "1/16 T",
            Self::Rate1_16D => "1/16 D",
            Self::Rate1_32 => "1/32",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum SeqMode {
    InternalSynth,  // Triggers internal synth voices directly
    ModCutoff,      // Modulates filter cutoff
    ModWTPos,       // Modulates wavetable position
    Off,
}

impl SeqMode {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::InternalSynth,
            1 => Self::ModCutoff,
            2 => Self::ModWTPos,
            _ => Self::Off,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StepSequencer {
    pub steps: [Step; MAX_STEPS],
    pub length: usize,       // 1 to 64
    pub rate: SeqRate,
    pub swing: f32,          // 0.0 to 0.75
    pub running: bool,
    pub mode: SeqMode,
    pub octave: i8,          // -2 to +2
    pub root_note: u8,       // Base MIDI note (e.g. 36 = C2)

    pub current_step: usize,
    sample_counter: f32,
    pub step_just_triggered: bool,
    pub mod_output: f32,     // Current smoothed modulation output (0.0 to 1.0)
    sample_rate: f32,
}

impl Default for StepSequencer {
    fn default() -> Self {
        let mut steps = [Step::default(); MAX_STEPS];
        // Create an energetic cyberpunk default sequence
        for (i, step) in steps.iter_mut().enumerate() {
            step.gate = i % 2 == 0 || i % 7 == 0 || i == 15 || i == 31;
            step.accent = i % 4 == 0;
            step.slide = i == 6 || i == 14 || i == 22 || i == 30;
            step.note_offset = match i % 8 {
                0 => 0,
                1 => 0,
                2 => 3,
                3 => 7,
                4 => 0,
                5 => 10,
                6 => 12,
                _ => 7,
            };
            step.velocity = if step.accent { 120 } else { 95 };
        }

        Self {
            steps,
            length: 16,
            rate: SeqRate::Rate1_16,
            swing: 0.0,
            running: false,
            mode: SeqMode::InternalSynth,
            octave: 0,
            root_note: 36, // C2
            current_step: 0,
            sample_counter: 0.0,
            step_just_triggered: false,
            mod_output: 0.0,
            sample_rate: 44100.0,
        }
    }
}

impl StepSequencer {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr.max(1000.0);
    }

    pub fn reset(&mut self) {
        self.current_step = 0;
        self.sample_counter = 0.0;
        self.step_just_triggered = false;
        self.mod_output = 0.0;
    }

    /// Advance one sample. If a step fires, returns `Some((midi_note, velocity, slide, accent))`
    #[inline(always)]
    pub fn process_sample(&mut self, bpm: f32) -> Option<(u8, f32, bool, bool)> {
        if !self.running || self.mode == SeqMode::Off {
            self.step_just_triggered = false;
            return None;
        }

        let effective_bpm = bpm.max(30.0);
        let beats_per_sec = effective_bpm / 60.0;
        let base_step_time = self.rate.to_beat_fraction() / beats_per_sec;
        let base_step_samples = base_step_time * self.sample_rate;

        // Apply swing on odd steps (every 2nd step)
        let swing_factor = if self.current_step % 2 == 1 {
            1.0 + self.swing.clamp(0.0, 0.75)
        } else {
            1.0 - self.swing.clamp(0.0, 0.75) * 0.5
        };

        let step_samples = base_step_samples * swing_factor;

        self.sample_counter += 1.0;
        if self.sample_counter >= step_samples {
            self.sample_counter = 0.0;
            self.current_step = (self.current_step + 1) % self.length.max(1).min(MAX_STEPS);
            self.step_just_triggered = true;

            let step = &self.steps[self.current_step];
            if step.gate {
                let note_val = (self.root_note as i32 + (self.octave as i32 * 12) + step.note_offset as i32)
                    .clamp(0, 127) as u8;
                let vel = (step.velocity as f32 / 127.0).clamp(0.05, 1.0);

                // Update mod output
                self.mod_output = (step.note_offset as f32 + 24.0) / 48.0;

                return Some((note_val, vel, step.slide, step.accent));
            }
        } else {
            self.step_just_triggered = false;
        }

        None
    }

    /// Clear all steps.
    pub fn clear(&mut self) {
        for s in self.steps.iter_mut() {
            *s = Step {
                note_offset: 0,
                velocity: 100,
                gate: false,
                slide: false,
                accent: false,
                tie: false,
            };
        }
    }

    /// Randomize sequence with musical cyberpunk intervals (pentatonic / minor / acid).
    pub fn randomize(&mut self, prng_seed: u32) {
        let mut s = prng_seed;
        let scale = [0, 3, 5, 7, 10, 12, 15, 17, 19, 22, 24]; // Minor pentatonic + octaves

        for (i, step) in self.steps.iter_mut().enumerate() {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            let r = s % 100;

            step.gate = r < 75;
            step.accent = (s >> 8) % 100 < 30;
            step.slide = (s >> 16) % 100 < 25;

            let note_idx = ((s >> 4) as usize) % scale.len();
            step.note_offset = scale[note_idx] as i8 - 12;
            step.velocity = if step.accent { 120 } else { 85 + ((s >> 2) % 30) as u8 };
            step.tie = i > 0 && !step.gate && (s >> 12) % 100 < 20;
        }
    }
}
