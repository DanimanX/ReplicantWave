//! 3D GPU-Accelerated Futuristic Cyberpunk Wavetable Visualizer.
//!
//! Renders a true 3D perspective waterfall mesh with volumetric GPU polygon fills,
//! depth fog, longitudinal wireframe ribbons, and real-time morph slice tracking.

use super::theme::CyberTheme;
use crate::audio::wavetable::Wavetable;
use nih_plug_egui::egui::{
    epaint::Shape, pos2, Color32, Mesh, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
};

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

    // 1. Futuristic Recessed Viewport with Chamfered Frame
    painter.rect_filled(rect, 4.0, Color32::from_rgb(8, 10, 16));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0_f32, CyberTheme::BORDER_DARK),
        StrokeKind::Inside,
    );

    // Subtle background scanlines
    let scanline_step = 6.0;
    let mut scan_y = rect.min.y + 2.0;
    while scan_y < rect.max.y {
        painter.line_segment(
            [pos2(rect.min.x + 2.0, scan_y), pos2(rect.max.x - 2.0, scan_y)],
            Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(15, 20, 32, 40)),
        );
        scan_y += scanline_step;
    }

    // 2. 3D Perspective Projection Setup
    // Z goes from 0.0 (deep in the background, top-right) to 1.0 (foreground, bottom-left)
    let margin_x = 10.0;
    let base_y_front = rect.max.y - 10.0;
    let base_y_back = rect.min.y + 16.0;
    let depth_offset_x = 22.0;

    // Helper: calculate 3D projected screen coordinates for a point (phase [0..1], amp [-1..1], z [0..1])
    let project_3d = |phase: f32, amp: f32, z: f32| -> Pos2 {
        // Perspective narrowing at distance
        let perspective_scale = 0.82 + 0.18 * z;
        let baseline_y = base_y_front - (1.0 - z) * (base_y_front - base_y_back);
        let start_x = rect.min.x + margin_x + (1.0 - z) * depth_offset_x;
        let avail_w = (rect.width() - margin_x * 2.0 - depth_offset_x) * perspective_scale;

        let x = start_x + phase * avail_w;
        let amp_scale = (rect.height() * 0.23) * perspective_scale;
        let y = baseline_y - amp * amp_scale;
        pos2(x, y)
    };

    // 3. 3D Cyber Grid Floor (Perspective Rails)
    let num_rails = 5;
    for r in 0..num_rails {
        let phase = r as f32 / (num_rails - 1) as f32;
        let p_back = project_3d(phase, 0.0, 0.0);
        let p_front = project_3d(phase, 0.0, 1.0);
        painter.line_segment(
            [p_back, p_front],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(color.r() / 3, color.g() / 3, color.b() / 3, 35)),
        );
    }

    // 4. Background 3D Wireframe Waterfall Slices
    let num_slices = 8;
    let num_points = 48;
    let total_frames = wt.num_frames().max(1);

    // Collect 3D points for all slices to allow drawing longitudinal cross-ribbons
    let mut slice_points: Vec<Vec<Pos2>> = Vec::with_capacity(num_slices);

    for s in 0..num_slices {
        let z = s as f32 / (num_slices - 1) as f32;
        let frame_idx = ((1.0 - z) * (total_frames - 1) as f32).round() as usize;

        // Depth fog color: distant slices fade into dark cyber neon
        let alpha = (25.0 + z * 55.0) as u8;
        let depth_tint = 0.45 + z * 0.55;
        let slice_col = Color32::from_rgba_unmultiplied(
            (color.r() as f32 * depth_tint) as u8,
            (color.g() as f32 * depth_tint) as u8,
            (color.b() as f32 * depth_tint) as u8,
            alpha,
        );

        let pts: Vec<Pos2> = (0..num_points).map(|i| {
            let phase = i as f32 / (num_points - 1) as f32;
            let amp = wt.sample_frame(frame_idx, phase);
            project_3d(phase, amp, z)
        }).collect();

        // Draw wireframe slice line
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment([pts[i], pts[i + 1]], Stroke::new(1.0_f32, slice_col));
        }

        slice_points.push(pts);
    }

    // 5. Longitudinal 3D Terrain Ribbons (Connecting harmonics across slices)
    let key_phases = [0, num_points / 4, num_points / 2, 3 * num_points / 4, num_points - 1];
    for &idx in &key_phases {
        for s in 0..num_slices.saturating_sub(1) {
            let p0 = slice_points[s][idx];
            let p1 = slice_points[s + 1][idx];
            let alpha = 28 + (s * 4) as u8;
            painter.line_segment(
                [p0, p1],
                Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(color.r() / 2, color.g() / 2, color.b() / 2, alpha)),
            );
        }
    }

    // 6. Active Morphed 3D Waveform (Volumetric GPU Mesh & Glowing Neon Core)
    // The active slice is placed in 3D at z_active = 1.0 - position
    let z_active = (1.0 - position.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let active_points: Vec<Pos2> = (0..num_points).map(|i| {
        let phase = i as f32 / (num_points - 1) as f32;
        let amp = wt.sample(position, phase);
        project_3d(phase, amp, z_active)
    }).collect();

    let baseline_points: Vec<Pos2> = (0..num_points).map(|i| {
        let phase = i as f32 / (num_points - 1) as f32;
        project_3d(phase, 0.0, z_active)
    }).collect();

    // GPU-Accelerated Volumetric Hologram Mesh
    // Generates hardware-interpolated gradient triangles from crest down to 3D baseline
    let mut mesh = Mesh::default();
    let mesh_top_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 125);
    let mesh_base_color = Color32::from_rgba_unmultiplied(color.r() / 4, color.g() / 4, color.b() / 4, 10);

    for i in 0..num_points.saturating_sub(1) {
        let base_idx = (i * 4) as u32;

        let v0 = active_points[i];
        let v1 = active_points[i + 1];
        let v2 = baseline_points[i];
        let v3 = baseline_points[i + 1];

        mesh.colored_vertex(v0, mesh_top_color);
        mesh.colored_vertex(v1, mesh_top_color);
        mesh.colored_vertex(v2, mesh_base_color);
        mesh.colored_vertex(v3, mesh_base_color);

        // Quad triangles: (v0, v1, v2) and (v1, v3, v2)
        mesh.add_triangle(base_idx, base_idx + 1, base_idx + 2);
        mesh.add_triangle(base_idx + 1, base_idx + 3, base_idx + 2);
    }
    painter.add(Shape::Mesh(mesh.into()));

    // Multi-pass glowing neon laser trace
    // Pass A: Wide ambient glow
    for i in 0..active_points.len().saturating_sub(1) {
        painter.line_segment(
            [active_points[i], active_points[i + 1]],
            Stroke::new(4.5_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 70)),
        );
    }
    // Pass B: Intense core laser
    for i in 0..active_points.len().saturating_sub(1) {
        painter.line_segment(
            [active_points[i], active_points[i + 1]],
            Stroke::new(1.8_f32, color),
        );
    }

    // 7. Max Crest Cyber Marker (Find highest peak and draw diamond target)
    let mut max_y = f32::INFINITY;
    let mut peak_pos = active_points[0];
    for &p in &active_points {
        if p.y < max_y {
            max_y = p.y;
            peak_pos = p;
        }
    }
    // Draw neon diamond target at peak
    let d_size = 3.0;
    painter.line_segment([pos2(peak_pos.x, peak_pos.y - d_size), pos2(peak_pos.x + d_size, peak_pos.y)], Stroke::new(1.5_f32, Color32::WHITE));
    painter.line_segment([pos2(peak_pos.x + d_size, peak_pos.y), pos2(peak_pos.x, peak_pos.y + d_size)], Stroke::new(1.5_f32, Color32::WHITE));
    painter.line_segment([pos2(peak_pos.x, peak_pos.y + d_size), pos2(peak_pos.x - d_size, peak_pos.y)], Stroke::new(1.5_f32, Color32::WHITE));
    painter.line_segment([pos2(peak_pos.x - d_size, peak_pos.y), pos2(peak_pos.x, peak_pos.y - d_size)], Stroke::new(1.5_f32, Color32::WHITE));

    // 8. Right-Side 3D Depth Rail Indicator
    let rail_x = rect.max.x - 7.0;
    let rail_y_top = rect.min.y + 14.0;
    let rail_y_bot = rect.max.y - 12.0;
    painter.line_segment(
        [pos2(rail_x, rail_y_top), pos2(rail_x, rail_y_bot)],
        Stroke::new(1.0_f32, CyberTheme::BORDER_DARK),
    );

    let cursor_y = rail_y_bot - position.clamp(0.0, 1.0) * (rail_y_bot - rail_y_top);
    painter.line_segment(
        [pos2(rail_x - 3.0, cursor_y), pos2(rail_x + 3.0, cursor_y)],
        Stroke::new(2.0_f32, color),
    );

    // 9. Cyberpunk Telemetry HUD
    let current_frame = (position * (total_frames - 1) as f32).round() as usize + 1;
    let hud_text = format!("3D WT // FRM {:02}/{} | {:02.0}%", current_frame, total_frames, position * 100.0);
    painter.text(
        pos2(rect.min.x + 8.0, rect.min.y + 4.0),
        nih_plug_egui::egui::Align2::LEFT_TOP,
        hud_text,
        nih_plug_egui::egui::FontId::monospace(8.5),
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 180),
    );

    // Corner HUD Brackets
    draw_hud_brackets(painter, rect);

    response
}

fn draw_hud_brackets(painter: &nih_plug_egui::egui::Painter, rect: Rect) {
    let len = 5.0;
    let s = Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 240, 255, 90));

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
