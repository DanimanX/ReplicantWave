//! NIGHT CITY HUD — radical cyberpunk editor for ReplicantWave.
//!
//! Layout: HUD header, preset bank strip, 2x2 synth grid + control deck, and a
//! full-width 64-step neon matrix. Painting uses a detached `UiModel` snapshot;
//! the engine mutex is only touched for microseconds at preset load/apply time.

pub mod knob;
pub mod model;
pub mod oscilloscope;
pub mod sequencer_view;
pub mod theme;
pub mod wavetable_view;

use crate::audio::filter::FilterType;
use crate::audio::fx::distortion::DistortionMode;
use crate::audio::oscillator::NoiseType;
use crate::audio::SynthEngine;
use crate::presets::factory::get_factory_presets;
use crate::presets::SynthPreset;
use knob::{draw_rotary_knob, draw_rotary_knob_log, draw_rotary_knob_usize};
use model::{EnvUi, FxUi, OscUi, UiModel};
use oscilloscope::draw_oscilloscope;
use sequencer_view::{draw_sequencer_deck, SequencerUiState};
use theme::CyberTheme;
use wavetable_view::{draw_wavetable_preview, WaveThumb};

use nih_plug_egui::egui::{
    pos2, vec2, Align, Button, Color32, ComboBox, Layout, Pos2, Rect, Response, RichText, Sense,
    Stroke, StrokeKind, Ui, UiBuilder,
};
use parking_lot::Mutex;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub const DONATION_URL: &str = "https://paypal.me/DanimanX?locale.x=en_US&country.x=CL";

const CATEGORIES: [&str; 9] = [
    "ALL", "BASS", "PAD", "DRONE", "LEAD", "ARP", "SEQ", "FX", "OTHER",
];

pub struct GuiSharedState {
    pub engine: Arc<Mutex<SynthEngine>>,
    pub scope: Arc<Mutex<crate::audio::ScopeBuffer>>,
    pub active_voices: Arc<AtomicUsize>,
    pub model: UiModel,
    pub presets: Vec<SynthPreset>,
    pub current_preset_idx: usize,
    pub pending_preset: Option<usize>,
    pub wave_thumbs: Vec<WaveThumb>,
    pub wave_names: Vec<String>,
    pub seq_ui_state: SequencerUiState,
    pub category: usize,
    pub live_step: usize,
}

impl GuiSharedState {
    pub fn new(engine: Arc<Mutex<SynthEngine>>) -> Self {
        let presets = get_factory_presets();
        let (model, scope, active_voices, wave_thumbs, wave_names) = {
            let mut guard = engine.lock();
            // Start on the first factory preset instead of the bare defaults.
            if let Some(p) = presets.first() {
                p.apply_to_engine(&mut guard);
            }
            let thumbs = guard
                .bank
                .tables
                .iter()
                .map(WaveThumb::from_wavetable)
                .collect();
            (
                UiModel::capture(&guard),
                guard.scope_data.clone(),
                guard.active_voice_count.clone(),
                thumbs,
                guard.bank.names(),
            )
        };

        Self {
            engine,
            scope,
            active_voices,
            model,
            presets,
            current_preset_idx: 0,
            pending_preset: None,
            wave_thumbs,
            wave_names,
            seq_ui_state: SequencerUiState::default(),
            category: 0,
            live_step: 0,
        }
    }

    fn queue_preset(&mut self, idx: usize) {
        if idx < self.presets.len() {
            self.current_preset_idx = idx;
            self.pending_preset = Some(idx);
        }
    }

    fn filtered_presets(&self) -> Vec<usize> {
        if self.category == 0 {
            return (0..self.presets.len()).collect();
        }
        let group = CATEGORIES[self.category];
        (0..self.presets.len())
            .filter(|&i| preset_group(&self.presets[i].category) == group)
            .collect()
    }
}

