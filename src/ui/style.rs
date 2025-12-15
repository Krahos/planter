//! Centralized theming system inspired by Sniffnet
//!
//! This module defines a theme struct that provides all colors and styling
//! constants for the application. Modify the THEME constant to change colors globally.

use iced::Color;

// ─── Theme Struct ────────────────────────────────────────────────────────────

/// Application theme containing all colors
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Primary color - used for focused elements, in-progress items
    pub primary: Color,
    /// Success color - used for completed items
    pub success: Color,
    /// Error color - used for error states
    pub error: Color,
    /// Main text color
    pub text: Color,
    /// Muted/secondary text color
    pub text_muted: Color,
    /// Main background color
    pub background: Color,
    /// Alternate background color - for striped rows, panels
    pub background_alt: Color,
    /// Border/grid color
    pub border: Color,
}

impl Theme {
    /// Check if theme is dark (for future dark mode support)
    pub fn is_dark(&self) -> bool {
        self.background.r + self.background.g + self.background.b < 1.5
    }
}

// ─── Theme Presets ───────────────────────────────────────────────────────────

pub const GRUVBOX_DARK: Theme = Theme {
    primary: Color::from_rgb(0.51, 0.65, 0.56),
    success: Color::from_rgb(0.72, 0.73, 0.15),
    error: Color::from_rgb(0.98, 0.29, 0.24),
    text: Color::from_rgb(0.92, 0.86, 0.70),
    text_muted: Color::from_rgb(0.66, 0.61, 0.53),
    background: Color::from_rgb(0.16, 0.15, 0.13),
    background_alt: Color::from_rgb(0.20, 0.18, 0.16),
    border: Color::from_rgb(0.33, 0.31, 0.28),
};

pub const GRUVBOX_LIGHT: Theme = Theme {
    primary: Color::from_rgb(0.27, 0.52, 0.53),
    success: Color::from_rgb(0.60, 0.59, 0.10),
    error: Color::from_rgb(0.80, 0.14, 0.11),
    text: Color::from_rgb(0.16, 0.15, 0.13),
    text_muted: Color::from_rgb(0.57, 0.51, 0.45),
    background: Color::from_rgb(0.98, 0.94, 0.84),
    background_alt: Color::from_rgb(0.92, 0.86, 0.70),
    border: Color::from_rgb(0.85, 0.76, 0.56),
};

pub const CATPPUCCIN_MOCHA: Theme = Theme {
    primary: Color::from_rgb(0.53, 0.71, 0.98),
    success: Color::from_rgb(0.65, 0.89, 0.63),
    error: Color::from_rgb(0.95, 0.55, 0.65),
    text: Color::from_rgb(0.80, 0.82, 0.92),
    text_muted: Color::from_rgb(0.59, 0.61, 0.74),
    background: Color::from_rgb(0.11, 0.11, 0.16),
    background_alt: Color::from_rgb(0.14, 0.14, 0.19),
    border: Color::from_rgb(0.24, 0.24, 0.31),
};

pub const NORD: Theme = Theme {
    primary: Color::from_rgb(0.51, 0.63, 0.76),
    success: Color::from_rgb(0.64, 0.75, 0.54),
    error: Color::from_rgb(0.75, 0.38, 0.42),
    text: Color::from_rgb(0.92, 0.93, 0.95),
    text_muted: Color::from_rgb(0.60, 0.64, 0.71),
    background: Color::from_rgb(0.18, 0.20, 0.25),
    background_alt: Color::from_rgb(0.23, 0.26, 0.32),
    border: Color::from_rgb(0.30, 0.34, 0.42),
};

/// Default theme
pub const THEME: Theme = NORD;

// ─── Spacing & Layout ────────────────────────────────────────────────────────

/// Extra small spacing (2px) - between rows in tables
pub const SPACING_XS: f32 = 2.0;
/// Small spacing (4px) - between cells in a row
pub const SPACING_SM: f32 = 4.0;
/// Medium spacing (8px) - general spacing
pub const SPACING_MD: f32 = 8.0;
/// Large spacing (16px) - section spacing
pub const SPACING_LG: f32 = 16.0;

/// Small padding (8px) - inside widgets
pub const PADDING_SM: f32 = 8.0;
/// Medium padding (16px) - around content
pub const PADDING_MD: f32 = 16.0;

// ─── Borders ─────────────────────────────────────────────────────────────────

/// Small border radius (4px) - cells, inputs
pub const BORDER_RADIUS_SM: f32 = 4.0;
/// Medium border radius (6px) - gantt bars
pub const BORDER_RADIUS_MD: f32 = 6.0;
/// Large border radius (8px) - panes, containers
pub const BORDER_RADIUS_LG: f32 = 8.0;

/// Thin border (1px) - standard borders
pub const BORDER_WIDTH_THIN: f32 = 1.0;
/// Thick border (2px) - focused elements
pub const BORDER_WIDTH_THICK: f32 = 2.0;

// ─── Widget Dimensions ───────────────────────────────────────────────────────

/// Standard cell width (120px)
pub const CELL_WIDTH: u32 = 120;
/// Standard cell height (48px)
pub const CELL_HEIGHT: u32 = 48;
/// Standard row height (48px)
pub const ROW_HEIGHT: f32 = 48.0;

// ─── Active Theme Accessor ───────────────────────────────────────────────────

/// Get the active theme (can be changed to support runtime theme selection)
pub fn theme() -> Theme {
    THEME
}
