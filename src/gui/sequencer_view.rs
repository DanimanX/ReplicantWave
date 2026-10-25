//! Full-width 64-step neon step matrix.
//!
//! The deck is always visible (no drawer), with transport, DAW sync, key-trig,
//! mode/length/rate, a 64-step radar and 16 editable pad columns.

use super::knob::{draw_rotary_knob_i8, param_chip, param_select, write_b, write_i};
use super::theme::CyberTheme;
use crate::audio::step_sequencer::{Step, MAX_STEPS};
use crate::params::SeqParams;
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui::{
    pos2, vec2, Align, Button, Color32, DragValue, Layout, Rect, RichText, Sense, Stroke,
    StrokeKind, Ui, UiBuilder,
};

#[derive(Clone, Debug)]
pub struct SequencerUiState {
    pub page: usize,
    pub auto_follow: bool,
}

impl Default for SequencerUiState {
    fn default() -> Self {
        Self {
            page: 0,
            auto_follow: true,
        }
    }
}

pub fn draw_sequencer_deck(
    ui: &mut Ui,
    setter: &ParamSetter,
    rect: Rect,
    seq: &SeqParams,
    steps: &mut Vec<Step>,
    state: &mut SequencerUiState,
    active_step: usize,
    t: f32,
) {
    let accent = CyberTheme::NEON_CYAN;
    CyberTheme::card_shell(ui.painter(), rect, accent);
    let content = CyberTheme::card_header(ui.painter(), rect, accent, "00", "STEP MATRIX // 64");
    let inner = Rect::from_min_max(
        pos2(content.min.x, content.min.y - 2.0),
        pos2(rect.max.x - 12.0, rect.max.y - 7.0),
    );

    let running = seq.running.value();
    let length = seq.length.value().clamp(1, MAX_STEPS as i32) as usize;

    if running && state.auto_follow {
        state.page = (active_step / 16).min(3);
    }

    let mut bpm_proxy = seq.bpm.value();

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
        |ui| {
            ui.set_min_width(inner.width());

            // ---- Transport / mode toolbar -------------------------------
            ui.horizontal(|ui| {
                let play_color = if running {
                    CyberTheme::NEON_LIME
                } else {
                    CyberTheme::NEON_CYAN
                };
                let label = if running { "■ STOP" } else { "▶ PLAY" };
                if ui
                    .add(super::neon_button(label, play_color, running))
                    .clicked()
                {
                    write_b(setter, &seq.running, !running);
                    if !running {
                        // Starting: make sure we are not in OFF mode.
                        if seq.mode.value() == 3 {
                            write_i(setter, &seq.mode, 0);
                        }
                        state.auto_follow = true;
                    }
                }
                param_chip(
                    ui,
                    setter,
                    &seq.host_sync,
                    "SYNC",
                    CyberTheme::NEON_CYAN,
                    54.0,
                )
                .on_hover_text("Follow the DAW transport (opt-in)");
                param_chip(
                    ui,
                    setter,
                    &seq.key_trigger,
                    "KEY",
                    CyberTheme::NEON_MAGENTA,
                    46.0,
                )
                .on_hover_text("Transpose an already running sequence from MIDI (opt-in)");

                ui.add_space(4.0);
                ui.label(
                    RichText::new("MODE")
                        .color(CyberTheme::TEXT_DIM)
                        .monospace()
                        .size(7.0),
                );
                let modes = ["SYNTH", "CUTOFF", "WT POS", "OFF"];
                for (i, name) in modes.iter().enumerate() {
                    let selected = seq.mode.value() == i as i32;
                    if ui
                        .add(category_button(
                            name,
                            selected,
                            CyberTheme::NEON_PURPLE,
                            8.0,
                        ))
                        .clicked()
                    {
                        write_i(setter, &seq.mode, i as i32);
                    }
                }

                ui.add_space(4.0);
                ui.label(
                    RichText::new("LEN")
                        .color(CyberTheme::TEXT_DIM)
                        .monospace()
                        .size(7.0),
                );
                for len in [16i32, 32, 48, 64] {
                    if ui
                        .add(category_button(
                            &format!("{}", len),
                            length as i32 == len,
                            CyberTheme::NEON_AMBER,
                            7.5,
                        ))
                        .clicked()
                    {
                        write_i(setter, &seq.length, len);
                    }
                }

                ui.add_space(4.0);
                ui.label(
                    RichText::new("RATE")
                        .color(CyberTheme::TEXT_DIM)
                        .monospace()
                        .size(7.0),
                );
                param_select(
                    ui,
                    setter,
                    &seq.rate,
                    &["1/4", "1/8", "1/8 T", "1/16", "1/16 T", "1/16 D", "1/32"],
                    64.0,
                );

                ui.add_space(4.0);
                ui.label(
                    RichText::new("BPM")
                        .color(CyberTheme::TEXT_DIM)
                        .monospace()
                        .size(7.0),
                );
                ui.add(
                    DragValue::new(&mut bpm_proxy)
                        .range(40.0..=240.0)
                        .speed(0.4)
                        .fixed_decimals(0),
                );
                if (bpm_proxy - seq.bpm.value()).abs() > 0.01 {
                    super::knob::write_f(setter, &seq.bpm, bpm_proxy);
                }
                ui.add_space(4.0);
                super::knob::param_slider(
                    ui,
                    setter,
                    &seq.swing,
                    "SWING",
                    104.0,
                    CyberTheme::NEON_CYAN,
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("STEP {:02}", (active_step % length) + 1))
                            .color(CyberTheme::NEON_LIME)
                            .monospace()
                            .strong()
                            .size(8.0),
                    );
                });
            });

            ui.add_space(2.0);

            // ---- Radar + page selector ----------------------------------
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("PAGE")
                        .color(CyberTheme::TEXT_DIM)
                        .monospace()
                        .size(7.0),
                );
                for page in 0..4usize {
                    let label = format!("{}-{}", page * 16 + 1, page * 16 + 16);
                    if ui
                        .add(category_button(
                            &label,
                            state.page == page,
                            CyberTheme::NEON_AMBER,
                            7.2,
                        ))
                        .clicked()
                    {
                        state.page = page;
                        state.auto_follow = false;
                    }
                }
                ui.add_space(6.0);
                let w = ui.available_width().max(120.0);
                draw_radar(
                    ui,
                    steps,
                    length,
                    running,
                    active_step,
                    state.page,
                    w,
                    18.0,
                    t,
                );
            });

            ui.add_space(2.0);

            // ---- 16 step columns ----------------------------------------
            let page_offset = state.page * 16;
            let num_visible = 16usize;
            let gap = 4.0;
            let col_w = ((ui.available_width() - (num_visible as f32 - 1.0) * gap)
                / num_visible as f32)
                .max(48.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for col in 0..num_visible {
                    let idx = page_offset + col;
                    if idx >= MAX_STEPS {
                        break;
                    }
                    let playing =
                        running && active_step % length == idx && state.page == active_step / 16;
                    draw_step_column(
                        ui,
                        &mut steps[idx],
                        idx,
                        col_w,
                        playing,
                        idx < length,
                        seq.root_note.value() + seq.octave.value() * 12,
                        t,
                    );
                    if col < num_visible - 1 {
                        ui.add_space(gap);
                    }
                }
            });
        },
    );

    CyberTheme::card_chrome(ui.painter(), rect, accent, t);
}