pub fn render_cyberpunk_gui(ui: &mut Ui, state: &mut GuiSharedState) {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f32())
        .unwrap_or(0.0);

    // 60 FPS for the animated scopes, without starving the audio thread.
    ui.ctx().request_repaint_after(Duration::from_millis(16));

    // Apply a queued preset with a non-blocking lock. Never hold the engine
    // mutex while painting.
    if let Some(idx) = state.pending_preset {
        if let Some(mut engine) = state.engine.try_lock() {
            if let Some(p) = state.presets.get(idx) {
                p.apply_to_engine(&mut engine);
                state.model = UiModel::capture(&engine);
            }
            state.pending_preset = None;
        }
    }

    // Live telemetry (separate atomics / short try_lock, no paint-time lock).
    if let Some(engine) = state.engine.try_lock() {
        state.live_step = engine.sequencer.current_step;
    }
    let active_voices = state.active_voices.load(Ordering::Relaxed);
    let scope_samples = {
        let mut out = [0.0f32; crate::audio::SCOPE_BUFFER_SIZE];
        if let Some(scope) = state.scope.try_lock() {
            for (i, s) in out.iter_mut().enumerate() {
                *s = scope.buffer[(scope.write_idx + i) % crate::audio::SCOPE_BUFFER_SIZE];
            }
        }
        out
    };

    // Base style.
    {
        let style = ui.style_mut();
        style.spacing.item_spacing = vec2(5.0, 3.0);
        style.spacing.button_padding = vec2(7.0, 2.0);
        style.visuals.panel_fill = CyberTheme::BG_VOID;
        style.visuals.window_fill = CyberTheme::BG_PANEL_ALT;
        style.visuals.extreme_bg_color = CyberTheme::BG_RECESSED;
        style.visuals.faint_bg_color = CyberTheme::BG_PANEL;
        style.visuals.hyperlink_color = CyberTheme::NEON_CYAN;
        style.visuals.selection.bg_fill = CyberTheme::alpha(CyberTheme::NEON_CYAN, 55);
        style.visuals.selection.stroke = Stroke::new(1.0_f32, CyberTheme::NEON_CYAN);
        style.visuals.widgets.noninteractive.fg_stroke =
            Stroke::new(1.0_f32, CyberTheme::TEXT_MUTED);
        style.visuals.widgets.inactive.bg_fill = CyberTheme::BG_RECESSED;
        style.visuals.widgets.inactive.weak_bg_fill = CyberTheme::BG_RECESSED;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, CyberTheme::TEXT_PRIMARY);
        style.visuals.widgets.inactive.corner_radius = 2.into();
        style.visuals.widgets.hovered.bg_fill = CyberTheme::alpha(CyberTheme::NEON_CYAN, 30);
        style.visuals.widgets.hovered.weak_bg_fill = CyberTheme::alpha(CyberTheme::NEON_CYAN, 30);
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, CyberTheme::NEON_CYAN);
        style.visuals.widgets.hovered.corner_radius = 2.into();
        style.visuals.widgets.active.bg_fill = CyberTheme::alpha(CyberTheme::NEON_CYAN, 60);
        style.visuals.widgets.active.weak_bg_fill = CyberTheme::alpha(CyberTheme::NEON_CYAN, 60);
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
        style.visuals.widgets.active.corner_radius = 2.into();
    }

    let full = ui.max_rect();
    CyberTheme::draw_backdrop(ui.painter(), full, t);

    let root = full.shrink(10.0);
    let top_h = 54.0;
    let strip_h = 30.0;
    let seq_h = 260.0;
    let gap = 6.0;
    let main_h = (root.height() - top_h - strip_h - seq_h - gap * 3.0).max(420.0);

    let top_rect = Rect::from_min_size(root.min, vec2(root.width(), top_h));
    let strip_rect = Rect::from_min_size(
        pos2(root.min.x, top_rect.max.y + gap),
        vec2(root.width(), strip_h),
    );
    let main_rect = Rect::from_min_size(
        pos2(root.min.x, strip_rect.max.y + gap),
        vec2(root.width(), main_h),
    );
    let seq_rect = Rect::from_min_size(
        pos2(root.min.x, main_rect.max.y + gap),
        vec2(root.width(), seq_h),
    );

    draw_top_bar(ui, top_rect, state, &scope_samples, active_voices, t);
    draw_category_strip(ui, strip_rect, state, t);
    draw_main_area(ui, main_rect, state, &scope_samples, active_voices, t);
    draw_sequencer_deck(
        ui,
        seq_rect,
        &mut state.model.seq,
        &mut state.seq_ui_state,
        state.live_step,
        t,
    );

    CyberTheme::draw_overlay(ui.painter(), full);

    // Push the final model back without ever blocking the audio callback.
    if let Some(mut engine) = state.engine.try_lock() {
        state.model.apply(&mut engine);
    }
}

fn preset_group(category: &str) -> &'static str {
    match category.to_ascii_uppercase().as_str() {
        "BASS" | "SUB" => "BASS",
        "PAD" | "CHOIR" | "ATMOSPHERE" => "PAD",
        "DRONE" => "DRONE",
        "LEAD" | "STAB" | "ACID" | "SYNC" => "LEAD",
        "ARP" => "ARP",
        "SEQUENCE" | "COMBAT" | "RAVE" => "SEQ",
        "FX" | "GLITCH" => "FX",
        _ => "OTHER",
    }
}

fn group_color(group: &str) -> Color32 {
    match group {
        "BASS" => CyberTheme::NEON_AMBER,
        "PAD" => CyberTheme::NEON_PURPLE,
        "DRONE" => CyberTheme::NEON_BLUE,
        "LEAD" => CyberTheme::NEON_MAGENTA,
        "ARP" => CyberTheme::NEON_LIME,
        "SEQ" => CyberTheme::NEON_CYAN,
        "FX" => CyberTheme::NEON_RED,
        _ => CyberTheme::NEON_AMBER,
    }
}

// ---------------------------------------------------------------------------
// Panels
// ---------------------------------------------------------------------------

fn panel(
    ui: &mut Ui,
    rect: Rect,
    accent: Color32,
    index: &str,
    title: &str,
    t: f32,
    add: impl FnOnce(&mut Ui),
) {
    CyberTheme::card_shell(ui.painter(), rect, accent);
    let content = CyberTheme::card_header(ui.painter(), rect, accent, index, title);
    let inner = Rect::from_min_max(
        pos2(content.min.x, content.min.y),
        pos2(rect.max.x - 12.0, rect.max.y - 7.0),
    );
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
        |ui| {
            ui.set_min_width(inner.width());
            add(ui);
        },
    );
    CyberTheme::card_chrome(ui.painter(), rect, accent, t);
}

