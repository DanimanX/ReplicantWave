pub mod knob;
pub mod oscilloscope;
pub mod sequencer_view;
pub mod theme;
pub mod wavetable_view;

use crate::audio::filter::FilterType;
use crate::audio::fx::distortion::DistortionMode;
use crate::audio::step_sequencer::SeqMode;
use crate::audio::SynthEngine;
use crate::presets::factory::get_factory_presets;
use crate::presets::SynthPreset;
use knob::{draw_rotary_knob, draw_rotary_knob_log, draw_rotary_knob_usize};
use oscilloscope::draw_oscilloscope;
use sequencer_view::{draw_sequencer_drawer, SequencerUiState};
use theme::CyberTheme;
use wavetable_view::draw_wavetable_preview;

use nih_plug_egui::egui::{
    Button, Color32, ComboBox, RichText, Stroke, Ui,
};
use parking_lot::Mutex;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub const DONATION_URL: &str = "https://paypal.me/DanimanX?locale.x=en_US&country.x=CL";

pub struct GuiSharedState {
    pub engine: Arc<Mutex<SynthEngine>>,
    pub seq_ui_state: SequencerUiState,
    pub presets: Vec<SynthPreset>,
    pub current_preset_idx: usize,
}

impl GuiSharedState {
    pub fn new(engine: Arc<Mutex<SynthEngine>>) -> Self {
        let presets = get_factory_presets();
        Self {
            engine,
            seq_ui_state: SequencerUiState::default(),
            presets,
            current_preset_idx: 0,
        }
    }
}

