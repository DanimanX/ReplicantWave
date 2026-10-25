//! NIGHT CITY HUD — radical cyberpunk editor for ReplicantWave.
//!
//! Every knob, toggle and selector is bound to a host-visible parameter. The
//! editor never mutates the DSP engine directly: it writes through the host
//! `ParamSetter`, and `ReplicantWaveParams::sync_to_engine` mirrors the values
//! into the engine on the audio thread. That keeps automation, MIDI learn,
//! generic-UI control and session restore working for all controls.

pub mod knob;
pub mod oscilloscope;
pub mod sequencer_view;
pub mod theme;
pub mod time;
pub mod wavetable_view;

use crate::audio::step_sequencer::Step;
use crate::audio::SynthEngine;
use crate::params::{
    ChorusParams, DelayParams, DistParams, EnvParams, FilterParams, NoiseParams, OscParams,
    ReplicantWaveParams, SubParams,
};
use crate::presets::factory::get_factory_presets;
use crate::presets::SynthPreset;
use knob::{knob_f, knob_i, param_chip, param_select, param_slider, write_i};
use nih_plug::prelude::ParamSetter;
use oscilloscope::draw_oscilloscope;
use sequencer_view::{draw_sequencer_deck, SequencerUiState};
use theme::CyberTheme;
use wavetable_view::{draw_wavetable_preview, WaveThumb};

use nih_plug_egui::egui::{
    pos2, vec2, Align, Button, Color32, Layout, Pos2, Rect, RichText, Sense, Stroke, StrokeKind,
    Ui, UiBuilder,
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
    pub params: Arc<ReplicantWaveParams>,
    pub scope: Arc<Mutex<crate::audio::ScopeBuffer>>,
    pub active_voices: Arc<AtomicUsize>,
    /// Editable copy of the persisted 64-step pattern.
    pub steps: Vec<Step>,
    pub steps_dirty: bool,
    pub presets: Vec<SynthPreset>,
    pub current_preset_idx: usize,
    pub preset_loaded: bool,
    pub pending_preset: Option<usize>,
    pub wave_thumbs: Vec<WaveThumb>,
    pub wave_names: Vec<String>,
    pub seq_ui_state: SequencerUiState,
    pub category: usize,
    pub live_step: usize,
    /// Pattern version currently shown, so a session restore refreshes the grid.
    pub seen_pattern_version: u64,
}

impl GuiSharedState {
    pub fn new(engine: Arc<Mutex<SynthEngine>>, params: Arc<ReplicantWaveParams>) -> Self {
        let presets = get_factory_presets();
        let (steps, pattern_version) = {
            let guard = params.pattern.lock();
            (guard.steps.clone(), guard.version)
        };
        let (scope, active_voices, wave_thumbs, wave_names) = {
            let guard = engine.lock();
            let thumbs = guard
                .bank
                .tables
                .iter()
                .map(WaveThumb::from_wavetable)
                .collect();
            (
                guard.scope_data.clone(),
                guard.active_voice_count.clone(),
                thumbs,
                guard.bank.names(),
            )
        };

        Self {
            engine,
            params,
            scope,
            active_voices,
            steps,
            steps_dirty: false,
            presets,
            current_preset_idx: 0,
            preset_loaded: false,
            pending_preset: None,
            wave_thumbs,
            wave_names,
            seq_ui_state: SequencerUiState::default(),
            category: 0,
            live_step: 0,
            seen_pattern_version: pattern_version,
        }
    }

    fn queue_preset(&mut self, idx: usize) {
        if idx < self.presets.len() {
            self.current_preset_idx = idx;
            self.pending_preset = Some(idx);
        }
    }