fn draw_top_bar(
    ui: &mut Ui,
    rect: Rect,
    state: &mut GuiSharedState,
    _scope: &[f32],
    active_voices: usize,
    t: f32,
) {
    let accent = CyberTheme::NEON_CYAN;
    CyberTheme::card_shell(ui.painter(), rect, accent);

    // Logo block (painted for precise HUD typography).
    let p = ui.painter();
    let lx = rect.min.x + 18.0;
    let ly = rect.min.y + 11.0;
    // Hexagonal emblem
    let c = pos2(lx + 12.0, ly + 15.0);
    let hex: Vec<Pos2> = (0..6)
        .map(|i| {
            let a = PI / 6.0 + i as f32 * PI / 3.0;
            pos2(c.x + a.cos() * 13.0, c.y + a.sin() * 13.0)
        })
        .collect();
    p.add(nih_plug_egui::egui::Shape::convex_polygon(
        hex,
        CyberTheme::alpha(accent, 26),
        Stroke::new(1.4_f32, accent),
    ));
    p.text(
        c,
        nih_plug_egui::egui::Align2::CENTER_CENTER,
        "RW",
        nih_plug_egui::egui::FontId::monospace(10.0),
        accent,
    );

    p.text(
        pos2(lx + 34.0, ly),
        nih_plug_egui::egui::Align2::LEFT_TOP,
        "REPLICANT",
        nih_plug_egui::egui::FontId::monospace(17.0),
        CyberTheme::TEXT_PRIMARY,
    );
    p.text(
        pos2(lx + 137.0, ly),
        nih_plug_egui::egui::Align2::LEFT_TOP,
        "WAVE",
        nih_plug_egui::egui::FontId::monospace(17.0),
        CyberTheme::NEON_MAGENTA,
    );
    p.text(
        pos2(lx + 35.0, ly + 23.0),
        nih_plug_egui::egui::Align2::LEFT_TOP,
        "NIGHT CITY AUDIO OS // NEON SERIES",
        nih_plug_egui::egui::FontId::monospace(6.8),
        CyberTheme::TEXT_DIM,
    );

    // Preset browser
    let bx = rect.min.x + 290.0;
    let by = rect.min.y + 9.0;
    let indices = state.filtered_presets();
    let pos = indices
        .iter()
        .position(|&i| i == state.current_preset_idx)
        .unwrap_or(0);

    let mut clicked_preset: Option<usize> = None;
    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(Rect::from_min_size(pos2(bx, by), vec2(470.0, 38.0)))
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            if ui
                .add(neon_button("◀", CyberTheme::NEON_CYAN, false))
                .on_hover_text("Previous preset")
                .clicked()
            {
                if !indices.is_empty() {
                    let ni = if pos == 0 { indices.len() - 1 } else { pos - 1 };
                    clicked_preset = Some(indices[ni]);
                }
            }

            let cur = &state.presets[state.current_preset_idx];
            let color = group_color(preset_group(&cur.category));
            ui.label(
                RichText::new(format!("{:03}", state.current_preset_idx + 1))
                    .color(CyberTheme::TEXT_DIM)
                    .monospace()
                    .size(9.0),
            );
            ComboBox::from_id_salt("night_city_preset")
                .selected_text(
                    RichText::new(format!("{}  //  {}", cur.name, cur.category.to_uppercase()))
                        .color(color)
                        .monospace()
                        .size(11.0),
                )
                .width(250.0)
                .show_ui(ui, |ui| {
                    for &idx in &indices {
                        let p = &state.presets[idx];
                        let label = format!("{:03}  {}  [{}]", idx + 1, p.name, p.category);
                        if ui
                            .selectable_label(idx == state.current_preset_idx, label)
                            .clicked()
                        {
                            clicked_preset = Some(idx);
                        }
                    }
                });

            if ui
                .add(neon_button("▶", CyberTheme::NEON_CYAN, false))
                .on_hover_text("Next preset")
                .clicked()
            {
                if !indices.is_empty() {
                    let ni = (pos + 1) % indices.len();
                    clicked_preset = Some(indices[ni]);
                }
            }
        },
    );
    if let Some(idx) = clicked_preset {
        state.queue_preset(idx);
    }

    // Voices LED meter + master + donate
    let right = rect.max.x - 18.0;
    let master_rect = Rect::from_min_size(pos2(right - 210.0, rect.min.y + 5.0), vec2(48.0, 44.0));
    ui.allocate_new_ui(UiBuilder::new().max_rect(master_rect), |ui| {
        draw_rotary_knob(
            ui,
            &mut state.model.master,
            0.0..=1.5,
            "MASTER",
            CyberTheme::NEON_AMBER,
        );
    });

    ui.allocate_new_ui(
        UiBuilder::new().max_rect(Rect::from_min_size(
            pos2(right - 330.0, rect.min.y + 10.0),
            vec2(112.0, 36.0),
        )),
        |ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(format!("VOICES {:02}/16", active_voices))
                        .color(if active_voices > 0 {
                            CyberTheme::NEON_LIME
                        } else {
                            CyberTheme::TEXT_MUTED
                        })
                        .monospace()
                        .strong()
                        .size(8.5),
                );
                let (r, _) = ui.allocate_exact_size(vec2(104.0, 6.0), Sense::hover());
                CyberTheme::led_meter(
                    ui.painter(),
                    r,
                    active_voices as f32 / 16.0,
                    CyberTheme::NEON_LIME,
                    16,
                    false,
                );
                ui.label(
                    RichText::new(format!("STEP {:02}", state.live_step + 1))
                        .color(CyberTheme::TEXT_DIM)
                        .monospace()
                        .size(7.0),
                );
            });
        },
    );

    if ui
        .allocate_new_ui(
            UiBuilder::new().max_rect(Rect::from_min_size(
                pos2(right - 430.0, rect.min.y + 14.0),
                vec2(90.0, 26.0),
            )),
            |ui| {
                ui.add(neon_button("DONATE ☕", CyberTheme::NEON_AMBER, true))
                    .on_hover_text("Support ReplicantWave on PayPal")
            },
        )
        .inner
        .clicked()
    {
        let _ = open::that(DONATION_URL);
    }

    // Animated scan tick marks on the right edge.
    for i in 0..6 {
        let x = rect.max.x - 8.0 - i as f32 * 4.0;
        let a = (60.0 + 40.0 * (t * 2.0 + i as f32).sin()) as u8;
        ui.painter().line_segment(
            [pos2(x, rect.min.y + 8.0), pos2(x, rect.max.y - 8.0)],
            Stroke::new(1.0_f32, CyberTheme::alpha(CyberTheme::NEON_CYAN, a / 4)),
        );
    }
}

