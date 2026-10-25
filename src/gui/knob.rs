//! Cyberpunk Neon HUD rotary knobs and compact controls.
//!
//! Two layers:
//! - `knob_core` / `slider_core` draw and edit a **normalised** value.
//! - `knob_f` / `knob_i` / `param_chip` / `param_select` / `param_slider` bind
//!   that to a `Param` and write it back through the host `ParamSetter`, so
//!   every control is automatable, MIDI-learnable and recorded by the DAW.

use super::theme::CyberTheme;
use nih_plug::prelude::{BoolParam, FloatParam, IntParam, Param, ParamSetter};
use nih_plug_egui::egui::{pos2, Color32, FontId, Pos2, Response, Sense, Stroke, Ui, Vec2};
use std::f32::consts::PI;

const ARC_START_ANGLE: f32 = -0.75 * PI; // 7 o'clock (-135 deg)
const ARC_END_ANGLE: f32 = 0.75 * PI; // 5 o'clock (+135 deg)
const ARC_TOTAL_SPAN: f32 = ARC_END_ANGLE - ARC_START_ANGLE; // 270 deg

// ---------------------------------------------------------------------------
// Param-bound widgets
// ---------------------------------------------------------------------------

/// Rotary knob bound to a `FloatParam`.
pub fn knob_f(
    ui: &mut Ui,
    setter: &ParamSetter,
    param: &FloatParam,
    label: &str,
    color: Color32,
) -> Response {
    let mut norm = param.preview_normalized(param.value());
    let readout = param.normalized_value_to_string(norm, true);
    let resp = knob_core(ui, &mut norm, label, color, 14.0, Some(&readout));
    commit_f32(setter, param, &resp, norm);
    resp
}

/// Rotary knob bound to an `IntParam` (discrete selector).
pub fn knob_i(
    ui: &mut Ui,
    setter: &ParamSetter,
    param: &IntParam,
    label: &str,
    color: Color32,
) -> Response {
    let mut norm = param.preview_normalized(param.value());
    let readout = param.normalized_value_to_string(norm, true);
    let resp = knob_core(ui, &mut norm, label, color, 14.0, Some(&readout));
    commit_i32(setter, param, &resp, norm);
    resp
}

fn commit_f32(setter: &ParamSetter, param: &FloatParam, resp: &Response, norm: f32) {
    if resp.drag_started() {
        setter.begin_set_parameter(param);
    }
    if resp.changed() {
        let value = param.preview_plain(norm.clamp(0.0, 1.0));
        let dragging = resp.dragged();
        if !dragging && !resp.drag_started() {
            setter.begin_set_parameter(param);
        }
        setter.set_parameter(param, value);
        if !dragging {
            setter.end_set_parameter(param);
        }
    }
    if resp.drag_stopped() {
        setter.end_set_parameter(param);
    }
}

fn commit_i32(setter: &ParamSetter, param: &IntParam, resp: &Response, norm: f32) {
    if resp.drag_started() {
        setter.begin_set_parameter(param);
    }
    if resp.changed() {
        let value = param.preview_plain(norm.clamp(0.0, 1.0));
        let dragging = resp.dragged();
        if !dragging && !resp.drag_started() {
            setter.begin_set_parameter(param);
        }
        setter.set_parameter(param, value);
        if !dragging {
            setter.end_set_parameter(param);
        }
    }
    if resp.drag_stopped() {
        setter.end_set_parameter(param);
    }
}

/// Neon chip toggle bound to a `BoolParam`.
pub fn param_chip(
    ui: &mut Ui,
    setter: &ParamSetter,
    param: &BoolParam,
    label: &str,
    color: Color32,
    width: f32,
) -> Response {
    let on = param.value();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 17.0), Sense::click());
    if resp.clicked() {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, !on);
        setter.end_set_parameter(param);
    }
    CyberTheme::chip(ui.painter(), rect, on, color, label);
    resp.on_hover_cursor(nih_plug_egui::egui::CursorIcon::PointingHand)
}

/// Combo box bound to an `IntParam` with human readable names.
pub fn param_select(
    ui: &mut Ui,
    setter: &ParamSetter,
    param: &IntParam,
    names: &[&str],
    width: f32,
) -> Response {
    let current = param.value();
    let text = names.get(current.max(0) as usize).copied().unwrap_or("—");

    let mut chosen = None;
    let resp = nih_plug_egui::egui::ComboBox::from_id_salt(("param_select", param.name()))
        .selected_text(
            nih_plug_egui::egui::RichText::new(text)
                .monospace()
                .size(9.0),
        )
        .width(width)
        .show_ui(ui, |ui| {
            for (i, name) in names.iter().enumerate() {
                if ui.selectable_label(current == i as i32, *name).clicked() {
                    chosen = Some(i as i32);
                }
            }
        })
        .response;

    if let Some(i) = chosen {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, i);
        setter.end_set_parameter(param);
    }
    resp
}

