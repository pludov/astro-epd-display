use crate::error::DrawingError;

use super::drawing_error::IntoDrawingError;
use super::{ColorFromTemplate, Point};
use embedded_graphics::{mono_font::MonoFont, text::Alignment, Drawable};
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};

use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextItem {
    pub value: String,
    pub position: Point,
    pub font: Option<String>,
    pub color: Option<String>,
    #[serde(with = "super::alignment", default = "super::alignment::default")]
    pub align: Option<Alignment>,
    // #[serde(default = "baseline::default", with = "baseline")]
    // pub baseline: Option<Baseline>,
}

pub fn resolve_font(font: &Option<String>) -> &MonoFont<'static> {
    match font {
        None => &embedded_graphics::mono_font::ascii::FONT_6X10,
        Some(font) => match font.to_ascii_uppercase().as_str() {
            "4X6" => &embedded_graphics::mono_font::ascii::FONT_4X6,
            "5X7" => &embedded_graphics::mono_font::ascii::FONT_5X7,
            "5X8" => &embedded_graphics::mono_font::ascii::FONT_5X8,
            "6X9" => &embedded_graphics::mono_font::ascii::FONT_6X9,
            "6X10" => &embedded_graphics::mono_font::ascii::FONT_6X10,
            "6X12" => &embedded_graphics::mono_font::ascii::FONT_6X12,
            "6X13" => &embedded_graphics::mono_font::ascii::FONT_6X13,
            "6X13_BOLD" => &embedded_graphics::mono_font::ascii::FONT_6X13_BOLD,
            "6X13_ITALIC" => &embedded_graphics::mono_font::ascii::FONT_6X13_ITALIC,
            "7X13" => &embedded_graphics::mono_font::ascii::FONT_7X13,
            "7X13_BOLD" => &embedded_graphics::mono_font::ascii::FONT_7X13_BOLD,
            "7X13_ITALIC" => &embedded_graphics::mono_font::ascii::FONT_7X13_ITALIC,
            "7X14" => &embedded_graphics::mono_font::ascii::FONT_7X14,
            "7X14_BOLD" => &embedded_graphics::mono_font::ascii::FONT_7X14_BOLD,
            "8X13" => &embedded_graphics::mono_font::ascii::FONT_8X13,
            "8X13_BOLD" => &embedded_graphics::mono_font::ascii::FONT_8X13_BOLD,
            "8X13_ITALIC" => &embedded_graphics::mono_font::ascii::FONT_8X13_ITALIC,
            "9X15" => &embedded_graphics::mono_font::ascii::FONT_9X15,
            "9X15_BOLD" => &embedded_graphics::mono_font::ascii::FONT_9X15_BOLD,
            "9X18" => &embedded_graphics::mono_font::ascii::FONT_9X18,
            "9X18_BOLD" => &embedded_graphics::mono_font::ascii::FONT_9X18_BOLD,
            "10X20" => &embedded_graphics::mono_font::ascii::FONT_10X20,
            _ => &embedded_graphics::mono_font::ascii::FONT_6X10,
        },
    }
}

