//! Cyberpunk Neon Theme, Hardware Styling, and Modular Widgets for ReplicantWave.

use nih_plug_egui::egui::{pos2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

pub struct CyberTheme;

impl CyberTheme {
    // Primary Cyberpunk colors
    pub const BG_VOID: Color32 = Color32::from_rgb(8, 9, 14);
    pub const BG_PANEL: Color32 = Color32::from_rgb(16, 18, 28);
    pub const BG_PANEL_ALT: Color32 = Color32::from_rgb(22, 26, 40);
    pub const BG_RECESSED: Color32 = Color32::from_rgb(6, 7, 11);

    pub const BORDER_DARK: Color32 = Color32::from_rgb(34, 40, 62);
    pub const BORDER_GLOW: Color32 = Color32::from_rgb(0, 240, 255);

    // Neon Accents
    pub const NEON_CYAN: Color32 = Color32::from_rgb(0, 240, 255);
    pub const NEON_PINK: Color32 = Color32::from_rgb(255, 0, 127);
    pub const NEON_LIME: Color32 = Color32::from_rgb(57, 255, 20);
    pub const NEON_AMBER: Color32 = Color32::from_rgb(255, 210, 0);
    pub const NEON_PURPLE: Color32 = Color32::from_rgb(168, 85, 247);

    // Text & Indicators
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(238, 244, 255);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(130, 144, 172);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(68, 76, 100);

    pub const LED_OFF: Color32 = Color32::from_rgb(26, 30, 44);
    pub const LED_ACTIVE: Color32 = Color32::from_rgb(57, 255, 20);
    pub const LED_ACCENT: Color32 = Color32::from_rgb(255, 0, 127);

    pub fn stroke_border() -> Stroke {
        Stroke::new(1.0_f32, Self::BORDER_DARK)
    }

    pub fn stroke_cyan() -> Stroke {
        Stroke::new(1.5_f32, Self::NEON_CYAN)
    }

    pub fn stroke_pink() -> Stroke {
        Stroke::new(1.5_f32, Self::NEON_PINK)
    }

    pub fn stroke_lime() -> Stroke {
        Stroke::new(1.5_f32, Self::NEON_LIME)
    }

    /// Draw metallic hex/slot screws in the corners of a modular rack module.
    pub fn draw_panel_screws(painter: &Painter, rect: Rect) {
        let pad = 6.0;
        let corners = [
            pos2(rect.min.x + pad, rect.min.y + pad),
            pos2(rect.max.x - pad, rect.min.y + pad),
            pos2(rect.min.x + pad, rect.max.y - pad),
            pos2(rect.max.x - pad, rect.max.y - pad),
        ];

        for &c in &corners {
            // Screw shadow
            painter.circle_filled(c + Vec2::new(0.5, 0.5), 2.8, Color32::from_rgb(4, 5, 8));
            // Screw metallic collar
            painter.circle_filled(c, 2.5, Color32::from_rgb(42, 48, 66));
            painter.circle_stroke(c, 2.5, Stroke::new(0.8_f32, Color32::from_rgb(72, 82, 112)));
            // Phillips / slot mark
            painter.line_segment(
                [c - Vec2::new(1.2, 0.0), c + Vec2::new(1.2, 0.0)],
                Stroke::new(0.8_f32, Color32::from_rgb(18, 20, 28)),
            );
        }
    }

    /// Draw a modular 1/4" analog audio patch socket (jack).
    pub fn draw_patch_socket(painter: &Painter, pos: Pos2, color: Color32) {
        // Outer hexagonal chassis bezel
        painter.circle_filled(pos, 6.0, Color32::from_rgb(22, 26, 38));
        painter.circle_stroke(pos, 6.0, Stroke::new(1.0_f32, Color32::from_rgb(55, 64, 88)));

        // Metallic collar ring with accent tint
        painter.circle_filled(pos, 4.2, Color32::from_rgb(48, 54, 74));
        painter.circle_stroke(
            pos,
            4.2,
            Stroke::new(0.8_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 180)),
        );

        // Center jack socket hole (deep void)
        painter.circle_filled(pos, 2.2, Color32::from_rgb(6, 7, 10));
    }

    /// Draw realistic draping analog patch cables between modular jacks.
    pub fn draw_patch_cable(painter: &Painter, start: Pos2, end: Pos2, color: Color32) {
        let num_steps = 24;
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let sag = (dist * 0.45).clamp(20.0, 75.0);

        let mut pts = Vec::with_capacity(num_steps + 1);
        for i in 0..=num_steps {
            let t = i as f32 / num_steps as f32;
            let x = start.x + t * dx;
            // Parabolic catenary sag under gravity
            let y_sag = 4.0 * sag * t * (1.0 - t);
            let y = start.y + t * dy + y_sag;
            pts.push(pos2(x, y));
        }

        // Pass 1: Drop Shadow
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[i] + Vec2::new(1.5, 3.5), pts[i + 1] + Vec2::new(1.5, 3.5)],
                Stroke::new(4.0_f32, Color32::from_rgba_unmultiplied(2, 3, 5, 95)),
            );
        }

        // Pass 2: Heavy Rubber Cable Body
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[i], pts[i + 1]],
                Stroke::new(3.2_f32, Color32::from_rgb(18, 20, 28)),
            );
        }

        // Pass 3: Subtle Cable Specular / Neon Trim Line
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[i], pts[i + 1]],
                Stroke::new(1.2_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 140)),
            );
        }

        // Plug strain relief boots
        painter.circle_filled(start, 4.0, Color32::from_rgb(28, 32, 44));
        painter.circle_filled(end, 4.0, Color32::from_rgb(28, 32, 44));
    }

    /// Monospace cybernetic circuit serial stamps.
    pub fn draw_tech_stamp(painter: &Painter, pos: Pos2, text: &str, color: Color32) {
        painter.text(
            pos,
            nih_plug_egui::egui::Align2::LEFT_TOP,
            text,
            FontId::monospace(7.5),
            color,
        );
    }
}
