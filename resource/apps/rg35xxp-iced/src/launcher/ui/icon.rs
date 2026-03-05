use crate::launcher;
use iced_core::{Color, Length};

include!(concat!(env!("OUT_DIR"), "/material_symbols_match.rs"));

#[derive(Debug, Clone, Copy)]
pub enum Style {
    Outlined,
    Sharp,
    Rounded,
}
pub fn outlined() -> IconManager {
    IconManager {
        style: Style::Outlined,
        size: 24,
        color: Color::BLACK,
    }
}
pub fn sharp() -> IconManager {
    IconManager {
        style: Style::Sharp,
        size: 24,
        color: Color::BLACK,
    }
}

pub fn rounded() -> IconManager {
    IconManager {
        style: Style::Rounded,
        size: 24,
        color: Color::BLACK,
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::Outlined
    }
}

pub struct IconManager {
    style: Style,
    size: u32,
    color: Color,
}
impl IconManager {
    pub fn icon(&self, name: &str) -> launcher::Element<'static> {
        let font = match self.style {
            Style::Outlined => "Material Symbols Outlined",
            Style::Sharp => "Material Symbols Rounded",
            Style::Rounded => "Material Symbols Rounded",
        };
        let char_check: char = get_icon_codepoint(name).unwrap_or('?');
        iced_widget::container(
            iced_widget::text(char_check)
                .font(iced::font::Font::with_name(font))
                .size(self.size)
                .color(self.color),
        )
        .width(self.size as f32)
        .height(self.size as f32)
        .center_x(Length::Shrink)
        .center_y(Length::Shrink)
        .into()
    }
    pub fn style(&self, style: Style) -> Self {
        Self {
            color: self.color,
            style,
            size: self.size,
        }
    }
    pub fn size(&self, s: u32) -> Self {
        Self {
            color: self.color,
            style: self.style,
            size: s,
        }
    }
    pub fn color(&self, color: Color) -> Self {
        Self {
            color,
            style: self.style,
            size: self.size,
        }
    }
}
impl Default for IconManager {
    fn default() -> Self {
        IconManager {
            style: Default::default(),
            size: 24,
            color: Color::BLACK,
        }
    }
}

pub fn icon(name: &str) -> launcher::Element<'static> {
    IconManager {
        color: Color::BLACK,
        style: Style::default(),
        size: 24,
    }
    .icon(name)
}
pub fn light(name: &str) -> launcher::Element<'static> {
    IconManager {
        color: Color::WHITE,
        style: Style::default(),
        size: 24,
    }
        .icon(name)
}
pub fn dark(name: &str) -> launcher::Element<'static> {
    IconManager {
        color: Color::WHITE,
        style: Style::default(),
        size: 24,
    }
        .icon(name)
}