fn draw_category_strip(ui: &mut Ui, rect: Rect, state: &mut GuiSharedState, t: f32) {
    CyberTheme::card_shell(ui.painter(), rect, CyberTheme::NEON_PURPLE);
    let inner = rect.shrink2(vec2(12.0, 4.0));
    let mut new_category = state.category;
    ui.allocate_new_ui(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            ui.label(
                RichText::new("BANK")
                    .color(CyberTheme::NEON_PURPLE)
                    .monospace()
                    .strong()
                    .size(8.5),
            );
            for (i, cat) in CATEGORIES.iter().enumerate() {
                let color = if *cat == "ALL" {
                    CyberTheme::NEON_CYAN
                } else {
                    group_color(cat)
                };
                if ui
                    .add(category_button(cat, state.category == i, color))
                    .clicked()
                {
                    new_category = i;
                }
            }

            ui.add_space(8.0);
            let count = if state.category == 0 {
                state.presets.len()
            } else {
                state.filtered_presets().len()
            };
            ui.label(
                RichText::new(format!("{:02} PATCHES", count))
                    .color(CyberTheme::TEXT_DIM)
                    .monospace()
                    .size(7.5),
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(neon_button("RND SEQ", CyberTheme::NEON_LIME, false))
                    .on_hover_text("Generate a new cyberpunk pattern")
                    .clicked()
                {
                    randomize_steps(&mut state.model.seq.steps, t.to_bits() ^ 0x9e37_79b9);
                }
                ui.label(
                    RichText::new("64-STEP SLIDING MATRIX")
                        .color(CyberTheme::alpha(CyberTheme::NEON_CYAN, 150))
                        .monospace()
                        .size(7.5),
                );
            });
        },
    );

    if new_category != state.category {
        state.category = new_category;
        if new_category != 0 {
            let list = state.filtered_presets();
            if !list.is_empty() && !list.contains(&state.current_preset_idx) {
                let idx = list[0];
                state.queue_preset(idx);
            }
        }
    }
}

fn draw_main_area(
    ui: &mut Ui,
    rect: Rect,
    state: &mut GuiSharedState,
    scope: &[f32],
    active_voices: usize,
    t: f32,
) {
    let gap = 8.0;
    let left_w = 610.0;
    let card_w = (left_w - gap) * 0.5;
    let row_h = (rect.height() - gap) * 0.5;
    let right_x = rect.min.x + left_w + gap;
    let right_w = rect.max.x - right_x;

    // OSC 1 / OSC 2
    let osc1_rect = Rect::from_min_size(rect.min, vec2(card_w, row_h));
    let osc2_rect = Rect::from_min_size(
        pos2(rect.min.x + card_w + gap, rect.min.y),
        vec2(card_w, row_h),
    );
    draw_osc_card(
        ui,
        osc1_rect,
        "01",
        "OSCILLATOR // A",
        CyberTheme::NEON_CYAN,
        &mut state.model.osc1,
        &state.wave_thumbs,
        &state.wave_names,
        t,
    );
    draw_osc_card(
        ui,
        osc2_rect,
        "02",
        "OSCILLATOR // B",
        CyberTheme::NEON_MAGENTA,
        &mut state.model.osc2,
        &state.wave_thumbs,
        &state.wave_names,
        t,
    );

    // SUB + NOISE / FILTER
    let low_y = rect.min.y + row_h + gap;
    let sub_rect = Rect::from_min_size(pos2(rect.min.x, low_y), vec2(card_w, row_h));
    let filter_rect =
        Rect::from_min_size(pos2(rect.min.x + card_w + gap, low_y), vec2(card_w, row_h));
    draw_sub_noise_card(ui, sub_rect, state, t);
    draw_filter_card(ui, filter_rect, state, t);

    // Right column: scope, envelopes, FX
    let scope_h = 158.0;
    let env_h = 172.0;
    let scope_rect = Rect::from_min_size(pos2(right_x, rect.min.y), vec2(right_w, scope_h));
    draw_scope_card(ui, scope_rect, scope, active_voices, state, t);

    let env_y = rect.min.y + scope_h + gap;
    let env_w = (right_w - gap) * 0.5;
    let amp_rect = Rect::from_min_size(pos2(right_x, env_y), vec2(env_w, env_h));
    let flt_env_rect = Rect::from_min_size(pos2(right_x + env_w + gap, env_y), vec2(env_w, env_h));
    draw_env_card(
        ui,
        amp_rect,
        "06",
        "AMP ENVELOPE",
        CyberTheme::NEON_PINK,
        &mut state.model.amp_env,
        t,
    );
    draw_env_card(
        ui,
        flt_env_rect,
        "07",
        "FILTER ENVELOPE",
        CyberTheme::NEON_CYAN,
        &mut state.model.filter_env,
        t,
    );

    let fx_y = env_y + env_h + gap;
    let fx_rect = Rect::from_min_size(
        pos2(right_x, fx_y),
        vec2(right_w, (rect.max.y - fx_y).max(120.0)),
    );
    draw_fx_card(ui, fx_rect, &mut state.model.fx, t);
}

fn draw_osc_card(
    ui: &mut Ui,
    rect: Rect,
    index: &str,
    title: &str,
    color: Color32,
    osc: &mut OscUi,
    thumbs: &[WaveThumb],
    names: &[String],
    t: f32,
) {
    panel(ui, rect, color, index, title, t, |ui| {
        // header + table selection
        ui.horizontal(|ui| {
            small_chip(ui, &mut osc.enabled, "ON", color, 48.0);
            ui.label(
                RichText::new("TABLE")
                    .color(CyberTheme::TEXT_DIM)
                    .monospace()
                    .size(7.0),
            );
            ComboBox::from_id_salt(format!("wt_combo_{}", index))
                .selected_text(
                    RichText::new(names.get(osc.table).cloned().unwrap_or_else(|| "—".into()))
                        .color(color)
                        .monospace()
                        .size(9.0),
                )
                .width(150.0)
                .show_ui(ui, |ui| {
                    for (i, name) in names.iter().enumerate() {
                        ui.selectable_value(&mut osc.table, i, name);
                    }
                });
        });

        if let Some(thumb) = thumbs.get(osc.table % thumbs.len().max(1)) {
            draw_wavetable_preview(ui, thumb, osc.pos, color, ui.available_width(), 96.0);
        }

        ui.horizontal(|ui| {
            draw_rotary_knob(ui, &mut osc.pos, 0.0..=1.0, "POS", color);
            draw_rotary_knob(ui, &mut osc.tune, -36.0..=36.0, "TUNE", color);
            draw_rotary_knob_usize(ui, &mut osc.unison, 1..=8, "UNISON", color);
            draw_rotary_knob(ui, &mut osc.detune, 0.0..=1.0, "DETUNE", color);
            draw_rotary_knob(ui, &mut osc.fine, -100.0..=100.0, "FINE", color);
        });
        ui.horizontal(|ui| {
            draw_rotary_knob(ui, &mut osc.level, 0.0..=1.5, "LEVEL", color);
            draw_rotary_knob(ui, &mut osc.spread, 0.0..=1.0, "SPREAD", color);
            draw_rotary_knob(ui, &mut osc.formant, -1.0..=1.0, "FORMANT", color);
            draw_rotary_knob(ui, &mut osc.bend, -1.0..=1.0, "BEND", color);
            draw_rotary_knob(ui, &mut osc.pan, -1.0..=1.0, "PAN", color);
        });
    });
}

