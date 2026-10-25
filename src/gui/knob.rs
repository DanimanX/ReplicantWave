//! Cyberpunk Analog Hardware Rotary Knob Widget.
//!
//! Provides authentic analog hardware synthesizer potentiometers with metallic knurled chassis,
//! glowing neon indicator arcs, smooth 3D metallic caps, fine-tuning modifiers, and mouse-wheel support.

use super::theme::CyberTheme;
use nih_plug_egui::egui::{pos2, Color32, FontId, Pos2, Response, Sense, Stroke, Ui, Vec2};
use std::f32::consts::PI;

const ARC_START_ANGLE: f32 = -0.75 * PI; // 7 o'clock (-135 deg)
const ARC_END_ANGLE: f32 = 0.75 * PI; // 5 o'clock (+135 deg)
const ARC_TOTAL_SPAN: f32 = ARC_END_ANGLE - ARC_START_ANGLE; // 270 deg

/// Draw a continuous float analog rotary knob.
pub fn draw_rotary_knob(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: &str,
    color: Color32,
) -> Response {
    draw_knob_internal(ui, value, range, label, color, false, 14.0, None)
}

/// Draw a logarithmic float analog rotary knob (ideal for frequency/cutoff, time).
pub fn draw_rotary_knob_log(
    ui: &mut Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    label: &str,
    color: Color32,
) -> Response {
    draw_knob_internal(ui, value, range, label, color, true, 14.0, None)
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

    let response = draw_knob_internal(
        ui,
        &mut f_val,
        min_f..=max_f,
        label,
        color,
        false,
        14.0,
        None,
    );
    *value = f_val.round() as usize;
    response
}

/// Draw an i8 analog rotary knob (for pitch semitones, etc.) with custom note readout.
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

    let custom_val_str = format!("{:+}", *value);
    let response = draw_knob_internal(
        ui,
        &mut f_val,
        min_f..=max_f,
        label,
        color,
        false,
        13.0,
        Some(&custom_val_str),
    );
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

    let custom_val_str = format!("{}", *value);
    let response = draw_knob_internal(
        ui,
        &mut f_val,
        min_f..=max_f,
        label,
        color,
        false,
        13.0,
        Some(&custom_val_str),
    );
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
    custom_readout: Option<&str>,
) -> Response {
    let total_width = knob_radius * 2.0 + 10.0;
    let total_height = knob_radius * 2.0 + 14.0; // Space for label below

    let (rect, mut response) = ui.allocate_exact_size(
        Vec2::new(total_width, total_height),
        Sense::click_and_drag(),
    );
    let painter = ui.painter();

    let min_val = *range.start();
    let max_val = *range.end();

    // Calculate normalized 0.0 .. 1.0 position
    let mut norm_val = if is_log {
        let log_min = min_val.ln();
        let log_max = max_val.ln();
        ((value.clamp(min_val, max_val).ln() - log_min) / (log_max - log_min)).clamp(0.0, 1.0)
    } else {
        ((*value - min_val) / (max_val - min_val)).clamp(0.0, 1.0)
    };

    let is_hovered = response.hovered() || response.dragged();

    // Handle mouse drag interaction
    if response.dragged() {
        let delta = response.drag_delta();
        let sensitivity = if ui.input(|i| i.modifiers.shift) {
            0.001
        } else {
            0.005
        };
        norm_val = (norm_val - delta.y * sensitivity + delta.x * sensitivity * 0.4).clamp(0.0, 1.0);

        *value = if is_log {
            let log_min = min_val.ln();
            let log_max = max_val.ln();
            (log_min + norm_val * (log_max - log_min)).exp()
        } else {
            min_val + norm_val * (max_val - min_val)
        };
        response.mark_changed();
    }

    // Mouse scroll wheel support
    if response.hovered() {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta.abs() > 0.1 {
            let step = if ui.input(|i| i.modifiers.shift) {
                0.005
            } else {
                0.02
            };
            norm_val = (norm_val + scroll_delta.signum() * step).clamp(0.0, 1.0);
            *value = if is_log {
                let log_min = min_val.ln();
                let log_max = max_val.ln();
                (log_min + norm_val * (log_max - log_min)).exp()
            } else {
                min_val + norm_val * (max_val - min_val)
            };
            response.mark_changed();
        }
    }

    // Double-click resets
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
    let center = pos2(rect.center().x, rect.min.y + knob_radius + 3.0);

    // 1. Ambient neon halo (stronger while hovered / dragged)
    let halo_alpha = if is_hovered { 60 } else { 26 };
    painter.circle_filled(
        center,
        knob_radius + 3.5,
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), halo_alpha),
    );

    // 2. Tick ring — 11 machined marks around the 270° sweep
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

    // 4. Dark glass cap with a single neon rim.
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
    let p_start = center + dir * 2.0;
    let p_end = center + dir * (dial_radius - 3.0);
    painter.line_segment([p_start, p_end], Stroke::new(1.6_f32, Color32::WHITE));
    painter.circle_filled(center + dir * (dial_radius - 3.2), 1.7, color);
    painter.circle_filled(center, 1.6, CyberTheme::BG_VOID);

    // 8. Label Text Underneath (Clean, matches new_ui.jpeg)
    let text_pos_label = pos2(rect.center().x, rect.max.y - 10.0);
    painter.text(
        text_pos_label,
        nih_plug_egui::egui::Align2::CENTER_TOP,
        label,
        FontId::monospace(7.6),
        if is_hovered {
            CyberTheme::TEXT_PRIMARY
        } else {
            CyberTheme::TEXT_MUTED
        },
    );

    // Value tooltip on hover
    if is_hovered {
        let full_val_str = match custom_readout {
            Some(s) => s.to_string(),
            None => format_val(*value, is_log),
        };
        response = response.on_hover_text(format!("{}: {}", label, full_val_str));
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

fn format_val(val: f32, is_log: bool) -> String {
    if is_log {
        if val >= 1000.0 {
            format!("{:.1}k", val / 1000.0)
        } else if val >= 10.0 {
            format!("{:.0}Hz", val)
        } else {
            format!("{:.2}s", val)
        }
    } else if val.abs() >= 100.0 {
        format!("{:.0}", val)
    } else if val.abs() >= 10.0 {
        format!("{:.1}", val)
    } else {
        format!("{:.2}", val)
    }
}
