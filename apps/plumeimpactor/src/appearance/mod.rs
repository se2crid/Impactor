use std::str::FromStr;

use iced::{Color, Theme, color};

mod button;
mod fonts;
mod picklist;

pub(crate) use button::{icon_button, p_button, s_button};
#[allow(unused)]
pub(crate) use fonts::{
    CHEVRON_BACK, DOWNLOAD, FILE, GEAR, MINUS, PLUS, SHARE, STAR, WRENCH, icon, icon_text,
    load_fonts,
};
pub(crate) use picklist::s_pick_list;

pub(crate) const THEME_CORNER_RADIUS: f32 = 6.0;
pub(crate) const THEME_FONT_SIZE: f32 = 13.0;
pub(crate) const THEME_PADDING: f32 = 10.0;
pub(crate) const THEME_ICON_SIZE: f32 = 13.0;

pub(crate) fn p_font() -> iced::Font {
    iced::Font {
        family: iced::font::Family::Monospace,
        weight: iced::font::Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AccentColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl AccentColor {
    pub(crate) const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub(crate) fn primary(self) -> Color {
        Color::from_rgb8(self.red, self.green, self.blue)
    }

    pub(crate) fn red(self) -> u8 {
        self.red
    }

    pub(crate) fn green(self) -> u8 {
        self.green
    }

    pub(crate) fn blue(self) -> u8 {
        self.blue
    }

    pub(crate) fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

impl Default for AccentColor {
    fn default() -> Self {
        Self::new(0xE8, 0x8A, 0xAB)
    }
}

impl std::fmt::Display for AccentColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for AccentColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.trim().trim_start_matches('#');

        if hex.len() != 6 {
            return Err(());
        }

        let red = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
        let green = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
        let blue = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;

        Ok(Self::new(red, green, blue))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlumeTheme {
    accent: AccentColor,
}

impl PlumeTheme {
    pub(crate) fn new(accent: AccentColor) -> Self {
        Self { accent }
    }

    pub(crate) fn to_iced_theme(self) -> Theme {
        self.plume_dark()
    }

    fn plume_dark(self) -> Theme {
        let accent = self.accent.primary();

        Theme::custom(
            format!("Plume Dark ({})", self.accent),
            iced::theme::Palette {
                background: color!(0x1a1a1a),
                text: color!(0xe8e8e8),
                primary: accent,
                success: lighten(accent, 0.08),
                danger: color!(0xe06070),
                warning: lighten(accent, 0.16),
            },
        )
    }
}

impl Default for PlumeTheme {
    fn default() -> Self {
        Self::new(AccentColor::default())
    }
}

pub(crate) fn lighten(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r + amount).min(1.0),
        g: (color.g + amount).min(1.0),
        b: (color.b + amount).min(1.0),
        a: color.a,
    }
}

pub(crate) fn darken(color: Color, amount: f32) -> Color {
    Color {
        r: (color.r - amount).max(0.0),
        g: (color.g - amount).max(0.0),
        b: (color.b - amount).max(0.0),
        a: color.a,
    }
}