fn draw_sub_noise_card(ui: &mut Ui, rect: Rect, state: &mut GuiSharedState, t: f32) {
    let color = CyberTheme::NEON_LIME;
    let model = &mut state.model;
    panel(ui, rect, color, "03", "SUB + NOISE", t, |ui| {
        ui.horizontal(|ui| {
            small_chip(ui, &mut model.sub_enabled, "SUB", color, 48.0);
            let waves = ["SINE", "TRI", "SAW", "PULSE 50", "PULSE 25", "PULSE 12"];
            ComboBox::from_id_salt("sub_wave")
                .selected_text(waves[model.sub_wave.min(waves.len() - 1)])
                .width(96.0)
                .show_ui(ui, |ui| {
                    for (i, w) in waves.iter().enumerate() {
                        ui.selectable_value(&mut model.sub_wave, i, *w);
                    }
                });
            for oct in [-2i8, -1, 0] {
                if ui
                    .add(category_button(
                        &format!("{:+}", oct),
                        model.sub_octave == oct,
                        CyberTheme::NEON_LIME,
                    ))
                    .clicked()
                {
                    model.sub_octave = oct;
                }
            }
        });
        ui.horizontal(|ui| {
            draw_rotary_knob(ui, &mut model.sub_level, 0.0..=1.0, "S LVL", color);
            draw_rotary_knob(ui, &mut model.sub_pan, -1.0..=1.0, "S PAN", color);
        });

        ui.add_space(3.0);
        ui.painter().line_segment(
            [
                pos2(ui.min_rect().min.x, ui.cursor().min.y),
                pos2(ui.min_rect().max.x, ui.cursor().min.y),
            ],
            Stroke::new(1.0_f32, CyberTheme::alpha(color, 40)),
        );
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            small_chip(
                ui,
                &mut model.noise_enabled,
                "NOISE",
                CyberTheme::NEON_CYAN,
                58.0,
            );
            let kinds = ["WHITE", "PINK", "CYBERBIT"];
            let idx = match model.noise_type {
                NoiseType::White => 0,
                NoiseType::Pink => 1,
                NoiseType::CyberBit => 2,
            };
            let mut sel = idx;
            ComboBox::from_id_salt("noise_type")
                .selected_text(kinds[idx])
                .width(96.0)
                .show_ui(ui, |ui| {
                    for (i, k) in kinds.iter().enumerate() {
                        ui.selectable_value(&mut sel, i, *k);
                    }
                });
            model.noise_type = match sel {
                0 => NoiseType::White,
                1 => NoiseType::Pink,
                _ => NoiseType::CyberBit,
            };
        });
        ui.horizontal(|ui| {
            draw_rotary_knob(
                ui,
                &mut model.noise_level,
                0.0..=1.0,
                "N LVL",
                CyberTheme::NEON_CYAN,
            );
            draw_rotary_knob(
                ui,
                &mut model.noise_pan,
                -1.0..=1.0,
                "N PAN",
                CyberTheme::NEON_CYAN,
            );
        });
    });
}

fn draw_filter_card(ui: &mut Ui, rect: Rect, state: &mut GuiSharedState, t: f32) {
    let color = CyberTheme::NEON_AMBER;
    let f = &mut state.model.filter;
    panel(ui, rect, color, "04", "MULTIMODE FILTER", t, |ui| {
        ui.horizontal(|ui| {
            small_chip(ui, &mut f.enabled, "ON", color, 44.0);
            let kinds = [
                "LP 12", "LP 24", "HP 12", "HP 24", "BP 12", "BP 24", "NOTCH",
            ];
            let idx = match f.kind {
                FilterType::LowPass12 => 0,
                FilterType::LowPass24 => 1,
                FilterType::HighPass12 => 2,
                FilterType::HighPass24 => 3,
                FilterType::BandPass12 => 4,
                FilterType::BandPass24 => 5,
                FilterType::Notch => 6,
            };
            let mut sel = idx;
            ComboBox::from_id_salt("filter_type")
                .selected_text(kinds[idx])
                .width(82.0)
                .show_ui(ui, |ui| {
                    for (i, k) in kinds.iter().enumerate() {
                        ui.selectable_value(&mut sel, i, *k);
                    }
                });
            f.kind = FilterType::from_index(sel);
            ui.label(
                RichText::new("ZDF")
                    .color(CyberTheme::TEXT_DIM)
                    .monospace()
                    .size(7.0),
            );
        });
        ui.horizontal(|ui| {
            draw_rotary_knob_log(ui, &mut f.cutoff, 20.0..=18000.0, "CUTOFF", color);
            draw_rotary_knob(ui, &mut f.reso, 0.0..=1.0, "RESO", color);
            draw_rotary_knob(ui, &mut f.drive, 1.0..=4.0, "DRIVE", CyberTheme::NEON_PINK);
            draw_rotary_knob(ui, &mut f.env_amount, -1.0..=1.0, "ENV MOD", color);
            draw_rotary_knob(ui, &mut f.key, 0.0..=1.0, "KEY", CyberTheme::NEON_CYAN);
        });
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("ROUTE")
                    .color(CyberTheme::TEXT_DIM)
                    .monospace()
                    .size(7.0),
            );
            small_chip(ui, &mut f.route_osc1, "OSC1", CyberTheme::NEON_CYAN, 52.0);
            small_chip(
                ui,
                &mut f.route_osc2,
                "OSC2",
                CyberTheme::NEON_MAGENTA,
                52.0,
            );
            small_chip(ui, &mut f.route_sub, "SUB", CyberTheme::NEON_LIME, 44.0);
            small_chip(
                ui,
                &mut f.route_noise,
                "NOISE",
                CyberTheme::NEON_AMBER,
                56.0,
            );
            draw_rotary_knob(ui, &mut f.vel, 0.0..=1.0, "VEL", CyberTheme::NEON_CYAN);
        });
        // ZDF magnitude response, so the filter can be read at a glance.
        let h = ui.available_height().min(52.0).max(34.0);
        if h > 30.0 {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
            draw_filter_response(ui, rect, f.kind, f.cutoff, f.reso, color);
        }
    });
}

