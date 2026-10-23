//! Preset definition and serialization.

pub mod factory;

use serde::{Deserialize, Serialize};
use crate::audio::filter::FilterType;
use crate::audio::fx::distortion::DistortionMode;
use crate::audio::oscillator::{NoiseType, SubWave};
use crate::audio::step_sequencer::{SeqMode, SeqRate, Step, MAX_STEPS};
use crate::audio::SynthEngine;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SynthPreset {
    pub name: String,
    pub category: String,
    pub author: String,
    pub description: String,

    // Osc 1
    pub osc1_table: usize,
    pub osc1_enabled: bool,
    pub osc1_unison: usize,
    pub osc1_tune: f32,
    pub osc1_fine: f32,
    pub osc1_level: f32,
    pub osc1_pos: f32,
    pub osc1_detune: f32,
    pub osc1_spread: f32,
    pub osc1_pan: f32,
    pub osc1_formant: f32,
    pub osc1_bend: f32,

    // Osc 2
    pub osc2_table: usize,
    pub osc2_enabled: bool,
    pub osc2_unison: usize,
    pub osc2_tune: f32,
    pub osc2_fine: f32,
    pub osc2_level: f32,
    pub osc2_pos: f32,
    pub osc2_detune: f32,
    pub osc2_spread: f32,
    pub osc2_pan: f32,
    pub osc2_formant: f32,
    pub osc2_bend: f32,

    // Sub & Noise
    pub sub_enabled: bool,
    pub sub_wave: usize,
    pub sub_octave: i8,
    pub sub_level: f32,
    pub sub_pan: f32,

    pub noise_enabled: bool,
    pub noise_type: usize,
    pub noise_level: f32,
    pub noise_pan: f32,

    // Filter
    pub filter_enabled: bool,
    pub filter_type: usize,
    pub filter_cutoff: f32,
    pub filter_reso: f32,
    pub filter_drive: f32,
    pub filter_key: f32,
    pub filter_vel: f32,
    pub filter_env_amount: f32,

    // Amp ADSR
    pub amp_attack: f32,
    pub amp_decay: f32,
    pub amp_sustain: f32,
    pub amp_release: f32,

    // Filter ADSR
    pub filter_attack: f32,
    pub filter_decay: f32,
    pub filter_sustain: f32,
    pub filter_release: f32,

    // FX
    pub dist_enabled: bool,
    pub dist_mode: usize,
    pub dist_amount: f32,
    pub dist_mix: f32,

    pub chorus_enabled: bool,
    pub chorus_rate: f32,
    pub chorus_depth: f32,
    pub chorus_mix: f32,

    pub delay_enabled: bool,
    pub delay_sync: bool,
    pub delay_time: f32,
    pub delay_fb: f32,
    pub delay_damp: f32,
    pub delay_mix: f32,

    pub reverb_enabled: bool,
    pub reverb_size: f32,
    pub reverb_damp: f32,
    pub reverb_mix: f32,

    pub gate_enabled: bool,

    // Sequencer
    pub seq_running: bool,
    pub seq_mode: usize,
    pub seq_length: usize,
    pub seq_rate: usize,
    pub seq_swing: f32,
    pub seq_steps: Vec<Step>,
}

