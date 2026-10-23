//! 64-Step Sequencer Drawer with sliding navigation bar, matching Replicant303 style.

use super::knob::{draw_rotary_knob, draw_rotary_knob_i8, draw_rotary_knob_u8};
use super::theme::CyberTheme;
use crate::audio::step_sequencer::{SeqMode, SeqRate, Step, StepSequencer, MAX_STEPS};
use nih_plug_egui::egui::{Button, Sense, Slider, Stroke, Ui, Vec2};

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
    ui.group(|ui| {
        ui.style_mut().visuals.widgets.inactive.bg_fill = CyberTheme::BG_PANEL;

        // Auto follow playhead across 64 steps (1-16, 17-32, 33-48, 49-64)
        if seq.running && state.auto_page_follow {
            let active_page = ((active_step / 16) * 16) as f32;
            state.sliding_bar_pos = active_page;
        }

        // 1. HEADER TAB: Clickable sliding drawer handle
        let arrow = if state.is_drawer_open { "▲ COLLAPSE" } else { "▼ EXPAND [64 STEPS]" };
        let tab_text = format!("⚡ 64-STEP PATTERN SEQUENCER // {} ⚡", arrow);

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

        // 2. TOP TOOLBAR: Play/Stop, Host Sync, Mode, Length, Rate, BPM, Swing, Follow, Random, Clear
        ui.horizontal(|ui| {
            // Play / Stop
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
            let sync_btn = Button::new(if seq.host_sync { "SYNC: DAW" } else { "SYNC: FREE" })
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.0_f32, sync_color));
            if ui.add(sync_btn).clicked() {
                seq.host_sync = !seq.host_sync;
            }

            ui.separator();

            // Mode
            ui.label(nih_plug_egui::egui::RichText::new("MODE:").color(CyberTheme::TEXT_MUTED).size(11.0));
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

            // Length (16, 32, 48, 64)
            ui.label(nih_plug_egui::egui::RichText::new("LEN:").color(CyberTheme::TEXT_MUTED).size(11.0));
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

            // Rate
            ui.label(nih_plug_egui::egui::RichText::new("RATE:").color(CyberTheme::TEXT_MUTED).size(11.0));
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

            // Rotary Knobs: BPM & SWING
            draw_rotary_knob(ui, &mut seq.internal_bpm, 40.0..=240.0, "BPM", CyberTheme::NEON_CYAN);
            draw_rotary_knob(ui, &mut seq.swing, 0.0..=0.75, "SWING", CyberTheme::NEON_AMBER);

            ui.separator();

            // Follow playhead toggle
            let follow_color = if state.auto_page_follow { CyberTheme::NEON_LIME } else { CyberTheme::TEXT_MUTED };
            let follow_btn = Button::new("FOLLOW")
                .fill(CyberTheme::BG_RECESSED)
                .stroke(Stroke::new(1.0_f32, follow_color));
            if ui.add(follow_btn).clicked() {
                state.auto_page_follow = !state.auto_page_follow;
            }

            // Utility buttons
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

        ui.add_space(6.0);

        // 3. SLIDING NAVIGATION BAR (The sliding bar requested like Replicant303)
        ui.horizontal(|ui| {
            ui.label(nih_plug_egui::egui::RichText::new("VIEWPORT [0..64]:").color(CyberTheme::NEON_AMBER).size(11.0));

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

            // Smooth Horizontal Sliding Bar
            ui.label(nih_plug_egui::egui::RichText::new("SLIDE:").color(CyberTheme::TEXT_MUTED).size(11.0));
            if ui.add_sized(
                [ui.available_width() - 10.0, 18.0],
                Slider::new(&mut state.sliding_bar_pos, 0.0..=48.0)
                    .show_value(false)
                    .text(""),
            ).changed() {
                state.auto_page_follow = false;
            }
        });

        ui.add_space(4.0);

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

                let is_current = seq.running && active_step == step_idx;
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
            // Little LED square
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

        // Note pitch rotary knob (-24 to +24 semitones)
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let base_note = 36 + step.note_offset as i32; // Centered around C2
        let name_idx = (base_note % 12).abs() as usize;
        let octave = (base_note / 12) - 1;
        let note_label = format!("{}{}", note_names[name_idx], octave);

        draw_rotary_knob_i8(ui, &mut step.note_offset, -24..=24, &note_label, CyberTheme::NEON_CYAN);

        // Velocity rotary knob (1 to 127)
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