fn draw_step_column(
    ui: &mut Ui,
    step: &mut Step,
    idx: usize,
    width: f32,
    playing: bool,
    in_length: bool,
    base_note: i32,
    t: f32,
) {
    ui.allocate_ui_with_layout(vec2(width, 150.0), Layout::top_down(Align::Center), |ui| {
        // Index + state LED
        ui.horizontal(|ui| {
            let (led, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
            let led_col = if playing {
                CyberTheme::NEON_LIME
            } else if step.gate && in_length {
                CyberTheme::NEON_CYAN
            } else {
                CyberTheme::LED_OFF
            };
            ui.painter().circle_filled(led.center(), 2.6, led_col);
            ui.label(
                RichText::new(format!("{:02}", idx + 1))
                    .color(if playing {
                        CyberTheme::NEON_LIME
                    } else if in_length {
                        CyberTheme::TEXT_MUTED
                    } else {
                        CyberTheme::TEXT_DIM
                    })
                    .monospace()
                    .strong()
                    .size(7.5),
            );
        });

        // Gate pad
        let pad_h = 56.0;
        let (pad_rect, pad_resp) = ui.allocate_exact_size(vec2(width - 4.0, pad_h), Sense::click());
        if pad_resp.clicked() {
            step.gate = !step.gate;
        }
        let accent = if step.accent {
            CyberTheme::NEON_AMBER
        } else if step.slide {
            CyberTheme::NEON_MAGENTA
        } else {
            CyberTheme::NEON_CYAN
        };
        let on = step.gate && in_length;
        let fill = if on {
            CyberTheme::alpha(accent, if playing { 90 } else { 45 })
        } else {
            Color32::from_rgb(8, 5, 18)
        };
        let stroke = if playing {
            Stroke::new(2.0_f32, CyberTheme::NEON_LIME)
        } else {
            Stroke::new(1.0_f32, if on { accent } else { CyberTheme::BORDER_DARK })
        };
        let cut = 6.0;
        let pts = vec![
            pos2(pad_rect.min.x + cut, pad_rect.min.y),
            pos2(pad_rect.max.x, pad_rect.min.y),
            pos2(pad_rect.max.x, pad_rect.max.y - cut),
            pos2(pad_rect.max.x - cut, pad_rect.max.y),
            pos2(pad_rect.min.x, pad_rect.max.y),
            pos2(pad_rect.min.x, pad_rect.min.y + cut),
        ];
        ui.painter().add(nih_plug_egui::egui::Shape::convex_polygon(
            pts, fill, stroke,
        ));
        if playing {
            let bloom =
                CyberTheme::alpha(CyberTheme::NEON_LIME, (40.0 + 50.0 * (t * 8.0).sin()) as u8);
            ui.painter().rect_stroke(
                pad_rect.expand(2.0),
                2.0,
                Stroke::new(2.0_f32, bloom),
                StrokeKind::Outside,
            );
        }
        // Note name
        let note = (base_note + step.note_offset as i32).clamp(0, 127);
        ui.painter().text(
            pos2(pad_rect.center().x, pad_rect.min.y + 14.0),
            nih_plug_egui::egui::Align2::CENTER_CENTER,
            note_name(note),
            nih_plug_egui::egui::FontId::monospace(11.0),
            if on {
                CyberTheme::TEXT_PRIMARY
            } else {
                CyberTheme::TEXT_DIM
            },
        );
        ui.painter().text(
            pos2(pad_rect.center().x, pad_rect.min.y + 30.0),
            nih_plug_egui::egui::Align2::CENTER_CENTER,
            if on { "ON" } else { "OFF" },
            nih_plug_egui::egui::FontId::monospace(7.0),
            if on { accent } else { CyberTheme::TEXT_DIM },
        );
        // Velocity bar at the bottom of the pad.
        let vel = step.velocity as f32 / 127.0;
        let bar = Rect::from_min_max(
            pos2(pad_rect.min.x + 6.0, pad_rect.max.y - 5.0),
            pos2(
                pad_rect.min.x + 6.0 + (pad_rect.width() - 12.0) * vel,
                pad_rect.max.y - 3.0,
            ),
        );
        ui.painter().rect_filled(
            bar,
            1.0,
            CyberTheme::alpha(accent, if on { 200 } else { 50 }),
        );

        // Accent / slide toggles
        ui.horizontal(|ui| {
            if ui
                .add(step_chip(
                    "A",
                    step.accent,
                    CyberTheme::NEON_AMBER,
                    width * 0.42,
                ))
                .on_hover_text("Accent")
                .clicked()
            {
                step.accent = !step.accent;
            }
            if ui
                .add(step_chip(
                    "S",
                    step.slide,
                    CyberTheme::NEON_MAGENTA,
                    width * 0.42,
                ))
                .on_hover_text("Slide / legato")
                .clicked()
            {
                step.slide = !step.slide;
            }
        });

        // Pitch knob
        let note_label = note_name(note);
        draw_rotary_knob_i8(ui, &mut step.note_offset, -24..=24, &note_label, accent);
    });
}

fn step_chip(label: &str, on: bool, color: Color32, width: f32) -> Button<'static> {
    Button::new(
        RichText::new(label.to_string())
            .color(if on {
                CyberTheme::BG_VOID
            } else {
                CyberTheme::TEXT_DIM
            })
            .monospace()
            .strong()
            .size(7.5),
    )
    .fill(if on {
        color
    } else {
        Color32::from_rgb(10, 6, 22)
    })
    .stroke(Stroke::new(
        1.0_f32,
        if on { color } else { CyberTheme::BORDER_DARK },
    ))
    .corner_radius(1.0)
    .min_size(vec2(width, 15.0))
}

