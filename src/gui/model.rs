//! UI parameter model.
//!
//! The editor never holds the engine mutex while painting. Each frame draws
//! from this plain-data snapshot, and the result is pushed back to the audio
//! engine in a short `try_lock` at the end of the frame. The audio callback is
//! therefore never blocked by a slow GUI repaint (a classic source of dropouts
//! and crackle when the editor is open).

use crate::audio::filter::FilterType;
use crate::audio::fx::distortion::DistortionMode;
use crate::audio::oscillator::{NoiseType, SubWave};
use crate::audio::step_sequencer::{SeqMode, SeqRate, Step, MAX_STEPS};
use crate::audio::SynthEngine;

#[derive(Clone, Debug)]
pub struct OscUi {
    pub enabled: bool,
    pub table: usize,
    pub unison: usize,
    pub tune: f32,
    pub fine: f32,
    pub level: f32,
    pub pos: f32,
    pub detune: f32,
    pub spread: f32,
    pub pan: f32,
    pub formant: f32,
    pub bend: f32,
}

#[derive(Clone, Debug)]
pub struct FilterUi {
    pub enabled: bool,
    pub kind: FilterType,
    pub cutoff: f32,
    pub reso: f32,
    pub drive: f32,
    pub key: f32,
    pub vel: f32,
    pub env_amount: f32,
    pub route_osc1: bool,
    pub route_osc2: bool,
    pub route_sub: bool,
    pub route_noise: bool,
}

