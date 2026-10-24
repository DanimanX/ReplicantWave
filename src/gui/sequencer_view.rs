//! 64-Step Pattern Sequencer Drawer matching new_ui.jpeg with analog rotary dial navigation,
//! swing, accent, slide, and gate.

use super::knob::{draw_rotary_knob, draw_rotary_knob_i8};
use super::theme::CyberTheme;
use crate::audio::step_sequencer::{SeqMode, SeqRate, Step, StepSequencer, MAX_STEPS};
use crate::audio::voice::Voice;
use nih_plug_egui::egui::{
    pos2, Button, Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};

pub struct SequencerUiState {
    pub is_drawer_open: bool,
    pub sliding_bar_pos: f32, // 0.0, 16.0, 32.0, 48.0 (page offsets)
    pub auto_page_follow: bool,
}

impl Default for SequencerUiState {
    fn default() -> Self {
        Self {
            is_drawer_open: true,
            sliding_bar_pos: 0.0,
            auto_page_follow: true,
        }
    }
}

pub fn draw_sequencer_drawer(
    ui: &mut Ui,
    seq: &mut StepSequencer,
    voices: &mut [Voice],
    state: &mut SequencerUiState,
    active_step: usize,
) {
    ui.group(|ui| {
        ui.style_mut().visuals.widgets.inactive.bg_fill = CyberTheme::BG_PANEL;

        // Auto follow playhead across 64 steps (1-16, 17-32, 33-48, 49-64)
        if seq.running && state.auto_page_follow {
            let active_page = ((active_step / 16) * 16) as f32;
            state.sliding_bar_pos = active_page;
        }

        // 1. HEADER TAB: Clickable drawer handle matching new_ui.jpeg
        let arrow = if state.is_drawer_open { "□ COLLAPSE" } else { "□ EXPAND" };
        let tab_text = format!("⚡ 64-STEP PATTERN SEQUENCER // {} ⚡", arrow);

        let tab_btn = Button::new(nih_plug_egui::egui::RichText::new(tab_text).color(CyberTheme::NEON_CYAN).strong().size(12.0))
            .fill(if state.is_drawer_open { CyberTheme::BG_PANEL_ALT } else { CyberTheme::BG_RECESSED })
            .stroke(Stroke::new(1.2_f32, CyberTheme::NEON_CYAN));

        if ui.add_sized([ui.available_width(), 24.0], tab_btn).clicked() {
            state.is_drawer_open = !state.is_drawer_open;
        }

        if !state.is_drawer_open {
            return;
        }

        ui.add_space(3.0);

        // 2. TOP TOOLBAR: Play/Stop, DAW Sync, Mode, Length, Rate, BPM, SWING
        ui.horizontal(|ui| {
            // Play / Stop Manual Button (Works 100% reliably!)
            let play_color = if seq.running { CyberTheme::NEON_LIME } else { CyberTheme::TEXT_PRIMARY };
            let play_text = if seq.running { "■ STOP" } else { "▶ PLAY" };
            let play_btn = Button::new(nih_plug_egui::egui::RichText::new(play_text).color(play_color).strong().size(11.0))
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.2_f32, if seq.running { CyberTheme::NEON_LIME } else { CyberTheme::NEON_CYAN }));

            if ui.add(play_btn).clicked() {
                seq.running = !seq.running;
                if seq.running {
                    if seq.mode == SeqMode::Off {
                        seq.mode = SeqMode::InternalSynth;
                    }
                    seq.reset();
                } else {
                    for v in voices.iter_mut() {
                        v.note_off();
                    }
                }
            }

            // Host Sync Toggle: SYNC: DAW
            let sync_color = if seq.host_sync { CyberTheme::NEON_CYAN } else { CyberTheme::TEXT_MUTED };
            let sync_text = if seq.host_sync { "SYNC: DAW" } else { "SYNC: FREE" };
            let sync_btn = Button::new(nih_plug_egui::egui::RichText::new(sync_text).color(sync_color).strong().size(11.0))
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.2_f32, sync_color));

            if ui.add(sync_btn).clicked() {
                seq.host_sync = !seq.host_sync;
            }

            ui.add_space(4.0);

            // Mode Selector: MODE: [SYNTH] [CUTOFF] [WT POS] [OFF]
            ui.label(nih_plug_egui::egui::RichText::new("MODE:").color(CyberTheme::TEXT_MUTED).size(11.0));
            let mode_names = ["SYNTH", "CUTOFF", "WT POS", "OFF"];
            for (idx, &name) in mode_names.iter().enumerate() {
                let is_sel = seq.mode == SeqMode::from_index(idx);
                let btn = Button::new(nih_plug_egui::egui::RichText::new(name)
                    .color(if is_sel { Color32::WHITE } else { CyberTheme::TEXT_MUTED })
                    .strong()
                    .size(10.5))
                    .fill(if is_sel { CyberTheme::NEON_PURPLE } else { CyberTheme::BG_RECESSED })
                    .stroke(Stroke::new(1.2_f32, if is_sel { CyberTheme::NEON_PURPLE } else { CyberTheme::BORDER_DARK }));
                if ui.add(btn).clicked() {
                    seq.mode = SeqMode::from_index(idx);
                }
            }

            ui.add_space(4.0);

            // Pattern Length: LEN: [16] [32] [48] [64]
            ui.label(nih_plug_egui::egui::RichText::new("LEN:").color(CyberTheme::TEXT_MUTED).size(11.0));
            for &l in &[16, 32, 48, 64] {
                let is_cur_len = seq.length == l;
                let btn = Button::new(nih_plug_egui::egui::RichText::new(format!("{}", l))
                    .color(if is_cur_len { Color32::from_rgb(10, 12, 16) } else { CyberTheme::TEXT_MUTED })
                    .strong()
                    .size(10.5))
                    .fill(if is_cur_len { CyberTheme::NEON_AMBER } else { CyberTheme::BG_RECESSED })
                    .stroke(Stroke::new(1.2_f32, if is_cur_len { CyberTheme::NEON_AMBER } else { CyberTheme::BORDER_DARK }));
                if ui.add(btn).clicked() {
                    seq.length = l;
                }
            }

            ui.add_space(4.0);

            // Rate (Timing Divisions): RATE: [1/4] [1/8] [1/8 T] [1/16] [1/16 T] [1/16 D] [1/32]
            ui.label(nih_plug_egui::egui::RichText::new("RATE:").color(CyberTheme::TEXT_MUTED).size(11.0));
            let rate_names = ["1/4", "1/8", "1/8 T", "1/16", "1/16 T", "1/16 D", "1/32"];
            for (idx, &name) in rate_names.iter().enumerate() {
                let is_cur_rate = seq.rate == SeqRate::from_index(idx);
                let btn = Button::new(nih_plug_egui::egui::RichText::new(name)
                    .color(if is_cur_rate { Color32::from_rgb(6, 8, 14) } else { CyberTheme::TEXT_MUTED })
                    .strong()
                    .size(10.0))
                    .fill(if is_cur_rate { CyberTheme::NEON_CYAN } else { CyberTheme::BG_RECESSED })
                    .stroke(Stroke::new(1.2_f32, if is_cur_rate { CyberTheme::NEON_CYAN } else { CyberTheme::BORDER_DARK }));
                if ui.add(btn).clicked() {
                    seq.rate = SeqRate::from_index(idx);
                }
            }

            // Right side knobs: BPM & SWING
            ui.with_layout(nih_plug_egui::egui::Layout::right_to_left(nih_plug_egui::egui::Align::Center), |ui| {
                draw_rotary_knob(ui, &mut seq.swing, 0.0..=0.75, "SWING", CyberTheme::NEON_CYAN);
                ui.add_space(4.0);
                draw_rotary_knob(ui, &mut seq.internal_bpm, 40.0..=240.0, "BPM", CyberTheme::NEON_CYAN);
            });
        });

        ui.add_space(3.0);

        // 3. ROW 2: VIEWPORT [0..64] PAGE SELECTOR & 64-STEP RADAR OVERVIEW BAR
        ui.horizontal(|ui| {
            ui.label(nih_plug_egui::egui::RichText::new("VIEWPORT [0..64]:").color(CyberTheme::NEON_AMBER).strong().size(11.0));

            let pages = [("1-16", 0.0), ("17-32", 16.0), ("33-48", 32.0), ("49-64", 48.0)];
            for (label, offset) in pages {
                let is_active_page = (state.sliding_bar_pos - offset).abs() < 1.0;
                let btn = Button::new(nih_plug_egui::egui::RichText::new(label)
                    .color(if is_active_page { Color32::from_rgb(10, 12, 16) } else { CyberTheme::TEXT_MUTED })
                    .strong()
                    .size(10.5))
                    .fill(if is_active_page { CyberTheme::NEON_AMBER } else { CyberTheme::BG_RECESSED })
                    .stroke(Stroke::new(1.0_f32, if is_active_page { CyberTheme::NEON_AMBER } else { CyberTheme::BORDER_DARK }));

                if ui.add(btn).clicked() {
                    state.sliding_bar_pos = offset;
                    state.auto_page_follow = false;
                }
            }

            ui.add_space(6.0);

            // SLIDE dial
            let mut slide_pct = if seq.steps[0].slide { 100.0f32 } else { 0.0f32 };
            ui.label(nih_plug_egui::egui::RichText::new("SLIDE:").color(CyberTheme::TEXT_MUTED).size(10.5));
            if draw_rotary_knob(ui, &mut slide_pct, 0.0..=100.0, "", CyberTheme::NEON_CYAN).changed() {
                let enable = slide_pct > 50.0;
                for s in seq.steps.iter_mut() {
                    s.slide = enable;
                }
            }

            ui.add_space(8.0);

            // Live 64-Step Radar Strip
            let avail_w = ui.available_width().max(100.0);
            draw_64_step_radar(ui, seq, active_step, state.sliding_bar_pos as usize, avail_w, 20.0);
        });

        ui.add_space(4.0);

        // 4. 16 STEP COLUMNS (Matching new_ui.jpeg: Step index, big ON/OFF button, pitch rotary knob)
        let page_offset = state.sliding_bar_pos as usize;
        let num_visible_steps = 16;
        let col_width = (ui.available_width() - (num_visible_steps as f32 - 1.0) * 4.0) / num_visible_steps as f32;

        ui.horizontal(|ui| {
            for col_i in 0..num_visible_steps {
                let step_idx = page_offset + col_i;
                if step_idx >= MAX_STEPS {
                    break;
                }

                let is_playing = seq.is_playback_running(true) && (active_step % seq.length.max(1)) == step_idx;
                let is_in_length = step_idx < seq.length;

                draw_step_column(ui, &mut seq.steps[step_idx], col_i + 1, col_width, is_playing, is_in_length);
                if col_i < num_visible_steps - 1 {
                    ui.add_space(4.0);
                }
            }
        });
    });
}

