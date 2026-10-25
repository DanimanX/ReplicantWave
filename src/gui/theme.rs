//! NIGHT CITY HUD — cyberpunk neon design system for ReplicantWave.
//!
//! This is a deliberate visual break from the old brushed-metal Eurorack look:
//! violet-black glass, hard cut-corner panels, neon outlines, scanlines and a
//! perspective city grid. All drawing here is pure egui geometry, so it stays
//! lightweight and resolution independent.

use nih_plug_egui::egui::{
    epaint::FontId, pos2, Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2,
};

pub struct CyberTheme;

impl CyberTheme {
    // Deep night-city background
    pub const BG_VOID: Color32 = Color32::from_rgb(4, 2, 10);
    pub const BG_DEEP: Color32 = Color32::from_rgb(6, 3, 14);
    pub const BG_PANEL: Color32 = Color32::from_rgb(13, 8, 27);
    pub const BG_PANEL_ALT: Color32 = Color32::from_rgb(18, 11, 38);
    pub const BG_RECESSED: Color32 = Color32::from_rgb(5, 3, 12);
    pub const GLASS: Color32 = Color32::from_rgb(12, 7, 26);

    pub const BORDER_DARK: Color32 = Color32::from_rgb(43, 29, 74);
    pub const BORDER_GLOW: Color32 = Color32::from_rgb(0, 245, 255);

    // Neon accents
    pub const NEON_CYAN: Color32 = Color32::from_rgb(0, 245, 255);
    pub const NEON_PINK: Color32 = Color32::from_rgb(255, 20, 147);
    pub const NEON_MAGENTA: Color32 = Color32::from_rgb(255, 0, 180);
    pub const NEON_LIME: Color32 = Color32::from_rgb(180, 255, 60);
    pub const NEON_AMBER: Color32 = Color32::from_rgb(255, 176, 46);
    pub const NEON_PURPLE: Color32 = Color32::from_rgb(155, 90, 255);
    pub const NEON_RED: Color32 = Color32::from_rgb(255, 45, 95);
    pub const NEON_BLUE: Color32 = Color32::from_rgb(70, 120, 255);

    // Text
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(232, 238, 255);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(143, 132, 180);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(82, 70, 118);

    pub const LED_OFF: Color32 = Color32::from_rgb(30, 20, 48);
    pub const LED_ACTIVE: Color32 = Color32::from_rgb(180, 255, 60);
    pub const LED_ACCENT: Color32 = Color32::from_rgb(255, 20, 147);

    pub fn alpha(color: Color32, a: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
    }

    pub fn stroke_border() -> Stroke {
        Stroke::new(1.0_f32, Self::BORDER_DARK)
    }

    pub fn stroke_cyan() -> Stroke {
        Stroke::new(1.2_f32, Self::NEON_CYAN)
    }

