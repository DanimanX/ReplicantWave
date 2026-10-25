//! Preset definition and serialization.

pub mod factory;

use crate::audio::filter::FilterType;
use crate::audio::fx::distortion::DistortionMode;
use crate::audio::oscillator::{NoiseType, SubWave};
use crate::audio::step_sequencer::{SeqMode, SeqRate, Step, MAX_STEPS};
use crate::audio::SynthEngine;
use crate::params::{PatternState, ReplicantWaveParams};
use nih_plug::prelude::{BoolParam, FloatParam, IntParam, ParamSetter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

        // Sequencer. Loading a patch never starts the transport: the pattern is
        // loaded and armed, and PLAY (or an explicit DAW sync) starts it.
        engine.sequencer.running = false;
        engine.sequencer.mode = SeqMode::from_index(self.seq_mode);
        engine.sequencer.length = self.seq_length.clamp(1, MAX_STEPS);
        engine.sequencer.rate = SeqRate::from_index(self.seq_rate);
        engine.sequencer.swing = self.seq_swing;

        for (i, step) in self.seq_steps.iter().take(MAX_STEPS).enumerate() {
            engine.sequencer.steps[i] = *step;
        }
    }
}

// ---------------------------------------------------------------------------
// Host-parameter application
// ---------------------------------------------------------------------------

fn set_f(setter: &ParamSetter, p: &FloatParam, v: f32) {
    setter.begin_set_parameter(p);
    setter.set_parameter(p, v);
    setter.end_set_parameter(p);
}

fn set_i(setter: &ParamSetter, p: &IntParam, v: i32) {
    setter.begin_set_parameter(p);
    setter.set_parameter(p, v);
    setter.end_set_parameter(p);
}

fn set_b(setter: &ParamSetter, p: &BoolParam, v: bool) {
    setter.begin_set_parameter(p);
    setter.set_parameter(p, v);
    setter.end_set_parameter(p);
}

impl SynthPreset {
    /// Steps of this patch, padded/truncated to the full 64-step grid.
    pub fn steps(&self) -> Vec<Step> {
        let mut steps = vec![
            Step {
                gate: false,
                ..Step::default()
            };
            MAX_STEPS
        ];
        for (dst, src) in steps.iter_mut().zip(self.seq_steps.iter()) {
            *dst = *src;
        }
        steps
    }