    fn apply_pending_preset(&mut self, setter: &ParamSetter) {
        let Some(idx) = self.pending_preset else {
            return;
        };
        if let Some(p) = self.presets.get(idx) {
            p.apply_to_params(setter, &self.params);
            p.write_pattern(&self.params.pattern);
            self.steps = p.steps();
            self.steps_dirty = false;
            self.seen_pattern_version = self.params.pattern.lock().version;
            self.preset_loaded = true;
        }
        self.pending_preset = None;
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

    fn flush_steps(&mut self) {
        if !self.steps_dirty {
            return;
        }
        let mut guard = self.params.pattern.lock();
        guard.steps = self.steps.clone();
        guard.version = guard.version.wrapping_add(1);
        self.seen_pattern_version = guard.version;
        self.steps_dirty = false;
    }

    /// Pick up a pattern that changed outside the editor (host session restore).
    fn refresh_steps_from_store(&mut self) {
        if self.steps_dirty {
            return;
        }
        let guard = self.params.pattern.lock();
        if guard.version != self.seen_pattern_version {
            self.steps = guard.steps.clone();
            self.seen_pattern_version = guard.version;
        }
    }
}

pub fn render_cyberpunk_gui(ui: &mut Ui, setter: &ParamSetter, state: &mut GuiSharedState) {
    let t = time::now_secs();

    ui.ctx().request_repaint_after(Duration::from_millis(16));

    state.refresh_steps_from_store();
    state.apply_pending_preset(setter);

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

    draw_top_bar(ui, setter, top_rect, state, active_voices, t);
    draw_category_strip(ui, strip_rect, state, t);
    draw_main_area(
        ui,
        setter,
        main_rect,
        state,
        &scope_samples,
        active_voices,
        t,
    );

    let params = state.params.clone();
    draw_sequencer_deck(
        ui,
        setter,
        seq_rect,
        &params.seq,
        &mut state.steps,
        &mut state.seq_ui_state,
        state.live_step,
        t,
    );

    CyberTheme::draw_overlay(ui.painter(), full);

    state.flush_steps();
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
    setter: &ParamSetter,
    rect: Rect,
    state: &mut GuiSharedState,
    active_voices: usize,
    t: f32,
) {
    let accent = CyberTheme::NEON_CYAN;
    CyberTheme::card_shell(ui.painter(), rect, accent);

    let p = ui.painter();
    let lx = rect.min.x + 18.0;
    let ly = rect.min.y + 11.0;
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
                && !indices.is_empty()
            {
                let ni = if pos == 0 { indices.len() - 1 } else { pos - 1 };
                clicked_preset = Some(indices[ni]);
            }

            let cur = &state.presets[state.current_preset_idx];
            let color = group_color(preset_group(&cur.category));
            let name = if state.preset_loaded {
                format!("{}  //  {}", cur.name, cur.category.to_uppercase())
            } else {
                "INIT PATCH  //  CLICK ▶".to_string()
            };
            ui.label(
                RichText::new(format!("{:03}", state.current_preset_idx + 1))
                    .color(CyberTheme::TEXT_DIM)
                    .monospace()
                    .size(9.0),
            );
            nih_plug_egui::egui::ComboBox::from_id_salt("night_city_preset")
                .selected_text(RichText::new(name).color(color).monospace().size(11.0))
                .width(250.0)
                .show_ui(ui, |ui| {
                    for &idx in &indices {
                        let pr = &state.presets[idx];
                        let label = format!("{:03}  {}  [{}]", idx + 1, pr.name, pr.category);
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
                && !indices.is_empty()
            {
                let ni = (pos + 1) % indices.len();
                clicked_preset = Some(indices[ni]);
            }
        },
    );
    if let Some(idx) = clicked_preset {
        state.queue_preset(idx);
    }

    let right = rect.max.x - 18.0;
    let master_rect = Rect::from_min_size(pos2(right - 210.0, rect.min.y + 5.0), vec2(48.0, 44.0));
    ui.allocate_new_ui(UiBuilder::new().max_rect(master_rect), |ui| {
        knob_f(
            ui,
            setter,
            &state.params.gain,
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
    let mut randomize = false;
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
                    randomize = true;
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

    if randomize {
        randomize_steps(&mut state.steps, t.to_bits() ^ 0x9e37_79b9);
        state.steps_dirty = true;
    }

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
    setter: &ParamSetter,
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

    let params = state.params.clone();
    let names: Vec<&str> = state.wave_names.iter().map(|s| s.as_str()).collect();

    let osc1_rect = Rect::from_min_size(rect.min, vec2(card_w, row_h));
    let osc2_rect = Rect::from_min_size(
        pos2(rect.min.x + card_w + gap, rect.min.y),
        vec2(card_w, row_h),
    );
    draw_osc_card(
        ui,
        setter,
        osc1_rect,
        "01",
        "OSCILLATOR // A",
        CyberTheme::NEON_CYAN,
        &params.osc1,
        &state.wave_thumbs,
        &names,
        t,
    );
    draw_osc_card(
        ui,
        setter,
        osc2_rect,
        "02",
        "OSCILLATOR // B",
        CyberTheme::NEON_MAGENTA,
        &params.osc2,
        &state.wave_thumbs,
        &names,
        t,
    );

    let low_y = rect.min.y + row_h + gap;
    let sub_rect = Rect::from_min_size(pos2(rect.min.x, low_y), vec2(card_w, row_h));
    let filter_rect =
        Rect::from_min_size(pos2(rect.min.x + card_w + gap, low_y), vec2(card_w, row_h));
    draw_sub_noise_card(ui, setter, sub_rect, &params.sub, &params.noise, t);
    draw_filter_card(
        ui,
        setter,
        filter_rect,
        &params.filter,
        &params.cutoff,
        &params.resonance,
        t,
    );

    let scope_h = 158.0;
    let env_h = 172.0;
    let scope_rect = Rect::from_min_size(pos2(right_x, rect.min.y), vec2(right_w, scope_h));
    draw_scope_card(ui, scope_rect, scope, active_voices, t);

    let env_y = rect.min.y + scope_h + gap;
    let env_w = (right_w - gap) * 0.5;
    let amp_rect = Rect::from_min_size(pos2(right_x, env_y), vec2(env_w, env_h));
    let flt_env_rect = Rect::from_min_size(pos2(right_x + env_w + gap, env_y), vec2(env_w, env_h));
    draw_env_card(
        ui,
        setter,
        amp_rect,
        "06",
        "AMP ENVELOPE",
        CyberTheme::NEON_PINK,
        &params.amp_env,
        t,
    );
    draw_env_card(
        ui,
        setter,
        flt_env_rect,
        "07",
        "FILTER ENVELOPE",
        CyberTheme::NEON_CYAN,
        &params.filter_env,
        t,
    );

    let fx_y = env_y + env_h + gap;
    let fx_rect = Rect::from_min_size(
        pos2(right_x, fx_y),
        vec2(right_w, (rect.max.y - fx_y).max(120.0)),
    );
    draw_fx_card(
        ui,
        setter,
        fx_rect,
        &params.dist,
        &params.chorus,
        &params.delay,
        &params.reverb,
        &params.gate.enabled,
        t,
    );
}

fn draw_osc_card(
    ui: &mut Ui,
    setter: &ParamSetter,
    rect: Rect,
    index: &str,
    title: &str,
    color: Color32,
    osc: &OscParams,
    thumbs: &[WaveThumb],
    names: &[&str],
    t: f32,
) {
    panel(ui, rect, color, index, title, t, |ui| {
        ui.horizontal(|ui| {
            param_chip(ui, setter, &osc.enabled, "ON", color, 48.0);
            ui.label(
                RichText::new("TABLE")
                    .color(CyberTheme::TEXT_DIM)
                    .monospace()
                    .size(7.0),
            );
            param_select(ui, setter, &osc.table, names, 150.0);
        });

        let table = osc.table.value().clamp(0, names.len() as i32 - 1) as usize;
        if let Some(thumb) = thumbs.get(table % thumbs.len().max(1)) {
            draw_wavetable_preview(
                ui,
                thumb,
                osc.position.value(),
                color,
                ui.available_width(),
                96.0,
            );
        }

        ui.horizontal(|ui| {
            knob_f(ui, setter, &osc.position, "POS", color);
            knob_f(ui, setter, &osc.tune, "TUNE", color);
            knob_i(ui, setter, &osc.unison, "UNISON", color);
            knob_f(ui, setter, &osc.detune, "DETUNE", color);
            knob_f(ui, setter, &osc.fine, "FINE", color);
        });
        ui.horizontal(|ui| {
            knob_f(ui, setter, &osc.level, "LEVEL", color);
            knob_f(ui, setter, &osc.spread, "SPREAD", color);
            knob_f(ui, setter, &osc.formant, "FORMANT", color);
            knob_f(ui, setter, &osc.bend, "BEND", color);
            knob_f(ui, setter, &osc.pan, "PAN", color);
        });
    });
}

fn draw_sub_noise_card(
    ui: &mut Ui,
    setter: &ParamSetter,
    rect: Rect,
    sub: &SubParams,
    noise: &NoiseParams,
    t: f32,
) {
    let color = CyberTheme::NEON_LIME;
    panel(ui, rect, color, "03", "SUB + NOISE", t, |ui| {
        ui.horizontal(|ui| {
            param_chip(ui, setter, &sub.enabled, "SUB", color, 48.0);
            param_select(
                ui,
                setter,
                &sub.wave,
                &["SINE", "TRI", "SAW", "PULSE 50", "PULSE 25", "PULSE 12"],
                96.0,
            );
            let octave = sub.octave.value();
            for oct in [-2i32, -1, 0] {
                if ui
                    .add(category_button(
                        &format!("{:+}", oct),
                        octave == oct,
                        CyberTheme::NEON_LIME,
                    ))
                    .clicked()
                {
                    write_i(setter, &sub.octave, oct);
                }
            }
        });
        ui.horizontal(|ui| {
            knob_f(ui, setter, &sub.level, "S LVL", color);
            knob_f(ui, setter, &sub.pan, "S PAN", color);
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
            param_chip(
                ui,
                setter,
                &noise.enabled,
                "NOISE",
                CyberTheme::NEON_CYAN,
                58.0,
            );
            param_select(
                ui,
                setter,
                &noise.noise_type,
                &["WHITE", "PINK", "CYBERBIT"],
                96.0,
            );
        });
        ui.horizontal(|ui| {
            knob_f(ui, setter, &noise.level, "N LVL", CyberTheme::NEON_CYAN);
            knob_f(ui, setter, &noise.pan, "N PAN", CyberTheme::NEON_CYAN);
        });
    });
}

fn draw_filter_card(
    ui: &mut Ui,
    setter: &ParamSetter,
    rect: Rect,
    filter: &FilterParams,
    cutoff: &nih_plug::prelude::FloatParam,
    resonance: &nih_plug::prelude::FloatParam,
    t: f32,
) {
    let color = CyberTheme::NEON_AMBER;
    panel(ui, rect, color, "04", "MULTIMODE FILTER", t, |ui| {
        ui.horizontal(|ui| {
            param_chip(ui, setter, &filter.enabled, "ON", color, 44.0);
            param_select(
                ui,
                setter,
                &filter.filter_type,
                &[
                    "LP 12", "LP 24", "HP 12", "HP 24", "BP 12", "BP 24", "NOTCH",
                ],
                82.0,
            );
            ui.label(
                RichText::new("ZDF")
                    .color(CyberTheme::TEXT_DIM)
                    .monospace()
                    .size(7.0),
            );
        });
        ui.horizontal(|ui| {
            knob_f(ui, setter, cutoff, "CUTOFF", color);
            knob_f(ui, setter, resonance, "RESO", color);
            knob_f(ui, setter, &filter.drive, "DRIVE", CyberTheme::NEON_PINK);
            knob_f(ui, setter, &filter.env_amount, "ENV MOD", color);
            knob_f(
                ui,
                setter,
                &filter.key_tracking,
                "KEY",
                CyberTheme::NEON_CYAN,
            );
        });
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("ROUTE")
                    .color(CyberTheme::TEXT_DIM)
                    .monospace()
                    .size(7.0),
            );
            param_chip(
                ui,
                setter,
                &filter.route_osc1,
                "OSC1",
                CyberTheme::NEON_CYAN,
                52.0,
            );
            param_chip(
                ui,
                setter,
                &filter.route_osc2,
                "OSC2",
                CyberTheme::NEON_MAGENTA,
                52.0,
            );
            param_chip(
                ui,
                setter,
                &filter.route_sub,
                "SUB",
                CyberTheme::NEON_LIME,
                44.0,
            );
            param_chip(
                ui,
                setter,
                &filter.route_noise,
                "NOISE",
                CyberTheme::NEON_AMBER,
                56.0,
            );
            knob_f(
                ui,
                setter,
                &filter.vel_tracking,
                "VEL",
                CyberTheme::NEON_CYAN,
            );
        });
        let h = ui.available_height().min(52.0).max(34.0);
        if h > 30.0 {
            let (resp_rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
            let kind = crate::audio::filter::FilterType::from_index(
                filter.filter_type.value().clamp(0, 6) as usize,
            );
            draw_filter_response(
                ui,
                resp_rect,
                kind,
                cutoff.value(),
                resonance.value(),
                color,
            );
        }
    });
}

fn draw_filter_response(
    ui: &mut Ui,
    rect: Rect,
    kind: crate::audio::filter::FilterType,
    cutoff: f32,
    reso: f32,
    color: Color32,
) {
    use crate::audio::filter::FilterType;
    let p = ui.painter();
    p.rect_filled(rect, 2.0, Color32::from_rgb(4, 2, 11));
    p.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, CyberTheme::alpha(color, 35)),
        StrokeKind::Inside,
    );
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
    let ct = (cutoff.max(f_min) / f_min).ln() / (f_max / f_min).ln();
    let cx = rect.min.x + 2.0 + ct.clamp(0.0, 1.0) * (rect.width() - 4.0);
    p.line_segment(
        [pos2(cx, rect.min.y + 10.0), pos2(cx, rect.max.y - 2.0)],
        Stroke::new(1.0_f32, CyberTheme::alpha(color, 130)),
    );
}

fn draw_scope_card(ui: &mut Ui, rect: Rect, scope: &[f32], active_voices: usize, t: f32) {
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
    setter: &ParamSetter,
    rect: Rect,
    index: &str,
    title: &str,
    color: Color32,
    env: &EnvParams,
    t: f32,
) {
    panel(ui, rect, color, index, title, t, |ui| {
        let (curve_rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::hover());
        draw_adsr(
            ui,
            curve_rect,
            env.attack.value(),
            env.decay.value(),
            env.sustain.value(),
            env.release.value(),
            color,
        );
        ui.horizontal(|ui| {
            knob_f(ui, setter, &env.attack, "ATK", color);
            knob_f(ui, setter, &env.decay, "DEC", color);
            knob_f(ui, setter, &env.sustain, "SUS", color);
            knob_f(ui, setter, &env.release, "REL", color);
        });
    });
}

fn draw_adsr(
    ui: &mut Ui,
    rect: Rect,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    color: Color32,
) {
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

    let total = attack + decay + 0.6 + release;
    let a = (attack / total).clamp(0.02, 0.5);
    let d = (decay / total).clamp(0.02, 0.4);
    let r = (release / total).clamp(0.03, 0.5);
    let sustain = sustain.clamp(0.0, 1.0);
    let (x0, y0) = (rect.min.x + 4.0, rect.max.y - 3.0);
    let w = rect.width() - 8.0;
    let h = rect.height() - 7.0;
    let xa = x0 + w * a;
    let xd = xa + w * d;
    let xs = (rect.max.x - 4.0 - w * r).max(xd);

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

#[allow(clippy::too_many_arguments)]
fn draw_fx_card(
    ui: &mut Ui,
    setter: &ParamSetter,
    rect: Rect,
    dist: &DistParams,
    chorus: &ChorusParams,
    delay: &DelayParams,
    reverb: &crate::params::ReverbParams,
    gate: &nih_plug::prelude::BoolParam,
    t: f32,
) {
    let color = CyberTheme::NEON_PURPLE;
    panel(ui, rect, color, "08", "FX RACK // SIGNAL CHAIN", t, |ui| {
        ui.horizontal(|ui| {
            param_chip(
                ui,
                setter,
                &dist.enabled,
                "DIST",
                CyberTheme::NEON_RED,
                58.0,
            );
            param_select(
                ui,
                setter,
                &dist.mode,
                &["OVERDRIVE", "CRUSHER", "FIRE", "GRIND"],
                86.0,
            );
            param_slider(
                ui,
                setter,
                &dist.amount,
                "DRIVE",
                180.0,
                CyberTheme::NEON_RED,
            );
            param_slider(ui, setter, &dist.mix, "MIX", 150.0, CyberTheme::NEON_RED);
        });
        ui.horizontal(|ui| {
            param_chip(
                ui,
                setter,
                &chorus.enabled,
                "CHORUS",
                CyberTheme::NEON_CYAN,
                68.0,
            );
            param_slider(ui, setter, &chorus.mix, "MIX", 180.0, CyberTheme::NEON_CYAN);
            param_slider(
                ui,
                setter,
                &chorus.rate,
                "RATE",
                150.0,
                CyberTheme::NEON_CYAN,
            );
        });
        ui.horizontal(|ui| {
            param_chip(
                ui,
                setter,
                &delay.enabled,
                "DELAY",
                CyberTheme::NEON_AMBER,
                58.0,
            );
            param_chip(
                ui,
                setter,
                &delay.pingpong,
                "P-P",
                CyberTheme::NEON_AMBER,
                48.0,
            );
            param_slider(ui, setter, &delay.mix, "MIX", 150.0, CyberTheme::NEON_AMBER);
            param_slider(
                ui,
                setter,
                &delay.feedback,
                "FDBK",
                130.0,
                CyberTheme::NEON_AMBER,
            );
            param_slider(
                ui,
                setter,
                &delay.time,
                "TIME",
                120.0,
                CyberTheme::NEON_AMBER,
            );
        });
        ui.horizontal(|ui| {
            param_chip(
                ui,
                setter,
                &reverb.enabled,
                "REVERB",
                CyberTheme::NEON_PURPLE,
                68.0,
            );
            param_slider(
                ui,
                setter,
                &reverb.room_size,
                "ROOM",
                180.0,
                CyberTheme::NEON_PURPLE,
            );
            param_slider(
                ui,
                setter,
                &reverb.mix,
                "MIX",
                150.0,
                CyberTheme::NEON_PURPLE,
            );
        });
        ui.horizontal(|ui| {
            param_chip(
                ui,
                setter,
                gate,
                "TRANCE GATE",
                CyberTheme::NEON_LIME,
                104.0,
            );
            param_chip(
                ui,
                setter,
                &delay.sync,
                "TEMPO LOCK",
                CyberTheme::NEON_AMBER,
                88.0,
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
    Button::new(txt)
        .fill(if filled {
            color
        } else {
            CyberTheme::alpha(color, 18)
        })
        .stroke(Stroke::new(1.0_f32, color))
        .corner_radius(2.0)
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

fn randomize_steps(steps: &mut [Step], seed: u32) {
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
        if i % 4 == 0 {
            step.gate = true;
            step.note_offset = 0;
        }
    }
}
