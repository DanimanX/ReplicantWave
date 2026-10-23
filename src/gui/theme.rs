//! Cyberpunk Neon Theme and Styling for ReplicatWave.

use nih_plug_egui::egui::{Color32, Stroke};

pub struct CyberTheme;

impl CyberTheme {
    // Primary Cyberpunk colors
    pub const BG_VOID: Color32 = Color32::from_rgb(10, 11, 16);
    pub const BG_PANEL: Color32 = Color32::from_rgb(17, 19, 29);
    pub const BG_PANEL_ALT: Color32 = Color32::from_rgb(24, 27, 42);
    pub const BG_RECESSED: Color32 = Color32::from_rgb(8, 9, 13);

    pub const BORDER_DARK: Color32 = Color32::from_rgb(34, 38, 58);
    pub const BORDER_GLOW: Color32 = Color32::from_rgb(0, 240, 255);

    // Neon Accents
    pub const NEON_CYAN: Color32 = Color32::from_rgb(0, 240, 255);
    pub const NEON_PINK: Color32 = Color32::from_rgb(255, 0, 127);
    pub const NEON_LIME: Color32 = Color32::from_rgb(57, 255, 20);
    pub const NEON_AMBER: Color32 = Color32::from_rgb(255, 210, 0);
    pub const NEON_PURPLE: Color32 = Color32::from_rgb(168, 85, 247);

    // Text & Indicators
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(235, 242, 255);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(130, 142, 170);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(70, 78, 102);

    pub const LED_OFF: Color32 = Color32::from_rgb(30, 34, 48);
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
}
