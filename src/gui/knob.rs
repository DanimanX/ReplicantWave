//! Cyberpunk Analog Rotary Knob Widget.
//!
//! Provides hardware-style analog rotary potentiometers with metallic caps,
//! illuminated neon angle arcs, smooth dragging, fine-tune modifiers, and logarithmic scaling.

use super::theme::CyberTheme;
use nih_plug_egui::egui::{
    pos2, Color32, FontId, Pos2, Response, Sense, Stroke, Ui, Vec2,
};
use std::f32::consts::PI;

const ARC_START_ANGLE: f32 = -0.75 * PI; // 7 o'clock (-135 deg)
const ARC_END_ANGLE: f32 = 0.75 * PI;    // 5 o'clock (+135 deg)
const ARC_TOTAL_SPAN: f32 = ARC_END_ANGLE - ARC_START_ANGLE; // 270 deg

/// Draw a continuous float analog rotary knob.
pub fn draw_rotary_knob(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: &str,
    color: Color32,
) -> Response {
    draw_knob_internal(ui, value, range, label, color, false, 13.5)
}

/// Draw a logarithmic float analog rotary knob (ideal for frequency/cutoff, time).
pub fn draw_rotary_knob_log(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: &str,
    color: Color32,
) -> Response {
    draw_knob_internal(ui, value, range, label, color, true, 13.5)
}

/// Draw an integer analog rotary knob (for unison, octaves, etc.).
pub fn draw_rotary_knob_usize(
    ui: &mut Ui,
    value: &mut usize,
    range: std::ops::RangeInclusive<usize>,
    label: &str,
    color: Color32,
) -> Response {
    let min_f = *range.start() as f32;
    let max_f = *range.end() as f32;
    let mut f_val = *value as f32;

    let response = draw_knob_internal(ui, &mut f_val, min_f..=max_f, label, color, false, 13.5);
    *value = f_val.round() as usize;
    response
}

/// Draw an i8 analog rotary knob (for pitch semitones, etc.).
pub fn draw_rotary_knob_i8(
    ui: &mut Ui,
    value: &mut i8,
    range: std::ops::RangeInclusive<i8>,
    label: &str,
    color: Color32,
) -> Response {
    let min_f = *range.start() as f32;
    let max_f = *range.end() as f32;
    let mut f_val = *value as f32;

    let response = draw_knob_internal(ui, &mut f_val, min_f..=max_f, label, color, false, 11.5);
    *value = f_val.round() as i8;
    response
}

/// Draw a u8 analog rotary knob (for velocity, etc.).
pub fn draw_rotary_knob_u8(
    ui: &mut Ui,
    value: &mut u8,
    range: std::ops::RangeInclusive<u8>,
    label: &str,
    color: Color32,
) -> Response {
    let min_f = *range.start() as f32;
    let max_f = *range.end() as f32;
    let mut f_val = *value as f32;

    let response = draw_knob_internal(ui, &mut f_val, min_f..=max_f, label, color, false, 11.5);
    *value = f_val.round() as u8;
    response
}

