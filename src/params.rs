//! Host-visible parameter tree.
//!
//! Every control in the editor is backed by a `Param`, so VST3/CLAP/AU hosts
//! can automate, MIDI-learn and remote-control all of them — not just a
//! handful. The editor never writes engine state directly: it writes params
//! through [`ParamSetter`], and [`ReplicantWaveParams::sync_to_engine`] copies
//! the current parameter values into the DSP engine once per process call.
//!
//! Legacy IDs (`gain`, `cutoff`, `resonance`) are intentionally kept flat and
//! unchanged so automation written with older releases still works.
//!
//! Wavetable *table selection* is a normalised choice param. The 64-step
//! pattern itself is session data, not a control surface, so it is persisted
//! separately (see [`PatternState`]) instead of being exposed as 384 params.

use crate::audio::filter::FilterType;
use crate::audio::fx::distortion::DistortionMode;
use crate::audio::oscillator::{NoiseType, SubWave};
use crate::audio::step_sequencer::{SeqMode, SeqRate, Step, MAX_STEPS};
use crate::audio::SynthEngine;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::Arc;

/// Number of frames available in the built-in wavetable bank.
pub const WAVETABLE_COUNT: i32 = 8;

// ---------------------------------------------------------------------------
// Small constructors, so the definitions stay readable and consistent.
// ---------------------------------------------------------------------------

fn s2v_f32() -> Arc<dyn Fn(&str) -> Option<f32> + Send + Sync> {
    Arc::new(|s| s.trim().parse().ok())
}

fn s2v_i32() -> Arc<dyn Fn(&str) -> Option<i32> + Send + Sync> {
    Arc::new(|s| s.trim().parse().ok())
}

fn v2s_i32_plain() -> Arc<dyn Fn(i32) -> String + Send + Sync> {
    Arc::new(|v| v.to_string())
}

fn f_unit(name: impl Into<String>, default: f32) -> FloatParam {
    FloatParam::new(
        name.into(),
        default,
        FloatRange::Linear { min: 0.0, max: 1.0 },
    )
    .with_value_to_string(formatters::v2s_f32_percentage(0))
    .with_string_to_value(formatters::s2v_f32_percentage())
}

fn f_bipolar(name: impl Into<String>, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Linear {
            min: -1.0,
            max: 1.0,
        },
    )
    .with_value_to_string(formatters::v2s_f32_percentage(0))
    .with_string_to_value(formatters::s2v_f32_percentage())
}

fn f_gain(name: impl Into<String>, default: f32, max: f32) -> FloatParam {
    FloatParam::new(name.into(), default, FloatRange::Linear { min: 0.0, max })
        .with_value_to_string(formatters::v2s_f32_percentage(0))
        .with_string_to_value(formatters::s2v_f32_percentage())
}

fn f_cents(name: impl Into<String>, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Linear {
            min: -100.0,
            max: 100.0,
        },
    )
    .with_unit(" ct")
    .with_value_to_string(formatters::v2s_f32_rounded(1))
    .with_string_to_value(s2v_f32())
}

fn f_semitones(name: impl Into<String>, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Linear {
            min: -36.0,
            max: 36.0,
        },
    )
    .with_step_size(1.0)
    .with_unit(" st")
    .with_value_to_string(formatters::v2s_f32_rounded(0))
    .with_string_to_value(s2v_f32())
}

fn f_hz(name: impl Into<String>, default: f32, min: f32, max: f32, skew: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Skewed {
            min,
            max,
            factor: FloatRange::skew_factor(skew),
        },
    )
    .with_unit(" Hz")
    .with_value_to_string(formatters::v2s_f32_hz_then_khz(1))
    .with_string_to_value(formatters::s2v_f32_hz_then_khz())
}

fn f_ms(name: impl Into<String>, default: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Skewed {
            min,
            max,
            factor: FloatRange::skew_factor(-1.0),
        },
    )
    .with_unit(" ms")
    .with_value_to_string(formatters::v2s_f32_rounded(0))
    .with_string_to_value(s2v_f32())
}

fn f_secs(name: impl Into<String>, default: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Skewed {
            min,
            max,
            factor: FloatRange::skew_factor(-2.0),
        },
    )
    .with_unit(" s")
    .with_value_to_string(formatters::v2s_f32_rounded(2))
    .with_string_to_value(s2v_f32())
}

