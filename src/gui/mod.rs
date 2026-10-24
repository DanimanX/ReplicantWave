pub mod knob;
pub mod oscilloscope;
pub mod sequencer_view;
pub mod theme;
pub mod wavetable_view;

use crate::audio::filter::FilterType;
use crate::audio::fx::distortion::DistortionMode;
use crate::audio::SynthEngine;
use crate::presets::factory::get_factory_presets;
use crate::presets::SynthPreset;
use knob::{draw_rotary_knob, draw_rotary_knob_log, draw_rotary_knob_usize};
use oscilloscope::draw_oscilloscope;
use sequencer_view::{draw_sequencer_drawer, SequencerUiState};
use theme::CyberTheme;
use wavetable_view::draw_wavetable_preview;

use nih_plug_egui::egui::{
    pos2, Button, Color32, ComboBox, Pos2, RichText, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use parking_lot::Mutex;
use std::f32::consts::PI;
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
    // Request continuous GPU repaint for 60 FPS smooth animations
    ui.ctx().request_repaint();

    ui.style_mut().visuals.panel_fill = CyberTheme::BG_VOID;
    ui.style_mut().visuals.window_fill = CyberTheme::BG_VOID;

    // Draw background chassis textures (cooling vents, IC 808 chip, telemetry codes, red stamp)
    let full_rect = ui.available_rect_before_wrap();
    CyberTheme::draw_chassis_decorations(ui.painter(), full_rect);

    let mut engine_guard = state.engine.lock();
    let engine = &mut *engine_guard;

    ui.vertical(|ui| {
        // =========================================================================
        // 1. TOP HEADER: LOGO, PRESETS, TELEMETRY, MASTER, DONATE (Matches new_ui.jpeg)
        // =========================================================================
        ui.horizontal(|ui| {
            // Cyberpunk Logo
            ui.label(
                RichText::new("⚡ REPLICANTWAVE ⚡")
                    .color(CyberTheme::NEON_CYAN)
                    .strong()
                    .size(15.5),
            );
            ui.label(
                RichText::new("// CYBERPUNK SYNTH")
                    .color(CyberTheme::TEXT_MUTED)
                    .size(10.0),
            );

            ui.add_space(8.0);

            // Preset Selector with Cyan Arrows and Amber Name Box
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

            if ui.button(RichText::new("▶").color(CyberTheme::NEON_CYAN).strong()).clicked() {
                state.current_preset_idx = (state.current_preset_idx + 1) % state.presets.len();
                state.presets[state.current_preset_idx].apply_to_engine(engine);
            }

            ui.add_space(12.0);

            // Live Telemetry Readout: VOICES: 12/16 | DSP: 64-BIT (Green monospace)
            let active_voices = engine.active_voice_count.load(Ordering::Relaxed);
            ui.label(
                RichText::new(format!("VOICES: {:02}/16 | DSP: 64-BIT", active_voices))
                    .color(CyberTheme::NEON_LIME)
                    .strong()
                    .size(11.0),
            );

            ui.add_space(10.0);

            // Master Volume Analog Rotary Knob
            draw_rotary_knob(ui, &mut engine.master_gain, 0.0..=1.5, "MASTER", CyberTheme::NEON_AMBER);

            ui.add_space(10.0);

            // Donation Button: ☕ DONATE / BUY ME A COFFEE (Golden Amber pill button)
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

            // Top-right industrial telemetry readout stamp
            ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("EATA:    0.5000006\nERROS:        0.00\nEXTFA:    01003100\nHAGINZS:      9.56")
                        .color(Color32::from_rgb(72, 85, 112))
                        .size(8.0)
                        .monospace(),
                );
            });
        });

        ui.add_space(4.0);
        ui.separator();

        // Modular patch cable connector coordinates
        let mut osc1_detune_jack: Option<Pos2> = None;
        let mut osc1_bend_jack: Option<Pos2> = None;
        let mut osc2_bend_jack: Option<Pos2> = None;

        let mut cable_dest_1: Option<Pos2> = None;
        let mut cable_dest_2: Option<Pos2> = None;
        let mut cable_dest_3: Option<Pos2> = None;

        // =========================================================================
        // 2. MAIN DSP SYNTH PANELS (3 Columns matching new_ui.jpeg)
        // =========================================================================
        let col_w = 394.0;
        let col_mid_w = 426.0;

        ui.horizontal(|ui| {
            // ---------------------------------------------------------------------
            // COLUMN 1: OSCILLATORS (OSC 1 & OSC 2)
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
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.voices[0].osc1.enabled, "");
                            ui.label(RichText::new("OSC 1:").color(CyberTheme::NEON_CYAN).strong());

                            // Wavetable Selector
                            let table_names = engine.bank.names();
                            let mut cur_table = engine.voices[0].osc1_table_idx;
                            ComboBox::from_id_salt("osc1_table_combo")
                                .selected_text(table_names.get(cur_table).cloned().unwrap_or_default())
                                .width(125.0)
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

                        // 3D GPU Holographic Wavetable Preview (Cyan)
                        let cur_wt1 = engine.bank.get_table(engine.voices[0].osc1_table_idx);
                        let cur_pos1 = engine.voices[0].osc1.position;
                        draw_wavetable_preview(ui, cur_wt1, cur_pos1, CyberTheme::NEON_CYAN, col_w - 24.0, 74.0);

                        // Row 1: POS, TUNE, UNISON, DETUNE + Jack
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
                            osc1_detune_jack = Some(pos2(det_resp.rect.max.x + 8.0, det_resp.rect.center().y));
                        });

                        // Row 2: LEVEL, SPREAD, FORMANT, BEND + Jack
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
                            let bend_resp = draw_rotary_knob(ui, &mut engine.voices[0].osc1.bend, -1.0..=1.0, "BEND", CyberTheme::NEON_CYAN);
                            if bend_resp.changed() {
                                let val = engine.voices[0].osc1.bend;
                                for v in engine.voices.iter_mut() { v.osc1.bend = val; }
                            }
                            osc1_bend_jack = Some(pos2(bend_resp.rect.max.x + 8.0, bend_resp.rect.center().y));
                        });
                    });

                    ui.add_space(2.0);

                    // --- OSC 2 ---
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.voices[0].osc2.enabled, "");
                            ui.label(RichText::new("OSC 2:").color(CyberTheme::NEON_PINK).strong());

                            // Wavetable Selector
                            let table_names = engine.bank.names();
                            let mut cur_table = engine.voices[0].osc2_table_idx;
                            ComboBox::from_id_salt("osc2_table_combo")
                                .selected_text(table_names.get(cur_table).cloned().unwrap_or_default())
                                .width(125.0)
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

                        // 3D GPU Holographic Wavetable Preview (Hot Pink / Magenta)
                        let cur_wt2 = engine.bank.get_table(engine.voices[0].osc2_table_idx);
                        let cur_pos2 = engine.voices[0].osc2.position;
                        draw_wavetable_preview(ui, cur_wt2, cur_pos2, CyberTheme::NEON_PINK, col_w - 24.0, 74.0);

                        // Row 1: POS, TUNE, UNISON, DETUNE
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

                        // Row 2: LEVEL, SPREAD, FORMANT, BEND + Jack
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
                            let bend_resp2 = draw_rotary_knob(ui, &mut engine.voices[0].osc2.bend, -1.0..=1.0, "BEND", CyberTheme::NEON_PINK);
                            if bend_resp2.changed() {
                                let val = engine.voices[0].osc2.bend;
                                for v in engine.voices.iter_mut() { v.osc2.bend = val; }
                            }
                            osc2_bend_jack = Some(pos2(bend_resp2.rect.max.x + 8.0, bend_resp2.rect.center().y));
                        });
                    });
                });
            });

            CyberTheme::draw_eurorack_screws(ui.painter(), osc_resp.response.rect);

            ui.add_space(6.0);

            // ---------------------------------------------------------------------
            // COLUMN 2: NEON OSCILLOSCOPE, MULTIMODE FILTER, SUB & NOISE
            // ---------------------------------------------------------------------
            let center_resp = ui.group(|ui| {
                ui.set_width(col_mid_w);
                ui.vertical(|ui| {
                    // Title: ⚡ NEON OSCILLOSCOPE
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚡ NEON OSCILLOSCOPE").color(CyberTheme::NEON_CYAN).strong().size(12.0));
                        ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                            ui.label(RichText::new("DEP VECTOR // 64-BIT").color(CyberTheme::TEXT_DIM).monospace().size(8.0));
                        });
                    });

                    // Live Oscilloscope Visualizer
                    let samples: Vec<f32> = {
                        let scope = engine.scope_data.lock();
                        scope.buffer.to_vec()
                    };
                    draw_oscilloscope(ui, &samples, col_mid_w - 24.0, 78.0);

                    ui.add_space(4.0);

                    // Multimode Filter Sub-panel
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
                                .width(88.0)
                                .show_ui(ui, |ui| {
                                    for (i, &name) in f_types.iter().enumerate() {
                                        if ui.selectable_value(&mut cur_type, i, name).clicked() {
                                            let ft = FilterType::from_index(cur_type);
                                            for v in engine.voices.iter_mut() { v.filter.filter_type = ft; }
                                        }
                                    }
                                });

                            ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                                ui.label(RichText::new("DULT1SXRT BS8TR RE 1:CIRA\nDONTOPEL COLCIP8 DATUHM").color(Color32::from_rgb(0, 180, 120)).monospace().size(6.8));
                            });
                        });

                        // Filter Knobs: CUTOFF, RESO, DRIVE, ENV MOD
                        ui.horizontal(|ui| {
                            let cut_resp = draw_rotary_knob_log(ui, &mut engine.voices[0].filter.cutoff_hz, 20.0..=18000.0, "CUTOFF", CyberTheme::NEON_AMBER);
                            if cut_resp.changed() {
                                let val = engine.voices[0].filter.cutoff_hz;
                                for v in engine.voices.iter_mut() { v.filter.cutoff_hz = val; }
                            }
                            cable_dest_1 = Some(pos2(cut_resp.rect.min.x - 6.0, cut_resp.rect.center().y));

                            if draw_rotary_knob(ui, &mut engine.voices[0].filter.resonance, 0.0..=1.0, "RESO", CyberTheme::NEON_AMBER).changed() {
                                let val = engine.voices[0].filter.resonance;
                                for v in engine.voices.iter_mut() { v.filter.resonance = val; }
                            }
                            if draw_rotary_knob(ui, &mut engine.voices[0].filter.drive, 1.0..=4.0, "DRIVE", CyberTheme::NEON_CYAN).changed() {
                                let val = engine.voices[0].filter.drive;
                                for v in engine.voices.iter_mut() { v.filter.drive = val; }
                            }
                            let env_resp = draw_rotary_knob(ui, &mut engine.voices[0].filter.env_amount, -1.0..=1.0, "ENV MOD", CyberTheme::NEON_AMBER);
                            if env_resp.changed() {
                                let val = engine.voices[0].filter.env_amount;
                                for v in engine.voices.iter_mut() { v.filter.env_amount = val; }
                            }
                            cable_dest_2 = Some(pos2(env_resp.rect.max.x + 8.0, env_resp.rect.center().y));
                        });

                        // Route Checkboxes: ROUTE: ☑ OSC1  ☑ OSC2  ☑ SUB
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

                    // Sub Oscillator & Noise Sub-panel
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.voices[0].sub.enabled, "");
                            ui.label(RichText::new("SUB OSC:").color(CyberTheme::NEON_LIME).strong().size(11.0));
                            let sub_resp = draw_rotary_knob(ui, &mut engine.voices[0].sub.level_gain, 0.0..=1.0, "SUB LVL", CyberTheme::NEON_LIME);
                            if sub_resp.changed() {
                                let val = engine.voices[0].sub.level_gain;
                                for v in engine.voices.iter_mut() { v.sub.level_gain = val; }
                            }
                            cable_dest_3 = Some(pos2(sub_resp.rect.min.x - 6.0, sub_resp.rect.center().y));

                            ui.add_space(14.0);

                            ui.checkbox(&mut engine.voices[0].noise.enabled, "");
                            ui.label(RichText::new("NOISE:").color(CyberTheme::TEXT_MUTED).strong().size(11.0));
                            if draw_rotary_knob(ui, &mut engine.voices[0].noise.level_gain, 0.0..=1.0, "NOISE LVL", CyberTheme::NEON_CYAN).changed() {
                                let val = engine.voices[0].noise.level_gain;
                                for v in engine.voices.iter_mut() { v.noise.level_gain = val; }
                            }
                        });

                        ui.label(RichText::new("00000000800000056").color(Color32::from_rgb(180, 150, 40)).monospace().size(7.0));
                    });
                });
            });

            CyberTheme::draw_eurorack_screws(ui.painter(), center_resp.response.rect);

            ui.add_space(6.0);

            // ---------------------------------------------------------------------
            // COLUMN 3: MODULATION & FX (ADSR Envelope & FX Processors)
            // ---------------------------------------------------------------------
            let fx_resp = ui.group(|ui| {
                ui.set_width(col_w);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚡ MODULATION & FX").color(CyberTheme::NEON_PURPLE).strong().size(12.0));
                        ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                            ui.label(RichText::new("CRFSCIOENITATION FORSING\n6EHOHE1.E9 TIL").color(Color32::from_rgb(220, 50, 70)).monospace().size(6.8));
                        });
                    });

                    // Amp ADSR Envelope
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

                    ui.add_space(2.0);

                    // FX Processors
                    ui.group(|ui| {
                        ui.label(RichText::new("FX PROCESSORS").color(CyberTheme::NEON_CYAN).strong().size(11.0));

                        // 1. Distortion (Magenta)
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
                                .width(94.0)
                                .show_ui(ui, |ui| {
                                    for (i, &name) in dist_modes.iter().enumerate() {
                                        if ui.selectable_value(&mut cur_dm, i, name).clicked() {
                                            engine.fx.distortion.mode = DistortionMode::from_index(cur_dm);
                                        }
                                    }
                                });

                            draw_rotary_knob(ui, &mut engine.fx.distortion.amount, 0.0..=1.0, "DRIVE", CyberTheme::NEON_PINK);
                            draw_mini_fx_monitor(ui, 0, CyberTheme::NEON_PINK, engine.fx.distortion.enabled);
                        });

                        // 2. Chorus (Cyan)
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.fx.chorus.enabled, "");
                            ui.label(RichText::new("CHORUS:").size(10.5).color(CyberTheme::NEON_CYAN).strong());
                            draw_rotary_knob(ui, &mut engine.fx.chorus.mix, 0.0..=1.0, "MIX", CyberTheme::NEON_CYAN);
                            draw_rotary_knob(ui, &mut engine.fx.chorus.rate_hz, 0.1..=8.0, "RATE", CyberTheme::NEON_CYAN);
                            draw_mini_fx_monitor(ui, 1, CyberTheme::NEON_CYAN, engine.fx.chorus.enabled);
                        });

                        // 3. Ping-Pong Delay (Amber)
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.fx.delay.enabled, "");
                            ui.label(RichText::new("P-P DELAY:").size(10.5).color(CyberTheme::NEON_AMBER).strong());
                            draw_rotary_knob(ui, &mut engine.fx.delay.mix, 0.0..=1.0, "MIX", CyberTheme::NEON_AMBER);
                            draw_rotary_knob(ui, &mut engine.fx.delay.feedback, 0.0..=0.9, "FDBK", CyberTheme::NEON_AMBER);
                            draw_mini_fx_monitor(ui, 2, CyberTheme::NEON_AMBER, engine.fx.delay.enabled);
                        });

                        // 4. Reverb (Purple)
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.fx.reverb.enabled, "");
                            ui.label(RichText::new("REVERB:").size(10.5).color(CyberTheme::NEON_PURPLE).strong());
                            draw_rotary_knob(ui, &mut engine.fx.reverb.room_size, 0.0..=1.0, "ROOM", CyberTheme::NEON_PURPLE);
                            draw_rotary_knob(ui, &mut engine.fx.reverb.mix, 0.0..=1.0, "MIX", CyberTheme::NEON_PURPLE);
                            draw_mini_fx_monitor(ui, 3, CyberTheme::NEON_PURPLE, engine.fx.reverb.enabled);
                        });

                        // 5. Trance Gate (Chopper)
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut engine.fx.gate.enabled, "");
                            ui.label(RichText::new("TRANCE GATE (CHOPPER)").size(10.5).color(CyberTheme::NEON_LIME));
                        });
                    });
                });
            });

            CyberTheme::draw_eurorack_screws(ui.painter(), fx_resp.response.rect);
        });

        // =========================================================================
        // 2B. MODULAR ANALOG PATCH CABLES OVERLAY
        // =========================================================================
        let painter = ui.painter();
        if let (Some(j1), Some(d1)) = (osc1_detune_jack, cable_dest_1) {
            CyberTheme::draw_patch_socket(painter, j1, CyberTheme::NEON_CYAN);
            CyberTheme::draw_patch_socket(painter, d1, CyberTheme::NEON_CYAN);
            CyberTheme::draw_patch_cable(painter, j1, d1, CyberTheme::NEON_CYAN);
        }

        if let (Some(j2), Some(d2)) = (osc1_bend_jack, cable_dest_2) {
            CyberTheme::draw_patch_socket(painter, j2, CyberTheme::NEON_CYAN);
            CyberTheme::draw_patch_socket(painter, d2, CyberTheme::NEON_CYAN);
            CyberTheme::draw_patch_cable(painter, j2, d2, CyberTheme::NEON_CYAN);
        }

        if let (Some(j3), Some(d3)) = (osc2_bend_jack, cable_dest_3) {
            CyberTheme::draw_patch_socket(painter, j3, CyberTheme::NEON_PINK);
            CyberTheme::draw_patch_socket(painter, d3, CyberTheme::NEON_PINK);
            CyberTheme::draw_patch_cable(painter, j3, d3, CyberTheme::NEON_PINK);
        }

        ui.add_space(4.0);

        // =========================================================================
        // 3. STEP SEQUENCER DRAWER (64 STEPS WITH ROTARY DIAL & MINI RADAR OVERVIEW)
        // =========================================================================
        let active_step = engine.sequencer.current_step;
        draw_sequencer_drawer(ui, &mut engine.sequencer, &mut engine.voices, &mut state.seq_ui_state, active_step);
    });
}