fn draw_filter_response(
    ui: &mut Ui,
    rect: Rect,
    kind: FilterType,
    cutoff: f32,
    reso: f32,
    color: Color32,
) {
    let p = ui.painter();
    p.rect_filled(rect, 2.0, Color32::from_rgb(4, 2, 11));
    p.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, CyberTheme::alpha(color, 35)),
        StrokeKind::Inside,
    );
    // Log-frequency grid.
    for i in 1..6 {
        let x = rect.min.x + rect.width() * i as f32 / 6.0;
        p.line_segment(
            [pos2(x, rect.min.y + 2.0), pos2(x, rect.max.y - 2.0)],
            Stroke::new(0.5_f32, CyberTheme::alpha(color, 14)),
        );
    }
    p.text(
        pos2(rect.min.x + 4.0, rect.min.y + 2.0),
        nih_plug_egui::egui::Align2::LEFT_TOP,
        "ZDF RESPONSE",
        nih_plug_egui::egui::FontId::monospace(6.2),
        CyberTheme::alpha(color, 160),
    );

    let f_min = 20.0f32;
    let f_max = 20000.0f32;
    let n = 96;
    let q = 0.707 + reso.clamp(0.0, 1.0) * 19.3;
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let freq = f_min * (f_max / f_min).powf(t);
        let x = freq / cutoff.max(1.0);
        let x2 = x * x;
        let mag = match kind {
            FilterType::LowPass12 => 1.0 / (1.0 + x2 * x2).sqrt(),
            FilterType::LowPass24 => 1.0 / (1.0 + x2),
            FilterType::HighPass12 => x2 / (1.0 + x2 * x2).sqrt(),
            FilterType::HighPass24 => x2 / (1.0 + x2),
            FilterType::BandPass12 => (x / q) / ((1.0 - x2).powi(2) + (x / q).powi(2)).sqrt(),
            FilterType::BandPass24 => {
                let a = (x / q) / ((1.0 - x2).powi(2) + (x / q).powi(2)).sqrt();
                a * a
            }
            FilterType::Notch => {
                let num = (1.0 - x2).abs();
                let den = ((1.0 - x2).powi(2) + (x / q).powi(2)).sqrt();
                num / den.max(1e-6)
            }
        }
        .clamp(0.0, 1.0);
        let db = 20.0 * mag.max(1e-4).log10();
        let y = rect.max.y - 4.0 - ((db.clamp(-48.0, 12.0) + 48.0) / 60.0) * (rect.height() - 10.0);
        pts.push(pos2(rect.min.x + 2.0 + t * (rect.width() - 4.0), y));
    }
    for i in 0..pts.len() - 1 {
        p.line_segment(
            [pts[i], pts[i + 1]],
            Stroke::new(4.0_f32, CyberTheme::alpha(color, 35)),
        );
    }
    for i in 0..pts.len() - 1 {
        p.line_segment([pts[i], pts[i + 1]], Stroke::new(1.3_f32, color));
    }
    // Cutoff marker.
    let ct = (cutoff.max(f_min) / f_min).ln() / (f_max / f_min).ln();
    let cx = rect.min.x + 2.0 + ct.clamp(0.0, 1.0) * (rect.width() - 4.0);
    p.line_segment(
        [pos2(cx, rect.min.y + 10.0), pos2(cx, rect.max.y - 2.0)],
        Stroke::new(1.0_f32, CyberTheme::alpha(color, 130)),
    );
}

fn draw_scope_card(
    ui: &mut Ui,
    rect: Rect,
    scope: &[f32],
    active_voices: usize,
    _state: &mut GuiSharedState,
    t: f32,
) {
    let color = CyberTheme::NEON_CYAN;
    panel(ui, rect, color, "05", "SIGNAL MONITOR", t, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("LIVE SIGNAL // PHOSPHOR VECTOR")
                    .color(color)
                    .monospace()
                    .strong()
                    .size(8.5),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("DSP IN {:02} VOICES", active_voices))
                        .color(CyberTheme::NEON_LIME)
                        .monospace()
                        .size(7.5),
                );
                ui.label(
                    RichText::new("48 kHz / 64-BIT")
                        .color(CyberTheme::TEXT_DIM)
                        .monospace()
                        .size(7.0),
                );
            });
        });
        let h = ui.available_height().max(70.0);
        draw_oscilloscope(ui, scope, ui.available_width(), h);
    });
}