    /// Write every patch value into the host-visible parameters.
    ///
    /// This is the path the editor uses, which means loading a preset is
    /// visible to the host, gets recorded as automation in write mode, and is
    /// saved with the session. The transport is never started.
    pub fn apply_to_params(&self, setter: &ParamSetter, p: &ReplicantWaveParams) {
        // OSC 1
        set_b(setter, &p.osc1.enabled, self.osc1_enabled);
        set_i(setter, &p.osc1.table, self.osc1_table as i32);
        set_f(setter, &p.osc1.position, self.osc1_pos);
        set_f(setter, &p.osc1.tune, self.osc1_tune);
        set_f(setter, &p.osc1.fine, self.osc1_fine);
        set_i(setter, &p.osc1.unison, self.osc1_unison as i32);
        set_f(setter, &p.osc1.detune, self.osc1_detune);
        set_f(setter, &p.osc1.spread, self.osc1_spread);
        set_f(setter, &p.osc1.formant, self.osc1_formant);
        set_f(setter, &p.osc1.bend, self.osc1_bend);
        set_f(setter, &p.osc1.level, self.osc1_level);
        set_f(setter, &p.osc1.pan, self.osc1_pan);

        // OSC 2
        set_b(setter, &p.osc2.enabled, self.osc2_enabled);
        set_i(setter, &p.osc2.table, self.osc2_table as i32);
        set_f(setter, &p.osc2.position, self.osc2_pos);
        set_f(setter, &p.osc2.tune, self.osc2_tune);
        set_f(setter, &p.osc2.fine, self.osc2_fine);
        set_i(setter, &p.osc2.unison, self.osc2_unison as i32);
        set_f(setter, &p.osc2.detune, self.osc2_detune);
        set_f(setter, &p.osc2.spread, self.osc2_spread);
        set_f(setter, &p.osc2.formant, self.osc2_formant);
        set_f(setter, &p.osc2.bend, self.osc2_bend);
        set_f(setter, &p.osc2.level, self.osc2_level);
        set_f(setter, &p.osc2.pan, self.osc2_pan);

        // SUB / NOISE
        set_b(setter, &p.sub.enabled, self.sub_enabled);
        set_i(setter, &p.sub.wave, self.sub_wave as i32);
        set_i(setter, &p.sub.octave, self.sub_octave as i32);
        set_f(setter, &p.sub.level, self.sub_level);
        set_f(setter, &p.sub.pan, self.sub_pan);

        set_b(setter, &p.noise.enabled, self.noise_enabled);
        set_i(setter, &p.noise.noise_type, self.noise_type as i32);
        set_f(setter, &p.noise.level, self.noise_level);
        set_f(setter, &p.noise.pan, self.noise_pan);

        // FILTER
        set_b(setter, &p.filter.enabled, self.filter_enabled);
        set_i(setter, &p.filter.filter_type, self.filter_type as i32);
        set_f(setter, &p.cutoff, self.filter_cutoff);
        set_f(setter, &p.resonance, self.filter_reso);
        set_f(setter, &p.filter.drive, self.filter_drive);
        set_f(setter, &p.filter.key_tracking, self.filter_key);
        set_f(setter, &p.filter.vel_tracking, self.filter_vel);
        set_f(setter, &p.filter.env_amount, self.filter_env_amount);
        // A patch always routes every source through the filter; the routing
        // matrix is a performance control the user can restrict afterwards.
        set_b(setter, &p.filter.route_osc1, true);
        set_b(setter, &p.filter.route_osc2, true);
        set_b(setter, &p.filter.route_sub, true);
        set_b(setter, &p.filter.route_noise, true);

        // ENVELOPES
        set_f(setter, &p.amp_env.attack, self.amp_attack);
        set_f(setter, &p.amp_env.decay, self.amp_decay);
        set_f(setter, &p.amp_env.sustain, self.amp_sustain);
        set_f(setter, &p.amp_env.release, self.amp_release);

        set_f(setter, &p.filter_env.attack, self.filter_attack);
        set_f(setter, &p.filter_env.decay, self.filter_decay);
        set_f(setter, &p.filter_env.sustain, self.filter_sustain);
        set_f(setter, &p.filter_env.release, self.filter_release);

        // FX
        set_b(setter, &p.dist.enabled, self.dist_enabled);
        set_i(setter, &p.dist.mode, self.dist_mode as i32);
        set_f(setter, &p.dist.amount, self.dist_amount);
        set_f(setter, &p.dist.mix, self.dist_mix);

        set_b(setter, &p.chorus.enabled, self.chorus_enabled);
        set_f(setter, &p.chorus.rate, self.chorus_rate);
        set_f(setter, &p.chorus.depth, self.chorus_depth);
        set_f(setter, &p.chorus.mix, self.chorus_mix);

        set_b(setter, &p.delay.enabled, self.delay_enabled);
        set_b(setter, &p.delay.sync, self.delay_sync);
        set_b(setter, &p.delay.pingpong, true);
        set_f(setter, &p.delay.time, self.delay_time);
        set_f(setter, &p.delay.feedback, self.delay_fb);
        set_f(setter, &p.delay.damping, self.delay_damp);
        set_f(setter, &p.delay.mix, self.delay_mix);

        set_b(setter, &p.reverb.enabled, self.reverb_enabled);
        set_f(setter, &p.reverb.room_size, self.reverb_size);
        set_f(setter, &p.reverb.damping, self.reverb_damp);
        set_f(setter, &p.reverb.mix, self.reverb_mix);

        set_b(setter, &p.gate.enabled, self.gate_enabled);

        // SEQUENCER: patch content only, transport stays stopped.
        set_i(setter, &p.seq.mode, self.seq_mode as i32);
        set_i(
            setter,
            &p.seq.length,
            self.seq_length.clamp(1, MAX_STEPS) as i32,
        );
        set_i(setter, &p.seq.rate, self.seq_rate as i32);
        set_f(setter, &p.seq.swing, self.seq_swing);
        set_b(setter, &p.seq.running, false);
    }

    /// Replace the persisted pattern with this patch's steps.
    pub fn write_pattern(&self, pattern: &Arc<parking_lot::Mutex<PatternState>>) {
        let mut guard = pattern.lock();
        guard.steps = self.steps();
        guard.version = guard.version.wrapping_add(1);
    }
}