/// Draws one authentic cyberpunk step column matching new_ui.jpeg:
/// 1. Top: small indicator square + step number `01`
/// 2. Middle: Large prominent glowing `ON` / `OFF` button
/// 3. Bottom: Analog rotary knob for note pitch (-24 to +24 semitones)
fn draw_step_column(
    ui: &mut Ui,
    step: &mut Step,
    step_num: usize,
    width: f32,
    is_playing: bool,
    is_in_length: bool,
) {
    ui.allocate_ui_with_layout(
        Vec2::new(width, 108.0),
        nih_plug_egui::egui::Layout::top_down(nih_plug_egui::egui::Align::Center),
        |ui| {
            // 1. Step Indicator & Number: ■ 01
            ui.horizontal(|ui| {
                let sq_col = if is_playing {
                    CyberTheme::NEON_LIME
                } else if step.gate && is_in_length {
                    CyberTheme::NEON_CYAN
                } else {
                    CyberTheme::TEXT_DIM
                };

                let (sq_rect, _) = ui.allocate_exact_size(Vec2::new(7.0, 7.0), Sense::hover());
                ui.painter().rect_filled(sq_rect, 1.0, sq_col);

                let num_text = format!("{:02}", step_num);
                ui.label(
                    nih_plug_egui::egui::RichText::new(num_text)
                        .color(if is_playing {
                            CyberTheme::NEON_LIME
                        } else if step.gate && is_in_length {
                            CyberTheme::TEXT_PRIMARY
                        } else {
                            CyberTheme::TEXT_MUTED
                        })
                        .strong()
                        .size(10.5),
                );
            });

            ui.add_space(3.0);

            // 2. Large Cyberpunk ON / OFF Button (Matches new_ui.jpeg!)
            let (btn_fill, btn_text_col, btn_stroke_col) = if step.gate && is_in_length {
                (CyberTheme::NEON_CYAN, Color32::from_rgb(6, 8, 14), CyberTheme::NEON_CYAN)
            } else {
                (Color32::from_rgb(14, 18, 26), Color32::from_rgb(76, 88, 114), CyberTheme::BORDER_DARK)
            };

            let gate_btn = Button::new(nih_plug_egui::egui::RichText::new(if step.gate { "ON" } else { "OFF" })
                .color(btn_text_col)
                .strong()
                .size(11.5))
                .fill(btn_fill)
                .stroke(Stroke::new(if is_playing { 2.0_f32 } else { 1.0_f32 }, if is_playing { CyberTheme::NEON_LIME } else { btn_stroke_col }));

            if ui.add_sized([width - 2.0, 28.0], gate_btn).clicked() {
                step.gate = !step.gate;
            }

            ui.add_space(3.0);

            // 3. Pitch Rotary Knob (-24 to +24 semitones) with Note Readout
            let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
            let base_note = 36 + step.note_offset as i32; // Centered around C2
            let name_idx = (base_note.rem_euclid(12)) as usize;
            let octave = (base_note / 12) - 1;
            let note_label = format!("{}{}", note_names[name_idx], octave);

            draw_rotary_knob_i8(ui, &mut step.note_offset, -24..=24, &note_label, CyberTheme::NEON_CYAN);
        },
    );
}