fn draw_knob_internal(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: &str,
    color: Color32,
    is_log: bool,
    knob_radius: f32,
) -> Response {
    let diameter = knob_radius * 2.0;
    let total_width = diameter + 8.0;
    let total_height = diameter + 14.0; // Space for label underneath

    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(total_width, total_height), Sense::click_and_drag());
    let painter = ui.painter();

    let min_val = *range.start();
    let max_val = *range.end();

    // Calculate normalized 0.0 .. 1.0 position
    let mut norm_val = if is_log {
        let log_min = min_val.ln();
        let log_max = max_val.ln();
        ((value.clamp(min_val, max_val).ln() - log_min) / (log_max - log_min)).clamp(0.0, 1.0)
    } else {
        (((*value - min_val) / (max_val - min_val))).clamp(0.0, 1.0)
    };

    // Handle mouse drag interaction
    if response.dragged() {
        let delta = response.drag_delta();
        // Shift key for 5x fine precision
        let sensitivity = if ui.input(|i| i.modifiers.shift) { 0.001 } else { 0.005 };
        // Dragging upwards increases value
        norm_val = (norm_val - delta.y * sensitivity + delta.x * sensitivity * 0.5).clamp(0.0, 1.0);

        *value = if is_log {
            let log_min = min_val.ln();
            let log_max = max_val.ln();
            (log_min + norm_val * (log_max - log_min)).exp()
        } else {
            min_val + norm_val * (max_val - min_val)
        };
        response.mark_changed();
    }

    // Double-click resets to center / default
    if response.double_clicked() {
        norm_val = if min_val < 0.0 && max_val > 0.0 {
            0.5
        } else if is_log {
            0.5
        } else {
            0.0
        };
        *value = if is_log {
            let log_min = min_val.ln();
            let log_max = max_val.ln();
            (log_min + norm_val * (log_max - log_min)).exp()
        } else {
            min_val + norm_val * (max_val - min_val)
        };
        response.mark_changed();
    }

    // Knob Center Position
    let center = pos2(rect.center().x, rect.min.y + knob_radius + 2.0);
    let is_hovered = response.hovered() || response.dragged();

    // 1. Inactive Track Arc (270 deg background)
    draw_arc(painter, center, knob_radius, ARC_START_ANGLE, ARC_END_ANGLE, Stroke::new(1.8_f32, Color32::from_rgb(28, 32, 48)));

    // 2. Active Glow Arc (from 7 o'clock to current position)
    let current_angle = ARC_START_ANGLE + norm_val * ARC_TOTAL_SPAN;
    let arc_glow_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), if is_hovered { 120 } else { 75 });
    draw_arc(painter, center, knob_radius, ARC_START_ANGLE, current_angle, Stroke::new(3.8_f32, arc_glow_color));
    draw_arc(painter, center, knob_radius, ARC_START_ANGLE, current_angle, Stroke::new(2.0_f32, color));

    // 3. Inner Metallic Dial Cap
    let cap_radius = knob_radius - 2.8;
    let cap_bg = if is_hovered {
        Color32::from_rgb(30, 34, 52)
    } else {
        Color32::from_rgb(18, 21, 32)
    };
    painter.circle_filled(center, cap_radius, cap_bg);
    painter.circle_stroke(center, cap_radius, Stroke::new(1.0_f32, Color32::from_rgb(45, 52, 78)));

    // 4. Indicator Pointer Needle
    let angle_rad = current_angle - (PI * 0.5); // Convert to polar angle where 0 is up
    let dir = Vec2::new(angle_rad.cos(), angle_rad.sin());
    let p_start = center + dir * (cap_radius * 0.35);
    let p_end = center + dir * (cap_radius - 0.5);

    painter.line_segment([p_start, p_end], Stroke::new(2.0_f32, Color32::WHITE));
    // Glowing neon needle tip
    painter.circle_filled(p_end, 1.2, color);

    // 5. Center Hub Dot
    painter.circle_filled(center, 2.5, Color32::from_rgb(10, 11, 16));

    // 6. Label & Value Text Underneath
    let text_pos = pos2(rect.center().x, rect.max.y - 10.0);
    painter.text(
        text_pos,
        nih_plug_egui::egui::Align2::CENTER_TOP,
        label,
        FontId::proportional(8.5),
        if is_hovered { CyberTheme::TEXT_PRIMARY } else { CyberTheme::TEXT_MUTED },
    );

    // Format value tooltip on hover
    if is_hovered {
        let val_str = format_val(*value, is_log);
        response = response.on_hover_text(format!("{}: {}", label, val_str));
    }

    response
}

fn draw_arc(
    painter: &nih_plug_egui::egui::Painter,
    center: Pos2,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    stroke: Stroke,
) {
    if (end_angle - start_angle).abs() < 0.01 {
        return;
    }

    let num_segments = 24;
    let step = (end_angle - start_angle) / num_segments as f32;

    for i in 0..num_segments {
        let a0 = start_angle + i as f32 * step - (PI * 0.5);
        let a1 = start_angle + (i + 1) as f32 * step - (PI * 0.5);

        let p0 = center + Vec2::new(a0.cos(), a0.sin()) * radius;
        let p1 = center + Vec2::new(a1.cos(), a1.sin()) * radius;

        painter.line_segment([p0, p1], stroke);
    }
}

fn format_val(val: f32, is_log: bool) -> String {
    if is_log {
        if val >= 1000.0 {
            format!("{:.1} kHz", val / 1000.0)
        } else if val < 0.1 {
            format!("{:.1} ms", val * 1000.0)
        } else {
            format!("{:.1} Hz", val)
        }
    } else if val.abs() < 0.001 {
        "0.00".into()
    } else if val.fract().abs() < 0.001 {
        format!("{:.0}", val)
    } else {
        format!("{:.2}", val)
    }
}
