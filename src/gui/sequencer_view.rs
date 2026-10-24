//! 64-Step Pattern Sequencer Drawer with analog rotary dial navigation, swing, accent, slide, and gate.
//! Authentic analog hardware controls with zero generic bars or sliders.

use super::knob::{draw_rotary_knob, draw_rotary_knob_i8, draw_rotary_knob_usize};
use super::theme::CyberTheme;
use crate::audio::step_sequencer::{SeqMode, SeqRate, Step, StepSequencer, MAX_STEPS};
use nih_plug_egui::egui::{pos2, Button, Color32, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

pub struct SequencerUiState {
    pub is_drawer_open: bool,
    pub sliding_bar_pos: f32, // 0.0 to 48.0 (viewport window over 64 steps)
    pub selected_step: usize,
    pub auto_page_follow: bool,
}

impl Default for SequencerUiState {
    fn default() -> Self {
        Self {
            is_drawer_open: true,
            sliding_bar_pos: 0.0,
            selected_step: 0,
            auto_page_follow: true,
        }
    }
}

pub fn draw_sequencer_drawer(
    ui: &mut Ui,
    seq: &mut StepSequencer,
    state: &mut SequencerUiState,
    active_step: usize,
) {
    let group_resp = ui.group(|ui| {
        ui.style_mut().visuals.widgets.inactive.bg_fill = CyberTheme::BG_PANEL;

        // Auto follow playhead across 64 steps (1-16, 17-32, 33-48, 49-64)
        if seq.running && state.auto_page_follow {
            let active_page = ((active_step / 16) * 16) as f32;
            state.sliding_bar_pos = active_page;
        }

        // 1. HEADER TAB: Clickable drawer handle matching new_ui.jpeg
        let arrow = if state.is_drawer_open { "⧉ COLLAPSE" } else { "⧉ EXPAND" };
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
            // Play / Stop
            let play_color = if seq.running { CyberTheme::NEON_LIME } else { CyberTheme::TEXT_PRIMARY };
            let play_btn = Button::new(nih_plug_egui::egui::RichText::new(if seq.running { "■ STOP" } else { "▶ PLAY" }).color(play_color).strong().size(11.0))
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.2_f32, if seq.running { CyberTheme::NEON_LIME } else { CyberTheme::BORDER_DARK }));
            if ui.add(play_btn).clicked() {
                seq.running = !seq.running;
                if seq.running {
                    seq.reset();
                }
            }

            // Host Sync Toggle: SYNC: DAW
            let sync_color = if seq.host_sync { CyberTheme::NEON_CYAN } else { CyberTheme::TEXT_MUTED };
            let sync_btn = Button::new(nih_plug_egui::egui::RichText::new(if seq.host_sync { "SYNC: DAW" } else { "SYNC: FREE" }).color(sync_color).strong().size(11.0))
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
                    .stroke(Stroke::new(1.0_f32, if is_sel { CyberTheme::NEON_PURPLE } else { CyberTheme::BORDER_DARK }));
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
                    .stroke(Stroke::new(1.0_f32, if is_cur_len { CyberTheme::NEON_AMBER } else { CyberTheme::BORDER_DARK }));
                if ui.add(btn).clicked() {
                    seq.length = l;
                }
            }

            ui.add_space(4.0);

            // Rate (Timing Divisions): RATE: [1/4] [1/8] [1/8 T] [1/16] [1/16 T] [1/16 D] [1/32]
            ui.label(nih_plug_egui::egui::RichText::new("RATE:").color(CyberTheme::TEXT_MUTED).size(11.0));
            for idx in 0..7 {
                let r = SeqRate::from_index(idx);
                let is_sel = seq.rate == r;
                let btn = Button::new(nih_plug_egui::egui::RichText::new(r.name())
                    .color(if is_sel { Color32::from_rgb(10, 12, 16) } else { CyberTheme::TEXT_MUTED })
                    .strong()
                    .size(10.0))
                    .fill(if is_sel { CyberTheme::NEON_CYAN } else { CyberTheme::BG_RECESSED })
                    .stroke(Stroke::new(1.0_f32, if is_sel { CyberTheme::NEON_CYAN } else { CyberTheme::BORDER_DARK }));
                if ui.add(btn).clicked() {
                    seq.rate = r;
                }
            }

            ui.add_space(6.0);

            // Analog Rotary Knobs: BPM & SWING
            draw_rotary_knob(ui, &mut seq.internal_bpm, 40.0..=240.0, "BPM", CyberTheme::NEON_CYAN);
            draw_rotary_knob(ui, &mut seq.swing, 0.0..=0.75, "SWING", CyberTheme::NEON_CYAN);

            // Utilities: Randomize, Clear, Follow
            let follow_color = if state.auto_page_follow { CyberTheme::NEON_LIME } else { CyberTheme::TEXT_DIM };
            let follow_btn = Button::new(nih_plug_egui::egui::RichText::new("FOLLOW").color(follow_color).size(10.0))
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.0_f32, follow_color));
            if ui.add(follow_btn).clicked() {
                state.auto_page_follow = !state.auto_page_follow;
            }
            if ui.button(nih_plug_egui::egui::RichText::new("🎲 RND").size(10.0)).clicked() {
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(0x12345678);
                seq.randomize(seed);
            }
            if ui.button(nih_plug_egui::egui::RichText::new("✖ CLR").size(10.0)).clicked() {
                seq.clear();
            }
        });

        ui.add_space(3.0);

        // 3. 64-STEP VIEWPORT NAVIGATION & PIANO-ROLL RADAR TIMELINE (Exactly matching new_ui.jpeg)
        ui.horizontal(|ui| {
            ui.label(nih_plug_egui::egui::RichText::new("VIEWPORT [0..64]:").color(CyberTheme::NEON_AMBER).size(11.0).strong());

            // Quick Page Buttons: [1-16], [17-32], [33-48], [49-64]
            let pages = [(0.0, "1-16"), (16.0, "17-32"), (32.0, "33-48"), (48.0, "49-64")];
            for &(page_start, label) in &pages {
                let is_active = (state.sliding_bar_pos - page_start).abs() < 1.0;
                let btn = Button::new(nih_plug_egui::egui::RichText::new(label)
                    .color(if is_active { Color32::from_rgb(10, 12, 16) } else { CyberTheme::TEXT_MUTED })
                    .strong()
                    .size(10.5))
                    .fill(if is_active { CyberTheme::NEON_AMBER } else { CyberTheme::BG_RECESSED })
                    .stroke(Stroke::new(1.0_f32, if is_active { CyberTheme::NEON_AMBER } else { CyberTheme::BORDER_DARK }));
                if ui.add(btn).clicked() {
                    state.sliding_bar_pos = page_start;
                    state.auto_page_follow = false;
                }
            }

            ui.add_space(4.0);

            // Analog Rotary Dial for Viewport scrolling across 64 steps
            let mut page_pos = state.sliding_bar_pos.round() as usize;
            if draw_rotary_knob_usize(ui, &mut page_pos, 0..=48, "SLIDE", CyberTheme::NEON_CYAN).changed() {
                state.sliding_bar_pos = page_pos as f32;
                state.auto_page_follow = false;
            }

            ui.add_space(6.0);

            // PIANO-ROLL TRACKER RADAR DISPLAY (Showing note bars for all 64 steps)
            let radar_w = ui.available_width() - 8.0;
            let (radar_rect, radar_resp) = ui.allocate_exact_size(Vec2::new(radar_w, 24.0), Sense::click_and_drag());
            let r_painter = ui.painter();

            // Background LCD frame
            r_painter.rect_filled(radar_rect, 2.0, Color32::from_rgb(6, 10, 16));
            r_painter.rect_stroke(radar_rect, 2.0, Stroke::new(1.0_f32, CyberTheme::BORDER_DARK), StrokeKind::Inside);

            // Pitch reference lines: GC2, C6S, GS3
            let y_lines = [
                (radar_rect.min.y + 5.0, "GC2"),
                (radar_rect.min.y + 12.0, "C6S"),
                (radar_rect.min.y + 19.0, "GS3"),
            ];
            for &(ly, l_name) in &y_lines {
                r_painter.line_segment(
                    [pos2(radar_rect.min.x + 28.0, ly), pos2(radar_rect.max.x - 4.0, ly)],
                    Stroke::new(0.6_f32, Color32::from_rgba_unmultiplied(0, 180, 200, 30)),
                );
                r_painter.text(
                    pos2(radar_rect.min.x + 4.0, ly - 4.0),
                    nih_plug_egui::egui::Align2::LEFT_TOP,
                    l_name,
                    nih_plug_egui::egui::FontId::monospace(6.5),
                    Color32::from_rgba_unmultiplied(0, 180, 200, 90),
                );
            }

            // Click or drag to jump viewport to any step
            if radar_resp.clicked() || radar_resp.dragged() {
                if let Some(mouse_pos) = radar_resp.interact_pointer_pos() {
                    let frac = ((mouse_pos.x - (radar_rect.min.x + 30.0)) / (radar_rect.width() - 34.0)).clamp(0.0, 1.0);
                    let target_step = (frac * 64.0).floor() as f32;
                    state.sliding_bar_pos = target_step.clamp(0.0, 48.0);
                    state.auto_page_follow = false;
                }
            }

            // Draw note pitch bars for all 64 steps
            let pip_w = (radar_rect.width() - 34.0) / 64.0;
            for i in 0..64 {
                let px = radar_rect.min.x + 30.0 + i as f32 * pip_w;
                let step = &seq.steps[i];
                let is_in_len = i < seq.length;

                if is_in_len && step.gate {
                    // Height mapped from note offset (-24..+24)
                    let norm_pitch = ((step.note_offset as f32 + 24.0) / 48.0).clamp(0.0, 1.0);
                    let bar_h = 4.0 + norm_pitch * 14.0;
                    let py = radar_rect.max.y - 2.0 - bar_h;
                    let bar_col = if step.accent { CyberTheme::NEON_AMBER } else { CyberTheme::NEON_CYAN };
                    r_painter.rect_filled(Rect::from_min_size(pos2(px, py), Vec2::new((pip_w - 0.6).max(1.0), bar_h)), 0.5, bar_col);
                } else {
                    let py = radar_rect.max.y - 3.0;
                    r_painter.rect_filled(Rect::from_min_size(pos2(px, py), Vec2::new((pip_w - 0.6).max(1.0), 1.5)), 0.0, Color32::from_rgb(18, 22, 32));
                }
            }

            // Highlight current 16-step viewport window
            let start_step_f = state.sliding_bar_pos.clamp(0.0, 48.0);
            let vp_x0 = radar_rect.min.x + 30.0 + start_step_f * pip_w;
            let vp_x1 = vp_x0 + 16.0 * pip_w;
            r_painter.rect_filled(
                Rect::from_min_max(pos2(vp_x0, radar_rect.min.y + 1.0), pos2(vp_x1, radar_rect.max.y - 1.0)),
                1.0,
                Color32::from_rgba_unmultiplied(255, 210, 0, 45),
            );
            r_painter.rect_stroke(
                Rect::from_min_max(pos2(vp_x0, radar_rect.min.y + 1.0), pos2(vp_x1, radar_rect.max.y - 1.0)),
                1.0,
                Stroke::new(1.2_f32, Color32::from_rgba_unmultiplied(255, 210, 0, 200)),
                StrokeKind::Outside,
            );

            // Highlight active playhead cursor
            if seq.is_playback_running(true) {
                let playhead_x = radar_rect.min.x + 30.0 + (active_step % seq.length.max(1)) as f32 * pip_w;
                r_painter.line_segment(
                    [pos2(playhead_x, radar_rect.min.y), pos2(playhead_x, radar_rect.max.y)],
                    Stroke::new(2.0_f32, CyberTheme::NEON_LIME),
                );
            }
        });

        ui.add_space(4.0);

        // 4. 16 VISIBLE STEP COLUMNS (Exactly matching new_ui.jpeg)
        let start_step = (state.sliding_bar_pos.round() as usize).min(48);
        let visible_count = 16;

        let avail_w = ui.available_width();
        let step_w = ((avail_w - 20.0) / visible_count as f32).clamp(42.0, 76.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for col in 0..visible_count {
                let step_idx = start_step + col;
                if step_idx >= MAX_STEPS {
                    break;
                }

                let is_current = seq.is_playback_running(true) && (active_step % seq.length.max(1)) == step_idx;
                let is_in_length = step_idx < seq.length;

                draw_single_step_column(
                    ui,
                    &mut seq.steps[step_idx],
                    step_idx,
                    is_current,
                    is_in_length,
                    step_w,
                );
            }
        });
    });

    CyberTheme::draw_panel_screws(ui.painter(), group_resp.response.rect);
}