/// Draws the 64-step mini radar timeline showing notes and current playhead cursor.
fn draw_64_step_radar(
    ui: &mut Ui,
    seq: &StepSequencer,
    active_step: usize,
    viewport_offset: usize,
    width: f32,
    height: f32,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();

    // Dark recessed tracker background
    painter.rect_filled(rect, 2.0, Color32::from_rgb(8, 10, 16));
    painter.rect_stroke(rect, 2.0, Stroke::new(1.0_f32, CyberTheme::BORDER_DARK), StrokeKind::Inside);

    // Octave guidelines & telemetry tags (GC2, C6S, GS3)
    let text_col = Color32::from_rgba_unmultiplied(0, 240, 255, 95);
    painter.text(pos2(rect.min.x + 3.0, rect.min.y + 1.0), nih_plug_egui::egui::Align2::LEFT_TOP, "GC2", FontId::monospace(7.0), text_col);
    painter.text(pos2(rect.min.x + 3.0, rect.center().y - 3.5), nih_plug_egui::egui::Align2::LEFT_TOP, "C6S", FontId::monospace(7.0), text_col);
    painter.text(pos2(rect.min.x + 3.0, rect.max.y - 8.0), nih_plug_egui::egui::Align2::LEFT_TOP, "GS3", FontId::monospace(7.0), text_col);

    let radar_left = rect.min.x + 24.0;
    let radar_right = rect.max.x - 4.0;
    let radar_w = (radar_right - radar_left).max(10.0);

    // Active Viewport Box Highlight (which 16-step page is visible)
    let vp_start_x = radar_left + (viewport_offset as f32 / MAX_STEPS as f32) * radar_w;
    let vp_end_x = radar_left + ((viewport_offset + 16) as f32 / MAX_STEPS as f32) * radar_w;
    let vp_rect = Rect::from_min_max(pos2(vp_start_x, rect.min.y + 1.0), pos2(vp_end_x, rect.max.y - 1.0));
    painter.rect_filled(vp_rect, 1.0, Color32::from_rgba_unmultiplied(255, 204, 0, 30));
    painter.rect_stroke(vp_rect, 1.0, Stroke::new(1.0_f32, CyberTheme::NEON_AMBER), StrokeKind::Inside);

    // Render 64 Steps as Mini Cyan/Amber Blocks
    let step_w = radar_w / MAX_STEPS as f32;
    for i in 0..MAX_STEPS {
        let x = radar_left + i as f32 * step_w;
        let is_in_pattern = i < seq.length;

        if is_in_pattern {
            let step = &seq.steps[i];
            if step.gate {
                let norm_pitch = ((step.note_offset as f32 + 24.0) / 48.0).clamp(0.1, 0.9);
                let block_y = rect.max.y - 2.0 - norm_pitch * (rect.height() - 5.0);
                let block_h = 4.0;
                let b_col = if step.accent {
                    CyberTheme::NEON_AMBER
                } else {
                    CyberTheme::NEON_CYAN
                };
                let b_rect = Rect::from_min_size(pos2(x + 0.5, block_y), Vec2::new(step_w.max(2.0) - 1.0, block_h));
                painter.rect_filled(b_rect, 0.5, b_col);
            }
        }
    }

    // Playhead cursor
    if seq.is_playback_running(true) {
        let play_idx = active_step % seq.length.max(1);
        let cursor_x = radar_left + (play_idx as f32 + 0.5) * step_w;
        painter.line_segment(
            [pos2(cursor_x, rect.min.y + 1.0), pos2(cursor_x, rect.max.y - 1.0)],
            Stroke::new(1.8_f32, CyberTheme::NEON_LIME),
        );
    }
}
