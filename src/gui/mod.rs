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
    pos2, Button, Color32, ComboBox, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
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

    // Draw authentic cyberpunk chassis background textures (heatsinks, microchips, red stamps, telemetry)
    let full_rect = ui.available_rect_before_wrap();
    CyberTheme::draw_chassis_decorations(ui.painter(), full_rect);

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
                    .size(16.0),
            );
            ui.label(
                RichText::new("// CYBERPUNK SYNTH")
                    .color(CyberTheme::TEXT_MUTED)
                    .size(10.5),
            );

            ui.add_space(8.0);

            // Preset Selector with cyan borders
            if ui.button(RichText::new("◀").color(CyberTheme::NEON_CYAN).strong()).clicked() {
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
                .width(185.0)
                .show_ui(ui, |ui| {
                    for (i, p) in state.presets.iter().enumerate() {
                        let text = format!("[{}] {}", p.category, p.name);
                        let _is_sel = state.current_preset_idx == i;
                        if ui.selectable_value(&mut state.current_preset_idx, i, text).clicked() {
                            state.presets[i].apply_to_engine(engine);
                        }
                    }
                });

            if ui.button(RichText::new("▶").color(CyberTheme::NEON_CYAN).strong()).clicked() {
                state.current_preset_idx = (state.current_preset_idx + 1) % state.presets.len();
                state.presets[state.current_preset_idx].apply_to_engine(engine);
            }

            ui.add_space(14.0);

            // Live Telemetry Readout
            let active_voices = engine.active_voice_count.load(Ordering::Relaxed);
            ui.label(
                RichText::new(format!("VOICES: {:02}/16 | DSP: 64-BIT", active_voices))
                    .color(CyberTheme::NEON_LIME)
                    .strong()
                    .size(11.0),
            );

            ui.add_space(10.0);

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
            .stroke(Stroke::new(1.2_f32, if seq_active { CyberTheme::NEON_LIME } else { CyberTheme::BORDER_DARK }));

            if ui.add(seq_btn).clicked() {
                if engine.sequencer.mode == SeqMode::Off {
                    engine.sequencer.mode = SeqMode::InternalSynth;
                }
                engine.sequencer.running = !engine.sequencer.running;
                if engine.sequencer.running {
                    engine.sequencer.reset();
                }
            }

            ui.add_space(10.0);

            // Master Volume Analog Rotary Knob
            draw_rotary_knob(ui, &mut engine.master_gain, 0.0..=1.5, "MASTER", CyberTheme::NEON_AMBER);

            ui.add_space(14.0);

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

            // Top-right industrial telemetry block matching new_ui.jpeg
            ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("EATA:     0.50000006\nERRORS:         0.00\nEXTTA:      01003100\nHAGINZS:        9.56")
                        .color(Color32::from_rgb(72, 85, 112))
                        .size(8.0)
                        .monospace(),
                );
            });
        });

        ui.add_space(4.0);

        // Coordinates for modular analog patch cables
        let mut cable_jack_1: Option<nih_plug_egui::egui::Pos2> = None;
        let mut cable_jack_2: Option<nih_plug_egui::egui::Pos2> = None;
        let mut cable_jack_3: Option<nih_plug_egui::egui::Pos2> = None;
        let mut cable_jack_4: Option<nih_plug_egui::egui::Pos2> = None;

        let mut cable_dest_1: Option<nih_plug_egui::egui::Pos2> = None;
        let mut cable_dest_2: Option<nih_plug_egui::egui::Pos2> = None;
        let mut cable_dest_3: Option<nih_plug_egui::egui::Pos2> = None;
        let mut cable_dest_4: Option<nih_plug_egui::egui::Pos2> = None;

        // =========================================================================
        // 2. MAIN DSP SYNTH PANELS (3 Columns evenly filling width)
        // =========================================================================
        let col_w = 394.0;
        ui.horizontal(|ui| {
            // ---------------------------------------------------------------------
            // COLUMN 1: OSCILLATOR 1 & OSCILLATOR 2 (Eurorack Panel)
            // ---------------------------------------------------------------------
            let osc_resp = ui.group(|ui| {
                ui.set_width(col_w);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚡ OSCILLATORS").color(CyberTheme::NEON_CYAN).strong().size(12.0));
                        ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                            ui.label(RichText::new("05M/ASEC//12AS05T").color(CyberTheme::TEXT_DIM).monospace().size(8.0));
                        });
                    });

                    // --- OSC 1 ---
                    let _osc1_resp = ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.voices[0].osc1.enabled, "");
                            ui.label(RichText::new("OSC 1:").color(CyberTheme::NEON_CYAN).strong());

                            // Wavetable name selector
                            let table_names = engine.bank.names();
                            let mut cur_table = engine.voices[0].osc1_table_idx;
                            ComboBox::from_id_salt("osc1_table_combo")
                                .selected_text(table_names.get(cur_table).cloned().unwrap_or_default())
                                .width(120.0)
                                .show_ui(ui, |ui| {
                                    for (i, name) in table_names.iter().enumerate() {
                                        if ui.selectable_value(&mut cur_table, i, name).clicked() {
                                            for v in engine.voices.iter_mut() {
                                                v.osc1_table_idx = cur_table;
                                            }
                                        }
                                    }
                                });

                            ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                                ui.label(RichText::new("C001/0002/ // 165057.KE").color(CyberTheme::TEXT_DIM).monospace().size(7.5));
                            });
                        });

                        // 3D GPU Holographic Wavetable Preview
                        let cur_wt1 = engine.bank.get_table(engine.voices[0].osc1_table_idx);
                        let cur_pos1 = engine.voices[0].osc1.position;
                        draw_wavetable_preview(ui, cur_wt1, cur_pos1, CyberTheme::NEON_CYAN, col_w - 24.0, 74.0);

                        // Osc 1 Rotary Knobs (Row 1: POS, TUNE, UNISON, DETUNE + Jack Sockets)
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
                            let det_resp = draw_rotary_knob(ui, &mut engine.voices[0].osc1.detune, 0.0..=1.0, "DETUNE", CyberTheme::NEON_CYAN);
                            if det_resp.changed() {
                                let val = engine.voices[0].osc1.detune;
                                for v in engine.voices.iter_mut() { v.osc1.detune = val; }
                            }

                            // Modular Patch Sockets 1 & 2 right next to DETUNE
                            let j1_pos = pos2(det_resp.rect.max.x + 12.0, det_resp.rect.min.y + 10.0);
                            let j2_pos = pos2(det_resp.rect.max.x + 12.0, det_resp.rect.min.y + 26.0);
                            cable_jack_1 = Some(j1_pos);
                            cable_jack_2 = Some(j2_pos);
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

                    ui.add_space(2.0);

                    // --- OSC 2 ---
                    let _osc2_resp = ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.voices[0].osc2.enabled, "");
                            ui.label(RichText::new("OSC 2:").color(CyberTheme::NEON_PINK).strong());

                            let table_names = engine.bank.names();
                            let mut cur_table = engine.voices[0].osc2_table_idx;
                            ComboBox::from_id_salt("osc2_table_combo")
                                .selected_text(table_names.get(cur_table).cloned().unwrap_or_default())
                                .width(120.0)
                                .show_ui(ui, |ui| {
                                    for (i, name) in table_names.iter().enumerate() {
                                        if ui.selectable_value(&mut cur_table, i, name).clicked() {
                                            for v in engine.voices.iter_mut() {
                                                v.osc2_table_idx = cur_table;
                                            }
                                        }
                                    }
                                });

                            ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                                ui.label(RichText::new("CQ916139 COG 26.918").color(CyberTheme::TEXT_DIM).monospace().size(7.5));
                            });
                        });

                        let cur_wt2 = engine.bank.get_table(engine.voices[0].osc2_table_idx);
                        let cur_pos2 = engine.voices[0].osc2.position;
                        draw_wavetable_preview(ui, cur_wt2, cur_pos2, CyberTheme::NEON_PINK, col_w - 24.0, 74.0);

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

                        // Osc 2 Rotary Knobs (Row 2: LEVEL, SPREAD, FORMANT, BEND + Jack Sockets)
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
                            let bend_resp = draw_rotary_knob(ui, &mut engine.voices[0].osc2.bend, -1.0..=1.0, "BEND", CyberTheme::NEON_PINK);
                            if bend_resp.changed() {
                                let val = engine.voices[0].osc2.bend;
                                for v in engine.voices.iter_mut() { v.osc2.bend = val; }
                            }

                            // Modular Patch Sockets 3 & 4 right next to BEND
                            let j3_pos = pos2(bend_resp.rect.max.x + 12.0, bend_resp.rect.min.y + 10.0);
                            let j4_pos = pos2(bend_resp.rect.max.x + 12.0, bend_resp.rect.min.y + 26.0);
                            cable_jack_3 = Some(j3_pos);
                            cable_jack_4 = Some(j4_pos);
                        });
                    });
                });
            });

            CyberTheme::draw_panel_screws(ui.painter(), osc_resp.response.rect);

            // ---------------------------------------------------------------------
            // COLUMN 2: MULTIMODE FILTER, SUB/NOISE, AND LIVE OSCILLOSCOPE
            // ---------------------------------------------------------------------
            let col2_resp = ui.group(|ui| {
                ui.set_width(col_w);
                ui.vertical(|ui| {
                    // Live Oscilloscope Display
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚡ NEON OSCILLOSCOPE").color(CyberTheme::NEON_CYAN).strong().size(12.0));
                        ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                            ui.label(RichText::new("DEP VECTOR // 64-BIT").color(CyberTheme::TEXT_DIM).monospace().size(8.0));
                        });
                    });

                    let samples: Vec<f32> = {
                        let scope = engine.scope_data.lock();
                        scope.buffer.to_vec()
                    };
                    draw_oscilloscope(ui, &samples, col_w - 24.0, 74.0);

                    ui.add_space(3.0);

                    // Multimode Filter Sub-panel with Military Green / Teal Styling
                    let flt_resp = ui.group(|ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = CyberTheme::BG_FILTER;

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
                                .selected_text(RichText::new(f_types[cur_type]).color(CyberTheme::NEON_AMBER).strong())
                                .width(90.0)
                                .show_ui(ui, |ui| {
                                    for (i, &name) in f_types.iter().enumerate() {
                                        if ui.selectable_value(&mut cur_type, i, name).clicked() {
                                            let ft = FilterType::from_index(cur_type);
                                            for v in engine.voices.iter_mut() { v.filter.filter_type = ft; }
                                        }
                                    }
                                });

                            ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                                ui.label(RichText::new("DULT1SWAT BSSTA RE 1.CIRA").color(CyberTheme::TEXT_DIM).monospace().size(7.5));
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

                    // Left-side Filter Jack Inputs
                    cable_dest_1 = Some(pos2(flt_resp.response.rect.min.x + 4.0, flt_resp.response.rect.center().y - 12.0));
                    cable_dest_2 = Some(pos2(flt_resp.response.rect.min.x + 4.0, flt_resp.response.rect.center().y + 4.0));
                    cable_dest_3 = Some(pos2(flt_resp.response.rect.min.x + 4.0, flt_resp.response.rect.center().y + 20.0));

                    ui.add_space(3.0);

                    // Sub Oscillator & Noise Generator Sub-panel
                    let sub_resp = ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.voices[0].sub.enabled, "");
                            ui.label(RichText::new("SUB OSC:").color(CyberTheme::NEON_LIME).strong().size(11.0));
                            if draw_rotary_knob(ui, &mut engine.voices[0].sub.level_gain, 0.0..=1.0, "SUB LVL", CyberTheme::NEON_LIME).changed() {
                                let val = engine.voices[0].sub.level_gain;
                                for v in engine.voices.iter_mut() { v.sub.level_gain = val; }
                            }

                            ui.add_space(20.0);

                            ui.checkbox(&mut engine.voices[0].noise.enabled, "");
                            ui.label(RichText::new("NOISE:").color(CyberTheme::NEON_CYAN).strong().size(11.0));
                            if draw_rotary_knob(ui, &mut engine.voices[0].noise.level_gain, 0.0..=1.0, "NOISE LVL", CyberTheme::NEON_CYAN).changed() {
                                let val = engine.voices[0].noise.level_gain;
                                for v in engine.voices.iter_mut() { v.noise.level_gain = val; }
                            }
                        });
                    });

                    // Sub Jack Input
                    cable_dest_4 = Some(pos2(sub_resp.response.rect.min.x + 4.0, sub_resp.response.rect.center().y));
                });
            });

            CyberTheme::draw_panel_screws(ui.painter(), col2_resp.response.rect);

            // ---------------------------------------------------------------------
            // COLUMN 3: ENVELOPES & CYBERPUNK FX CHAIN WITH REAL-TIME MONITORS
            // ---------------------------------------------------------------------
            let col3_resp = ui.group(|ui| {
                ui.set_width(col_w);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚡ MODULATION & FX").color(CyberTheme::NEON_PURPLE).strong().size(12.0));
                        ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                            ui.label(RichText::new("CRESCENDATION FORSING").color(CyberTheme::TEXT_DIM).monospace().size(8.0));
                        });
                    });

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

                    // FX Section with Mini Real-Time Response Monitors
                    ui.group(|ui| {
                        ui.label(RichText::new("FX PROCESSORS").color(CyberTheme::NEON_CYAN).strong().size(11.0));

                        // Distortion
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.fx.distortion.enabled, "");
                            ui.label(RichText::new("DIST:").size(10.5).color(CyberTheme::NEON_PINK).strong());
                            let dist_modes = ["OVERDRIVE", "CRUSHER", "FIRE", "GRIND"];
                            let mut cur_dm = match engine.fx.distortion.mode {
                                DistortionMode::SoftOverdrive => 0,
                                DistortionMode::BitCrusher => 1,
                                DistortionMode::FireAmp => 2,
                                DistortionMode::GrindAmp => 3,
                            };
                            ComboBox::from_id_salt("dist_mode_combo")
                                .selected_text(dist_modes[cur_dm])
                                .width(88.0)
                                .show_ui(ui, |ui| {
                                    for (i, &name) in dist_modes.iter().enumerate() {
                                        if ui.selectable_value(&mut cur_dm, i, name).clicked() {
                                            engine.fx.distortion.mode = DistortionMode::from_index(cur_dm);
                                        }
                                    }
                                });

                            // Horizontal cyan drive meter line matching new_ui.jpeg
                            let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(42.0, 6.0), Sense::hover());
                            ui.painter().rect_filled(bar_rect, 1.0, Color32::from_rgb(10, 16, 24));
                            let fill_w = bar_rect.width() * engine.fx.distortion.amount.clamp(0.0, 1.0);
                            ui.painter().rect_filled(Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_rect.height())), 1.0, CyberTheme::NEON_CYAN);

                            draw_rotary_knob(ui, &mut engine.fx.distortion.amount, 0.0..=1.0, "DRIVE", CyberTheme::NEON_PINK);
                            draw_mini_fx_monitor(ui, 0, CyberTheme::NEON_PINK, engine.fx.distortion.enabled);
                        });

                        // Chorus
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.fx.chorus.enabled, "");
                            ui.label(RichText::new("CHORUS:").size(10.5).color(CyberTheme::NEON_CYAN).strong());
                            draw_rotary_knob(ui, &mut engine.fx.chorus.mix, 0.0..=1.0, "MIX", CyberTheme::NEON_CYAN);
                            draw_rotary_knob(ui, &mut engine.fx.chorus.rate_hz, 0.1..=8.0, "RATE", CyberTheme::NEON_CYAN);
                            draw_mini_fx_monitor(ui, 1, CyberTheme::NEON_CYAN, engine.fx.chorus.enabled);
                        });

                        // Delay
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.fx.delay.enabled, "");
                            ui.label(RichText::new("P-P DELAY:").size(10.5).color(CyberTheme::NEON_AMBER).strong());
                            draw_rotary_knob(ui, &mut engine.fx.delay.mix, 0.0..=1.0, "MIX", CyberTheme::NEON_AMBER);
                            draw_rotary_knob(ui, &mut engine.fx.delay.feedback, 0.0..=0.9, "FDBK", CyberTheme::NEON_AMBER);
                            draw_mini_fx_monitor(ui, 2, CyberTheme::NEON_AMBER, engine.fx.delay.enabled);
                        });

                        // Reverb
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.fx.reverb.enabled, "");
                            ui.label(RichText::new("REVERB:").size(10.5).color(CyberTheme::NEON_PURPLE).strong());
                            draw_rotary_knob(ui, &mut engine.fx.reverb.room_size, 0.0..=1.0, "ROOM", CyberTheme::NEON_PURPLE);
                            draw_rotary_knob(ui, &mut engine.fx.reverb.mix, 0.0..=1.0, "MIX", CyberTheme::NEON_PURPLE);
                            draw_mini_fx_monitor(ui, 3, CyberTheme::NEON_PURPLE, engine.fx.reverb.enabled);
                        });

                        // Trance Gate
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.fx.gate.enabled, "");
                            ui.label(RichText::new("TRANCE GATE (CHOPPER)").size(10.5).color(CyberTheme::NEON_LIME).strong());
                        });
                    });
                });
            });

            CyberTheme::draw_panel_screws(ui.painter(), col3_resp.response.rect);
        });

        // =========================================================================
        // 2B. MODULAR ANALOG PATCH CABLES OVERLAY (4 High-Realism Physical Cables)
        // =========================================================================
        let painter = ui.painter();
        if let (Some(j1), Some(d1)) = (cable_jack_1, cable_dest_1) {
            CyberTheme::draw_patch_socket(painter, j1, CyberTheme::NEON_CYAN);
            CyberTheme::draw_patch_socket(painter, d1, CyberTheme::NEON_CYAN);
            CyberTheme::draw_patch_cable(painter, j1, d1, CyberTheme::NEON_CYAN);
        }

        if let (Some(j2), Some(d2)) = (cable_jack_2, cable_dest_2) {
            CyberTheme::draw_patch_socket(painter, j2, CyberTheme::NEON_CYAN);
            CyberTheme::draw_patch_socket(painter, d2, CyberTheme::NEON_CYAN);
            CyberTheme::draw_patch_cable(painter, j2, d2, CyberTheme::NEON_CYAN);
        }

        if let (Some(j3), Some(d3)) = (cable_jack_3, cable_dest_3) {
            CyberTheme::draw_patch_socket(painter, j3, CyberTheme::NEON_PINK);
            CyberTheme::draw_patch_socket(painter, d3, CyberTheme::NEON_PINK);
            CyberTheme::draw_patch_cable(painter, j3, d3, CyberTheme::NEON_PINK);
        }

        if let (Some(j4), Some(d4)) = (cable_jack_4, cable_dest_4) {
            CyberTheme::draw_patch_socket(painter, j4, CyberTheme::NEON_PINK);
            CyberTheme::draw_patch_socket(painter, d4, CyberTheme::NEON_PINK);
            CyberTheme::draw_patch_cable(painter, j4, d4, CyberTheme::NEON_PINK);
        }

        ui.add_space(3.0);

        // =========================================================================
        // 3. STEP SEQUENCER DRAWER (64 STEPS WITH ROTARY DIAL & MINI OVERVIEW RADAR)
        // =========================================================================
        let active_step = engine.sequencer.current_step;
        draw_sequencer_drawer(ui, &mut engine.sequencer, &mut state.seq_ui_state, active_step);
    });
}