    /// Full-window backdrop: vertical neon gradient, synthwave floor grid,
    /// drifting data glyphs and a soft magenta horizon bloom.
    pub fn draw_backdrop(painter: &Painter, rect: Rect, t: f32) {
        // Vertical gradient in coarse bands (cheap, no shader required).
        let bands = 42;
        for i in 0..bands {
            let f = i as f32 / (bands - 1) as f32;
            let col = lerp_col(Color32::from_rgb(13, 5, 30), Color32::from_rgb(3, 2, 8), f);
            let y0 = rect.min.y + rect.height() * (i as f32 / bands as f32);
            let y1 = rect.min.y + rect.height() * ((i + 1) as f32 / bands as f32);
            painter.rect_filled(
                Rect::from_min_max(pos2(rect.min.x, y0), pos2(rect.max.x, y1 + 1.0)),
                0.0,
                col,
            );
        }

        // Magenta horizon bloom behind the upper HUD.
        let sun_c = pos2(rect.center().x, rect.min.y + 8.0);
        for i in (1..=14).rev() {
            let r = 30.0 + i as f32 * 16.0;
            painter.circle_filled(sun_c, r, Self::alpha(Self::NEON_MAGENTA, 3));
        }

        // Synthwave perspective floor. Vanishing point sits under the scope.
        let vp = pos2(rect.center().x, rect.min.y + rect.height() * 0.56);
        let grid_col = Self::alpha(Self::NEON_MAGENTA, 26);
        let bottom = rect.max.y;
        for i in 0..=26 {
            let f = i as f32 / 26.0;
            let x_bottom = rect.min.x - rect.width() * 0.4 + f * rect.width() * 1.8;
            painter.line_segment([vp, pos2(x_bottom, bottom)], Stroke::new(1.0_f32, grid_col));
        }
        for i in 0..14 {
            let f = i as f32 / 13.0;
            let y = vp.y + (bottom - vp.y) * f * f;
            painter.line_segment(
                [pos2(rect.min.x, y), pos2(rect.max.x, y)],
                Stroke::new(1.0_f32, Self::alpha(Self::NEON_MAGENTA, 18)),
            );
        }

        // Drifting glyph columns (deterministic pseudo random, no RNG state).
        let glyphs = [
            "01", "10", "//", "電", "夢", "NC", "2077", "///", "ERROR", "SIG",
        ];
        for i in 0..18u32 {
            let h = (i.wrapping_mul(2_654_435_761u32) & 0xffff) as f32 / 65535.0;
            let v = (i.wrapping_mul(40_503u32).wrapping_add(7) & 0xffff) as f32 / 65535.0;
            let x = rect.min.x + 18.0 + h * (rect.width() - 36.0);
            let drift = (t * (6.0 + v * 10.0) + h * 300.0) % (rect.height() + 60.0);
            let y = rect.min.y + drift - 30.0;
            let glyph = glyphs[(i as usize) % glyphs.len()];
            let col = Self::alpha(
                if i % 3 == 0 {
                    Self::NEON_CYAN
                } else {
                    Self::NEON_PURPLE
                },
                22,
            );
            painter.text(
                pos2(x, y),
                nih_plug_egui::egui::Align2::LEFT_TOP,
                glyph,
                FontId::monospace(8.0),
                col,
            );
        }

        // Thin animated neon rails on both edges.
        let pulse = 0.5 + 0.5 * (t * 1.7).sin();
        let rail = Self::alpha(Self::NEON_CYAN, (40.0 + pulse * 70.0) as u8);
        painter.line_segment(
            [
                pos2(rect.min.x + 4.0, rect.min.y + 40.0),
                pos2(rect.min.x + 4.0, rect.max.y - 40.0),
            ],
            Stroke::new(1.2_f32, rail),
        );
        painter.line_segment(
            [
                pos2(rect.max.x - 4.0, rect.min.y + 40.0),
                pos2(rect.max.x - 4.0, rect.max.y - 40.0),
            ],
            Stroke::new(
                1.2_f32,
                Self::alpha(Self::NEON_MAGENTA, (40.0 + pulse * 70.0) as u8),
            ),
        );
    }

    /// A hard cut-corner glass panel. The background is painted immediately so
    /// child widgets land on top; chrome is painted afterwards by `card_chrome`.
    pub fn card_shell(painter: &Painter, rect: Rect, accent: Color32) {
        let cut = 10.0;
        let pts = cut_corner_points(rect, cut);

        // Ambient neon halo (thick, low alpha outline).
        painter.add(Shape::convex_polygon(
            pts.clone(),
            Color32::TRANSPARENT,
            Stroke::new(4.0_f32, Self::alpha(accent, 18)),
        ));
        // Glass body.
        painter.add(Shape::convex_polygon(
            pts,
            Self::GLASS,
            Stroke::new(1.0_f32, Self::alpha(accent, 95)),
        ));

        // Inner top highlight line.
        painter.line_segment(
            [
                pos2(rect.min.x + cut + 2.0, rect.min.y + 1.0),
                pos2(rect.max.x - 6.0, rect.min.y + 1.0),
            ],
            Stroke::new(1.0_f32, Self::alpha(accent, 55)),
        );
        // Accent wedge in the top-left cut corner.
        painter.add(Shape::convex_polygon(
            vec![
                pos2(rect.min.x, rect.min.y + cut),
                pos2(rect.min.x + cut, rect.min.y),
                pos2(rect.min.x + cut + 18.0, rect.min.y),
                pos2(rect.min.x, rect.min.y + cut + 18.0),
            ],
            Self::alpha(accent, 70),
            Stroke::NONE,
        ));
    }

