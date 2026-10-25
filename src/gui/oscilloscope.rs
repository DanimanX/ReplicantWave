//! Neon 3D Vector Oscilloscope Audio Visualizer.
//!
//! Renders hardware-accelerated volumetric audio traces, dual-layer phosphor persistence,
//! dynamic level meters, continuous holographic cyber waves, and cyberpunk HUD telemetry.

use super::theme::CyberTheme;
use nih_plug_egui::egui::{
    epaint::Shape, pos2, Color32, Mesh, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use std::f32::consts::PI;

pub fn draw_oscilloscope(ui: &mut Ui, samples: &[f32], width: f32, height: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();

    // 1. Recessed CRT background with subtle glow border
    painter.rect_filled(rect, 4.0, Color32::from_rgb(7, 9, 15));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.2_f32, CyberTheme::BORDER_DARK),
        StrokeKind::Inside,
    );

    // Dynamic horizontal scanlines
    let scanline_step = 5.0;
    let mut scan_y = rect.min.y + 2.0;
    while scan_y < rect.max.y {
        painter.line_segment(
            [
                pos2(rect.min.x + 2.0, scan_y),
                pos2(rect.max.x - 2.0, scan_y),
            ],
            Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(10, 16, 26, 40)),
        );
        scan_y += scanline_step;
    }

    // 2. Cyberpunk Reticle & Scanlines
    let center_y = rect.center().y;
    let grid_color = Color32::from_rgba_unmultiplied(0, 240, 255, 30);

    // Center dashed reticle
    let mut x_ret = rect.min.x + 8.0;
    while x_ret < rect.max.x - 16.0 {
        painter.line_segment(
            [pos2(x_ret, center_y), pos2(x_ret + 3.0, center_y)],
            Stroke::new(1.0_f32, grid_color),
        );
        x_ret += 8.0;
    }

    // Vertical divisions
    for i in 1..4 {
        let x = rect.min.x + (rect.width() * i as f32 / 4.0);
        painter.line_segment(
            [pos2(x, rect.min.y + 4.0), pos2(x, rect.max.y - 4.0)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 240, 255, 20)),
        );
    }

    // 3. Audio Peak Level Meter (Right margin)
    let meter_x = rect.max.x - 9.0;
    let mut peak: f32 = 0.0;
    for &s in samples.iter().take(256) {
        let abs = s.abs();
        if abs > peak {
            peak = abs;
        }
    }

    let meter_h = ((rect.height() - 14.0) * peak.clamp(0.0, 1.0)).max(3.0);
    painter.line_segment(
        [
            pos2(meter_x, rect.max.y - 7.0),
            pos2(meter_x, rect.max.y - 7.0 - meter_h),
        ],
        Stroke::new(3.0_f32, CyberTheme::NEON_PINK),
    );

    // High-resolution clock for living vector trace animation
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f32())
        .unwrap_or(0.0);

    // 4. Waveform Coordinates
    let num_points = 120;
    let draw_w = rect.width() - 26.0;
    let step_x = draw_w / (num_points - 1) as f32;
    let amp_scale = rect.height() * 0.38;

    let points: Vec<Pos2> = if peak > 0.005 && !samples.is_empty() {
        // Live audio trace
        (0..num_points)
            .map(|i| {
                let sample_idx = (i * samples.len()) / num_points;
                let s = samples.get(sample_idx).copied().unwrap_or(0.0);
                let x = rect.min.x + 8.0 + i as f32 * step_x;
                let y =
                    center_y - (s * amp_scale).clamp(-rect.height() * 0.44, rect.height() * 0.44);
                pos2(x, y)
            })
            .collect()
    } else {
        // Living holographic cyber vector carrier wave (matches new_ui.jpeg vector scope)
        (0..num_points)
            .map(|i| {
                let frac = i as f32 / (num_points - 1) as f32;
                let x = rect.min.x + 8.0 + i as f32 * step_x;
                let ripple1 = (frac * 14.0 * PI - t * 6.5).sin() * 0.38;
                let ripple2 = (frac * 26.0 * PI + t * 4.0).sin() * 0.16;
                let burst = if frac > 0.70 {
                    (frac * 36.0 * PI - t * 12.0).sin() * 0.35
                } else {
                    0.0
                };
                let wave_y = ripple1 + ripple2 + burst;
                let y = center_y
                    - (wave_y * amp_scale).clamp(-rect.height() * 0.44, rect.height() * 0.44);
                pos2(x, y)
            })
            .collect()
    };

    // 5. GPU Volumetric Ribbon (Hardware Triangle Mesh)
    let mut mesh = Mesh::default();
    let top_color = Color32::from_rgba_unmultiplied(0, 240, 255, 75);
    let base_color = Color32::from_rgba_unmultiplied(0, 240, 255, 0);

    for i in 0..num_points.saturating_sub(1) {
        let base_idx = (i * 4) as u32;

        let v0 = points[i];
        let v1 = points[i + 1];
        let v2 = pos2(points[i].x, center_y);
        let v3 = pos2(points[i + 1].x, center_y);

        mesh.colored_vertex(v0, top_color);
        mesh.colored_vertex(v1, top_color);
        mesh.colored_vertex(v2, base_color);
        mesh.colored_vertex(v3, base_color);

        mesh.add_triangle(base_idx, base_idx + 1, base_idx + 2);
        mesh.add_triangle(base_idx + 1, base_idx + 3, base_idx + 2);
    }
    painter.add(Shape::Mesh(mesh.into()));

    // 6. Phosphor Persistence 3D Echo Layer (Violet afterglow)
    for i in 0..points.len().saturating_sub(1) {
        let p0 = pos2(points[i].x - 1.2, points[i].y + 1.2);
        let p1 = pos2(points[i + 1].x - 1.2, points[i + 1].y + 1.2);
        painter.line_segment(
            [p0, p1],
            Stroke::new(2.8_f32, Color32::from_rgba_unmultiplied(168, 85, 247, 70)),
        );
    }

    // 7. Outer Cyan Neon Bloom Halo
    for i in 0..points.len().saturating_sub(1) {
        painter.line_segment(
            [points[i], points[i + 1]],
            Stroke::new(4.5_f32, Color32::from_rgba_unmultiplied(0, 240, 255, 65)),
        );
    }

    // 8. Solid Core Laser Beam
    for i in 0..points.len().saturating_sub(1) {
        painter.line_segment(
            [points[i], points[i + 1]],
            Stroke::new(1.8_f32, CyberTheme::NEON_CYAN),
        );
    }

    // 9. Telemetry Readout (Matches new_ui.jpeg: DEP VECTOR SCOPE // 64-BIT SIND)
    painter.text(
        pos2(rect.min.x + 8.0, rect.min.y + 4.0),
        nih_plug_egui::egui::Align2::LEFT_TOP,
        "DEP VECTOR SCOPE // 64-BIT SIND",
        nih_plug_egui::egui::FontId::monospace(8.5),
        Color32::from_rgba_unmultiplied(0, 240, 255, 180),
    );

    // Corner HUD brackets
    draw_hud_brackets(painter, rect);

    response
}

fn draw_hud_brackets(painter: &nih_plug_egui::egui::Painter, rect: Rect) {
    let len = 5.0;
    let color = Color32::from_rgba_unmultiplied(0, 240, 255, 95);
    let s = Stroke::new(1.0_f32, color);

    // Top-Left
    painter.line_segment([rect.min, pos2(rect.min.x + len, rect.min.y)], s);
    painter.line_segment([rect.min, pos2(rect.min.x, rect.min.y + len)], s);

    // Top-Right
    let tr = pos2(rect.max.x, rect.min.y);
    painter.line_segment([tr, pos2(tr.x - len, tr.y)], s);
    painter.line_segment([tr, pos2(tr.x, tr.y + len)], s);

    // Bottom-Left
    let bl = pos2(rect.min.x, rect.max.y);
    painter.line_segment([bl, pos2(bl.x + len, bl.y)], s);
    painter.line_segment([bl, pos2(bl.x, bl.y - len)], s);

    // Bottom-Right
    painter.line_segment([rect.max, pos2(rect.max.x - len, rect.max.y)], s);
    painter.line_segment([rect.max, pos2(rect.max.x, rect.max.y - len)], s);
}
