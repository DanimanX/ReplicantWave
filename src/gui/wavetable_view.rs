//! Holographic wavetable display for the NIGHT CITY HUD.
//!
//! The editor works from compact pre-sampled thumbnails (`WaveThumb`) so it
//! never needs to borrow the audio engine's wavetable bank while painting.

use super::theme::CyberTheme;
use crate::audio::wavetable::Wavetable;
use nih_plug_egui::egui::{
    epaint::FontId, epaint::Shape, pos2, Color32, Mesh, Pos2, Response, Sense, Stroke, StrokeKind,
    Ui, Vec2,
};

const THUMB_SLICES: usize = 10;
const THUMB_POINTS: usize = 96;

#[derive(Clone, Debug, Default)]
pub struct WaveThumb {
    pub name: String,
    pub slices: Vec<Vec<f32>>,
    pub num_frames: usize,
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
        Self {
            name: wt.name.clone(),
            slices,
            num_frames: nf,
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
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f32())
        .unwrap_or(0.0);

    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();

    // Recessed CRT well with a cut top-right corner.
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

    let base_y_front = rect.max.y - 8.0;
    let base_y_back = rect.min.y + 13.0;
    let margin_x = 8.0;
    let depth_x = 17.0;
    let project = |phase: f32, amp: f32, z: f32| -> Pos2 {
        let persp = 0.80 + 0.20 * z;
        let baseline = base_y_front - (1.0 - z) * (base_y_front - base_y_back);
        let start_x = rect.min.x + margin_x + (1.0 - z) * depth_x;
        let avail_w = (rect.width() - margin_x * 2.0 - depth_x) * persp;
        let x = start_x + phase * avail_w;
        let amp_scale = rect.height() * 0.27 * persp;
        pos2(x, baseline - amp * amp_scale)
    };

    let scan_z = (t * 0.8).sin().abs() * 0.85 + 0.05;
    let mut slices: Vec<Vec<Pos2>> = Vec::with_capacity(THUMB_SLICES);

    for s in 0..THUMB_SLICES {
        let z = s as f32 / (THUMB_SLICES - 1) as f32;
        let dist = (z - scan_z).abs();
        let lit = (1.0 - dist / 0.25).max(0.0);
        let alpha = (18.0 + z * 70.0 + lit * 110.0).min(230.0) as u8;
        let tint = 0.35 + z * 0.65;
        let col = Color32::from_rgba_unmultiplied(
            (color.r() as f32 * tint).min(255.0) as u8,
            (color.g() as f32 * tint).min(255.0) as u8,
            (color.b() as f32 * tint).min(255.0) as u8,
            alpha,
        );
        let pts: Vec<Pos2> = (0..THUMB_POINTS)
            .map(|i| {
                let phase = i as f32 / (THUMB_POINTS - 1) as f32;
                let ripple = 1.0 + 0.05 * (phase * 15.0 - t * 2.8 + z * 6.0).sin();
                project(phase, wt.sample_slice_public(s, phase) * ripple, z)
            })
            .collect();
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[i], pts[i + 1]],
                Stroke::new(if lit > 0.2 { 1.4_f32 } else { 0.9_f32 }, col),
            );
        }
        slices.push(pts);
    }

    // Longitudinal ribbons give the stack a holographic volume.
    for &idx in &[THUMB_POINTS / 4, THUMB_POINTS / 2, 3 * THUMB_POINTS / 4] {
        for s in 0..slices.len().saturating_sub(1) {
            painter.line_segment(
                [slices[s][idx], slices[s + 1][idx]],
                Stroke::new(0.7_f32, CyberTheme::alpha(color, 34)),
            );
        }
    }

    // Active morphed wave.
    let z_active = 1.0 - position.clamp(0.0, 1.0);
    let active: Vec<Pos2> = (0..THUMB_POINTS)
        .map(|i| {
            let phase = i as f32 / (THUMB_POINTS - 1) as f32;
            let pulse = 1.0 + 0.035 * (phase * 17.0 - t * 3.6).sin();
            project(phase, wt.sample(position, phase) * pulse, z_active)
        })
        .collect();
    let baseline: Vec<Pos2> = (0..THUMB_POINTS)
        .map(|i| project(i as f32 / (THUMB_POINTS - 1) as f32, 0.0, z_active))
        .collect();

    // Filled translucent ribbon under the active trace.
    let mut mesh = Mesh::default();
    let top = CyberTheme::alpha(color, 100);
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
