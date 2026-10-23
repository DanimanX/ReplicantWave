//! 3D GPU-Accelerated Animated Futuristic Cyberpunk Wavetable Visualizer.
//!
//! Renders a dynamic 3D perspective waterfall mesh with real-time orbital camera motion,
//! volumetric GPU polygon fills, sweeping depth laser scanners, harmonic cross-ribbons,
//! crest energy particles, and cybernetic telemetry HUD.

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
    // Request continuous GPU repaint for smooth 60 FPS futuristic animation
    ui.ctx().request_repaint();

    // High-resolution monotonic clock for rock-solid fluid animation
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f32())
        .unwrap_or(0.0);

    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();

    // 1. Futuristic Recessed Viewport Frame
    painter.rect_filled(rect, 4.0, Color32::from_rgb(6, 8, 14));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0_f32, CyberTheme::BORDER_DARK),
        StrokeKind::Inside,
    );

    // Dynamic horizontal scanlines
    let scanline_step = 6.0;
    let mut scan_y = rect.min.y + 2.0;
    while scan_y < rect.max.y {
        painter.line_segment(
            [pos2(rect.min.x + 2.0, scan_y), pos2(rect.max.x - 2.0, scan_y)],
            Stroke::new(0.5_f32, Color32::from_rgba_unmultiplied(12, 16, 28, 45)),
        );
        scan_y += scanline_step;
    }

    // 2. 3D Camera Orbit & Perspective Projection
    // Smooth cinematic breathing camera (yaw and pitch rotate gently in real time)
    let cam_yaw = (t * 0.7).sin() * 0.07;
    let cam_pitch = 0.22 + (t * 0.4).cos() * 0.04;
    let margin_x = 12.0;

    let base_y_front = rect.max.y - 10.0;
    let base_y_back = rect.min.y + 16.0;
    let depth_offset_x = 24.0 + cam_yaw * 40.0;

    // Helper: calculate 3D projected screen coordinates for a point (phase [0..1], amp [-1..1], z [0..1])
    let project_3d = |phase: f32, amp: f32, z: f32| -> Pos2 {
        let perspective_scale = 0.80 + 0.20 * z;
        let baseline_y = base_y_front - (1.0 - z) * (base_y_front - base_y_back);
        let start_x = rect.min.x + margin_x + (1.0 - z) * depth_offset_x;
        let avail_w = (rect.width() - margin_x * 2.0 - depth_offset_x) * perspective_scale;

        let x = start_x + phase * avail_w;
        let amp_scale = (rect.height() * 0.25) * perspective_scale * (1.0 + cam_pitch * 0.5);
        let y = baseline_y - amp * amp_scale;
        pos2(x, y)
    };

    // 3. 3D Cyber Matrix Floor Grid (Perspective Rails)
    let num_rails = 6;
    for r in 0..num_rails {
        let phase = r as f32 / (num_rails - 1) as f32;
        let p_back = project_3d(phase, 0.0, 0.0);
        let p_front = project_3d(phase, 0.0, 1.0);
        painter.line_segment(
            [p_back, p_front],
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(color.r() / 4, color.g() / 4, color.b() / 4, 40)),
        );
    }

    // 4. Background 3D Wireframe Waterfall Slices
    let num_slices = 8;
    let num_points = 54;
    let total_frames = wt.num_frames().max(1);

    // Laser scanner position continuously sweeping back and forth through depth
    let scan_z = (t * 0.9).sin().abs();

    let mut slice_points: Vec<Vec<Pos2>> = Vec::with_capacity(num_slices);

    for s in 0..num_slices {
        let z = s as f32 / (num_slices - 1) as f32;
        let frame_idx = ((1.0 - z) * (total_frames - 1) as f32).round() as usize;

        // Is this slice currently illuminated by the sweeping 3D laser?
        let dist_to_scan = (z - scan_z).abs();
        let is_scanned = dist_to_scan < 0.15;
        let scan_boost = if is_scanned { (1.0 - dist_to_scan / 0.15) * 85.0 } else { 0.0 };

        // Depth fog color with animated wave pulse
        let base_alpha = 30.0 + z * 55.0 + scan_boost;
        let depth_tint = 0.40 + z * 0.60;
        let slice_col = Color32::from_rgba_unmultiplied(
            ((color.r() as f32 * depth_tint) + scan_boost).min(255.0) as u8,
            ((color.g() as f32 * depth_tint) + scan_boost).min(255.0) as u8,
            ((color.b() as f32 * depth_tint) + scan_boost).min(255.0) as u8,
            base_alpha.min(240.0) as u8,
        );

        let pts: Vec<Pos2> = (0..num_points).map(|i| {
            let phase = i as f32 / (num_points - 1) as f32;
            // Living 3D undulating wave motion along the holographic terrain
            let ripple = 1.0 + 0.08 * (phase * 14.0 - t * 3.5 + z * 7.0).sin();
            let amp = wt.sample_frame(frame_idx, phase) * ripple;
            project_3d(phase, amp, z)
        }).collect();

        // Draw wireframe slice line
        let stroke_w = if is_scanned { 1.6_f32 } else { 1.0_f32 };
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment([pts[i], pts[i + 1]], Stroke::new(stroke_w, slice_col));
        }

        slice_points.push(pts);
    }

    // 5. Longitudinal 3D Harmonics Ribbons
    let key_phases = [0, num_points / 4, num_points / 2, 3 * num_points / 4, num_points - 1];
    for &idx in &key_phases {
        for s in 0..num_slices.saturating_sub(1) {
            let p0 = slice_points[s][idx];
            let p1 = slice_points[s + 1][idx];
            let alpha = 24 + (s * 5) as u8;
            painter.line_segment(
                [p0, p1],
                Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(color.r() / 3, color.g() / 3, color.b() / 3, alpha)),
            );
        }
    }

    // 6. Active Morphed 3D Waveform (Volumetric GPU Triangle Mesh & Glowing Laser Core)
    let z_active = (1.0 - position.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let active_points: Vec<Pos2> = (0..num_points).map(|i| {
        let phase = i as f32 / (num_points - 1) as f32;
        // Animated living harmonic pulsation across the active wave
        let pulse = 1.0 + 0.05 * (phase * 16.0 - t * 4.0).sin();
        let amp = wt.sample(position, phase) * pulse;
        project_3d(phase, amp, z_active)
    }).collect();

    let baseline_points: Vec<Pos2> = (0..num_points).map(|i| {
        let phase = i as f32 / (num_points - 1) as f32;
        project_3d(phase, 0.0, z_active)
    }).collect();

    // GPU Volumetric Holographic Mesh (Hardware Triangles)
    let mut mesh = Mesh::default();
    let energy_pulse = 135.0 + (t * 5.0).sin() * 30.0;
    let mesh_top_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), energy_pulse as u8);
    let mesh_base_color = Color32::from_rgba_unmultiplied(color.r() / 4, color.g() / 4, color.b() / 4, 12);

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
    // Pass A: Ambient Bloom Glow
    for i in 0..active_points.len().saturating_sub(1) {
        painter.line_segment(
            [active_points[i], active_points[i + 1]],
            Stroke::new(4.5_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 85)),
        );
    }
    // Pass B: Intense Core Laser Beam
    for i in 0..active_points.len().saturating_sub(1) {
        painter.line_segment(
            [active_points[i], active_points[i + 1]],
            Stroke::new(1.8_f32, color),
        );
    }

    // 7. Dynamic Cyber Energy Orb & Peak Target Marker
    let mut max_y = f32::INFINITY;
    let mut peak_pos = active_points[0];
    for &p in &active_points {
        if p.y < max_y {
            max_y = p.y;
            peak_pos = p;
        }
    }

    // Peak neon diamond target
    let d_size = 3.2;
    painter.line_segment([pos2(peak_pos.x, peak_pos.y - d_size), pos2(peak_pos.x + d_size, peak_pos.y)], Stroke::new(1.5_f32, Color32::WHITE));
    painter.line_segment([pos2(peak_pos.x + d_size, peak_pos.y), pos2(peak_pos.x, peak_pos.y + d_size)], Stroke::new(1.5_f32, Color32::WHITE));
    painter.line_segment([pos2(peak_pos.x, peak_pos.y + d_size), pos2(peak_pos.x - d_size, peak_pos.y)], Stroke::new(1.5_f32, Color32::WHITE));
    painter.line_segment([pos2(peak_pos.x - d_size, peak_pos.y), pos2(peak_pos.x, peak_pos.y - d_size)], Stroke::new(1.5_f32, Color32::WHITE));

    // Traveling Cyber Energy Orb orbiting the active wave crest
    let orb_phase = (t * 0.45).fract();
    let orb_idx = (orb_phase * (active_points.len() - 1) as f32).floor() as usize;
    if orb_idx < active_points.len() {
        let orb_pos = active_points[orb_idx];
        painter.circle_filled(orb_pos, 2.5, Color32::WHITE);
        painter.circle_stroke(orb_pos, 5.0, Stroke::new(1.2_f32, color));
    }

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
        [pos2(rail_x - 3.5, cursor_y), pos2(rail_x + 3.5, cursor_y)],
        Stroke::new(2.0_f32, color),
    );

    // 9. Cyberpunk Telemetry HUD
    let current_frame = (position * (total_frames - 1) as f32).round() as usize + 1;
    let hud_text = format!("3D WT // FRM {:02}/{} | POS: {:02.0}%", current_frame, total_frames, position * 100.0);
    painter.text(
        pos2(rect.min.x + 8.0, rect.min.y + 4.0),
        nih_plug_egui::egui::Align2::LEFT_TOP,
        hud_text,
        nih_plug_egui::egui::FontId::monospace(8.5),
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 195),
    );

    // Corner HUD Brackets
    draw_hud_brackets(painter, rect);

    response
}

fn draw_hud_brackets(painter: &nih_plug_egui::egui::Painter, rect: Rect) {
    let len = 5.0;
    let s = Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 240, 255, 95));

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
