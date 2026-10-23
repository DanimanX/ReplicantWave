//! 64-Step Pattern Sequencer Drawer with analog rotary dial navigation, swing, accent, slide, and gate.
//! Authentic analog hardware controls with zero generic bars or sliders.

use super::knob::{draw_rotary_knob, draw_rotary_knob_i8, draw_rotary_knob_u8, draw_rotary_knob_usize};
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

        // 1. HEADER TAB: Clickable drawer handle with live telemetry
        let status_str = if seq.is_playback_running(true) {
            "● RUNNING"
        } else {
            "○ STOPPED"
        };
        let arrow = if state.is_drawer_open { "▲ COLLAPSE" } else { "▼ EXPAND [64 STEPS]" };
        let tab_text = format!(
            "⚡ 64-STEP PATTERN SEQUENCER // {} // STEP {:02}/{} // {} ⚡",
            status_str,
            (active_step % seq.length.max(1)) + 1,
            seq.length,
            arrow
        );

        let tab_btn = Button::new(tab_text)
            .fill(if state.is_drawer_open { CyberTheme::BG_PANEL_ALT } else { CyberTheme::BG_RECESSED })
            .stroke(Stroke::new(1.0_f32, CyberTheme::NEON_CYAN));

        if ui.add_sized([ui.available_width(), 26.0], tab_btn).clicked() {
            state.is_drawer_open = !state.is_drawer_open;
        }

        if !state.is_drawer_open {
            return;
        }

        ui.add_space(4.0);

        // 2. TOP TOOLBAR: Play/Stop, DAW Sync, Key Trig, Latch, Mode, Length, Rate, Knobs, Follow, Random, Clear
        ui.horizontal(|ui| {
            // Play / Stop Manual Toggle
            let play_color = if seq.running { CyberTheme::NEON_LIME } else { CyberTheme::TEXT_MUTED };
            let play_btn = Button::new(if seq.running { "■ STOP" } else { "▶ PLAY" })
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.5_f32, play_color));
            if ui.add(play_btn).clicked() {
                seq.running = !seq.running;
                if seq.running {
                    seq.reset();
                }
            }

            // Host Sync Toggle
            let sync_color = if seq.host_sync { CyberTheme::NEON_CYAN } else { CyberTheme::TEXT_MUTED };
            let sync_btn = Button::new(if seq.host_sync { "DAW SYNC" } else { "FREE RUN" })
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.0_f32, sync_color));
            if ui.add(sync_btn).clicked() {
                seq.host_sync = !seq.host_sync;
            }

            // Key Trigger Toggle
            let key_color = if seq.key_trigger { CyberTheme::NEON_AMBER } else { CyberTheme::TEXT_MUTED };
            let key_btn = Button::new(if seq.key_trigger { "KEY TRIG" } else { "NO TRIG" })
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.0_f32, key_color));
            if ui.add(key_btn).clicked() {
                seq.key_trigger = !seq.key_trigger;
            }

            // Latch Toggle
            let latch_color = if seq.latch { CyberTheme::NEON_PINK } else { CyberTheme::TEXT_MUTED };
            let latch_btn = Button::new(if seq.latch { "LATCH" } else { "MOMENT" })
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.0_f32, latch_color));
            if ui.add(latch_btn).clicked() {
                seq.latch = !seq.latch;
            }

            ui.separator();

            // Mode Selector
            ui.label(nih_plug_egui::egui::RichText::new("MODE:").color(CyberTheme::TEXT_MUTED).size(10.5));
            let mode_names = ["SYNTH", "CUTOFF", "WT POS", "OFF"];
            for (idx, &name) in mode_names.iter().enumerate() {
                let is_sel = seq.mode == SeqMode::from_index(idx);
                let btn = Button::new(name)
                    .fill(if is_sel { CyberTheme::NEON_PURPLE } else { CyberTheme::BG_RECESSED })
                    .stroke(Stroke::new(1.0_f32, if is_sel { CyberTheme::TEXT_PRIMARY } else { CyberTheme::BORDER_DARK }));
                if ui.add(btn).clicked() {
                    seq.mode = SeqMode::from_index(idx);
                }
            }

            ui.separator();

            // Pattern Length: 16, 32, 48, 64
            ui.label(nih_plug_egui::egui::RichText::new("LEN:").color(CyberTheme::TEXT_MUTED).size(10.5));
            for &l in &[16, 32, 48, 64] {
                let is_cur_len = seq.length == l;
                let btn = Button::new(format!("{}", l))
                    .fill(if is_cur_len { CyberTheme::NEON_AMBER } else { CyberTheme::BG_RECESSED })
                    .stroke(Stroke::new(1.0_f32, if is_cur_len { CyberTheme::TEXT_PRIMARY } else { CyberTheme::BORDER_DARK }));
                if ui.add(btn).clicked() {
                    seq.length = l;
                }
            }

            ui.separator();

            // Rate (Timing Divisions)
            ui.label(nih_plug_egui::egui::RichText::new("RATE:").color(CyberTheme::TEXT_MUTED).size(10.5));
            for idx in 0..7 {
                let r = SeqRate::from_index(idx);
                let is_sel = seq.rate == r;
                let btn = Button::new(r.name())
                    .fill(if is_sel { CyberTheme::NEON_CYAN } else { CyberTheme::BG_RECESSED })
                    .stroke(Stroke::new(1.0_f32, CyberTheme::BORDER_DARK));
                if ui.add(btn).clicked() {
                    seq.rate = r;
                }
            }

            ui.separator();

            // Analog Rotary Knobs: BPM & SWING
            draw_rotary_knob(ui, &mut seq.internal_bpm, 40.0..=240.0, "BPM", CyberTheme::NEON_CYAN);
            draw_rotary_knob(ui, &mut seq.swing, 0.0..=0.75, "SWING", CyberTheme::NEON_AMBER);

            ui.separator();

            // Follow Playhead Toggle
            let follow_color = if state.auto_page_follow { CyberTheme::NEON_LIME } else { CyberTheme::TEXT_MUTED };
            let follow_btn = Button::new("FOLLOW")
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.0_f32, follow_color));
            if ui.add(follow_btn).clicked() {
                state.auto_page_follow = !state.auto_page_follow;
            }

            // Utilities: Randomize & Clear
            if ui.button("🎲 RND").clicked() {
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(0x12345678);
                seq.randomize(seed);
            }
            if ui.button("✖ CLR").clicked() {
                seq.clear();
            }
        });

        ui.add_space(3.0);

        // 3. 64-STEP VIEWPORT NAVIGATION: ANALOG ROTARY DIAL & QUICK PAGE BUTTONS (NO SLIDERS/BARS)
        ui.horizontal(|ui| {
            ui.label(nih_plug_egui::egui::RichText::new("PAGE [64 STEPS]:").color(CyberTheme::NEON_AMBER).size(11.0).strong());

            // Quick Page Buttons
            let pages = [(0.0, "1-16"), (16.0, "17-32"), (32.0, "33-48"), (48.0, "49-64")];
            for &(page_start, label) in &pages {
                let is_active = (state.sliding_bar_pos - page_start).abs() < 1.0;
                let btn = Button::new(label)
                    .fill(if is_active { CyberTheme::NEON_AMBER } else { CyberTheme::BG_RECESSED })
                    .stroke(Stroke::new(1.0_f32, if is_active { CyberTheme::TEXT_PRIMARY } else { CyberTheme::BORDER_DARK }));
                if ui.add(btn).clicked() {
                    state.sliding_bar_pos = page_start;
                    state.auto_page_follow = false; // Allow manual browsing
                }
            }

            ui.add_space(8.0);

            // Analog Rotary Dial for Viewport scrolling across all 64 steps
            let mut page_pos = state.sliding_bar_pos.round() as usize;
            if draw_rotary_knob_usize(ui, &mut page_pos, 0..=48, "VIEW [0..48]", CyberTheme::NEON_AMBER).changed() {
                state.sliding_bar_pos = page_pos as f32;
                state.auto_page_follow = false;
            }

            ui.add_space(8.0);
            let cur_start = state.sliding_bar_pos.round() as usize + 1;
            let cur_end = (cur_start + 15).min(64);
            ui.label(nih_plug_egui::egui::RichText::new(format!("// VIEWING STEPS {:02}..{:02} OF 64", cur_start, cur_end))
                .color(CyberTheme::TEXT_MUTED)
                .size(10.5));
        });

        ui.add_space(2.0);

        // 3B. 64-STEP OVERVIEW RADAR TIMELINE BAR (Interactive Overview matching new_ui.jpeg)
        let radar_w = ui.available_width() - 8.0;
        let (radar_rect, radar_resp) = ui.allocate_exact_size(Vec2::new(radar_w, 12.0), Sense::click_and_drag());
        let r_painter = ui.painter();

        // Background groove
        r_painter.rect_filled(radar_rect, 2.0, Color32::from_rgb(10, 12, 18));
        r_painter.rect_stroke(radar_rect, 2.0, Stroke::new(1.0_f32, CyberTheme::BORDER_DARK), StrokeKind::Inside);

        // Click or drag to jump viewport to any step
        if radar_resp.clicked() || radar_resp.dragged() {
            if let Some(mouse_pos) = radar_resp.interact_pointer_pos() {
                let frac = ((mouse_pos.x - radar_rect.min.x) / radar_rect.width()).clamp(0.0, 1.0);
                let target_step = (frac * 64.0).floor() as f32;
                state.sliding_bar_pos = target_step.clamp(0.0, 48.0);
                state.auto_page_follow = false;
            }
        }

        // Draw 64 mini step pips
        let pip_w = (radar_rect.width() - 8.0) / 64.0;
        for i in 0..64 {
            let px = radar_rect.min.x + 4.0 + i as f32 * pip_w;
            let py = radar_rect.min.y + 2.0;
            let step = &seq.steps[i];
            let is_in_len = i < seq.length;

            let pip_col = if !is_in_len {
                Color32::from_rgb(20, 22, 30)
            } else if step.accent && step.gate {
                CyberTheme::NEON_AMBER
            } else if step.gate {
                CyberTheme::NEON_CYAN
            } else {
                Color32::from_rgb(26, 30, 44)
            };

            r_painter.rect_filled(Rect::from_min_size(pos2(px, py), Vec2::new(pip_w - 0.8, 8.0)), 1.0, pip_col);
        }

        // Highlight current 16-step viewport bracket
        let start_step_f = state.sliding_bar_pos.clamp(0.0, 48.0);
        let vp_x0 = radar_rect.min.x + 4.0 + start_step_f * pip_w;
        let vp_x1 = vp_x0 + 16.0 * pip_w;
        r_painter.rect_stroke(
            Rect::from_min_max(pos2(vp_x0, radar_rect.min.y + 1.0), pos2(vp_x1, radar_rect.max.y - 1.0)),
            2.0,
            Stroke::new(1.2_f32, Color32::from_rgba_unmultiplied(255, 210, 0, 190)),
            StrokeKind::Outside,
        );

        // Highlight active playhead cursor
        if seq.is_playback_running(true) {
            let playhead_x = radar_rect.min.x + 4.0 + (active_step % seq.length.max(1)) as f32 * pip_w;
            r_painter.line_segment(
                [pos2(playhead_x, radar_rect.min.y), pos2(playhead_x, radar_rect.max.y)],
                Stroke::new(2.0_f32, CyberTheme::NEON_LIME),
            );
        }

        ui.add_space(2.0);

        // 4. 16 VISIBLE STEPS GRID: mapped from sliding_bar_pos
        let start_step = (state.sliding_bar_pos.round() as usize).min(48);
        let visible_count = 16;

        let avail_w = ui.available_width();
        let step_w = ((avail_w - 36.0) / visible_count as f32).clamp(36.0, 58.0);

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
    ui.vertical(|ui| {
        ui.set_width(width);

        // Header: Step number & LED playhead
        let led_color = if is_playing {
            CyberTheme::NEON_LIME
        } else if step.gate && is_in_length {
            CyberTheme::NEON_CYAN
        } else {
            CyberTheme::LED_OFF
        };

        let num_color = if is_playing {
            CyberTheme::NEON_LIME
        } else if is_in_length {
            CyberTheme::TEXT_PRIMARY
        } else {
            CyberTheme::TEXT_DIM
        };

        ui.horizontal(|ui| {
            // LED square
            let (rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
            ui.painter().rect_filled(rect, 1.0, led_color);
            ui.label(nih_plug_egui::egui::RichText::new(format!("{:02}", step_num + 1)).size(10.0).color(num_color));
        });

        // Gate Toggle Button
        let gate_fill = if step.gate {
            if is_in_length { CyberTheme::NEON_CYAN } else { CyberTheme::BG_PANEL_ALT }
        } else {
            CyberTheme::BG_RECESSED
        };
        let gate_btn = Button::new(if step.gate { "ON" } else { "OFF" })
            .fill(gate_fill)
            .stroke(Stroke::new(1.0_f32, CyberTheme::BORDER_DARK));
        if ui.add_sized([width, 18.0], gate_btn).clicked() {
            step.gate = !step.gate;
        }

        // Note pitch analog rotary knob (-24 to +24 semitones) with musical note name
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let base_note = 36 + step.note_offset as i32; // Centered around C2
        let name_idx = (base_note % 12).abs() as usize;
        let octave = (base_note / 12) - 1;
        let note_label = format!("{}{}", note_names[name_idx], octave);

        draw_rotary_knob_i8(ui, &mut step.note_offset, -24..=24, &note_label, CyberTheme::NEON_CYAN);

        // Velocity analog rotary knob (1 to 127)
        draw_rotary_knob_u8(ui, &mut step.velocity, 1..=127, "VEL", CyberTheme::NEON_AMBER);

        // Slide Button
        let slide_btn = Button::new("SLD")
            .fill(if step.slide { CyberTheme::NEON_PINK } else { CyberTheme::BG_RECESSED })
            .stroke(Stroke::new(1.0_f32, CyberTheme::BORDER_DARK));
        if ui.add_sized([width, 16.0], slide_btn).clicked() {
            step.slide = !step.slide;
        }

        // Accent Button
        let acc_btn = Button::new("ACC")
            .fill(if step.accent { CyberTheme::NEON_AMBER } else { CyberTheme::BG_RECESSED })
            .stroke(Stroke::new(1.0_f32, CyberTheme::BORDER_DARK));
        if ui.add_sized([width, 16.0], acc_btn).clicked() {
            step.accent = !step.accent;
        }
    });
}