    pub fn card_chrome(painter: &Painter, rect: Rect, accent: Color32, t: f32) {
        // Corner brackets.
        let len = 9.0;
        let s = Stroke::new(1.6_f32, accent);
        let c = rect.shrink(3.0);
        // bottom-right bracket
        painter.line_segment([pos2(c.max.x - len, c.max.y), pos2(c.max.x, c.max.y)], s);
        painter.line_segment([pos2(c.max.x, c.max.y - len), pos2(c.max.x, c.max.y)], s);
        // bottom-left
        painter.line_segment([pos2(c.min.x, c.max.y), pos2(c.min.x + len, c.max.y)], s);
        painter.line_segment([pos2(c.min.x, c.max.y - len), pos2(c.min.x, c.max.y)], s);

        // Animated telemetry ticks along the right edge.
        let pulse = (t * 2.0).sin() * 0.5 + 0.5;
        for i in 0..5 {
            let y = rect.max.y - 18.0 - i as f32 * 9.0;
            let w = 3.0 + pulse * 5.0 * ((i + 1) as f32 / 5.0);
            painter.line_segment(
                [pos2(rect.max.x - 5.0 - w, y), pos2(rect.max.x - 5.0, y)],
                Stroke::new(1.0_f32, Self::alpha(accent, 130)),
            );
        }
    }

    /// Card title row: index tag, title, underline. Returns the content rect.
    pub fn card_header(
        painter: &Painter,
        rect: Rect,
        accent: Color32,
        index: &str,
        title: &str,
    ) -> Rect {
        let x = rect.min.x + 14.0;
        let y = rect.min.y + 9.0;
        painter.text(
            pos2(x, y),
            nih_plug_egui::egui::Align2::LEFT_TOP,
            index,
            FontId::monospace(8.0),
            Self::alpha(accent, 170),
        );
        painter.text(
            pos2(x + 26.0, y - 1.0),
            nih_plug_egui::egui::Align2::LEFT_TOP,
            title,
            FontId::monospace(11.0),
            accent,
        );
        let line_y = y + 17.0;
        painter.line_segment(
            [pos2(x, line_y), pos2(rect.max.x - 14.0, line_y)],
            Stroke::new(1.0_f32, Self::alpha(accent, 42)),
        );
        Rect::from_min_max(
            pos2(rect.min.x + 12.0, line_y + 7.0),
            pos2(rect.max.x - 12.0, rect.max.y - 8.0),
        )
    }