/// Compact horizontal neon slider bound to a `FloatParam`.
pub fn param_slider(
    ui: &mut Ui,
    setter: &ParamSetter,
    param: &FloatParam,
    label: &str,
    width: f32,
    color: Color32,
) -> Response {
    let mut norm = param.preview_normalized(param.value());
    let text = param.normalized_value_to_string(norm, false);
    let resp = slider_core(ui, &mut norm, label, width, color, &text);
    commit_f32(setter, param, &resp, norm);
    resp
}

// ---------------------------------------------------------------------------
// Direct parameter writes (used for discrete buttons)
// ---------------------------------------------------------------------------

/// Write an `IntParam` value, skipping the gesture when it is already set.
pub fn write_i(setter: &ParamSetter, param: &IntParam, value: i32) {
    if param.value() == value {
        return;
    }
    setter.begin_set_parameter(param);
    setter.set_parameter(param, value);
    setter.end_set_parameter(param);
}

/// Write a `BoolParam` value, skipping the gesture when it is already set.
pub fn write_b(setter: &ParamSetter, param: &BoolParam, value: bool) {
    if param.value() == value {
        return;
    }
    setter.begin_set_parameter(param);
    setter.set_parameter(param, value);
    setter.end_set_parameter(param);
}

/// Write a `FloatParam` value, skipping the gesture when it is already set.
pub fn write_f(setter: &ParamSetter, param: &FloatParam, value: f32) {
    if (param.value() - value).abs() < f32::EPSILON {
        return;
    }
    setter.begin_set_parameter(param);
    setter.set_parameter(param, value);
    setter.end_set_parameter(param);
}

// ---------------------------------------------------------------------------
// Normalised drawing cores
// ---------------------------------------------------------------------------

/// Draw and edit a rotary knob that operates on a 0..1 normalised value.
pub fn knob_core(
    ui: &mut Ui,
    norm: &mut f32,
    label: &str,
    color: Color32,
    knob_radius: f32,
    readout: Option<&str>,
) -> Response {
    let total_width = knob_radius * 2.0 + 10.0;
    let total_height = knob_radius * 2.0 + 14.0;

    let (rect, mut response) = ui.allocate_exact_size(
        Vec2::new(total_width, total_height),
        Sense::click_and_drag(),
    );
    let painter = ui.painter();

    let mut norm_val = norm.clamp(0.0, 1.0);
    let is_hovered = response.hovered() || response.dragged();

    if response.dragged() {
        let delta = response.drag_delta();
        let sensitivity = if ui.input(|i| i.modifiers.shift) {
            0.001
        } else {
            0.005
        };
        norm_val = (norm_val - delta.y * sensitivity + delta.x * sensitivity * 0.4).clamp(0.0, 1.0);
        *norm = norm_val;
        response.mark_changed();
    }

    if response.hovered() {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta.abs() > 0.1 {
            let step = if ui.input(|i| i.modifiers.shift) {
                0.005
            } else {
                0.02
            };
            norm_val = (norm_val + scroll_delta.signum() * step).clamp(0.0, 1.0);
            *norm = norm_val;
            response.mark_changed();
        }
    }

    if response.double_clicked() {
        // Snap back to the centre of the sweep, a neutral and predictable reset.
        norm_val = 0.5;
        *norm = norm_val;
        response.mark_changed();
    }

    let center = pos2(rect.center().x, rect.min.y + knob_radius + 3.0);

    // 1. Ambient neon halo
    let halo_alpha = if is_hovered { 60 } else { 26 };
    painter.circle_filled(
        center,
        knob_radius + 3.5,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), halo_alpha),
    );

    // 2. Tick ring
    for i in 0..=10 {
        let a = ARC_START_ANGLE + (i as f32 / 10.0) * ARC_TOTAL_SPAN - PI * 0.5;
        let r0 = knob_radius + 4.2;
        let r1 = if i % 5 == 0 {
            knob_radius + 7.0
        } else {
            knob_radius + 5.8
        };
        painter.line_segment(
            [
                pos2(center.x + a.cos() * r0, center.y + a.sin() * r0),
                pos2(center.x + a.cos() * r1, center.y + a.sin() * r1),
            ],
            Stroke::new(
                1.0_f32,
                if i % 5 == 0 {
                    CyberTheme::TEXT_MUTED
                } else {
                    CyberTheme::TEXT_DIM
                },
            ),
        );
    }

    // 3. Track arc + glowing value arc
    let cur_angle = ARC_START_ANGLE + norm_val * ARC_TOTAL_SPAN;
    draw_arc(
        painter,
        center,
        knob_radius + 1.5,
        ARC_START_ANGLE,
        ARC_END_ANGLE,
        Stroke::new(2.0_f32, Color32::from_rgb(30, 20, 48)),
    );
    if norm_val > 0.005 {
        draw_arc(
            painter,
            center,
            knob_radius + 1.5,
            ARC_START_ANGLE,
            cur_angle,
            Stroke::new(
                6.0_f32,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 45),
            ),
        );
        draw_arc(
            painter,
            center,
            knob_radius + 1.5,
            ARC_START_ANGLE,
            cur_angle,
            Stroke::new(2.2_f32, color),
        );
    }

    // 4. Dark glass cap with a single neon rim
    let dial_radius = knob_radius - 0.5;
    painter.circle_filled(center, dial_radius, Color32::from_rgb(9, 5, 20));
    painter.circle_stroke(
        center,
        dial_radius,
        Stroke::new(
            1.2_f32,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 150),
        ),
    );
    painter.circle_filled(center, dial_radius - 2.0, Color32::from_rgb(15, 9, 30));
    painter.circle_stroke(
        center,
        dial_radius - 2.0,
        Stroke::new(0.7_f32, Color32::from_rgb(46, 30, 76)),
    );

    // 5. Indicator needle
    let dir = Vec2::new((cur_angle - PI * 0.5).cos(), (cur_angle - PI * 0.5).sin());
    painter.line_segment(
        [center + dir * 2.0, center + dir * (dial_radius - 3.0)],
        Stroke::new(1.6_f32, Color32::WHITE),
    );
    painter.circle_filled(center + dir * (dial_radius - 3.2), 1.7, color);
    painter.circle_filled(center, 1.6, CyberTheme::BG_VOID);

    // 6. Label
    painter.text(
        pos2(rect.center().x, rect.max.y - 10.0),
        nih_plug_egui::egui::Align2::CENTER_TOP,
        label,
        FontId::monospace(7.6),
        if is_hovered {
            CyberTheme::TEXT_PRIMARY
        } else {
            CyberTheme::TEXT_MUTED
        },
    );

    if is_hovered {
        if let Some(readout) = readout {
            response = response.on_hover_text(format!("{}: {}", label, readout));
        }
    }

    response
}

