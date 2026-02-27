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
pub(crate) enum PlumeTheme {
    PlumeDark,
}

impl PlumeTheme {
    pub(crate) fn to_iced_theme(self) -> Theme {
        Self::plume_dark()
    }

    fn plume_dark() -> Theme {
        Theme::custom(
            "Plume Dark".to_string(),
            iced::theme::Palette {
                background: color!(0x1a1a1a),
                text: color!(0xe8e8e8),
                primary: color!(0xe88aab),
                success: color!(0xd4a0b0),
                danger: color!(0xe06070),
                warning: color!(0xf0a0b0),
            },
        )
    }
}

impl Default for PlumeTheme {
    fn default() -> Self {
        Self::PlumeDark
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