    /// Scanlines + vignette + window corner HUD, painted over the finished frame.
    pub fn draw_overlay(painter: &Painter, rect: Rect) {
        let mut y = rect.min.y;
        while y < rect.max.y {
            painter.line_segment(
                [pos2(rect.min.x, y), pos2(rect.max.x, y)],
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 16)),
            );
            y += 4.0;
        }

        // Vignette edges.
        let w = rect.width();
        let h = rect.height();
        for i in 0..8 {
            let a = (2 + i * 3) as u8;
            let inset = i as f32 * 2.0;
            painter.rect_stroke(
                rect.shrink(inset),
                0.0,
                Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, a)),
                StrokeKind::Inside,
            );
        }

        // Outer window HUD frame.
        let frame = Stroke::new(1.2_f32, Self::alpha(Self::NEON_CYAN, 120));
        let c = rect.shrink(3.0);
        let len = 18.0;
        painter.line_segment([c.left_top(), pos2(c.min.x + len, c.min.y)], frame);
        painter.line_segment([c.left_top(), pos2(c.min.x, c.min.y + len)], frame);
        painter.line_segment([pos2(c.max.x - len, c.max.y), c.right_bottom()], frame);
        painter.line_segment([pos2(c.max.x, c.max.y - len), c.right_bottom()], frame);

        painter.text(
            pos2(rect.min.x + 10.0, rect.max.y - 13.0),
            nih_plug_egui::egui::Align2::LEFT_TOP,
            "NIGHT CITY // AUDIO OPERATING SYSTEM // BUILD 2077.11",
            FontId::monospace(7.0),
            Self::alpha(Self::NEON_CYAN, 90),
        );
        painter.text(
            pos2(rect.max.x - 10.0, rect.max.y - 13.0),
            nih_plug_egui::egui::Align2::RIGHT_TOP,
            format!("{:08X}", (w as u32) << 16 | (h as u32)),
            FontId::monospace(7.0),
            Self::alpha(Self::NEON_MAGENTA, 90),
        );
    }

    /// Neon LED horizontal/vertical meter.
    pub fn led_meter(
        painter: &Painter,
        rect: Rect,
        value: f32,
        accent: Color32,
        segments: usize,
        vertical: bool,
    ) {
        let n = segments.max(1);
        for i in 0..n {
            let f = i as f32 / n as f32;
            let active = f < value;
            let col = if active {
                if f > 0.82 {
                    Self::NEON_RED
                } else if f > 0.64 {
                    Self::NEON_AMBER
                } else {
                    accent
                }
            } else {
                Self::LED_OFF
            };
            let (r, pad) = if vertical {
                let sh = rect.height() / n as f32;
                (
                    Rect::from_min_size(
                        pos2(rect.min.x, rect.max.y - (i + 1) as f32 * sh),
                        Vec2::new(rect.width(), (sh - 2.0).max(1.0)),
                    ),
                    0.0,
                )
            } else {
                let sw = rect.width() / n as f32;
                (
                    Rect::from_min_size(
                        pos2(rect.min.x + i as f32 * sw, rect.min.y),
                        Vec2::new((sw - 2.0).max(1.0), rect.height()),
                    ),
                    0.0,
                )
            };
            let _ = pad;
            painter.rect_filled(r, 1.0, col);
        }
    }

    /// Small neon chip/toggle used for boolean parameters.
    pub fn chip(painter: &Painter, rect: Rect, on: bool, color: Color32, label: &str) {
        let fill = if on {
            Self::alpha(color, 55)
        } else {
            Color32::from_rgb(10, 6, 22)
        };
        let stroke = Stroke::new(1.0_f32, if on { color } else { Self::BORDER_DARK });
        let cut = 4.0;
        painter.add(Shape::convex_polygon(
            cut_corner_points(rect, cut),
            fill,
            stroke,
        ));
        painter.circle_filled(
            pos2(rect.min.x + 8.0, rect.center().y),
            2.4,
            if on { color } else { Self::LED_OFF },
        );
        painter.text(
            pos2(rect.min.x + 15.0, rect.center().y),
            nih_plug_egui::egui::Align2::LEFT_CENTER,
            label,
            FontId::monospace(8.0),
            if on {
                Self::TEXT_PRIMARY
            } else {
                Self::TEXT_MUTED
            },
        );
    }
}

fn cut_corner_points(rect: Rect, cut: f32) -> Vec<Pos2> {
    vec![
        pos2(rect.min.x + cut, rect.min.y),
        pos2(rect.max.x, rect.min.y),
        pos2(rect.max.x, rect.max.y - cut),
        pos2(rect.max.x - cut, rect.max.y),
        pos2(rect.min.x, rect.max.y),
        pos2(rect.min.x, rect.min.y + cut),
    ]
}

fn lerp_col(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}