fn draw_single_step_column(
    ui: &mut Ui,
    step: &mut Step,
    step_num: usize,
    is_playing: bool,
    is_in_length: bool,
    width: f32,
) {
    let col_bg = if is_playing {
        Color32::from_rgb(13, 22, 24)
    } else if step.gate && is_in_length {
        Color32::from_rgb(11, 16, 24)
    } else {
        Color32::from_rgb(9, 12, 18)
    };

    let col_border = if is_playing {
        Stroke::new(1.5_f32, CyberTheme::NEON_LIME)
    } else {
        Stroke::new(1.0_f32, Color32::from_rgb(26, 32, 48))
    };

    nih_plug_egui::egui::Frame::NONE
        .fill(col_bg)
        .stroke(col_border)
        .corner_radius(3.0)
        .inner_margin(nih_plug_egui::egui::Margin::symmetric(2, 4))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.vertical_centered(|ui| {
                // 1. Header: Cyan square pip + Step Number (e.g. ■ 01)
                let pip_color = if is_playing {
                    CyberTheme::NEON_LIME
                } else if step.gate && is_in_length {
                    CyberTheme::NEON_CYAN
                } else {
                    Color32::from_rgb(22, 28, 40)
                };

                let num_color = if is_playing {
                    CyberTheme::NEON_LIME
                } else if is_in_length {
                    CyberTheme::TEXT_PRIMARY
                } else {
                    CyberTheme::TEXT_DIM
                };

                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(7.0, 7.0), Sense::hover());
                    ui.painter().rect_filled(rect, 1.0, pip_color);
                    ui.label(nih_plug_egui::egui::RichText::new(format!("{:02}", step_num + 1)).size(10.5).color(num_color).strong());
                });

                ui.add_space(2.0);

                // 2. Note pitch analog rotary knob (-24 to +24 semitones) with musical note name (C2, D#2, etc.)
                let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
                let base_note = 36 + step.note_offset as i32; // Centered around C2
                let name_idx = (base_note % 12).abs() as usize;
                let octave = (base_note / 12) - 1;
                let note_label = format!("{}{}", note_names[name_idx], octave);

                let knob_col = if is_playing {
                    CyberTheme::NEON_LIME
                } else if step.accent {
                    CyberTheme::NEON_AMBER
                } else {
                    CyberTheme::NEON_CYAN
                };

                draw_rotary_knob_i8(ui, &mut step.note_offset, -24..=24, &note_label, knob_col);

                ui.add_space(3.0);

                // 3. Compact Step Modifiers Row: SLD (Slide), ACC (Accent), TIE (Tie)
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    let third_w = (width - 12.0) / 3.0;

                    let sld_btn = Button::new(nih_plug_egui::egui::RichText::new("S")
                        .color(if step.slide { Color32::WHITE } else { CyberTheme::TEXT_DIM })
                        .strong()
                        .size(9.5))
                        .fill(if step.slide { CyberTheme::NEON_PINK } else { CyberTheme::BG_RECESSED })
                        .stroke(Stroke::new(1.0_f32, if step.slide { CyberTheme::NEON_PINK } else { CyberTheme::BORDER_DARK }));
                    if ui.add_sized([third_w, 18.0], sld_btn).clicked() {
                        step.slide = !step.slide;
                    }

                    let acc_btn = Button::new(nih_plug_egui::egui::RichText::new("A")
                        .color(if step.accent { Color32::from_rgb(10, 12, 16) } else { CyberTheme::TEXT_DIM })
                        .strong()
                        .size(9.5))
                        .fill(if step.accent { CyberTheme::NEON_AMBER } else { CyberTheme::BG_RECESSED })
                        .stroke(Stroke::new(1.0_f32, if step.accent { CyberTheme::NEON_AMBER } else { CyberTheme::BORDER_DARK }));
                    if ui.add_sized([third_w, 18.0], acc_btn).clicked() {
                        step.accent = !step.accent;
                    }

                    let tie_btn = Button::new(nih_plug_egui::egui::RichText::new("T")
                        .color(if step.tie { Color32::WHITE } else { CyberTheme::TEXT_DIM })
                        .strong()
                        .size(9.5))
                        .fill(if step.tie { CyberTheme::NEON_PURPLE } else { CyberTheme::BG_RECESSED })
                        .stroke(Stroke::new(1.0_f32, if step.tie { CyberTheme::NEON_PURPLE } else { CyberTheme::BORDER_DARK }));
                    if ui.add_sized([third_w, 18.0], tie_btn).clicked() {
                        step.tie = !step.tie;
                    }
                });

                ui.add_space(4.0);

                // 4. Large Vibrant Cyberpunk ON / OFF Button AT THE BOTTOM (Matching new_ui.jpeg)
                let (btn_fill, btn_text_col, btn_stroke_col) = if step.gate && is_in_length {
                    (CyberTheme::NEON_CYAN, Color32::from_rgb(6, 8, 14), CyberTheme::NEON_CYAN)
                } else {
                    (Color32::from_rgb(14, 18, 26), Color32::from_rgb(76, 88, 114), CyberTheme::BORDER_DARK)
                };

                let gate_btn = Button::new(nih_plug_egui::egui::RichText::new(if step.gate { "ON" } else { "OFF" })
                    .color(btn_text_col)
                    .strong()
                    .size(12.0))
                    .fill(btn_fill)
                    .stroke(Stroke::new(if is_playing { 1.8_f32 } else { 1.0_f32 }, if is_playing { CyberTheme::NEON_LIME } else { btn_stroke_col }));

                if ui.add_sized([width - 2.0, 52.0], gate_btn).clicked() {
                    step.gate = !step.gate;
                }
            });
        });
}