fn i_choice(name: impl Into<String>, default: i32, names: &'static [&'static str]) -> IntParam {
    let max = names.len().saturating_sub(1) as i32;
    IntParam::new(
        name,
        default.clamp(0, max),
        IntRange::Linear { min: 0, max },
    )
    .with_value_to_string(Arc::new(move |v| {
        names
            .get(v.clamp(0, max) as usize)
            .map(|s| (*s).to_string())
            .unwrap_or_else(|| v.to_string())
    }))
}

fn i_range(name: impl Into<String>, default: i32, min: i32, max: i32) -> IntParam {
    IntParam::new(name.into(), default, IntRange::Linear { min, max })
        .with_value_to_string(v2s_i32_plain())
        .with_string_to_value(s2v_i32())
}

// ---------------------------------------------------------------------------
// Groups
// ---------------------------------------------------------------------------

#[derive(Params)]
pub struct OscParams {
    #[id = "on"]
    pub enabled: BoolParam,
    #[id = "table"]
    pub table: IntParam,
    #[id = "pos"]
    pub position: FloatParam,
    #[id = "tune"]
    pub tune: FloatParam,
    #[id = "fine"]
    pub fine: FloatParam,
    #[id = "unison"]
    pub unison: IntParam,
    #[id = "detune"]
    pub detune: FloatParam,
    #[id = "spread"]
    pub spread: FloatParam,
    #[id = "formant"]
    pub formant: FloatParam,
    #[id = "bend"]
    pub bend: FloatParam,
    #[id = "level"]
    pub level: FloatParam,
    #[id = "pan"]
    pub pan: FloatParam,
}

impl OscParams {
    fn new(label: &str, default_table: i32, default_tune: f32, enabled: bool) -> Self {
        Self {
            enabled: BoolParam::new(format!("{label} Enabled"), enabled),
            table: i_choice(
                format!("{label} Wavetable"),
                default_table,
                &[
                    "Analog Morph",
                    "Cyber Neuro",
                    "Digital FM",
                    "Vocaloid Synth",
                    "Cyber Acid 303",
                    "Dystopian Steel",
                    "PWM Hyper",
                    "Sub Zero",
                ],
            ),
            position: f_unit(format!("{label} Position"), 0.15),
            tune: f_semitones(format!("{label} Tune"), default_tune),
            fine: f_cents(format!("{label} Fine"), 0.0),
            unison: IntParam::new(
                format!("{label} Unison"),
                2,
                IntRange::Linear { min: 1, max: 8 },
            ),
            detune: f_unit(format!("{label} Detune"), 0.15),
            spread: f_unit(format!("{label} Spread"), 0.6),
            formant: f_bipolar(format!("{label} Formant"), 0.0),
            bend: f_bipolar(format!("{label} Bend"), 0.0),
            level: f_gain(format!("{label} Level"), 0.8, 1.5),
            pan: f_bipolar(format!("{label} Pan"), 0.0),
        }
    }
}

#[derive(Params)]
pub struct SubParams {
    #[id = "on"]
    pub enabled: BoolParam,
    #[id = "wave"]
    pub wave: IntParam,
    #[id = "oct"]
    pub octave: IntParam,
    #[id = "level"]
    pub level: FloatParam,
    #[id = "pan"]
    pub pan: FloatParam,
}

#[derive(Params)]
pub struct NoiseParams {
    #[id = "on"]
    pub enabled: BoolParam,
    #[id = "type"]
    pub noise_type: IntParam,
    #[id = "level"]
    pub level: FloatParam,
    #[id = "pan"]
    pub pan: FloatParam,
}

#[derive(Params)]
pub struct FilterParams {
    #[id = "on"]
    pub enabled: BoolParam,
    #[id = "type"]
    pub filter_type: IntParam,
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "key"]
    pub key_tracking: FloatParam,
    #[id = "vel"]
    pub vel_tracking: FloatParam,
    #[id = "envmod"]
    pub env_amount: FloatParam,
    #[id = "route_osc1"]
    pub route_osc1: BoolParam,
    #[id = "route_osc2"]
    pub route_osc2: BoolParam,
    #[id = "route_sub"]
    pub route_sub: BoolParam,
    #[id = "route_noise"]
    pub route_noise: BoolParam,
}

#[derive(Params)]
pub struct EnvParams {
    #[id = "atk"]
    pub attack: FloatParam,
    #[id = "dec"]
    pub decay: FloatParam,
    #[id = "sus"]
    pub sustain: FloatParam,
    #[id = "rel"]
    pub release: FloatParam,
}