pub fn draw_text<D, TargetColor>(display: &mut D, text: &TextItem) -> Result<(), DrawingError>
where
    D: DrawTarget<Color = TargetColor, Error: IntoDrawingError>,
    TargetColor: PixelColor + ColorFromTemplate,
{
    let style = MonoTextStyleBuilder::new()
        .font(resolve_font(&text.font))
        .text_color(TargetColor::resolve(&text.color))
        .build();
    let text_style = TextStyleBuilder::new()
        .baseline(Baseline::Top)
        .alignment(text.align.unwrap_or(Alignment::Left))
        .build();

    Text::with_text_style(&text.value, text.position.clone().into(), style, text_style)
        .draw(display)
        .map(|_| ())
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::super::Primitive;
    use super::super::*;
    use super::*;

    #[test]
    fn test_render_default_font() {
        let display = render(
            Size {
                width: 90,
                height: 128,
            },
            vec![Primitive::Text(TextItem {
                value: "Hello, World!".to_string(),
                position: Point { x: 0, y: 0 },
                font: None,
                color: Some("0".to_string()),
                align: None,
            })],
            Some(Rectangle::new(
                embedded_graphics::prelude::Point { x: 0, y: 0 },
                Size {
                    width: 81,
                    height: 12,
                },
            )),
        );

        assert_eq!(
            String::from("\n") + &display,
            r#"
▀███▀████████▀▀████▀▀█████████████████████▀███▀██████████████▀▀███████▀███▀██████
 ███ ██▀▀▀████ █████ ████▀▀▀██████████████ ███ ██▀▀▀██▀█▀▀████ ████▀▀█ ███ ██████
 ▄▄▄ █ ▀▀▀ ███ █████ ███ ███ █████████████ █ █ █ ███ █ ▄██▄███ ███ ██▄ ███ ██████
 ███ █▄▀▀▀███▀ ▀███▀ ▀██▄▀▀▀▄███ ▄████████ ▄█▄ █▄▀▀▀▄█ ██████▀ ▀██▄▀▀▄ ███▀██████
███████████████████████████████▄█████████████████████████████████████████████████
█████████████████████████████████████████████████████████████████████████████████
"#
        );
    }

    #[test]
    fn test_render_small_font() {
        let display = render(
            Size {
                width: 90,
                height: 128,
            },
            vec![Primitive::Text(TextItem {
                value: "Hello, World!".to_string(),
                position: Point { x: 0, y: 0 },
                font: Some("4x6".to_string()),
                color: Some("0".to_string()),
                align: None,
            })],
            Some(Rectangle::new(
                embedded_graphics::prelude::Point { x: 0, y: 0 },
                Size {
                    width: 81,
                    height: 12,
                },
            )),
        );

        assert_eq!(
            String::from("\n") + &display,
            r#"
 █ ██▀██▄ ██▄ ███▀██████████ █ ██▀██▀█▀█▄ ███▀ ██ ███████████████████████████████
 ▄ █ ▀▄██ ███ ██ █ █████████   █ █ █ ▄███ ██ █ ██▄███████████████████████████████
▄█▄██▄▄█▄▄▄█▄▄▄██▄██▀▄██████▄█▄██▄██▄███▄▄▄██▄▄██▄███████████████████████████████
█████████████████████████████████████████████████████████████████████████████████
█████████████████████████████████████████████████████████████████████████████████
█████████████████████████████████████████████████████████████████████████████████
"#
        );
    }

    #[test]
    fn test_render_center() {
        let display = render(
            Size {
                width: 90,
                height: 128,
            },
            vec![Primitive::Text(TextItem {
                value: "Hello, World!".to_string(),
                position: Point { x: 40, y: 0 },
                font: Some("4x6".to_string()),
                color: Some("0".to_string()),
                align: Some(Alignment::Center),
            })],
            Some(Rectangle::new(
                embedded_graphics::prelude::Point { x: 0, y: 0 },
                Size {
                    width: 81,
                    height: 12,
                },
            )),
        );

        assert_eq!(
            String::from("\n") + &display,
            r#"
███████████████ █ ██▀██▄ ██▄ ███▀██████████ █ ██▀██▀█▀█▄ ███▀ ██ ████████████████
███████████████ ▄ █ ▀▄██ ███ ██ █ █████████   █ █ █ ▄███ ██ █ ██▄████████████████
███████████████▄█▄██▄▄█▄▄▄█▄▄▄██▄██▀▄██████▄█▄██▄██▄███▄▄▄██▄▄██▄████████████████
█████████████████████████████████████████████████████████████████████████████████
█████████████████████████████████████████████████████████████████████████████████
█████████████████████████████████████████████████████████████████████████████████
"#
        );
    }
}
