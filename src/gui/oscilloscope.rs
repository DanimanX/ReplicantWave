//! Neon 3D Vector Oscilloscope Audio Visualizer.
//!
//! Renders hardware-accelerated volumetric audio traces, dual-layer phosphor persistence,
//! dynamic level meters, and cyberpunk HUD telemetry.

use super::theme::CyberTheme;
use nih_plug_egui::egui::{
    epaint::Shape, pos2, Color32, Mesh, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
};

pub fn draw_oscilloscope(ui: &mut Ui, samples: &[f32], width: f32, height: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();

    // 1. Recessed background with subtle glow border
    painter.rect_filled(rect, 4.0, Color32::from_rgb(8, 10, 16));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0_f32, CyberTheme::BORDER_DARK),
        StrokeKind::Inside,
    );

    // 2. Cyberpunk Reticle & Scanlines
    let center_y = rect.center().y;
    let grid_color = Color32::from_rgba_unmultiplied(0, 240, 255, 25);

    // Center dashed reticle
    let mut x_ret = rect.min.x + 8.0;
    while x_ret < rect.max.x - 8.0 {
        painter.line_segment([pos2(x_ret, center_y), pos2(x_ret + 3.0, center_y)], Stroke::new(1.0_f32, grid_color));
        x_ret += 7.0;
    }

    // Vertical quarter divisions
    for i in 1..4 {
        let x = rect.min.x + (rect.width() * i as f32 / 4.0);
        painter.line_segment(
            [pos2(x, rect.min.y + 4.0), pos2(x, rect.max.y - 4.0)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 240, 255, 18)),
        );
    }

    // 3. Audio Level Meter (Right margin)
    let meter_x = rect.max.x - 8.0;
    let mut peak: f32 = 0.0;
    for &s in samples.iter().take(256) {
        let abs = s.abs();
        if abs > peak {
            peak = abs;
        }
    }
    let meter_h = (rect.height() - 14.0) * peak.clamp(0.0, 1.0);
    painter.line_segment(
        [pos2(meter_x, rect.max.y - 7.0), pos2(meter_x, rect.max.y - 7.0 - meter_h)],
        Stroke::new(3.0_f32, if peak > 0.9 { CyberTheme::NEON_PINK } else { CyberTheme::NEON_LIME }),
    );

    if samples.is_empty() {
        // Idle baseline pulse
        painter.line_segment(
            [pos2(rect.min.x + 8.0, center_y), pos2(rect.max.x - 14.0, center_y)],
            Stroke::new(1.5_f32, Color32::from_rgba_unmultiplied(0, 240, 255, 90)),
        );
        draw_hud_brackets(painter, rect);
        return response;
    }

    // 4. Waveform Coordinates
    let num_points = samples.len().min(128);
    let draw_w = rect.width() - 24.0;
    let step_x = draw_w / (num_points - 1) as f32;
    let amp_scale = rect.height() * 0.40;

    let points: Vec<Pos2> = samples.iter().take(num_points).enumerate().map(|(i, &s)| {
        let x = rect.min.x + 8.0 + i as f32 * step_x;
        let y = center_y - (s * amp_scale).clamp(-rect.height() * 0.45, rect.height() * 0.45);
        pos2(x, y)
    }).collect();

    // 5. GPU-Accelerated Volumetric Ribbon (Gradient Mesh)
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

    // 6. Phosphor Persistence Afterglow Layer (3D Echo effect)
    for i in 0..points.len().saturating_sub(1) {
        let p0 = pos2(points[i].x - 1.0, points[i].y + 1.0);
        let p1 = pos2(points[i + 1].x - 1.0, points[i + 1].y + 1.0);
        painter.line_segment(
            [p0, p1],
            Stroke::new(2.5_f32, Color32::from_rgba_unmultiplied(168, 85, 247, 65)),
        );
    }

    // 7. Outer Cyan Neon Halo
    for i in 0..points.len().saturating_sub(1) {
        painter.line_segment(
            [points[i], points[i + 1]],
            Stroke::new(4.0_f32, Color32::from_rgba_unmultiplied(0, 240, 255, 60)),
        );
    }

    // 8. Solid Core Laser Beam
    for i in 0..points.len().saturating_sub(1) {
        painter.line_segment(
            [points[i], points[i + 1]],
            Stroke::new(1.6_f32, CyberTheme::NEON_CYAN),
        );
    }

    // 9. Telemetry Readout
    painter.text(
        pos2(rect.min.x + 8.0, rect.min.y + 4.0),
        nih_plug_egui::egui::Align2::LEFT_TOP,
        "DSP VECTOR SCOPE // 64-BIT SIMD",
        nih_plug_egui::egui::FontId::monospace(8.5),
        Color32::from_rgba_unmultiplied(0, 240, 255, 170),
    );

    // Corner HUD brackets
    draw_hud_brackets(painter, rect);

    response
}

fn draw_hud_brackets(painter: &nih_plug_egui::egui::Painter, rect: Rect) {
    let len = 5.0;
    let color = Color32::from_rgba_unmultiplied(0, 240, 255, 90);
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