fn draw_env_card(
    ui: &mut Ui,
    rect: Rect,
    index: &str,
    title: &str,
    color: Color32,
    env: &mut EnvUi,
    t: f32,
) {
    panel(ui, rect, color, index, title, t, |ui| {
        // ADSR curve
        let (curve_rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::hover());
        draw_adsr(ui, curve_rect, env, color);
        ui.horizontal(|ui| {
            draw_rotary_knob_log(ui, &mut env.attack, 0.001..=5.0, "ATK", color);
            draw_rotary_knob_log(ui, &mut env.decay, 0.01..=5.0, "DEC", color);
            draw_rotary_knob(ui, &mut env.sustain, 0.0..=1.0, "SUS", color);
            draw_rotary_knob_log(ui, &mut env.release, 0.01..=5.0, "REL", color);
        });
    });
}

fn draw_adsr(ui: &mut Ui, rect: Rect, env: &EnvUi, color: Color32) {
    let p = ui.painter();
    p.rect_filled(rect, 2.0, Color32::from_rgb(4, 2, 11));
    p.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, CyberTheme::alpha(color, 40)),
        StrokeKind::Inside,
    );
    for i in 0..4 {
        let x = rect.min.x + rect.width() * (i as f32 + 1.0) / 5.0;
        p.line_segment(
            [pos2(x, rect.min.y + 3.0), pos2(x, rect.max.y - 3.0)],
            Stroke::new(0.5_f32, CyberTheme::alpha(color, 16)),
        );
    }

    let total = env.attack + env.decay + 0.6 + env.release;
    let a = (env.attack / total).clamp(0.02, 0.5);
    let d = (env.decay / total).clamp(0.02, 0.4);
    let r = (env.release / total).clamp(0.03, 0.5);
    let sustain = env.sustain.clamp(0.0, 1.0);
    let (x0, y0) = (rect.min.x + 4.0, rect.max.y - 3.0);
    let w = rect.width() - 8.0;
    let h = rect.height() - 7.0;
    let xa = x0 + w * a;
    let xd = xa + w * d;
    let xs = rect.max.x - 4.0 - w * r;
    if xs < xd {
        // long release: keep the sustain segment short
    }
    let xs = xs.max(xd);

    let pts = vec![
        pos2(x0, y0),
        pos2(xa, rect.min.y + 3.0),
        pos2(xd, rect.min.y + 3.0 + h * (1.0 - sustain)),
        pos2(xs, rect.min.y + 3.0 + h * (1.0 - sustain)),
        pos2(rect.max.x - 4.0, y0),
    ];
    for i in 0..pts.len() - 1 {
        p.line_segment(
            [pts[i], pts[i + 1]],
            Stroke::new(4.0_f32, CyberTheme::alpha(color, 35)),
        );
    }
    for i in 0..pts.len() - 1 {
        p.line_segment([pts[i], pts[i + 1]], Stroke::new(1.5_f32, color));
    }
    for pt in &pts[1..pts.len() - 1] {
        p.circle_filled(*pt, 2.0, Color32::WHITE);
    }
}

fn draw_fx_card(ui: &mut Ui, rect: Rect, fx: &mut FxUi, t: f32) {
    let color = CyberTheme::NEON_PURPLE;
    panel(ui, rect, color, "08", "FX RACK // SIGNAL CHAIN", t, |ui| {
        // 1. Distortion
        ui.horizontal(|ui| {
            small_chip(ui, &mut fx.dist_enabled, "DIST", CyberTheme::NEON_RED, 58.0);
            let modes = ["OVERDRIVE", "CRUSHER", "FIRE", "GRIND"];
            let idx = match fx.dist_mode {
                DistortionMode::SoftOverdrive => 0,
                DistortionMode::BitCrusher => 1,
                DistortionMode::FireAmp => 2,
                DistortionMode::GrindAmp => 3,
            };
            let mut sel = idx;
            ComboBox::from_id_salt("dist_mode")
                .selected_text(modes[idx])
                .width(86.0)
                .show_ui(ui, |ui| {
                    for (i, m) in modes.iter().enumerate() {
                        ui.selectable_value(&mut sel, i, *m);
                    }
                });
            fx.dist_mode = match sel {
                0 => DistortionMode::SoftOverdrive,
                1 => DistortionMode::BitCrusher,
                2 => DistortionMode::FireAmp,
                _ => DistortionMode::GrindAmp,
            };
            neon_slider(
                ui,
                &mut fx.dist_amount,
                0.0..=1.0,
                "DRIVE",
                180.0,
                CyberTheme::NEON_RED,
            );
            neon_slider(
                ui,
                &mut fx.dist_mix,
                0.0..=1.0,
                "MIX",
                150.0,
                CyberTheme::NEON_RED,
            );
        });
        // 2. Chorus
        ui.horizontal(|ui| {
            small_chip(
                ui,
                &mut fx.chorus_enabled,
                "CHORUS",
                CyberTheme::NEON_CYAN,
                68.0,
            );
            neon_slider(
                ui,
                &mut fx.chorus_mix,
                0.0..=1.0,
                "MIX",
                180.0,
                CyberTheme::NEON_CYAN,
            );
            neon_slider(
                ui,
                &mut fx.chorus_rate,
                0.1..=8.0,
                "RATE",
                150.0,
                CyberTheme::NEON_CYAN,
            );
        });
        // 3. Ping-pong delay
        ui.horizontal(|ui| {
            small_chip(
                ui,
                &mut fx.delay_enabled,
                "DELAY",
                CyberTheme::NEON_AMBER,
                58.0,
            );
            small_chip(
                ui,
                &mut fx.delay_pingpong,
                "P-P",
                CyberTheme::NEON_AMBER,
                48.0,
            );
            neon_slider(
                ui,
                &mut fx.delay_mix,
                0.0..=1.0,
                "MIX",
                150.0,
                CyberTheme::NEON_AMBER,
            );
            neon_slider(
                ui,
                &mut fx.delay_fb,
                0.0..=0.9,
                "FDBK",
                130.0,
                CyberTheme::NEON_AMBER,
            );
            neon_slider(
                ui,
                &mut fx.delay_time,
                20.0..=1200.0,
                "TIME",
                120.0,
                CyberTheme::NEON_AMBER,
            );
        });
        // 4. Reverb
        ui.horizontal(|ui| {
            small_chip(
                ui,
                &mut fx.reverb_enabled,
                "REVERB",
                CyberTheme::NEON_PURPLE,
                68.0,
            );
            neon_slider(
                ui,
                &mut fx.reverb_size,
                0.0..=1.0,
                "ROOM",
                180.0,
                CyberTheme::NEON_PURPLE,
            );
            neon_slider(
                ui,
                &mut fx.reverb_mix,
                0.0..=1.0,
                "MIX",
                150.0,
                CyberTheme::NEON_PURPLE,
            );
        });
        // 5. Trance gate
        ui.horizontal(|ui| {
            small_chip(
                ui,
                &mut fx.gate_enabled,
                "TRANCE GATE",
                CyberTheme::NEON_LIME,
                104.0,
            );
            ui.label(
                RichText::new("16-STEP CHOPPER // TEMPO LOCKED")
                    .color(CyberTheme::TEXT_DIM)
                    .monospace()
                    .size(7.0),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new("GATE → CHORUS → DRIVE → DELAY → REVERB")
                        .color(CyberTheme::alpha(CyberTheme::NEON_PURPLE, 180))
                        .monospace()
                        .size(6.8),
                );
            });
        });
    });
}