fn category_button(text: &str, selected: bool, color: Color32, size: f32) -> Button<'static> {
    Button::new(
        RichText::new(text.to_string())
            .color(if selected {
                CyberTheme::BG_VOID
            } else {
                CyberTheme::TEXT_MUTED
            })
            .monospace()
            .strong()
            .size(size),
    )
    .fill(if selected {
        color
    } else {
        Color32::from_rgb(10, 6, 22)
    })
    .stroke(Stroke::new(
        1.0_f32,
        if selected {
            color
        } else {
            CyberTheme::BORDER_DARK
        },
    ))
    .corner_radius(1.5)
}

fn note_name(note: i32) -> String {
    let names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let n = note.rem_euclid(12) as usize;
    let oct = note.div_euclid(12) - 1;
    format!("{}{}", names[n], oct)
}

#[allow(clippy::too_many_arguments)]
fn draw_radar(
    ui: &mut Ui,
    steps: &[Step],
    length: usize,
    running: bool,
    active_step: usize,
    page: usize,
    width: f32,
    height: f32,
    t: f32,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 1.0, Color32::from_rgb(4, 2, 11));
    p.rect_stroke(
        rect,
        1.0,
        Stroke::new(1.0_f32, CyberTheme::BORDER_DARK),
        StrokeKind::Inside,
    );

    let step_w = rect.width() / MAX_STEPS as f32;
    for i in 0..MAX_STEPS {
        let x = rect.min.x + i as f32 * step_w;
        let step = &steps[i];
        let in_len = i < length;
        if in_len && step.gate {
            let norm = ((step.note_offset as f32 + 24.0) / 48.0).clamp(0.15, 0.9);
            let h = 4.0 + norm * (rect.height() - 7.0);
            let y = rect.max.y - 2.0 - h;
            let col = if i == active_step {
                CyberTheme::NEON_LIME
            } else if step.accent {
                CyberTheme::NEON_AMBER
            } else {
                CyberTheme::alpha(CyberTheme::NEON_CYAN, 180)
            };
            p.rect_filled(
                Rect::from_min_size(pos2(x + 0.5, y), vec2((step_w - 1.0).max(1.0), h)),
                0.5,
                col,
            );
        }
    }

    // Viewport bracket
    let vp = Rect::from_min_max(
        pos2(rect.min.x + page as f32 * 16.0 * step_w, rect.min.y + 1.0),
        pos2(
            rect.min.x + (page as f32 + 1.0) * 16.0 * step_w,
            rect.max.y - 1.0,
        ),
    );
    p.rect_stroke(
        vp,
        1.0,
        Stroke::new(1.0_f32, CyberTheme::NEON_AMBER),
        StrokeKind::Inside,
    );

    // Animated playhead
    if running {
        let x = rect.min.x + (active_step as f32 + 0.5) * step_w;
        let a = (120.0 + 100.0 * (t * 9.0).sin()) as u8;
        p.line_segment(
            [pos2(x, rect.min.y + 1.0), pos2(x, rect.max.y - 1.0)],
            Stroke::new(1.6_f32, CyberTheme::alpha(CyberTheme::NEON_LIME, a)),
        );
    }
}