/// Mini animated oscilloscope curve monitor for FX processors.
fn draw_mini_fx_monitor(ui: &mut Ui, mode: u8, color: Color32, active: bool) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(50.0, 22.0), Sense::hover());
    let painter = ui.painter();

    // Dark recessed CRT monitor background
    painter.rect_filled(rect, 2.0, Color32::from_rgb(7, 9, 14));
    let border_col = if active {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 140)
    } else {
        CyberTheme::BORDER_DARK
    };
    painter.rect_stroke(rect, 2.0, Stroke::new(1.0_f32, border_col), StrokeKind::Inside);

    if !active {
        // Flat inactive bypass line
        let mid_y = rect.center().y;
        painter.line_segment(
            [pos2(rect.min.x + 3.0, mid_y), pos2(rect.max.x - 3.0, mid_y)],
            Stroke::new(1.0_f32, Color32::from_rgb(32, 38, 52)),
        );
        return;
    }

    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f32())
        .unwrap_or(0.0);

    let num_pts = 18;
    let mut pts = Vec::with_capacity(num_pts);

    for i in 0..num_pts {
        let frac = i as f32 / (num_pts - 1) as f32; // 0..1
        let x = rect.min.x + 3.0 + frac * (rect.width() - 6.0);
        let y_norm = match mode {
            0 => {
                // Distortion S-curve transfer function
                let x_sym = (frac - 0.5) * 2.0;
                let y_val = ((x_sym * 2.5).tanh() + (frac * 6.0 * PI - t * 6.0).sin() * 0.15).clamp(-1.0, 1.0);
                0.5 - y_val * 0.38
            }
            1 => {
                // Chorus dual-phase sine modulation wave
                let y_val = (frac * 3.5 * PI + t * 4.5).sin() * 0.8 + (frac * 7.0 * PI - t * 2.5).cos() * 0.2;
                0.5 - y_val * 0.36
            }
            2 => {
                // Delay stereo ping-pong bounce taps
                let tap = ((frac * 5.5 + t * 2.5).fract() - 0.5).abs();
                let decay = 1.0 - frac * 0.55;
                0.5 - (tap * 2.0 - 0.5) * decay * 0.40
            }
            _ => {
                // Reverb exponential diffusion decay envelope
                let decay = (-frac * 3.2).exp();
                let noise = ((frac * 24.0 + t * 3.5).sin() * 0.14) * decay;
                0.78 - (decay * 0.62 + noise)
            }
        };
        let y = rect.min.y + y_norm * rect.height();
        pts.push(pos2(x, y));
    }

    for i in 0..pts.len().saturating_sub(1) {
        painter.line_segment([pts[i], pts[i + 1]], Stroke::new(1.3_f32, color));
    }
}
