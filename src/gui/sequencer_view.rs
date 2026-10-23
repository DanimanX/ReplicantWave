//! 64-Step Sequencer Drawer with sliding navigation bar, matching Replicant303 style.

use super::theme::CyberTheme;
use crate::audio::step_sequencer::{SeqMode, SeqRate, Step, StepSequencer, MAX_STEPS};
use nih_plug_egui::egui::{Button, Sense, Slider, Stroke, Ui, Vec2};

pub struct SequencerUiState {
    pub is_drawer_open: bool,
    pub sliding_bar_pos: f32, // 0.0 to 48.0 (viewport window over 64 steps)
    pub selected_step: usize,
}

impl Default for SequencerUiState {
    fn default() -> Self {
        Self {
            is_drawer_open: true,
            sliding_bar_pos: 0.0,
            selected_step: 0,
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

        // 1. HEADER TAB: Clickable sliding drawer handle like Replicant303
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

        // 2. TOP TOOLBAR: Play/Stop, Mode, Length, Rate, Swing, Random, Clear
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

            // Length
            ui.label(nih_plug_egui::egui::RichText::new("LENGTH:").color(CyberTheme::TEXT_MUTED).size(11.0));
            ui.add(Slider::new(&mut seq.length, 1..=MAX_STEPS).text(""));
            for &l in &[16, 32, 48, 64] {
                if ui.button(format!("{}", l)).clicked() {
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

            // Swing
            ui.label(nih_plug_egui::egui::RichText::new("SWING:").color(CyberTheme::TEXT_MUTED).size(11.0));
            ui.add(Slider::new(&mut seq.swing, 0.0..=0.75).show_value(false));

            ui.separator();

            // Utility buttons
            if ui.button("🎲 RANDOM").clicked() {
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(0x12345678);
                seq.randomize(seed);
            }
            if ui.button("✖ CLEAR").clicked() {
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
                }
            }

            ui.add_space(8.0);

            // Smooth Horizontal Sliding Bar
            ui.label(nih_plug_egui::egui::RichText::new("SLIDE:").color(CyberTheme::TEXT_MUTED).size(11.0));
            ui.add_sized(
                [ui.available_width() - 10.0, 18.0],
                Slider::new(&mut state.sliding_bar_pos, 0.0..=48.0)
                    .show_value(false)
                    .text(""),
            );
        });

        ui.add_space(4.0);

        // 4. 16 VISIBLE STEPS GRID: mapped from sliding_bar_pos
        let start_step = (state.sliding_bar_pos.round() as usize).min(48);
        let visible_count = 16;

        let avail_w = ui.available_width();
        let step_w = ((avail_w - 40.0) / visible_count as f32).max(42.0);

        ui.horizontal(|ui| {
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

        // Note pitch slider (-24 to +24 semitones)
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let base_note = 36 + step.note_offset as i32; // Centered around C2
        let name_idx = (base_note % 12).abs() as usize;
        let octave = (base_note / 12) - 1;
        let note_label = format!("{}{}", note_names[name_idx], octave);

        ui.add_sized(
            [width, 16.0],
            Slider::new(&mut step.note_offset, -24..=24).show_value(false),
        );
        ui.label(nih_plug_egui::egui::RichText::new(note_label).size(10.0).color(CyberTheme::NEON_CYAN));

        // Velocity slider (1 to 127)
        ui.add_sized(
            [width, 14.0],
            Slider::new(&mut step.velocity, 1..=127).show_value(false),
        );

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
