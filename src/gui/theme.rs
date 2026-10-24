//! Cyberpunk Neon Theme, Hardware Styling, and Modular Widgets for ReplicantWave.

use nih_plug_egui::egui::{pos2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

pub struct CyberTheme;

impl CyberTheme {
    // Primary Cyberpunk colors
    pub const BG_VOID: Color32 = Color32::from_rgb(8, 9, 14);
    pub const BG_PANEL: Color32 = Color32::from_rgb(14, 17, 26);
    pub const BG_PANEL_ALT: Color32 = Color32::from_rgb(20, 24, 38);
    pub const BG_FILTER: Color32 = Color32::from_rgb(13, 27, 23);
    pub const BG_RECESSED: Color32 = Color32::from_rgb(6, 7, 11);

    pub const BORDER_DARK: Color32 = Color32::from_rgb(34, 40, 62);
    pub const BORDER_FILTER: Color32 = Color32::from_rgb(28, 58, 48);
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
        // Outer hexagonal chassis nut
        let hex_r = 7.0;
        let mut hex_pts = Vec::with_capacity(6);
        for i in 0..6 {
            let angle = i as f32 * std::f32::consts::PI / 3.0;
            hex_pts.push(pos2(pos.x + hex_r * angle.cos(), pos.y + hex_r * angle.sin()));
        }
        for i in 0..6 {
            painter.line_segment(
                [hex_pts[i], hex_pts[(i + 1) % 6]],
                Stroke::new(1.2_f32, Color32::from_rgb(52, 60, 82)),
            );
        }
        painter.circle_filled(pos, 5.8, Color32::from_rgb(20, 24, 34));

        // Metallic chrome collar ring with accent tint
        painter.circle_filled(pos, 4.4, Color32::from_rgb(58, 66, 90));
        painter.circle_stroke(
            pos,
            4.4,
            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 190)),
        );

        // Center jack socket hole (deep void)
        painter.circle_filled(pos, 2.5, Color32::from_rgb(4, 5, 8));
    }

    /// Draw realistic draping analog patch cables between modular jacks.
    pub fn draw_patch_cable(painter: &Painter, start: Pos2, end: Pos2, color: Color32) {
        let num_steps = 28;
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let sag = (dist * 0.48).clamp(25.0, 90.0);

        let mut pts = Vec::with_capacity(num_steps + 1);
        for i in 0..=num_steps {
            let t = i as f32 / num_steps as f32;
            let x = start.x + t * dx;
            // Parabolic catenary sag under physical gravity
            let y_sag = 4.0 * sag * t * (1.0 - t);
            let y = start.y + t * dy + y_sag;
            pts.push(pos2(x, y));
        }

        // Pass 1: Realistic Drop Shadow (Wide & Soft)
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[i] + Vec2::new(2.5, 5.0), pts[i + 1] + Vec2::new(2.5, 5.0)],
                Stroke::new(8.0_f32, Color32::from_rgba_unmultiplied(1, 2, 3, 110)),
            );
        }

        // Pass 2: Heavy Rubber Cable Body (Thick, Dark Textured Jacket)
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[i], pts[i + 1]],
                Stroke::new(5.2_f32, Color32::from_rgb(14, 16, 22)),
            );
        }

        // Pass 3: 3D Cylindrical Volume Highlight (Specular Reflection)
        for i in 0..pts.len().saturating_sub(1) {
            painter.line_segment(
                [pts[i] - Vec2::new(0.0, 1.2), pts[i + 1] - Vec2::new(0.0, 1.2)],
                Stroke::new(1.6_f32, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 110)),
            );
        }

        // Shiny chrome barrel plugs & strain-relief boots at both endpoints
        for &(pt, is_start) in &[(start, true), (end, false)] {
            // Rubber strain relief boot
            painter.circle_filled(pt, 5.2, Color32::from_rgb(22, 26, 36));
            // Metal barrel cylinder
            let dir_offset = if is_start { Vec2::new(3.0, 1.0) } else { Vec2::new(-3.0, 1.0) };
            painter.circle_filled(pt + dir_offset, 3.8, Color32::from_rgb(110, 125, 155));
            painter.circle_stroke(pt + dir_offset, 3.8, Stroke::new(1.0_f32, Color32::WHITE));
        }
    }

    /// Monospace cybernetic circuit serial stamps.
    pub fn draw_tech_stamp(painter: &Painter, pos: Pos2, text: &str, color: Color32) {
        painter.text(
            pos,
            nih_plug_egui::egui::Align2::LEFT_TOP,
            text,
            FontId::monospace(8.0),
            color,
        );
    }

    /// Draw authentic cyberpunk chassis background textures (heatsinks, PCB chip, stamps).
    pub fn draw_chassis_decorations(painter: &Painter, rect: Rect) {
        // 1. Top-Left Heatsink Vents
        let vent_x = rect.min.x + 14.0;
        let vent_y = rect.min.y + 4.0;
        for i in 0..6 {
            let vx = vent_x + i as f32 * 5.0;
            painter.rect_filled(
                Rect::from_min_size(pos2(vx, vent_y), Vec2::new(2.5, 12.0)),
                1.0,
                Color32::from_rgb(12, 14, 20),
            );
            painter.line_segment(
                [pos2(vx + 0.5, vent_y), pos2(vx + 0.5, vent_y + 12.0)],
                Stroke::new(1.0_f32, Color32::from_rgb(24, 28, 40)),
            );
        }

        // 2. Red Stamp "ADVERT ISED" under the title
        let stamp_pos = pos2(rect.min.x + 280.0, rect.min.y + 28.0);
        let stamp_rect = Rect::from_min_size(stamp_pos, Vec2::new(76.0, 14.0));
        painter.rect_stroke(
            stamp_rect,
            1.0,
            Stroke::new(1.2_f32, Color32::from_rgba_unmultiplied(220, 40, 60, 200)),
            nih_plug_egui::egui::StrokeKind::Outside,
        );
        painter.text(
            stamp_rect.center(),
            nih_plug_egui::egui::Align2::CENTER_CENTER,
            "ADVERT ISED",
            FontId::monospace(8.0),
            Color32::from_rgba_unmultiplied(240, 60, 80, 230),
        );

        // 3. Right-Edge PCB Edge Connector & Microchip
        let chip_x = rect.max.x - 28.0;
        let chip_y = rect.min.y + 180.0;
        // Dark green PCB substrate track
        let pcb_rect = Rect::from_min_max(pos2(chip_x - 8.0, rect.min.y + 80.0), pos2(rect.max.x, rect.max.y - 120.0));
        painter.rect_filled(pcb_rect, 0.0, Color32::from_rgb(8, 18, 15));
        painter.line_segment(
            [pcb_rect.left_top(), pcb_rect.left_bottom()],
            Stroke::new(1.0_f32, Color32::from_rgb(16, 42, 34)),
        );

        // Copper circuit bus lines
        for l in 0..4 {
            let lx = chip_x - 4.0 + l as f32 * 5.0;
            painter.line_segment(
                [pos2(lx, pcb_rect.min.y + 10.0), pos2(lx, pcb_rect.max.y - 10.0)],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 180, 120, 60)),
            );
        }

        // Microchip Body
        let ic_rect = Rect::from_min_size(pos2(chip_x, chip_y), Vec2::new(24.0, 50.0));
        painter.rect_filled(ic_rect, 2.0, Color32::from_rgb(18, 22, 28));
        painter.rect_stroke(
            ic_rect,
            2.0,
            Stroke::new(1.0_f32, Color32::from_rgb(45, 52, 68)),
            nih_plug_egui::egui::StrokeKind::Inside,
        );

        // Silver IC Pins
        for p in 0..8 {
            let py = ic_rect.min.y + 5.0 + p as f32 * 5.5;
            painter.line_segment(
                [pos2(ic_rect.min.x - 4.0, py), pos2(ic_rect.min.x, py)],
                Stroke::new(1.5_f32, Color32::from_rgb(160, 175, 200)),
            );
        }

        // IC Chip Label
        painter.text(
            ic_rect.center(),
            nih_plug_egui::egui::Align2::CENTER_CENTER,
            "CORE\n808",
            FontId::monospace(7.0),
            Color32::from_rgb(90, 105, 130),
        );

        // 4. Industrial Telemetry Codes in Background Spaces
        Self::draw_tech_stamp(painter, pos2(rect.min.x + 580.0, rect.min.y + 32.0), "010110010 0000100100", Color32::from_rgb(38, 46, 64));
        Self::draw_tech_stamp(painter, pos2(rect.max.x - 220.0, rect.min.y + 32.0), "01101100 1111111100", Color32::from_rgb(38, 46, 64));
        Self::draw_tech_stamp(painter, pos2(rect.min.x + 380.0, rect.min.y + 440.0), "000000000800000056", Color32::from_rgb(32, 42, 54));
        Self::draw_tech_stamp(painter, pos2(rect.max.x - 340.0, rect.max.y - 380.0), "DC001 0001111", Color32::from_rgb(36, 44, 58));
    }
}
