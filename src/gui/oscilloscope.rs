//! Neon Oscilloscope Audio Visualizer.

use super::theme::CyberTheme;
use nih_plug_egui::egui::{pos2, Color32, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

pub fn draw_oscilloscope(ui: &mut Ui, samples: &[f32], width: f32, height: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();

    // Recessed background
    painter.rect_filled(rect, 4.0, CyberTheme::BG_RECESSED);
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0_f32, CyberTheme::BORDER_DARK), StrokeKind::Inside);

    // Subtle cyberpunk grid lines
    let center_y = rect.center().y;
    painter.line_segment(
        [pos2(rect.min.x, center_y), pos2(rect.max.x, center_y)],
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 240, 255, 30)),
    );

    for i in 1..4 {
        let x = rect.min.x + (rect.width() * i as f32 / 4.0);
        painter.line_segment(
            [pos2(x, rect.min.y), pos2(x, rect.max.y)],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 240, 255, 20)),
        );
    }

    if samples.is_empty() {
        return response;
    }

    // Draw glowing neon waveform trace
    let num_points = samples.len().min(256);
    let step_x = rect.width() / (num_points - 1) as f32;
    let amp_scale = rect.height() * 0.42;

    let points: Vec<Pos2> = samples.iter().take(num_points).enumerate().map(|(i, &s)| {
        let x = rect.min.x + i as f32 * step_x;
        let y = center_y - (s * amp_scale).clamp(-rect.height() * 0.48, rect.height() * 0.48);
        pos2(x, y)
    }).collect();

    // Outer glow pass
    for i in 0..points.len().saturating_sub(1) {
        painter.line_segment(
            [points[i], points[i + 1]],
            Stroke::new(3.0_f32, Color32::from_rgba_unmultiplied(0, 240, 255, 60)),
        );
    }

    // Bright core pass
    for i in 0..points.len().saturating_sub(1) {
        painter.line_segment(
            [points[i], points[i + 1]],
            Stroke::new(1.5_f32, CyberTheme::NEON_CYAN),
        );
    }

    // Corner HUD brackets
    draw_hud_brackets(painter, rect);

    response
}

fn draw_hud_brackets(painter: &nih_plug_egui::egui::Painter, rect: Rect) {
    let len = 6.0;
    let color = Color32::from_rgba_unmultiplied(0, 240, 255, 120);
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
