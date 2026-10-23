//! Cyberpunk Analog Hardware Rotary Knob Widget.
//!
//! Provides authentic analog hardware synthesizer potentiometers with metallic knurled chassis,
//! perimeter graduation tick marks, glowing neon indicator arcs, smooth 3D metallic caps,
//! fine-tuning modifiers, mouse-wheel support, and live value readouts.

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

    let response = draw_knob_internal(ui, &mut f_val, min_f..=max_f, label, color, false, 14.0, None);
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
    let response = draw_knob_internal(ui, &mut f_val, min_f..=max_f, label, color, false, 11.0, Some(&custom_val_str));
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
    let response = draw_knob_internal(ui, &mut f_val, min_f..=max_f, label, color, false, 11.0, Some(&custom_val_str));
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
    let total_width = knob_radius * 2.0 + 12.0;
    let total_height = knob_radius * 2.0 + 18.0; // Space for label + live value readout

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

    let is_hovered = response.hovered() || response.dragged();

    // Handle mouse drag interaction
    if response.dragged() {
        let delta = response.drag_delta();
        // Shift key for 5x fine precision
        let sensitivity = if ui.input(|i| i.modifiers.shift) { 0.001 } else { 0.005 };
        // Dragging upwards increases value
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

    // Mouse scroll wheel support on hover
    if response.hovered() {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta.abs() > 0.1 {
            let step = if ui.input(|i| i.modifiers.shift) { 0.005 } else { 0.02 };
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
    let center = pos2(rect.center().x, rect.min.y + knob_radius + 4.0);

    // 1. Outer Chassis Graduation Ticks (11 Analog Hardware Dots/Tick Marks)
    let num_ticks = 11;
    let tick_radius_inner = knob_radius + 2.4;
    let tick_radius_outer = knob_radius + 4.2;

    for i in 0..num_ticks {
        let tick_frac = i as f32 / (num_ticks - 1) as f32;
        let tick_angle = ARC_START_ANGLE + tick_frac * ARC_TOTAL_SPAN - (PI * 0.5);
        let dir = Vec2::new(tick_angle.cos(), tick_angle.sin());

        let p_in = center + dir * tick_radius_inner;
        let p_out = center + dir * tick_radius_outer;

        let is_active_tick = tick_frac <= norm_val + 0.01;
        let tick_color = if is_active_tick {
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), if is_hovered { 255 } else { 190 })
        } else {
            Color32::from_rgb(32, 38, 56)
        };

        painter.line_segment([p_in, p_out], Stroke::new(1.0_f32, tick_color));
    }

    // 2. Inactive Track Arc (270 deg background)
    draw_arc(painter, center, knob_radius, ARC_START_ANGLE, ARC_END_ANGLE, Stroke::new(2.0_f32, Color32::from_rgb(24, 28, 42)));

    // 3. Active Glow Arc (from 7 o'clock to current position)
    let current_angle = ARC_START_ANGLE + norm_val * ARC_TOTAL_SPAN;
    let arc_glow_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), if is_hovered { 140 } else { 85 });
    draw_arc(painter, center, knob_radius, ARC_START_ANGLE, current_angle, Stroke::new(4.2_f32, arc_glow_color));
    draw_arc(painter, center, knob_radius, ARC_START_ANGLE, current_angle, Stroke::new(2.2_f32, color));

    // 4. Outer Knurled Potentiometer Rim (Hardware Bezel)
    let bezel_radius = knob_radius - 1.8;
    painter.circle_filled(center, bezel_radius, Color32::from_rgb(14, 16, 24));
    painter.circle_stroke(center, bezel_radius, Stroke::new(1.0_f32, Color32::from_rgb(40, 46, 68)));

    // 5. Inner Metallic Machined Cap with Specular Drop
    let cap_radius = bezel_radius - 1.6;
    let cap_fill = if is_hovered {
        Color32::from_rgb(32, 38, 58)
    } else {
        Color32::from_rgb(20, 24, 36)
    };
    painter.circle_filled(center, cap_radius, cap_fill);

    // Subtle brushed metal highlight ring
    painter.circle_stroke(center, cap_radius, Stroke::new(1.2_f32, Color32::from_rgb(52, 60, 88)));

    // 6. High-Contrast Indicator Pointer Needle with Glowing Pip
    let angle_rad = current_angle - (PI * 0.5); // Convert to polar angle where 0 is up
    let dir = Vec2::new(angle_rad.cos(), angle_rad.sin());
    let p_start = center + dir * (cap_radius * 0.25);
    let p_end = center + dir * (cap_radius - 0.5);

    // Needle shadow
    painter.line_segment([p_start + Vec2::new(0.5, 0.5), p_end + Vec2::new(0.5, 0.5)], Stroke::new(2.2_f32, Color32::from_rgb(5, 6, 9)));
    // Needle core
    painter.line_segment([p_start, p_end], Stroke::new(2.0_f32, Color32::WHITE));
    // Glowing neon needle tip pip
    painter.circle_filled(p_end, 1.4, color);

    // 7. Center Hub Dot (vintage synth collar)
    painter.circle_filled(center, 2.2, Color32::from_rgb(10, 11, 16));
    painter.circle_stroke(center, 2.2, Stroke::new(0.8_f32, Color32::from_rgb(60, 68, 96)));

    // 8. Label & Live Value Text Underneath (Analog Synth readout)
    let text_pos_label = pos2(rect.center().x, rect.max.y - 12.0);
    painter.text(
        text_pos_label,
        nih_plug_egui::egui::Align2::CENTER_TOP,
        label,
        FontId::monospace(8.0),
        if is_hovered { CyberTheme::TEXT_PRIMARY } else { CyberTheme::TEXT_MUTED },
    );

    // Live formatted value text (self-documenting analog synth faceplate)
    let val_str = match custom_readout {
        Some(s) => s.to_string(),
        None => format_val(*value, is_log),
    };
    let text_pos_val = pos2(rect.center().x, rect.max.y - 3.0);
    painter.text(
        text_pos_val,
        nih_plug_egui::egui::Align2::CENTER_TOP,
        val_str,
        FontId::monospace(7.2),
        if is_hovered { color } else { Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 180) },
    );

    // Format value tooltip on hover
    if is_hovered {
        let full_val_str = format_val(*value, is_log);
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
            format!("{:.1}k", val / 1000.0)
        } else if val < 0.1 {
            format!("{:.0}ms", val * 1000.0)
        } else {
            format!("{:.0}Hz", val)
        }
    } else if val.abs() < 0.001 {
        "0.00".into()
    } else if val.fract().abs() < 0.001 {
        format!("{:.0}", val)
    } else {
        format!("{:.2}", val)
    }
}