// ---------------------------------------------------------------------------
// Small widgets
// ---------------------------------------------------------------------------

pub(crate) fn neon_button(text: &str, color: Color32, filled: bool) -> Button<'static> {
    let txt = RichText::new(text.to_string())
        .color(if filled { CyberTheme::BG_VOID } else { color })
        .monospace()
        .strong()
        .size(9.5);
    let btn = Button::new(txt)
        .fill(if filled {
            color
        } else {
            CyberTheme::alpha(color, 18)
        })
        .stroke(Stroke::new(1.0_f32, color))
        .corner_radius(2.0);
    btn
}

pub(crate) fn category_button(text: &str, selected: bool, color: Color32) -> Button<'static> {
    let txt = RichText::new(text.to_string())
        .color(if selected {
            CyberTheme::BG_VOID
        } else {
            CyberTheme::TEXT_MUTED
        })
        .monospace()
        .strong()
        .size(7.6);
    Button::new(txt)
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

fn small_chip(ui: &mut Ui, on: &mut bool, label: &str, color: Color32, width: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 17.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
    }
    CyberTheme::chip(ui.painter(), rect, *on, color, label);
    resp.on_hover_cursor(nih_plug_egui::egui::CursorIcon::PointingHand)
}

pub(crate) fn neon_slider(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: &str,
    width: f32,
    color: Color32,
) -> Response {
    let (rect, mut resp) = ui.allocate_exact_size(vec2(width, 22.0), Sense::click_and_drag());
    let min = *range.start();
    let max = *range.end();
    let span = (max - min).max(1e-6);
    let track = Rect::from_min_max(
        pos2(rect.min.x + 38.0, rect.center().y - 2.5),
        pos2(rect.max.x - 30.0, rect.center().y + 2.5),
    );

    if (resp.dragged() || resp.clicked()) && track.width() > 1.0 {
        if let Some(p) = resp.interact_pointer_pos() {
            let n = ((p.x - track.min.x) / track.width()).clamp(0.0, 1.0);
            *value = min + n * span;
            resp.mark_changed();
        }
    }

    let norm = ((*value - min) / span).clamp(0.0, 1.0);
    let p = ui.painter();
    p.text(
        pos2(rect.min.x, rect.center().y),
        nih_plug_egui::egui::Align2::LEFT_CENTER,
        label,
        nih_plug_egui::egui::FontId::monospace(7.0),
        CyberTheme::TEXT_MUTED,
    );
    p.rect_filled(track, 1.0, Color32::from_rgb(24, 14, 42));
    let fill = Rect::from_min_max(
        track.min,
        pos2(track.min.x + track.width() * norm, track.max.y),
    );
    p.rect_filled(fill, 1.0, CyberTheme::alpha(color, 55));
    p.line_segment(
        [pos2(fill.max.x, track.min.y), pos2(fill.max.x, track.max.y)],
        Stroke::new(1.6_f32, color),
    );
    p.circle_filled(pos2(fill.max.x, track.center().y), 2.4, color);
    p.text(
        pos2(rect.max.x, rect.center().y),
        nih_plug_egui::egui::Align2::RIGHT_CENTER,
        if span > 100.0 {
            format!("{:.0}", value)
        } else if span > 10.0 {
            format!("{:.1}", value)
        } else {
            format!("{:.2}", value)
        },
        nih_plug_egui::egui::FontId::monospace(6.8),
        CyberTheme::TEXT_PRIMARY,
    );

    resp
}

fn randomize_steps(steps: &mut [crate::audio::step_sequencer::Step; 64], seed: u32) {
    let scale = [0i8, 2, 3, 5, 7, 8, 10, 12, 15, 17, 19, 22, 24];
    let mut s = seed | 1;
    for (i, step) in steps.iter_mut().enumerate() {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        step.gate = (s % 100) < 72;
        step.accent = (s >> 8) % 100 < 28;
        step.slide = (s >> 16) % 100 < 22;
        step.tie = false;
        step.velocity = if step.accent {
            122
        } else {
            84 + ((s >> 4) % 34) as u8
        };
        step.note_offset = scale[((s >> 7) as usize) % scale.len()] - 12;
        // Keep a strong downbeat every 4 steps.
        if i % 4 == 0 {
            step.gate = true;
            step.note_offset = 0;
        }
    }
}
