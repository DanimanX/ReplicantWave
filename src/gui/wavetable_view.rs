//! Holographic wavetable display for the NIGHT CITY HUD.
//!
//! The editor works from compact pre-sampled thumbnails (`WaveThumb`) so it
//! never needs to borrow the audio engine's wavetable bank while painting.
//!
//! All motion is driven by [`super::time::now_secs`], a monotonic clock with
//! sub-frame resolution. Do **not** feed `SystemTime` epoch seconds into an
//! `f32`: at ~1.79e9 s the ULP is ~213 s, which froze every animation.

use super::theme::CyberTheme;
use crate::audio::wavetable::Wavetable;
use nih_plug_egui::egui::{
    epaint::FontId, epaint::Shape, pos2, Color32, Mesh, Pos2, Response, Sense, Stroke, StrokeKind,
    Ui, Vec2,
};

const THUMB_SLICES: usize = 11;
const THUMB_POINTS: usize = 96;

#[derive(Clone, Debug, Default)]
pub struct WaveThumb {
    pub name: String,
    pub slices: Vec<Vec<f32>>,
    pub num_frames: usize,
    /// Largest absolute sample in the table, used to auto-fit the display.
    pub peak: f32,
}

impl WaveThumb {
    pub fn from_wavetable(wt: &Wavetable) -> Self {
        let nf = wt.num_frames().max(1);
        let mut slices = Vec::with_capacity(THUMB_SLICES);
        for s in 0..THUMB_SLICES {
            let frame = ((s as f32 / (THUMB_SLICES - 1) as f32) * (nf - 1) as f32).round() as usize;
            let mut pts = Vec::with_capacity(THUMB_POINTS);
            for i in 0..THUMB_POINTS {
                let phase = i as f32 / (THUMB_POINTS - 1) as f32;
                pts.push(wt.sample_frame(frame, phase));
            }
            slices.push(pts);
        }
        let peak = slices
            .iter()
            .flat_map(|s| s.iter())
            .fold(0.0f32, |acc, v| acc.max(v.abs()))
            .max(0.05);
        Self {
            name: wt.name.clone(),
            slices,
            num_frames: nf,
            peak,
        }
    }

    #[inline]
    fn sample_slice(&self, idx: usize, phase: f32) -> f32 {
        let slice = &self.slices[idx.min(self.slices.len().saturating_sub(1))];
        let n = slice.len();
        if n == 0 {
            return 0.0;
        }
        let x = phase.clamp(0.0, 0.9999) * (n - 1) as f32;
        let i0 = x.floor() as usize;
        let i1 = (i0 + 1).min(n - 1);
        slice[i0] + (x - i0 as f32) * (slice[i1] - slice[i0])
    }

    pub fn sample(&self, position: f32, phase: f32) -> f32 {
        let n = self.slices.len();
        if n == 1 {
            return self.sample_slice(0, phase);
        }
        let p = position.clamp(0.0, 1.0) * (n - 1) as f32;
        let i0 = p.floor() as usize;
        let i1 = (i0 + 1).min(n - 1);
        let frac = p - i0 as f32;
        let a = self.sample_slice(i0, phase);
        let b = self.sample_slice(i1, phase);
        a + frac * (b - a)
    }
}

