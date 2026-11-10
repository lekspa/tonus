use super::color;
use ratatui::style::Color;
use serde::Deserialize;

const DEFAULT_THEME: Theme = Theme {
    background: color::COD_GRAY,
    foreground: color::SILVER,
    surface: color::MINE_SHAFT,
    selection_background: color::ATHS_SPECIAL,
    selection_foreground: color::COD_GRAY,
    accent: color::FIERY_ORANGE,
    on_accent: color::ATHS_SPECIAL,
    success: Color::LightGreen,
    failure: Color::LightRed,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    /// Background color
    pub background: Color,
    /// Text color
    pub foreground: Color,
    /// Table header and progress background
    pub surface: Color,
    /// Selected table row background
    pub selection_background: Color,
    /// Selected table row foreground
    pub selection_foreground: Color,
    /// Accent color
    pub accent: Color,
    /// Text color on accent backgrounds
    pub on_accent: Color,
    /// Success text color
    pub success: Color,
    /// Error text color
    pub failure: Color,
}

impl Default for Theme {
    fn default() -> Self {
        DEFAULT_THEME
    }
}
