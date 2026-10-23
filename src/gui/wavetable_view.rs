//! Wavetable 2D/3D Morph Visualizer.

use super::theme::CyberTheme;
use crate::audio::wavetable::Wavetable;
use nih_plug_egui::egui::{pos2, Color32, Pos2, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

pub fn draw_wavetable_preview(
    ui: &mut Ui,
    wt: &Wavetable,
    position: f32,
    color: Color32,
    width: f32,
    height: f32,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();

    // Recessed background
    painter.rect_filled(rect, 3.0, CyberTheme::BG_RECESSED);
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, CyberTheme::BORDER_DARK), StrokeKind::Inside);

    let center_y = rect.center().y;
    let num_points = 64;
    let step_x = rect.width() / (num_points - 1) as f32;
    let amp_scale = rect.height() * 0.40;

    // Background ghost frame: render first and last frames in dim trace
    if wt.num_frames() > 1 {
        let draw_ghost = |frame_idx: usize, alpha: u8| {
            let pts: Vec<Pos2> = (0..num_points).map(|i| {
                let phase = i as f32 / (num_points - 1) as f32;
                let s = wt.sample_frame(frame_idx, phase);
                pos2(rect.min.x + i as f32 * step_x, center_y - s * amp_scale * 0.7)
            }).collect();

            for j in 0..pts.len().saturating_sub(1) {
                painter.line_segment(
                    [pts[j], pts[j + 1]],
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)),
                );
            }
        };

        draw_ghost(0, 30);
        draw_ghost(wt.num_frames() - 1, 30);
    }

    // Active morphed waveform
    let pts: Vec<Pos2> = (0..num_points).map(|i| {
        let phase = i as f32 / (num_points - 1) as f32;
        let s = wt.sample(position, phase);
        pos2(rect.min.x + i as f32 * step_x, center_y - s * amp_scale)
    }).collect();

    // Glow pass
    for i in 0..pts.len().saturating_sub(1) {
        painter.line_segment(
            [pts[i], pts[i + 1]],
            Stroke::new(3.0_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 65)),
        );
    }

    // Main line pass
    for i in 0..pts.len().saturating_sub(1) {
        painter.line_segment(
            [pts[i], pts[i + 1]],
            Stroke::new(1.5_f32, color),
        );
    }

    // Position progress bar underneath
    let bar_y = rect.max.y - 2.5;
    let bar_w = rect.width() * position.clamp(0.0, 1.0);
    painter.line_segment(
        [pos2(rect.min.x, bar_y), pos2(rect.min.x + bar_w, bar_y)],
        Stroke::new(2.0_f32, color),
    );

    response
}