pub fn draw_wavetable_preview(
    ui: &mut Ui,
    wt: &WaveThumb,
    position: f32,
    color: Color32,
    width: f32,
    height: f32,
) -> Response {
    let t = super::time::now_secs();
    // Keep the hologram alive even if the parent stops asking for repaints.
    ui.ctx().request_repaint();

    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    // Hard-clip to the viewport: a deep 3D stack must never bleed into the
    // knobs below the display.
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));

    // Recessed CRT well.
    painter.rect_filled(rect, 2.0, Color32::from_rgb(4, 2, 11));
    painter.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, CyberTheme::alpha(color, 55)),
        StrokeKind::Inside,
    );

    // Fine grid.
    for i in 1..8 {
        let x = rect.min.x + rect.width() * i as f32 / 8.0;
        painter.line_segment(
            [pos2(x, rect.min.y + 2.0), pos2(x, rect.max.y - 2.0)],
            Stroke::new(0.6_f32, CyberTheme::alpha(color, 18)),
        );
    }
    for i in 1..3 {
        let y = rect.min.y + rect.height() * i as f32 / 3.0;
        painter.line_segment(
            [pos2(rect.min.x + 2.0, y), pos2(rect.max.x - 2.0, y)],
            Stroke::new(0.6_f32, CyberTheme::alpha(color, 16)),
        );
    }

    let margin_x = 8.0;

    // Vertical budget: the wave excursion is reserved first, and the slice
    // baselines share whatever is left. This guarantees every slice fits,
    // regardless of how hot the wavetable is.
    let top_y = rect.min.y + 12.0;
    let bottom_y = rect.max.y - 10.0;
    let span_y = (bottom_y - top_y).max(24.0);
    let fit = (1.0 / wt.peak).clamp(0.55, 1.6);

    // --- Orbital camera -----------------------------------------------------
    // Slow cinematic breathing so the stack reads as a volume, not a still.
    let cam_yaw = (t * 0.62).sin() * 9.0; // horizontal parallax in px
    let cam_pitch = 0.5 + 0.5 * (t * 0.44 + 1.1).sin(); // 0..1 depth tilt
    let depth_x = 15.0 + cam_pitch * 9.0;

    // Amplitudes peak at <= 1.0 by construction; the pitch breathing only ever
    // shrinks the excursion so the reserved budget still holds.
    let amp_budget = (span_y * 0.36 * fit).min(span_y * 0.44);
    let band = (span_y - 2.0 * amp_budget).max(10.0);
    let y_front = top_y + amp_budget + band;
    let y_back = top_y + amp_budget;
    let amp_base = amp_budget * (0.90 + 0.10 * cam_pitch);

    let project = move |phase: f32, amp: f32, z: f32| -> Pos2 {
        let persp = 0.78 + 0.22 * z;
        let baseline = y_back + z * (y_front - y_back);
        let start_x = rect.min.x + margin_x + (1.0 - z) * depth_x + cam_yaw * (1.0 - z);
        let avail_w =
            (rect.width() - margin_x * 2.0 - depth_x) * persp * (1.0 + 0.04 * cam_yaw / 9.0);
        let x = start_x + phase * avail_w;
        pos2(x, baseline - amp.clamp(-1.05, 1.05) * amp_base * persp)
    };

    // Traveling depth scan (triangle so it sweeps back and forth without
    // teleporting at the turn-around).
    let scan_phase = (t * 0.42).fract();
    let scan_z = if scan_phase < 0.5 {
        scan_phase * 2.0
    } else {
        (1.0 - scan_phase) * 2.0
    };

    let mut slices: Vec<Vec<Pos2>> = Vec::with_capacity(THUMB_SLICES);

    for s in 0..THUMB_SLICES {
        let z = s as f32 / (THUMB_SLICES - 1) as f32;
        let dist = (z - scan_z).abs();
        let lit = (1.0 - dist / 0.22).max(0.0);
        let alpha = (22.0 + z * 62.0 + lit * 130.0).min(240.0) as u8;
        let tint = 0.34 + z * 0.66 + lit * 0.25;
        let col = Color32::from_rgba_unmultiplied(
            (color.r() as f32 * tint).min(255.0) as u8,
            (color.g() as f32 * tint).min(255.0) as u8,
            (color.b() as f32 * tint).min(255.0) as u8,
            alpha,
        );

        // Each slice undulates on its own phase so the stack visibly breathes.
        let slice_phase = z * 5.2 + s as f32 * 0.4;
        let pts: Vec<Pos2> = (0..THUMB_POINTS)
            .map(|i| {
                let phase = i as f32 / (THUMB_POINTS - 1) as f32;
                let ripple = 1.0 + 0.085 * (phase * 12.0 - t * 2.4 + slice_phase).sin();
                let drift = 0.022 * (phase * 5.0 + t * 1.3 + slice_phase).sin();
                project(phase, wt.sample_slice_public(s, phase) * ripple + drift, z)
            })
            .collect();

        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[i], pts[i + 1]],
                Stroke::new(if lit > 0.25 { 1.5_f32 } else { 0.9_f32 }, col),
            );
        }
        // Glowing sweep band around the scan plane.
        if lit > 0.3 {
            for i in 0..pts.len().saturating_sub(1) {
                painter.line_segment(
                    [pts[i], pts[i + 1]],
                    Stroke::new(4.0_f32, CyberTheme::alpha(color, (lit * 45.0) as u8)),
                );
            }
        }
        slices.push(pts);
    }

    // Longitudinal ribbons give the stack a holographic volume.
    for &idx in &[
        THUMB_POINTS / 5,
        2 * THUMB_POINTS / 5,
        THUMB_POINTS / 2,
        3 * THUMB_POINTS / 5,
        4 * THUMB_POINTS / 5,
    ] {
        for s in 0..slices.len().saturating_sub(1) {
            painter.line_segment(
                [slices[s][idx], slices[s + 1][idx]],
                Stroke::new(0.7_f32, CyberTheme::alpha(color, 30)),
            );
        }
    }

    // --- Active morphed wave -------------------------------------------------
    let z_active = 1.0 - position.clamp(0.0, 1.0);
    let active: Vec<Pos2> = (0..THUMB_POINTS)
        .map(|i| {
            let phase = i as f32 / (THUMB_POINTS - 1) as f32;
            // Faster, deeper pulse so the focused wave clearly moves.
            let pulse = 1.0 + 0.07 * (phase * 15.0 - t * 3.4).sin();
            let drift = 0.018 * (phase * 6.0 + t * 1.9).sin();
            project(phase, wt.sample(position, phase) * pulse + drift, z_active)
        })
        .collect();
    let baseline: Vec<Pos2> = (0..THUMB_POINTS)
        .map(|i| project(i as f32 / (THUMB_POINTS - 1) as f32, 0.0, z_active))
        .collect();

    // Filled translucent ribbon under the active trace.
    let mut mesh = Mesh::default();
    let energy = 0.5 + 0.5 * (t * 2.2).sin();
    let top = CyberTheme::alpha(color, (80.0 + energy * 55.0) as u8);
    let bot = CyberTheme::alpha(color, 8);
    for i in 0..THUMB_POINTS.saturating_sub(1) {
        let base = (i * 4) as u32;
        mesh.colored_vertex(active[i], top);
        mesh.colored_vertex(active[i + 1], top);
        mesh.colored_vertex(baseline[i], bot);
        mesh.colored_vertex(baseline[i + 1], bot);
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base + 1, base + 3, base + 2);
    }
    painter.add(Shape::Mesh(mesh.into()));

    // Bloom + core trace.
    for i in 0..active.len().saturating_sub(1) {
        painter.line_segment(
            [active[i], active[i + 1]],
            Stroke::new(5.0_f32, CyberTheme::alpha(color, 48)),
        );
    }
    for i in 0..active.len().saturating_sub(1) {
        painter.line_segment([active[i], active[i + 1]], Stroke::new(1.7_f32, color));
    }

    // Peak marker.
    if let Some(peak) = active.iter().min_by(|a, b| a.y.partial_cmp(&b.y).unwrap()) {
        let d = 3.0;
        painter.line_segment(
            [pos2(peak.x - d, peak.y), pos2(peak.x + d, peak.y)],
            Stroke::new(1.2_f32, Color32::WHITE),
        );
        painter.line_segment(
            [pos2(peak.x, peak.y - d), pos2(peak.x, peak.y + d)],
            Stroke::new(1.2_f32, Color32::WHITE),
        );
    }

    // Energy orb orbiting the active crest: unmistakable motion cue.
    let orb_phase = (t * 0.55).fract();
    let orb_idx = (orb_phase * (active.len() - 1) as f32) as usize;
    if let Some(orb) = active.get(orb_idx) {
        painter.circle_filled(*orb, 2.4, Color32::WHITE);
        painter.circle_stroke(
            *orb,
            5.5,
            Stroke::new(1.2_f32, CyberTheme::alpha(color, 210)),
        );
        painter.circle_filled(*orb, 7.5, CyberTheme::alpha(color, 22));
    }

    // Position rail.
    let rail_x = rect.max.x - 5.0;
    painter.line_segment(
        [
            pos2(rail_x, rect.min.y + 14.0),
            pos2(rail_x, rect.max.y - 10.0),
        ],
        Stroke::new(1.0_f32, CyberTheme::alpha(color, 40)),
    );
    let ry = rect.max.y - 10.0 - position.clamp(0.0, 1.0) * (rect.height() - 24.0);
    painter.line_segment(
        [pos2(rail_x - 3.0, ry), pos2(rail_x + 3.0, ry)],
        Stroke::new(2.0_f32, color),
    );

    painter.text(
        pos2(rect.min.x + 5.0, rect.min.y + 3.0),
        nih_plug_egui::egui::Align2::LEFT_TOP,
        format!(
            "WT {:02} // {}",
            (position * (wt.num_frames.saturating_sub(1)) as f32).round() as usize + 1,
            wt.name
        ),
        FontId::monospace(7.5),
        CyberTheme::alpha(color, 150),
    );

    response
}

impl WaveThumb {
    // A public accessor used by the painter above; keeps indices clamped.
    fn sample_slice_public(&self, idx: usize, phase: f32) -> f32 {
        self.sample_slice(idx, phase)
    }
}