impl EnvParams {
    fn new(label: &str, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack: f_secs(format!("{label} Attack"), attack, 0.001, 8.0),
            decay: f_secs(format!("{label} Decay"), decay, 0.01, 8.0),
            sustain: f_unit(format!("{label} Sustain"), sustain),
            release: f_secs(format!("{label} Release"), release, 0.01, 8.0),
        }
    }
}

#[derive(Params)]
pub struct DistParams {
    #[id = "on"]
    pub enabled: BoolParam,
    #[id = "mode"]
    pub mode: IntParam,
    #[id = "amount"]
    pub amount: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
}

#[derive(Params)]
pub struct ChorusParams {
    #[id = "on"]
    pub enabled: BoolParam,
    #[id = "rate"]
    pub rate: FloatParam,
    #[id = "depth"]
    pub depth: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
}

#[derive(Params)]
pub struct DelayParams {
    #[id = "on"]
    pub enabled: BoolParam,
    #[id = "sync"]
    pub sync: BoolParam,
    #[id = "pingpong"]
    pub pingpong: BoolParam,
    #[id = "time"]
    pub time: FloatParam,
    #[id = "fb"]
    pub feedback: FloatParam,
    #[id = "damp"]
    pub damping: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
}

#[derive(Params)]
pub struct ReverbParams {
    #[id = "on"]
    pub enabled: BoolParam,
    #[id = "size"]
    pub room_size: FloatParam,
    #[id = "damp"]
    pub damping: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
}

#[derive(Params)]
pub struct GateParams {
    #[id = "on"]
    pub enabled: BoolParam,
}

#[derive(Params)]
pub struct SeqParams {
    #[id = "run"]
    pub running: BoolParam,
    /// Opt-in: follow the DAW transport. Never starts a stopped sequence.
    #[id = "sync"]
    pub host_sync: BoolParam,
    /// Opt-in: transpose an *already running* sequence from MIDI.
    #[id = "key"]
    pub key_trigger: BoolParam,
    #[id = "mode"]
    pub mode: IntParam,
    #[id = "len"]
    pub length: IntParam,
    #[id = "rate"]
    pub rate: IntParam,
    #[id = "swing"]
    pub swing: FloatParam,
    #[id = "bpm"]
    pub bpm: FloatParam,
    #[id = "oct"]
    pub octave: IntParam,
    #[id = "root"]
    pub root_note: IntParam,
}

// ---------------------------------------------------------------------------
// The full parameter tree
// ---------------------------------------------------------------------------

#[derive(Params)]
pub struct ReplicantWaveParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    /// Master output level (linear gain, displayed in dB).
    ///
    /// Kept with the legacy `gain` ID so old automation lanes still resolve.
    #[id = "gain"]
    pub gain: FloatParam,

    /// Filter cutoff, kept with the legacy `cutoff` ID.
    #[id = "cutoff"]
    pub cutoff: FloatParam,

    /// Filter resonance, kept with the legacy `resonance` ID.
    #[id = "resonance"]
    pub resonance: FloatParam,

    #[nested(id_prefix = "osc1", group = "Oscillator 1")]
    pub osc1: OscParams,
    #[nested(id_prefix = "osc2", group = "Oscillator 2")]
    pub osc2: OscParams,
    #[nested(id_prefix = "sub", group = "Sub Oscillator")]
    pub sub: SubParams,
    #[nested(id_prefix = "noise", group = "Noise")]
    pub noise: NoiseParams,
    #[nested(id_prefix = "flt", group = "Filter")]
    pub filter: FilterParams,
    #[nested(id_prefix = "amp", group = "Amp Envelope")]
    pub amp_env: EnvParams,
    #[nested(id_prefix = "fenv", group = "Filter Envelope")]
    pub filter_env: EnvParams,
    #[nested(id_prefix = "dist", group = "Distortion")]
    pub dist: DistParams,
    #[nested(id_prefix = "chorus", group = "Chorus")]
    pub chorus: ChorusParams,
    #[nested(id_prefix = "delay", group = "Delay")]
    pub delay: DelayParams,
    #[nested(id_prefix = "verb", group = "Reverb")]
    pub reverb: ReverbParams,
    #[nested(id_prefix = "gate", group = "Trance Gate")]
    pub gate: GateParams,
    #[nested(id_prefix = "seq", group = "Sequencer")]
    pub seq: SeqParams,

    /// 64-step pattern. Session data, not a control surface.
    #[persist = "pattern"]
    pub pattern: Arc<parking_lot::Mutex<PatternState>>,
}