/// Mini real-time oscilloscope curve monitor for FX processors.
fn draw_mini_fx_monitor(ui: &mut Ui, mode: u8, color: Color32, active: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(48.0, 22.0), Sense::hover());
    let painter = ui.painter();

    // Dark recessed CRT monitor background
    painter.rect_filled(rect, 2.0, Color32::from_rgb(7, 9, 14));
    let border_col = if active {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 130)
    } else {
        CyberTheme::BORDER_DARK
    };
    painter.rect_stroke(rect, 2.0, Stroke::new(1.0_f32, border_col), StrokeKind::Inside);

    if !active {
        // Flat inactive bypass line
        let mid_y = rect.center().y;
        painter.line_segment(
            [pos2(rect.min.x + 3.0, mid_y), pos2(rect.max.x - 3.0, mid_y)],
            Stroke::new(1.0_f32, Color32::from_rgb(35, 40, 56)),
        );
        return;
    }

    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f32())
        .unwrap_or(0.0);

    let num_pts = 16;
    let mut pts = Vec::with_capacity(num_pts);

    for i in 0..num_pts {
        let frac = i as f32 / (num_pts - 1) as f32; // 0..1
        let x = rect.min.x + 3.0 + frac * (rect.width() - 6.0);
        let y_norm = match mode {
            0 => {
                // Distortion S-curve transfer function
                let x_sym = (frac - 0.5) * 2.0;
                let y_val = (x_sym * 2.4).tanh();
                0.5 - y_val * 0.36
            }
            1 => {
                // Chorus dual-phase sine modulation wave
                let y_val = (frac * 3.0 * std::f32::consts::PI + t * 4.0).sin();
                0.5 - y_val * 0.35
            }
            2 => {
                // Delay stereo ping-pong bounce taps
                let tap = ((frac * 5.0 + t * 2.0).fract() - 0.5).abs();
                let decay = 1.0 - frac * 0.6;
                0.5 - (tap * 2.0 - 0.5) * decay * 0.38
            }
            _ => {
                // Reverb exponential diffusion decay envelope
                let decay = (-frac * 3.2).exp();
                let noise = ((frac * 24.0 + t * 3.0).sin() * 0.12) * decay;
                0.78 - (decay * 0.60 + noise)
            }
        };
        let y = rect.min.y + y_norm * rect.height();
        pts.push(pos2(x, y));
    }

    for i in 0..pts.len().saturating_sub(1) {
        painter.line_segment([pts[i], pts[i + 1]], Stroke::new(1.3_f32, color));
    }
}
