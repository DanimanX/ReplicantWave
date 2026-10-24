//! Cyberpunk Neon Theme and Hardware Styling for ReplicantWave.

use nih_plug_egui::egui::{
    epaint::FontId, pos2, Color32, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2,
};

pub struct CyberTheme;

impl CyberTheme {
    // Primary Cyberpunk Chassis & Panel Colors
    pub const BG_VOID: Color32 = Color32::from_rgb(8, 10, 15);
    pub const BG_PANEL: Color32 = Color32::from_rgb(14, 17, 26);
    pub const BG_PANEL_ALT: Color32 = Color32::from_rgb(20, 24, 36);
    pub const BG_RECESSED: Color32 = Color32::from_rgb(7, 9, 14);

    pub const BORDER_DARK: Color32 = Color32::from_rgb(32, 38, 56);
    pub const BORDER_GLOW: Color32 = Color32::from_rgb(0, 240, 255);

    // Neon Accents
    pub const NEON_CYAN: Color32 = Color32::from_rgb(0, 240, 255);
    pub const NEON_PINK: Color32 = Color32::from_rgb(255, 0, 128);
    pub const NEON_LIME: Color32 = Color32::from_rgb(57, 255, 20);
    pub const NEON_AMBER: Color32 = Color32::from_rgb(255, 204, 0);
    pub const NEON_PURPLE: Color32 = Color32::from_rgb(170, 85, 250);

    // Text & Indicators
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(238, 244, 255);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(125, 138, 168);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(68, 76, 98);

    pub const LED_OFF: Color32 = Color32::from_rgb(26, 30, 44);
    pub const LED_ACTIVE: Color32 = Color32::from_rgb(57, 255, 20);
    pub const LED_ACCENT: Color32 = Color32::from_rgb(255, 0, 128);

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

    /// Draws realistic Eurorack hex/cross mounting screws at the 4 corners of a panel.
    pub fn draw_eurorack_screws(painter: &Painter, rect: Rect) {
        let margin = 7.0;
        let corners = [
            pos2(rect.min.x + margin, rect.min.y + margin),
            pos2(rect.max.x - margin, rect.min.y + margin),
            pos2(rect.min.x + margin, rect.max.y - margin),
            pos2(rect.max.x - margin, rect.max.y - margin),
        ];

        for &c in &corners {
            // 1. Dark countersunk bevel well
            painter.circle_filled(c, 4.5, Color32::from_rgb(12, 14, 20));
            // 2. Metallic washer ring
            painter.circle_stroke(c, 4.2, Stroke::new(1.0_f32, Color32::from_rgb(55, 64, 84)));
            // 3. Steel screw face
            painter.circle_filled(c, 3.2, Color32::from_rgb(34, 40, 54));
            // 4. Highlight sheen
            painter.circle_stroke(c, 3.0, Stroke::new(0.6_f32, Color32::from_rgb(85, 98, 126)));
            // 5. Cross screwdriver slot
            let slot_len = 2.0;
            let slot_col = Color32::from_rgb(8, 10, 15);
            painter.line_segment([pos2(c.x - slot_len, c.y), pos2(c.x + slot_len, c.y)], Stroke::new(1.0_f32, slot_col));
            painter.line_segment([pos2(c.x, c.y - slot_len), pos2(c.x, c.y + slot_len)], Stroke::new(1.0_f32, slot_col));
        }
    }