/// Serializable sequencer pattern (persisted with the host session).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PatternState {
    /// `Vec` because serde has no built-in support for `[T; 64]`.
    pub steps: Vec<Step>,
    /// Bumped by the editor whenever `steps` change, so the audio thread can
    /// pick the pattern up without locking every block.
    #[serde(skip)]
    pub version: u64,
}

impl Default for PatternState {
    fn default() -> Self {
        let seq = crate::audio::step_sequencer::StepSequencer::default();
        Self {
            steps: seq.steps.to_vec(),
            version: 1,
        }
    }
}

impl Default for ReplicantWaveParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(1300, 920),

            // 0.6 keeps the same loudness as the 1.0.x engine default.
            gain: FloatParam::new(
                "Master Gain",
                0.6,
                FloatRange::Linear { min: 0.0, max: 1.5 },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            cutoff: f_hz("Filter Cutoff", 2500.0, 20.0, 20_000.0, -2.0),
            resonance: f_unit("Filter Resonance", 0.2),

            osc1: OscParams::new("OSC 1", 1, 0.0, true),
            osc2: OscParams::new("OSC 2", 7, -12.0, false),
            sub: SubParams {
                enabled: BoolParam::new("Sub Enabled", false),
                wave: i_choice(
                    "Sub Wave",
                    0,
                    &[
                        "Sine",
                        "Triangle",
                        "Saw",
                        "Pulse 50%",
                        "Pulse 25%",
                        "Pulse 12%",
                    ],
                ),
                octave: i_range("Sub Octave", -1, -2, 0),
                level: f_unit("Sub Level", 0.5),
                pan: f_bipolar("Sub Pan", 0.0),
            },
            noise: NoiseParams {
                enabled: BoolParam::new("Noise Enabled", false),
                noise_type: i_choice("Noise Type", 0, &["White", "Pink", "CyberBit"]),
                level: f_unit("Noise Level", 0.2),
                pan: f_bipolar("Noise Pan", 0.0),
            },
            filter: FilterParams {
                enabled: BoolParam::new("Filter Enabled", true),
                filter_type: i_choice(
                    "Filter Type",
                    1,
                    &[
                        "LP 12", "LP 24", "HP 12", "HP 24", "BP 12", "BP 24", "Notch",
                    ],
                ),
                drive: FloatParam::new(
                    "Filter Drive",
                    1.0,
                    FloatRange::Linear { min: 1.0, max: 4.0 },
                )
                .with_value_to_string(formatters::v2s_f32_rounded(2))
                .with_string_to_value(s2v_f32()),
                key_tracking: f_unit("Filter Key Tracking", 0.0),
                vel_tracking: f_unit("Filter Velocity Tracking", 0.0),
                env_amount: f_bipolar("Filter Envelope Amount", 0.0),
                route_osc1: BoolParam::new("Filter Route OSC 1", true),
                route_osc2: BoolParam::new("Filter Route OSC 2", true),
                route_sub: BoolParam::new("Filter Route Sub", true),
                route_noise: BoolParam::new("Filter Route Noise", true),
            },
            amp_env: EnvParams::new("Amp", 0.01, 0.2, 0.7, 0.3),
            filter_env: EnvParams::new("Filter", 0.05, 0.3, 0.2, 0.3),
            dist: DistParams {
                enabled: BoolParam::new("Distortion Enabled", false),
                mode: i_choice(
                    "Distortion Mode",
                    0,
                    &["Overdrive", "Crusher", "Fire", "Grind"],
                ),
                amount: f_unit("Distortion Drive", 0.2),
                mix: f_unit("Distortion Mix", 1.0),
            },
            chorus: ChorusParams {
                enabled: BoolParam::new("Chorus Enabled", false),
                rate: f_hz("Chorus Rate", 1.5, 0.05, 8.0, -1.0),
                depth: f_ms("Chorus Depth", 3.0, 0.1, 12.0),
                mix: f_unit("Chorus Mix", 0.5),
            },
            delay: DelayParams {
                enabled: BoolParam::new("Delay Enabled", false),
                sync: BoolParam::new("Delay Sync", true),
                pingpong: BoolParam::new("Delay Ping-Pong", true),
                time: f_ms("Delay Time", 300.0, 10.0, 2000.0),
                feedback: f_unit("Delay Feedback", 0.35),
                damping: f_unit("Delay Damping", 0.3),
                mix: f_unit("Delay Mix", 0.35),
            },
            reverb: ReverbParams {
                enabled: BoolParam::new("Reverb Enabled", false),
                room_size: f_unit("Reverb Room Size", 0.75),
                damping: f_unit("Reverb Damping", 0.3),
                mix: f_unit("Reverb Mix", 0.3),
            },
            gate: GateParams {
                enabled: BoolParam::new("Trance Gate Enabled", false),
            },
            seq: SeqParams {
                running: BoolParam::new("Sequencer Play", false).with_value_to_string(Arc::new(
                    |v| {
                        if v {
                            "Playing".to_string()
                        } else {
                            "Stopped".to_string()
                        }
                    },
                )),
                host_sync: BoolParam::new("Sequencer DAW Sync", false),
                key_trigger: BoolParam::new("Sequencer Key Trigger", false),
                mode: i_choice("Sequencer Mode", 3, &["Synth", "Cutoff", "WT Pos", "Off"]),
                length: i_range("Sequencer Length", 16, 1, MAX_STEPS as i32),
                rate: i_choice(
                    "Sequencer Rate",
                    3,
                    &["1/4", "1/8", "1/8T", "1/16", "1/16T", "1/16D", "1/32"],
                ),
                swing: FloatParam::new(
                    "Sequencer Swing",
                    0.0,
                    FloatRange::Linear {
                        min: 0.0,
                        max: 0.75,
                    },
                )
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
                bpm: FloatParam::new(
                    "Sequencer BPM",
                    125.0,
                    FloatRange::Linear {
                        min: 40.0,
                        max: 240.0,
                    },
                )
                .with_unit(" BPM")
                .with_value_to_string(formatters::v2s_f32_rounded(0))
                .with_string_to_value(s2v_f32()),
                octave: IntParam::new("Sequencer Octave", 0, IntRange::Linear { min: -2, max: 2 }),
                root_note: IntParam::new(
                    "Sequencer Root Note",
                    36,
                    IntRange::Linear { min: 0, max: 127 },
                )
                .with_value_to_string(formatters::v2s_i32_note_formatter())
                .with_string_to_value(formatters::s2v_i32_note_formatter()),
            },

            pattern: Arc::new(parking_lot::Mutex::new(PatternState::default())),
        }
    }
}