#[derive(Clone, Debug)]
pub struct EnvUi {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

#[derive(Clone, Debug)]
pub struct FxUi {
    pub dist_enabled: bool,
    pub dist_mode: DistortionMode,
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
    pub delay_pingpong: bool,
    pub reverb_enabled: bool,
    pub reverb_size: f32,
    pub reverb_damp: f32,
    pub reverb_mix: f32,
    pub gate_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct SeqUi {
    pub running: bool,
    pub host_sync: bool,
    pub key_trigger: bool,
    pub latch: bool,
    pub mode: SeqMode,
    pub length: usize,
    pub rate: SeqRate,
    pub swing: f32,
    pub bpm: f32,
    pub octave: i8,
    pub root_note: u8,
    pub steps: [Step; MAX_STEPS],
}

#[derive(Clone, Debug)]
pub struct UiModel {
    pub master: f32,
    pub osc1: OscUi,
    pub osc2: OscUi,
    pub sub_enabled: bool,
    pub sub_wave: usize,
    pub sub_octave: i8,
    pub sub_level: f32,
    pub sub_pan: f32,
    pub noise_enabled: bool,
    pub noise_type: NoiseType,
    pub noise_level: f32,
    pub noise_pan: f32,
    pub filter: FilterUi,
    pub amp_env: EnvUi,
    pub filter_env: EnvUi,
    pub fx: FxUi,
    pub seq: SeqUi,
}

impl UiModel {
    pub fn capture(engine: &SynthEngine) -> Self {
        let v = &engine.voices[0];
        let mk_osc = |osc: &crate::audio::oscillator::WavetableOscillator, table: usize| OscUi {
            enabled: osc.enabled,
            table,
            unison: osc.unison,
            tune: osc.tune_semitones,
            fine: osc.fine_cents,
            level: osc.level_gain,
            pos: osc.position,
            detune: osc.detune,
            spread: osc.spread,
            pan: osc.pan,
            formant: osc.formant,
            bend: osc.bend,
        };
        let mut steps = [Step::default(); MAX_STEPS];
        steps.copy_from_slice(&engine.sequencer.steps);

        Self {
            master: engine.master_gain,
            osc1: mk_osc(&v.osc1, v.osc1_table_idx),
            osc2: mk_osc(&v.osc2, v.osc2_table_idx),
            sub_enabled: v.sub.enabled,
            sub_wave: match v.sub.wave {
                SubWave::Sine => 0,
                SubWave::Triangle => 1,
                SubWave::Saw => 2,
                SubWave::Pulse50 => 3,
                SubWave::Pulse25 => 4,
                SubWave::Pulse12 => 5,
            },
            sub_octave: v.sub.octave_offset,
            sub_level: v.sub.level_gain,
            sub_pan: v.sub.pan,
            noise_enabled: v.noise.enabled,
            noise_type: v.noise.noise_type,
            noise_level: v.noise.level_gain,
            noise_pan: v.noise.pan,
            filter: FilterUi {
                enabled: v.filter.enabled,
                kind: v.filter.filter_type,
                cutoff: v.filter.cutoff_hz,
                reso: v.filter.resonance,
                drive: v.filter.drive,
                key: v.filter.key_tracking,
                vel: v.filter.vel_tracking,
                env_amount: v.filter.env_amount,
                route_osc1: v.filter.route_osc1,
                route_osc2: v.filter.route_osc2,
                route_sub: v.filter.route_sub,
                route_noise: v.filter.route_noise,
            },
            amp_env: EnvUi {
                attack: v.amp_env.attack_s,
                decay: v.amp_env.decay_s,
                sustain: v.amp_env.sustain_level,
                release: v.amp_env.release_s,
            },
            filter_env: EnvUi {
                attack: v.filter_env.attack_s,
                decay: v.filter_env.decay_s,
                sustain: v.filter_env.sustain_level,
                release: v.filter_env.release_s,
            },
            fx: FxUi {
                dist_enabled: engine.fx.distortion.enabled,
                dist_mode: engine.fx.distortion.mode,
                dist_amount: engine.fx.distortion.amount,
                dist_mix: engine.fx.distortion.mix,
                chorus_enabled: engine.fx.chorus.enabled,
                chorus_rate: engine.fx.chorus.rate_hz,
                chorus_depth: engine.fx.chorus.depth_ms,
                chorus_mix: engine.fx.chorus.mix,
                delay_enabled: engine.fx.delay.enabled,
                delay_sync: engine.fx.delay.sync,
                delay_time: engine.fx.delay.time_ms,
                delay_fb: engine.fx.delay.feedback,
                delay_damp: engine.fx.delay.damping,
                delay_mix: engine.fx.delay.mix,
                delay_pingpong: engine.fx.delay.ping_pong,
                reverb_enabled: engine.fx.reverb.enabled,
                reverb_size: engine.fx.reverb.room_size,
                reverb_damp: engine.fx.reverb.damping,
                reverb_mix: engine.fx.reverb.mix,
                gate_enabled: engine.fx.gate.enabled,
            },
            seq: SeqUi {
                running: engine.sequencer.running,
                host_sync: engine.sequencer.host_sync,
                key_trigger: engine.sequencer.key_trigger,
                latch: engine.sequencer.latch,
                mode: engine.sequencer.mode,
                length: engine.sequencer.length,
                rate: engine.sequencer.rate,
                swing: engine.sequencer.swing,
                bpm: engine.sequencer.internal_bpm,
                octave: engine.sequencer.octave,
                root_note: engine.sequencer.root_note,
                steps,
            },
        }
    }