pub fn render_cyberpunk_gui(ui: &mut Ui, state: &mut GuiSharedState) {
    ui.style_mut().visuals.panel_fill = CyberTheme::BG_VOID;
    ui.style_mut().visuals.window_fill = CyberTheme::BG_VOID;

    let mut engine_guard = state.engine.lock();
    let engine = &mut *engine_guard;

    // Outer container with cyberpunk chamfered margins
    ui.vertical(|ui| {
        // =========================================================================
        // 1. TOP HEADER: BRAND, PRESETS, TELEMETRY, MASTER, AND DONATION BUTTON
        // =========================================================================
        ui.horizontal(|ui| {
            // Cyberpunk Logo
            ui.label(
                RichText::new("⚡ REPLICANTWAVE ⚡")
                    .color(CyberTheme::NEON_CYAN)
                    .strong()
                    .size(15.0),
            );
            ui.label(
                RichText::new("// CYBERPUNK SYNTH")
                    .color(CyberTheme::TEXT_DIM)
                    .size(10.0),
            );

            ui.add_space(8.0);

            // Preset Selector
            if ui.button(RichText::new("◀").color(CyberTheme::NEON_CYAN)).clicked() {
                if state.current_preset_idx > 0 {
                    state.current_preset_idx -= 1;
                } else {
                    state.current_preset_idx = state.presets.len() - 1;
                }
                state.presets[state.current_preset_idx].apply_to_engine(engine);
            }

            let cur_name = state.presets[state.current_preset_idx].name.clone();
            ComboBox::from_id_salt("preset_combo")
                .selected_text(RichText::new(&cur_name).color(CyberTheme::NEON_AMBER).strong())
                .width(180.0)
                .show_ui(ui, |ui| {
                    for (i, p) in state.presets.iter().enumerate() {
                        let text = format!("[{}] {}", p.category, p.name);
                        let _is_sel = state.current_preset_idx == i;
                        if ui.selectable_value(&mut state.current_preset_idx, i, text).clicked() {
                            state.presets[i].apply_to_engine(engine);
                        }
                    }
                });

            if ui.button(RichText::new("▶").color(CyberTheme::NEON_CYAN)).clicked() {
                state.current_preset_idx = (state.current_preset_idx + 1) % state.presets.len();
                state.presets[state.current_preset_idx].apply_to_engine(engine);
            }

            ui.add_space(12.0);

            // Live Telemetry Readout
            let active_voices = engine.active_voice_count.load(Ordering::Relaxed);
            ui.label(
                RichText::new(format!("VOICES: {:02}/16 | DSP: 64-BIT", active_voices))
                    .color(CyberTheme::NEON_LIME)
                    .size(11.0),
            );

            ui.add_space(8.0);

            // Master 64-Step Pattern Sequencer Quick Toggle
            let seq_active = engine.sequencer.is_playback_running(true);
            let seq_text = if seq_active { "⚡ SEQ: ON" } else { "⚡ SEQ: OFF" };
            let seq_btn = Button::new(
                RichText::new(seq_text)
                    .color(if seq_active { Color32::from_rgb(10, 11, 16) } else { CyberTheme::TEXT_PRIMARY })
                    .strong()
                    .size(10.5),
            )
            .fill(if seq_active { CyberTheme::NEON_LIME } else { CyberTheme::BG_RECESSED })
            .stroke(Stroke::new(1.0_f32, if seq_active { CyberTheme::NEON_LIME } else { CyberTheme::BORDER_DARK }));

            if ui.add(seq_btn).clicked() {
                if engine.sequencer.mode == SeqMode::Off {
                    engine.sequencer.mode = SeqMode::InternalSynth;
                }
                engine.sequencer.running = !engine.sequencer.running;
                if engine.sequencer.running {
                    engine.sequencer.reset();
                }
            }

            ui.add_space(8.0);

            // Master Volume Analog Rotary Knob
            draw_rotary_knob(ui, &mut engine.master_gain, 0.0..=1.5, "MASTER", CyberTheme::NEON_AMBER);

            ui.add_space(12.0);

            // =====================================================================
            // DONATION BUTTON (Required by user!)
            // =====================================================================
            let donate_btn = Button::new(
                RichText::new("☕ DONATE / BUY ME A COFFEE")
                    .color(Color32::from_rgb(10, 11, 16))
                    .strong()
                    .size(11.0),
            )
            .fill(CyberTheme::NEON_AMBER)
            .stroke(Stroke::new(1.5_f32, Color32::WHITE));

            if ui.add(donate_btn).clicked() {
                let _ = open::that(DONATION_URL);
            }
        });

        ui.add_space(8.0);
        ui.separator();

        // =========================================================================
        // 2. MAIN DSP SYNTH PANELS (3 Columns)
        // =========================================================================
        ui.horizontal(|ui| {
            // ---------------------------------------------------------------------
            // COLUMN 1: OSCILLATOR 1 & OSCILLATOR 2
            // ---------------------------------------------------------------------
            ui.group(|ui| {
                ui.set_width(320.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("⚡ OSCILLATORS").color(CyberTheme::NEON_CYAN).strong());

                // --- OSC 1 ---
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut engine.voices[0].osc1.enabled, "");
                        ui.label(RichText::new("OSC 1:").color(CyberTheme::NEON_CYAN).strong());

                        // Wavetable name selector
                        let table_names = engine.bank.names();
                        let mut cur_table = engine.voices[0].osc1_table_idx;
                        ComboBox::from_id_salt("osc1_table_combo")
                            .selected_text(table_names.get(cur_table).cloned().unwrap_or_default())
                            .show_ui(ui, |ui| {
                                for (i, name) in table_names.iter().enumerate() {
                                    if ui.selectable_value(&mut cur_table, i, name).clicked() {
                                        for v in engine.voices.iter_mut() {
                                            v.osc1_table_idx = cur_table;
                                        }
                                    }
                                }
                            });
                    });

                    // Wavetable Morph Preview
                    let cur_wt1 = engine.bank.get_table(engine.voices[0].osc1_table_idx);
                    let cur_pos1 = engine.voices[0].osc1.position;
                    draw_wavetable_preview(ui, cur_wt1, cur_pos1, CyberTheme::NEON_CYAN, 296.0, 68.0);

                    // Osc 1 Rotary Knobs (Row 1: POS, TUNE, UNISON, DETUNE)
                    ui.horizontal(|ui| {
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc1.position, 0.0..=1.0, "POS", CyberTheme::NEON_CYAN).changed() {
                            let val = engine.voices[0].osc1.position;
                            for v in engine.voices.iter_mut() { v.osc1.position = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc1.tune_semitones, -36.0..=36.0, "TUNE", CyberTheme::NEON_CYAN).changed() {
                            let val = engine.voices[0].osc1.tune_semitones;
                            for v in engine.voices.iter_mut() { v.osc1.tune_semitones = val; }
                        }
                        if draw_rotary_knob_usize(ui, &mut engine.voices[0].osc1.unison, 1..=8, "UNISON", CyberTheme::NEON_CYAN).changed() {
                            let val = engine.voices[0].osc1.unison;
                            for v in engine.voices.iter_mut() { v.osc1.unison = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc1.detune, 0.0..=1.0, "DETUNE", CyberTheme::NEON_CYAN).changed() {
                            let val = engine.voices[0].osc1.detune;
                            for v in engine.voices.iter_mut() { v.osc1.detune = val; }
                        }
                    });

                    // Osc 1 Rotary Knobs (Row 2: LEVEL, SPREAD, FORMANT, BEND)
                    ui.horizontal(|ui| {
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc1.level_gain, 0.0..=1.5, "LEVEL", CyberTheme::NEON_CYAN).changed() {
                            let val = engine.voices[0].osc1.level_gain;
                            for v in engine.voices.iter_mut() { v.osc1.level_gain = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc1.spread, 0.0..=1.0, "SPREAD", CyberTheme::NEON_CYAN).changed() {
                            let val = engine.voices[0].osc1.spread;
                            for v in engine.voices.iter_mut() { v.osc1.spread = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc1.formant, -1.0..=1.0, "FORMANT", CyberTheme::NEON_CYAN).changed() {
                            let val = engine.voices[0].osc1.formant;
                            for v in engine.voices.iter_mut() { v.osc1.formant = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc1.bend, -1.0..=1.0, "BEND", CyberTheme::NEON_CYAN).changed() {
                            let val = engine.voices[0].osc1.bend;
                            for v in engine.voices.iter_mut() { v.osc1.bend = val; }
                        }
                    });
                });

                ui.add_space(4.0);

                // --- OSC 2 ---
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut engine.voices[0].osc2.enabled, "");
                        ui.label(RichText::new("OSC 2:").color(CyberTheme::NEON_PINK).strong());

                        let table_names = engine.bank.names();
                        let mut cur_table = engine.voices[0].osc2_table_idx;
                        ComboBox::from_id_salt("osc2_table_combo")
                            .selected_text(table_names.get(cur_table).cloned().unwrap_or_default())
                            .show_ui(ui, |ui| {
                                for (i, name) in table_names.iter().enumerate() {
                                    if ui.selectable_value(&mut cur_table, i, name).clicked() {
                                        for v in engine.voices.iter_mut() {
                                            v.osc2_table_idx = cur_table;
                                        }
                                    }
                                }
                            });
                    });

                    let cur_wt2 = engine.bank.get_table(engine.voices[0].osc2_table_idx);
                    let cur_pos2 = engine.voices[0].osc2.position;
                    draw_wavetable_preview(ui, cur_wt2, cur_pos2, CyberTheme::NEON_PINK, 296.0, 68.0);

                    // Osc 2 Rotary Knobs (Row 1: POS, TUNE, UNISON, DETUNE)
                    ui.horizontal(|ui| {
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc2.position, 0.0..=1.0, "POS", CyberTheme::NEON_PINK).changed() {
                            let val = engine.voices[0].osc2.position;
                            for v in engine.voices.iter_mut() { v.osc2.position = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc2.tune_semitones, -36.0..=36.0, "TUNE", CyberTheme::NEON_PINK).changed() {
                            let val = engine.voices[0].osc2.tune_semitones;
                            for v in engine.voices.iter_mut() { v.osc2.tune_semitones = val; }
                        }
                        if draw_rotary_knob_usize(ui, &mut engine.voices[0].osc2.unison, 1..=8, "UNISON", CyberTheme::NEON_PINK).changed() {
                            let val = engine.voices[0].osc2.unison;
                            for v in engine.voices.iter_mut() { v.osc2.unison = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc2.detune, 0.0..=1.0, "DETUNE", CyberTheme::NEON_PINK).changed() {
                            let val = engine.voices[0].osc2.detune;
                            for v in engine.voices.iter_mut() { v.osc2.detune = val; }
                        }
                    });

                    // Osc 2 Rotary Knobs (Row 2: LEVEL, SPREAD, FORMANT, BEND)
                    ui.horizontal(|ui| {
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc2.level_gain, 0.0..=1.5, "LEVEL", CyberTheme::NEON_PINK).changed() {
                            let val = engine.voices[0].osc2.level_gain;
                            for v in engine.voices.iter_mut() { v.osc2.level_gain = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc2.spread, 0.0..=1.0, "SPREAD", CyberTheme::NEON_PINK).changed() {
                            let val = engine.voices[0].osc2.spread;
                            for v in engine.voices.iter_mut() { v.osc2.spread = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc2.formant, -1.0..=1.0, "FORMANT", CyberTheme::NEON_PINK).changed() {
                            let val = engine.voices[0].osc2.formant;
                            for v in engine.voices.iter_mut() { v.osc2.formant = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].osc2.bend, -1.0..=1.0, "BEND", CyberTheme::NEON_PINK).changed() {
                            let val = engine.voices[0].osc2.bend;
                            for v in engine.voices.iter_mut() { v.osc2.bend = val; }
                        }
                    });
                });
                });
            });

            // ---------------------------------------------------------------------
            // COLUMN 2: MULTIMODE FILTER, SUB/NOISE, AND LIVE OSCILLOSCOPE
            // ---------------------------------------------------------------------
            ui.group(|ui| {
                ui.set_width(330.0);
                ui.vertical(|ui| {
                    // Live Oscilloscope Display
                    ui.label(RichText::new("⚡ NEON OSCILLOSCOPE").color(CyberTheme::NEON_CYAN).strong());
                let samples: Vec<f32> = {
                    let scope = engine.scope_data.lock();
                    scope.buffer.to_vec()
                };
                draw_oscilloscope(ui, &samples, 310.0, 72.0);

                ui.add_space(4.0);

                // Multimode Filter
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut engine.voices[0].filter.enabled, "");
                        ui.label(RichText::new("MULTIMODE FILTER").color(CyberTheme::NEON_AMBER).strong());

                        let f_types = ["LP 12", "LP 24", "HP 12", "HP 24", "BP 12", "BP 24", "NOTCH"];
                        let mut cur_type = match engine.voices[0].filter.filter_type {
                            FilterType::LowPass12 => 0,
                            FilterType::LowPass24 => 1,
                            FilterType::HighPass12 => 2,
                            FilterType::HighPass24 => 3,
                            FilterType::BandPass12 => 4,
                            FilterType::BandPass24 => 5,
                            FilterType::Notch => 6,
                        };

                        ComboBox::from_id_salt("flt_type_combo")
                            .selected_text(f_types[cur_type])
                            .show_ui(ui, |ui| {
                                for (i, &name) in f_types.iter().enumerate() {
                                    if ui.selectable_value(&mut cur_type, i, name).clicked() {
                                        let ft = FilterType::from_index(cur_type);
                                        for v in engine.voices.iter_mut() { v.filter.filter_type = ft; }
                                    }
                                }
                            });
                    });

                    // Filter Analog Rotary Knobs (Row 1: CUTOFF, RESO, DRIVE, ENV MOD)
                    ui.horizontal(|ui| {
                        if draw_rotary_knob_log(ui, &mut engine.voices[0].filter.cutoff_hz, 20.0..=18000.0, "CUTOFF", CyberTheme::NEON_AMBER).changed() {
                            let val = engine.voices[0].filter.cutoff_hz;
                            for v in engine.voices.iter_mut() { v.filter.cutoff_hz = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].filter.resonance, 0.0..=1.0, "RESO", CyberTheme::NEON_AMBER).changed() {
                            let val = engine.voices[0].filter.resonance;
                            for v in engine.voices.iter_mut() { v.filter.resonance = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].filter.drive, 1.0..=4.0, "DRIVE", CyberTheme::NEON_AMBER).changed() {
                            let val = engine.voices[0].filter.drive;
                            for v in engine.voices.iter_mut() { v.filter.drive = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].filter.env_amount, -1.0..=1.0, "ENV MOD", CyberTheme::NEON_AMBER).changed() {
                            let val = engine.voices[0].filter.env_amount;
                            for v in engine.voices.iter_mut() { v.filter.env_amount = val; }
                        }
                    });

                    // Routing Toggles
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("ROUTE:").color(CyberTheme::TEXT_MUTED).size(10.0));
                        if ui.checkbox(&mut engine.voices[0].filter.route_osc1, "OSC1").changed() {
                            let b = engine.voices[0].filter.route_osc1;
                            for v in engine.voices.iter_mut() { v.filter.route_osc1 = b; }
                        }
                        if ui.checkbox(&mut engine.voices[0].filter.route_osc2, "OSC2").changed() {
                            let b = engine.voices[0].filter.route_osc2;
                            for v in engine.voices.iter_mut() { v.filter.route_osc2 = b; }
                        }
                        if ui.checkbox(&mut engine.voices[0].filter.route_sub, "SUB").changed() {
                            let b = engine.voices[0].filter.route_sub;
                            for v in engine.voices.iter_mut() { v.filter.route_sub = b; }
                        }
                    });
                });

                ui.add_space(4.0);

                // Sub Oscillator & Noise Generator
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut engine.voices[0].sub.enabled, "");
                        ui.label(RichText::new("SUB OSC:").color(CyberTheme::NEON_LIME).strong().size(11.0));
                        if draw_rotary_knob(ui, &mut engine.voices[0].sub.level_gain, 0.0..=1.0, "SUB LVL", CyberTheme::NEON_LIME).changed() {
                            let val = engine.voices[0].sub.level_gain;
                            for v in engine.voices.iter_mut() { v.sub.level_gain = val; }
                        }

                        ui.add_space(14.0);

                        ui.checkbox(&mut engine.voices[0].noise.enabled, "");
                        ui.label(RichText::new("NOISE:").color(CyberTheme::TEXT_MUTED).strong().size(11.0));
                        if draw_rotary_knob(ui, &mut engine.voices[0].noise.level_gain, 0.0..=1.0, "NOISE LVL", CyberTheme::TEXT_MUTED).changed() {
                            let val = engine.voices[0].noise.level_gain;
                            for v in engine.voices.iter_mut() { v.noise.level_gain = val; }
                        }
                    });
                });
                });
            });

            // ---------------------------------------------------------------------
            // COLUMN 3: ENVELOPES & CYBERPUNK FX CHAIN
            // ---------------------------------------------------------------------
            ui.group(|ui| {
                ui.set_width(330.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("⚡ MODULATION & FX").color(CyberTheme::NEON_PURPLE).strong());

                // Amp ADSR
                ui.group(|ui| {
                    ui.label(RichText::new("AMP ADSR ENVELOPE").color(CyberTheme::TEXT_PRIMARY).strong().size(11.0));
                    ui.horizontal(|ui| {
                        if draw_rotary_knob_log(ui, &mut engine.voices[0].amp_env.attack_s, 0.001..=5.0, "ATTACK", CyberTheme::NEON_PURPLE).changed() {
                            let val = engine.voices[0].amp_env.attack_s;
                            for v in engine.voices.iter_mut() { v.amp_env.attack_s = val; }
                        }
                        if draw_rotary_knob_log(ui, &mut engine.voices[0].amp_env.decay_s, 0.01..=5.0, "DECAY", CyberTheme::NEON_PURPLE).changed() {
                            let val = engine.voices[0].amp_env.decay_s;
                            for v in engine.voices.iter_mut() { v.amp_env.decay_s = val; }
                        }
                        if draw_rotary_knob(ui, &mut engine.voices[0].amp_env.sustain_level, 0.0..=1.0, "SUSTAIN", CyberTheme::NEON_PURPLE).changed() {
                            let val = engine.voices[0].amp_env.sustain_level;
                            for v in engine.voices.iter_mut() { v.amp_env.sustain_level = val; }
                        }
                        if draw_rotary_knob_log(ui, &mut engine.voices[0].amp_env.release_s, 0.01..=5.0, "RELEASE", CyberTheme::NEON_PURPLE).changed() {
                            let val = engine.voices[0].amp_env.release_s;
                            for v in engine.voices.iter_mut() { v.amp_env.release_s = val; }
                        }
                    });
                });

                // FX Section
                ui.group(|ui| {
                    ui.label(RichText::new("FX PROCESSORS").color(CyberTheme::NEON_CYAN).strong().size(11.0));

                    // Distortion
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut engine.fx.distortion.enabled, "");
                        ui.label(RichText::new("DIST:").size(10.0).color(CyberTheme::NEON_PINK));
                        let dist_modes = ["OVERDRIVE", "CRUSHER", "FIRE", "GRIND"];
                        let mut cur_dm = match engine.fx.distortion.mode {
                            DistortionMode::SoftOverdrive => 0,
                            DistortionMode::BitCrusher => 1,
                            DistortionMode::FireAmp => 2,
                            DistortionMode::GrindAmp => 3,
                        };
                        ComboBox::from_id_salt("dist_mode_combo")
                            .selected_text(dist_modes[cur_dm])
                            .width(82.0)
                            .show_ui(ui, |ui| {
                                for (i, &name) in dist_modes.iter().enumerate() {
                                    if ui.selectable_value(&mut cur_dm, i, name).clicked() {
                                        engine.fx.distortion.mode = DistortionMode::from_index(cur_dm);
                                    }
                                }
                            });
                        draw_rotary_knob(ui, &mut engine.fx.distortion.amount, 0.0..=1.0, "DRIVE", CyberTheme::NEON_PINK);
                    });

                    // Chorus
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut engine.fx.chorus.enabled, "");
                        ui.label(RichText::new("CHORUS:").size(10.0).color(CyberTheme::NEON_CYAN));
                        draw_rotary_knob(ui, &mut engine.fx.chorus.mix, 0.0..=1.0, "MIX", CyberTheme::NEON_CYAN);
                        draw_rotary_knob(ui, &mut engine.fx.chorus.rate_hz, 0.1..=8.0, "RATE", CyberTheme::NEON_CYAN);
                    });

                    // Delay
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut engine.fx.delay.enabled, "");
                        ui.label(RichText::new("P-P DELAY:").size(10.0).color(CyberTheme::NEON_AMBER));
                        draw_rotary_knob(ui, &mut engine.fx.delay.mix, 0.0..=1.0, "MIX", CyberTheme::NEON_AMBER);
                        draw_rotary_knob(ui, &mut engine.fx.delay.feedback, 0.0..=0.9, "FDBK", CyberTheme::NEON_AMBER);
                    });

                    // Reverb
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut engine.fx.reverb.enabled, "");
                        ui.label(RichText::new("REVERB:").size(10.0).color(CyberTheme::NEON_PURPLE));
                        draw_rotary_knob(ui, &mut engine.fx.reverb.room_size, 0.0..=1.0, "ROOM", CyberTheme::NEON_PURPLE);
                        draw_rotary_knob(ui, &mut engine.fx.reverb.mix, 0.0..=1.0, "MIX", CyberTheme::NEON_PURPLE);
                    });

                    // Trance Gate
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut engine.fx.gate.enabled, "");
                        ui.label(RichText::new("TRANCE GATE (CHOPPER)").size(10.0).color(CyberTheme::NEON_LIME));
                    });
                });
                });
            });
        });

        ui.add_space(6.0);

        // =========================================================================
        // 3. STEP SEQUENCER DRAWER (UP TO 64 STEPS WITH SLIDING BAR)
        // =========================================================================
        let active_step = engine.sequencer.current_step;
        draw_sequencer_drawer(ui, &mut engine.sequencer, &mut state.seq_ui_state, active_step);
    });
}