impl ReplicantWaveParams {
    /// Copy the host-visible parameters into the DSP engine.
    ///
    /// Called once per process call. Parameter reads are relaxed atomics, so
    /// this is realtime-safe and cheap enough for every block.
    pub fn sync_to_engine(&self, engine: &mut SynthEngine) {
        engine.master_gain = self.gain.value();

        let table1 = self.osc1.table.value().clamp(0, WAVETABLE_COUNT - 1) as usize;
        let table2 = self.osc2.table.value().clamp(0, WAVETABLE_COUNT - 1) as usize;
        for v in engine.voices.iter_mut() {
            apply_osc(&self.osc1, &mut v.osc1, &mut v.osc1_table_idx, table1);
            apply_osc(&self.osc2, &mut v.osc2, &mut v.osc2_table_idx, table2);
        }

        let sub_wave = SubWave::from_index(self.sub.wave.value().clamp(0, 5) as usize);
        let noise_type = match self.noise.noise_type.value() {
            0 => NoiseType::White,
            1 => NoiseType::Pink,
            _ => NoiseType::CyberBit,
        };
        let filter_type =
            FilterType::from_index(self.filter.filter_type.value().clamp(0, 6) as usize);

        for v in engine.voices.iter_mut() {
            v.sub.enabled = self.sub.enabled.value();
            v.sub.wave = sub_wave;
            v.sub.octave_offset = self.sub.octave.value().clamp(-2, 0) as i8;
            v.sub.level_gain = self.sub.level.value();
            v.sub.pan = self.sub.pan.value();

            v.noise.enabled = self.noise.enabled.value();
            v.noise.noise_type = noise_type;
            v.noise.level_gain = self.noise.level.value();
            v.noise.pan = self.noise.pan.value();

            v.filter.enabled = self.filter.enabled.value();
            v.filter.filter_type = filter_type;
            v.filter.cutoff_hz = self.cutoff.value().clamp(20.0, 20_000.0);
            v.filter.resonance = self.resonance.value();
            v.filter.drive = self.filter.drive.value();
            v.filter.key_tracking = self.filter.key_tracking.value();
            v.filter.vel_tracking = self.filter.vel_tracking.value();
            v.filter.env_amount = self.filter.env_amount.value();
            v.filter.route_osc1 = self.filter.route_osc1.value();
            v.filter.route_osc2 = self.filter.route_osc2.value();
            v.filter.route_sub = self.filter.route_sub.value();
            v.filter.route_noise = self.filter.route_noise.value();

            v.amp_env.attack_s = self.amp_env.attack.value();
            v.amp_env.decay_s = self.amp_env.decay.value();
            v.amp_env.sustain_level = self.amp_env.sustain.value();
            v.amp_env.release_s = self.amp_env.release.value();

            v.filter_env.attack_s = self.filter_env.attack.value();
            v.filter_env.decay_s = self.filter_env.decay.value();
            v.filter_env.sustain_level = self.filter_env.sustain.value();
            v.filter_env.release_s = self.filter_env.release.value();
        }

        engine.fx.distortion.enabled = self.dist.enabled.value();
        engine.fx.distortion.mode =
            DistortionMode::from_index(self.dist.mode.value().clamp(0, 3) as usize);
        engine.fx.distortion.amount = self.dist.amount.value();
        engine.fx.distortion.mix = self.dist.mix.value();

        engine.fx.chorus.enabled = self.chorus.enabled.value();
        engine.fx.chorus.rate_hz = self.chorus.rate.value();
        engine.fx.chorus.depth_ms = self.chorus.depth.value();
        engine.fx.chorus.mix = self.chorus.mix.value();

        engine.fx.delay.enabled = self.delay.enabled.value();
        engine.fx.delay.sync = self.delay.sync.value();
        engine.fx.delay.ping_pong = self.delay.pingpong.value();
        engine.fx.delay.time_ms = self.delay.time.value();
        engine.fx.delay.feedback = self.delay.feedback.value();
        engine.fx.delay.damping = self.delay.damping.value();
        engine.fx.delay.mix = self.delay.mix.value();

        engine.fx.reverb.enabled = self.reverb.enabled.value();
        engine.fx.reverb.room_size = self.reverb.room_size.value();
        engine.fx.reverb.damping = self.reverb.damping.value();
        engine.fx.reverb.mix = self.reverb.mix.value();

        engine.fx.gate.enabled = self.gate.enabled.value();

        // Sequencer: transport and clock are parameters; the pattern is session
        // state and is applied separately from `self.pattern`.
        let was_running = engine.sequencer.running;
        let running = self.seq.running.value();
        engine.sequencer.running = running;
        engine.sequencer.host_sync = self.seq.host_sync.value();
        engine.sequencer.key_trigger = self.seq.key_trigger.value();
        engine.sequencer.mode = SeqMode::from_index(self.seq.mode.value().clamp(0, 3) as usize);
        engine.sequencer.length = (self.seq.length.value().clamp(1, MAX_STEPS as i32)) as usize;
        engine.sequencer.rate = SeqRate::from_index(self.seq.rate.value().clamp(0, 6) as usize);
        engine.sequencer.swing = self.seq.swing.value();
        engine.sequencer.internal_bpm = self.seq.bpm.value();
        engine.sequencer.octave = self.seq.octave.value().clamp(-2, 2) as i8;
        engine.sequencer.root_note = self.seq.root_note.value().clamp(0, 127) as u8;

        if running && !was_running {
            engine.sequencer.reset();
        } else if !running && was_running {
            engine.stop_all_notes();
        }
    }

    /// Copy the persisted pattern into the engine if it changed.
    ///
    /// Uses `try_lock` so the audio thread can never block on the editor.
    pub fn apply_pattern_if_changed(&self, engine: &mut SynthEngine, seen: &mut u64) {
        if let Some(guard) = self.pattern.try_lock() {
            if guard.version != *seen {
                for (dst, src) in engine.sequencer.steps.iter_mut().zip(guard.steps.iter()) {
                    *dst = *src;
                }
                *seen = guard.version;
            }
        }
    }
}

/// Copy one oscillator's parameters into one voice's oscillator.
fn apply_osc(
    p: &OscParams,
    osc: &mut crate::audio::oscillator::WavetableOscillator,
    table_idx: &mut usize,
    table: usize,
) {
    osc.enabled = p.enabled.value();
    *table_idx = table;
    osc.position = p.position.value();
    osc.tune_semitones = p.tune.value();
    osc.fine_cents = p.fine.value();
    osc.unison = p.unison.value().clamp(1, 8) as usize;
    osc.detune = p.detune.value();
    osc.spread = p.spread.value();
    osc.formant = p.formant.value();
    osc.bend = p.bend.value();
    osc.level_gain = p.level.value();
    osc.pan = p.pan.value();
}