    /// Push the snapshot into the engine. Called with a short `try_lock`.
    pub fn apply(&self, engine: &mut SynthEngine) {
        engine.master_gain = self.master;

        for v in engine.voices.iter_mut() {
            v.osc1.enabled = self.osc1.enabled;
            v.osc1_table_idx = self.osc1.table;
            v.osc1.unison = self.osc1.unison;
            v.osc1.tune_semitones = self.osc1.tune;
            v.osc1.fine_cents = self.osc1.fine;
            v.osc1.level_gain = self.osc1.level;
            v.osc1.position = self.osc1.pos;
            v.osc1.detune = self.osc1.detune;
            v.osc1.spread = self.osc1.spread;
            v.osc1.pan = self.osc1.pan;
            v.osc1.formant = self.osc1.formant;
            v.osc1.bend = self.osc1.bend;

            v.osc2.enabled = self.osc2.enabled;
            v.osc2_table_idx = self.osc2.table;
            v.osc2.unison = self.osc2.unison;
            v.osc2.tune_semitones = self.osc2.tune;
            v.osc2.fine_cents = self.osc2.fine;
            v.osc2.level_gain = self.osc2.level;
            v.osc2.position = self.osc2.pos;
            v.osc2.detune = self.osc2.detune;
            v.osc2.spread = self.osc2.spread;
            v.osc2.pan = self.osc2.pan;
            v.osc2.formant = self.osc2.formant;
            v.osc2.bend = self.osc2.bend;

            v.sub.enabled = self.sub_enabled;
            v.sub.wave = SubWave::from_index(self.sub_wave);
            v.sub.octave_offset = self.sub_octave;
            v.sub.level_gain = self.sub_level;
            v.sub.pan = self.sub_pan;

            v.noise.enabled = self.noise_enabled;
            v.noise.noise_type = self.noise_type;
            v.noise.level_gain = self.noise_level;
            v.noise.pan = self.noise_pan;

            v.filter.enabled = self.filter.enabled;
            v.filter.filter_type = self.filter.kind;
            v.filter.cutoff_hz = self.filter.cutoff;
            v.filter.resonance = self.filter.reso;
            v.filter.drive = self.filter.drive;
            v.filter.key_tracking = self.filter.key;
            v.filter.vel_tracking = self.filter.vel;
            v.filter.env_amount = self.filter.env_amount;
            v.filter.route_osc1 = self.filter.route_osc1;
            v.filter.route_osc2 = self.filter.route_osc2;
            v.filter.route_sub = self.filter.route_sub;
            v.filter.route_noise = self.filter.route_noise;

            v.amp_env.attack_s = self.amp_env.attack;
            v.amp_env.decay_s = self.amp_env.decay;
            v.amp_env.sustain_level = self.amp_env.sustain;
            v.amp_env.release_s = self.amp_env.release;

            v.filter_env.attack_s = self.filter_env.attack;
            v.filter_env.decay_s = self.filter_env.decay;
            v.filter_env.sustain_level = self.filter_env.sustain;
            v.filter_env.release_s = self.filter_env.release;
        }

        engine.fx.distortion.enabled = self.fx.dist_enabled;
        engine.fx.distortion.mode = self.fx.dist_mode;
        engine.fx.distortion.amount = self.fx.dist_amount;
        engine.fx.distortion.mix = self.fx.dist_mix;

        engine.fx.chorus.enabled = self.fx.chorus_enabled;
        engine.fx.chorus.rate_hz = self.fx.chorus_rate;
        engine.fx.chorus.depth_ms = self.fx.chorus_depth;
        engine.fx.chorus.mix = self.fx.chorus_mix;

        engine.fx.delay.enabled = self.fx.delay_enabled;
        engine.fx.delay.sync = self.fx.delay_sync;
        engine.fx.delay.time_ms = self.fx.delay_time;
        engine.fx.delay.feedback = self.fx.delay_fb;
        engine.fx.delay.damping = self.fx.delay_damp;
        engine.fx.delay.mix = self.fx.delay_mix;
        engine.fx.delay.ping_pong = self.fx.delay_pingpong;

        engine.fx.reverb.enabled = self.fx.reverb_enabled;
        engine.fx.reverb.room_size = self.fx.reverb_size;
        engine.fx.reverb.damping = self.fx.reverb_damp;
        engine.fx.reverb.mix = self.fx.reverb_mix;

        engine.fx.gate.enabled = self.fx.gate_enabled;

        let was_running = engine.sequencer.running;
        engine.sequencer.host_sync = self.seq.host_sync;
        engine.sequencer.key_trigger = self.seq.key_trigger;
        engine.sequencer.latch = self.seq.latch;
        engine.sequencer.mode = self.seq.mode;
        engine.sequencer.length = self.seq.length.clamp(1, MAX_STEPS);
        engine.sequencer.rate = self.seq.rate;
        engine.sequencer.swing = self.seq.swing;
        engine.sequencer.internal_bpm = self.seq.bpm;
        engine.sequencer.octave = self.seq.octave;
        engine.sequencer.root_note = self.seq.root_note;
        engine.sequencer.steps.copy_from_slice(&self.seq.steps);
        engine.sequencer.running = self.seq.running;

        if self.seq.running && !was_running {
            engine.sequencer.reset();
        } else if !self.seq.running && was_running {
            engine.stop_all_notes();
        }
    }
}