/// Draw and edit a compact horizontal slider on a 0..1 normalised value.
pub fn slider_core(
    ui: &mut Ui,
    norm: &mut f32,
    label: &str,
    width: f32,
    color: Color32,
    readout: &str,
) -> Response {
    let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(width, 22.0), Sense::click_and_drag());
    let track = nih_plug_egui::egui::Rect::from_min_max(
        pos2(rect.min.x + 38.0, rect.center().y - 2.5),
        pos2(rect.max.x - 34.0, rect.center().y + 2.5),
    );

    let mut value = norm.clamp(0.0, 1.0);
    if (resp.dragged() || resp.clicked()) && track.width() > 1.0 {
        if let Some(p) = resp.interact_pointer_pos() {
            value = ((p.x - track.min.x) / track.width()).clamp(0.0, 1.0);
            *norm = value;
            resp.mark_changed();
        }
    }

    let p = ui.painter();
    p.text(
        pos2(rect.min.x, rect.center().y),
        nih_plug_egui::egui::Align2::LEFT_CENTER,
        label,
        FontId::monospace(7.0),
        CyberTheme::TEXT_MUTED,
    );
    p.rect_filled(track, 1.0, Color32::from_rgb(24, 14, 42));
    let fill = nih_plug_egui::egui::Rect::from_min_max(
        track.min,
        pos2(track.min.x + track.width() * value, track.max.y),
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
        readout,
        FontId::monospace(6.8),
        CyberTheme::TEXT_PRIMARY,
    );

    resp
}

fn draw_arc(
    painter: &nih_plug_egui::egui::Painter,
    center: Pos2,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    stroke: Stroke,
) {
    let segments = 24;
    let total_span = end_angle - start_angle;
    if total_span.abs() < 0.001 {
        return;
    }

    let mut prev_pt = pos2(
        center.x + (start_angle - PI * 0.5).cos() * radius,
        center.y + (start_angle - PI * 0.5).sin() * radius,
    );

    for s in 1..=segments {
        let frac = s as f32 / segments as f32;
        let angle = start_angle + frac * total_span - (PI * 0.5);
        let pt = pos2(
            center.x + angle.cos() * radius,
            center.y + angle.sin() * radius,
        );
        painter.line_segment([prev_pt, pt], stroke);
        prev_pt = pt;
    }
}

// ---------------------------------------------------------------------------
// Non-parameter knob (sequencer step pitch, which is pattern data)
// ---------------------------------------------------------------------------

/// Rotary knob over a plain integer range, used for sequencer step offsets.
pub fn draw_rotary_knob_i8(
    ui: &mut Ui,
    value: &mut i8,
    range: std::ops::RangeInclusive<i8>,
    label: &str,
    color: Color32,
) -> Response {
    let min = *range.start() as f32;
    let max = *range.end() as f32;
    let mut norm = ((*value as f32 - min) / (max - min)).clamp(0.0, 1.0);
    let readout = format!("{:+}", *value);
    let resp = knob_core(ui, &mut norm, label, color, 14.0, Some(&readout));
    if resp.changed() {
        *value = (min + norm * (max - min)).round().clamp(min, max) as i8;
    }
    resp
}

/// Rotary knob over a plain float range (non-parameter UI values).
pub fn draw_rotary_knob(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: &str,
    color: Color32,
) -> Response {
    let min = *range.start();
    let max = *range.end();
    let mut norm = ((*value - min) / (max - min)).clamp(0.0, 1.0);
    let readout = format!("{:.2}", *value);
    let resp = knob_core(ui, &mut norm, label, color, 14.0, Some(&readout));
    if resp.changed() {
        *value = min + norm * (max - min);
    }
    resp
}