    /// Draws a realistic circular metallic patch socket.
    pub fn draw_patch_socket(painter: &Painter, pos: Pos2, color: Color32) {
        // Outer knurled hex nut
        painter.circle_filled(pos, 6.0, Color32::from_rgb(22, 26, 36));
        painter.circle_stroke(pos, 5.8, Stroke::new(1.2_f32, Color32::from_rgb(58, 68, 90)));
        // Colored washer indicator
        painter.circle_stroke(pos, 4.5, Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 180)));
        // Dark center jack hole
        painter.circle_filled(pos, 3.0, Color32::from_rgb(6, 8, 12));
        painter.circle_stroke(pos, 3.0, Stroke::new(0.8_f32, Color32::from_rgb(16, 20, 28)));
    }

    /// Draws a realistic 3D Eurorack modular patch cable with physical gravity catenary curve and drop shadow.
    pub fn draw_patch_cable(painter: &Painter, start: Pos2, end: Pos2, color: Color32) {
        let num_points = 28;
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Gravity sag factor proportional to span
        let sag = (dist * 0.28).clamp(30.0, 75.0);

        let mut pts: Vec<Pos2> = Vec::with_capacity(num_points);
        for i in 0..num_points {
            let t = i as f32 / (num_points - 1) as f32;
            let x = start.x + t * dx;
            let linear_y = start.y + t * dy;
            let sag_y = 4.0 * t * (1.0 - t) * sag;
            pts.push(pos2(x, linear_y + sag_y));
        }

        // Pass 1: Soft Ambient Drop Shadow onto Faceplate
        let shadow_offset = Vec2::new(3.0, 5.0);
        let shadow_col = Color32::from_rgba_unmultiplied(0, 0, 0, 85);
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[i] + shadow_offset, pts[i + 1] + shadow_offset],
                Stroke::new(5.0_f32, shadow_col),
            );
        }

        // Pass 2: Thick Dark Braided Rubber Outer Core
        let cable_body_col = Color32::from_rgb(18, 22, 30);
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment([pts[i], pts[i + 1]], Stroke::new(4.2_f32, cable_body_col));
        }

        // Pass 3: Neon Color Glow/Tint along the Cord
        let cable_tint = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 125);
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment([pts[i], pts[i + 1]], Stroke::new(2.4_f32, cable_tint));
        }

        // Pass 4: Top Specular Sheen (Subtle light reflection)
        let sheen_col = Color32::from_rgba_unmultiplied(220, 235, 255, 95);
        let sheen_offset = Vec2::new(0.0, -0.8);
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[i] + sheen_offset, pts[i + 1] + sheen_offset],
                Stroke::new(0.9_f32, sheen_col),
            );
        }

        // Strain Relief & Jack Barrels at the Ends
        for &(pt, is_start) in &[(start, true), (end, false)] {
            let dir = if is_start {
                Vec2::new(1.0, 0.4).normalized()
            } else {
                Vec2::new(-0.8, -0.6).normalized()
            };
            let barrel_p1 = pt;
            let barrel_p2 = pt + dir * 9.0;
            // Metal barrel
            painter.line_segment([barrel_p1, barrel_p2], Stroke::new(5.0_f32, Color32::from_rgb(130, 142, 164)));
            // Rubber strain sleeve
            let sleeve_p2 = barrel_p2 + dir * 6.0;
            painter.line_segment([barrel_p2, sleeve_p2], Stroke::new(3.8_f32, Color32::from_rgb(28, 32, 44)));
        }
    }

    /// Background industrial textures: cooling vents, red stencil stamps, and HUD telemetry codes.
    pub fn draw_chassis_decorations(painter: &Painter, rect: Rect) {
        // 1. Top-Left Cooling Vents
        let vent_x = rect.min.x + 16.0;
        let vent_y = rect.min.y + 3.0;
        for i in 0..6 {
            let vx = vent_x + i as f32 * 5.0;
            painter.rect_filled(
                Rect::from_min_size(pos2(vx, vent_y), Vec2::new(2.5, 11.0)),
                1.0,
                Color32::from_rgb(10, 13, 18),
            );
            painter.line_segment(
                [pos2(vx + 0.5, vent_y), pos2(vx + 0.5, vent_y + 11.0)],
                Stroke::new(1.0_f32, Color32::from_rgb(22, 28, 40)),
            );
        }

        // 2. Red Stencil Stamp "ADVERTISED" below the logo
        let stamp_pos = pos2(rect.min.x + 284.0, rect.min.y + 28.0);
        let stamp_rect = Rect::from_min_size(stamp_pos, Vec2::new(74.0, 13.0));
        painter.rect_stroke(
            stamp_rect,
            1.0,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(210, 45, 65, 180)),
            StrokeKind::Outside,
        );
        painter.text(
            stamp_rect.center(),
            nih_plug_egui::egui::Align2::CENTER_CENTER,
            "ADVERTISED",
            FontId::monospace(8.0),
            Color32::from_rgba_unmultiplied(230, 60, 80, 220),
        );

        // 3. Right-Edge PCB Edge Connector & Microchip (CORE 808)
        let chip_x = rect.max.x - 24.0;
        let chip_y = rect.min.y + 175.0;
        let pcb_rect = Rect::from_min_max(pos2(chip_x - 8.0, rect.min.y + 70.0), pos2(rect.max.x, rect.max.y - 120.0));
        painter.rect_filled(pcb_rect, 0.0, Color32::from_rgb(7, 16, 14));
        painter.line_segment(
            [pcb_rect.left_top(), pcb_rect.left_bottom()],
            Stroke::new(1.0_f32, Color32::from_rgb(14, 38, 30)),
        );

        // Copper circuit bus lines
        for l in 0..4 {
            let lx = chip_x - 4.0 + l as f32 * 5.0;
            painter.line_segment(
                [pos2(lx, pcb_rect.min.y + 10.0), pos2(lx, pcb_rect.max.y - 10.0)],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 180, 120, 50)),
            );
        }

        // Microchip Body
        let ic_rect = Rect::from_min_size(pos2(chip_x, chip_y), Vec2::new(22.0, 48.0));
        painter.rect_filled(ic_rect, 2.0, Color32::from_rgb(16, 20, 26));
        painter.rect_stroke(
            ic_rect,
            2.0,
            Stroke::new(1.0_f32, Color32::from_rgb(42, 50, 65)),
            StrokeKind::Inside,
        );

        // Silver IC Pins
        for p in 0..8 {
            let py = ic_rect.min.y + 4.5 + p as f32 * 5.5;
            painter.line_segment(
                [pos2(ic_rect.min.x - 4.0, py), pos2(ic_rect.min.x, py)],
                Stroke::new(1.4_f32, Color32::from_rgb(150, 165, 190)),
            );
        }

        // IC Chip Label
        painter.text(
            ic_rect.center(),
            nih_plug_egui::egui::Align2::CENTER_CENTER,
            "CORE\n808",
            FontId::monospace(6.8),
            Color32::from_rgb(85, 100, 125),
        );

        // 4. Background Telemetry Code Strings
        let code_col = Color32::from_rgb(34, 42, 58);
        painter.text(pos2(rect.min.x + 580.0, rect.min.y + 32.0), nih_plug_egui::egui::Align2::LEFT_TOP, "010110010 0000100100", FontId::monospace(8.0), code_col);
        painter.text(pos2(rect.max.x - 230.0, rect.min.y + 32.0), nih_plug_egui::egui::Align2::LEFT_TOP, "01101100 1111111100", FontId::monospace(8.0), code_col);
        painter.text(pos2(rect.min.x + 380.0, rect.min.y + 440.0), nih_plug_egui::egui::Align2::LEFT_TOP, "00000000800000056", FontId::monospace(8.0), Color32::from_rgb(30, 38, 50));
        painter.text(pos2(rect.max.x - 330.0, rect.max.y - 365.0), nih_plug_egui::egui::Align2::LEFT_TOP, "DC001 0001111", FontId::monospace(8.0), Color32::from_rgb(32, 40, 54));
    }
}