impl SynthPreset {
    pub fn apply_to_engine(&self, engine: &mut SynthEngine) {
        for v in engine.voices.iter_mut() {
            // Osc 1
            v.osc1_table_idx = self.osc1_table;
            v.osc1.enabled = self.osc1_enabled;
            v.osc1.unison = self.osc1_unison;
            v.osc1.tune_semitones = self.osc1_tune;
            v.osc1.fine_cents = self.osc1_fine;
            v.osc1.level_gain = self.osc1_level;
            v.osc1.position = self.osc1_pos;
            v.osc1.detune = self.osc1_detune;
            v.osc1.spread = self.osc1_spread;
            v.osc1.pan = self.osc1_pan;
            v.osc1.formant = self.osc1_formant;
            v.osc1.bend = self.osc1_bend;

            // Osc 2
            v.osc2_table_idx = self.osc2_table;
            v.osc2.enabled = self.osc2_enabled;
            v.osc2.unison = self.osc2_unison;
            v.osc2.tune_semitones = self.osc2_tune;
            v.osc2.fine_cents = self.osc2_fine;
            v.osc2.level_gain = self.osc2_level;
            v.osc2.position = self.osc2_pos;
            v.osc2.detune = self.osc2_detune;
            v.osc2.spread = self.osc2_spread;
            v.osc2.pan = self.osc2_pan;
            v.osc2.formant = self.osc2_formant;
            v.osc2.bend = self.osc2_bend;

            // Sub & Noise
            v.sub.enabled = self.sub_enabled;
            v.sub.wave = SubWave::from_index(self.sub_wave);
            v.sub.octave_offset = self.sub_octave;
            v.sub.level_gain = self.sub_level;
            v.sub.pan = self.sub_pan;

            v.noise.enabled = self.noise_enabled;
            v.noise.noise_type = NoiseType::from_index(self.noise_type);
            v.noise.level_gain = self.noise_level;
            v.noise.pan = self.noise_pan;

            // Filter
            v.filter.enabled = self.filter_enabled;
            v.filter.filter_type = FilterType::from_index(self.filter_type);
            v.filter.cutoff_hz = self.filter_cutoff;
            v.filter.resonance = self.filter_reso;
            v.filter.drive = self.filter_drive;
            v.filter.key_tracking = self.filter_key;
            v.filter.vel_tracking = self.filter_vel;
            v.filter.env_amount = self.filter_env_amount;

            // Envelopes
            v.amp_env.attack_s = self.amp_attack;
            v.amp_env.decay_s = self.amp_decay;
            v.amp_env.sustain_level = self.amp_sustain;
            v.amp_env.release_s = self.amp_release;

            v.filter_env.attack_s = self.filter_attack;
            v.filter_env.decay_s = self.filter_decay;
            v.filter_env.sustain_level = self.filter_sustain;
            v.filter_env.release_s = self.filter_release;
        }

        // FX
        engine.fx.distortion.enabled = self.dist_enabled;
        engine.fx.distortion.mode = DistortionMode::from_index(self.dist_mode);
        engine.fx.distortion.amount = self.dist_amount;
        engine.fx.distortion.mix = self.dist_mix;

        engine.fx.chorus.enabled = self.chorus_enabled;
        engine.fx.chorus.rate_hz = self.chorus_rate;
        engine.fx.chorus.depth_ms = self.chorus_depth;
        engine.fx.chorus.mix = self.chorus_mix;

        engine.fx.delay.enabled = self.delay_enabled;
        engine.fx.delay.sync = self.delay_sync;
        engine.fx.delay.time_ms = self.delay_time;
        engine.fx.delay.feedback = self.delay_fb;
        engine.fx.delay.damping = self.delay_damp;
        engine.fx.delay.mix = self.delay_mix;

        engine.fx.reverb.enabled = self.reverb_enabled;
        engine.fx.reverb.room_size = self.reverb_size;
        engine.fx.reverb.damping = self.reverb_damp;
        engine.fx.reverb.mix = self.reverb_mix;

        engine.fx.gate.enabled = self.gate_enabled;

        // Sequencer
        engine.sequencer.running = self.seq_running;
        engine.sequencer.mode = SeqMode::from_index(self.seq_mode);
        engine.sequencer.length = self.seq_length.clamp(1, MAX_STEPS);
        engine.sequencer.rate = SeqRate::from_index(self.seq_rate);
        engine.sequencer.swing = self.seq_swing;

        for (i, step) in self.seq_steps.iter().take(MAX_STEPS).enumerate() {
            engine.sequencer.steps[i] = *step;
        }
    }
}
