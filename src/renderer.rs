use crate::{
    binary_framebuffer::{BinarisedColor, BinaryFrameBuffer},
    error::Error,
};
use embedded_graphics::Drawable;
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};
use serde::{Deserialize, Serialize};
use yaml_merge_keys::serde_yaml;

pub trait ColorFromTemplate {
    fn resolve(color: &str) -> Self;
}

// EPD specific implem.
impl ColorFromTemplate for BinaryColor {
    fn resolve(color: &str) -> Self {
        match color {
            "black" => BinaryColor::Off,
            "0" => BinaryColor::Off,
            "white" => BinaryColor::On,
            "1" => BinaryColor::On,
            _ => BinaryColor::Off,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Into<embedded_graphics::geometry::Point> for Point {
    fn into(self) -> embedded_graphics::geometry::Point {
        embedded_graphics::geometry::Point::new(self.x, self.y)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextItem {
    pub value: String,
    pub position: Point,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dummy {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Primitive {
    #[serde(rename = "x-ignore")]
    Dummy(Dummy),
    Text(TextItem),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct PrimitiveWrapper(#[serde(with = "serde_yaml::with::singleton_map")] pub Primitive);

pub fn parse(yaml: serde_yaml::Value) -> Result<Vec<Primitive>, Error> {
    let mut primitives = Vec::new();

    let mut id = 1;
    let sequence = yaml.as_sequence();
    if sequence.is_none() {
        return Ok(primitives);
    }
    for item in sequence.unwrap() {
        let primitive = serde_yaml::from_value::<PrimitiveWrapper>(item.clone())
            .map_err(|e| Error::InvalidPrimitive(id, e))?;

        primitives.push(primitive.0);
        id += 1;
    }

    Ok(primitives)
}

pub fn draw<D, TargetColor>(display: &mut D, primitives: Vec<Primitive>) -> Result<(), Error>
where
    D: DrawTarget<Color = TargetColor>,
    TargetColor: PixelColor + ColorFromTemplate,
{
    for primitive in primitives {
        println!("Rendering {:?}", primitive);
        match primitive {
            Primitive::Dummy(_) => {}
            Primitive::Text(text) => {
                let style = MonoTextStyleBuilder::new()
                    .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
                    .text_color(TargetColor::resolve("white"))
                    .build();
                let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

                // FIXME: mess with custom errors... :-(
                Text::with_text_style(&text.value, text.position.into(), style, text_style)
                    .draw(display)
                    .map_err(|_e| println!("drawing error"));
            }
        }
    }
    Ok(())
}

fn map_ascii(a: u8, b: u8) -> char {
    if a == 0 && b == 0 {
        ' '
    } else {
        if a == 0 {
            '▄'
        } else {
            if b == 0 {
                '▀'
            } else {
                '█'
            }
        }
    }
}

pub fn to_display_string<C: BinarisedColor>(
    buffer: &BinaryFrameBuffer<C>,
    bounds: Option<Size>,
) -> String {
    let mut result = String::new();
    let width = buffer.width();

    let bounds = bounds.unwrap_or(Size {
        width: width,
        height: buffer.height(),
    });

    let get_pixel = |x: u32, y: u32| -> u8 {
        let index = (y * width + x) as usize;

        buffer.get_bit(index) as u8
    };

    let maxx = bounds.width;
    let maxy = bounds.height;

    // Iterator over two lines
    for y in 0..maxy / 2 {
        for x in 0..maxx {
            let v1 = get_pixel(x, y * 2);
            let v2 = get_pixel(x, y * 2 + 1);
            result.push(map_ascii(v1, v2));
        }
        result.push_str("\n");
    }
    result
}

#[cfg(test)]
mod tests {
    use embedded_graphics::pixelcolor::BinaryColor;
    use embedded_graphics_framebuf::FrameBuf;

    use crate::binary_framebuffer::BinaryFrameBuffer;

    use super::*;

    #[test]
    fn test_parse() {
        let yaml = serde_yaml::from_str(
            r#"
        -
          x-ignore: {}
        -
          text:
            value: "Hello, World!"
            position:
              x: 0
              y: 0
        "#,
        )
        .unwrap();

        let result = parse(yaml);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Primitive::Dummy(Dummy {}));
        assert_eq!(
            result[1],
            Primitive::Text(TextItem {
                value: "Hello, World!".to_string(),
                position: Point { x: 0, y: 0 }
            })
        );
    }

    #[test]
    fn test_bit_rendering() {
        let w = 40;
        let size = Size {
            width: 40,
            height: 26,
        };

        let mut buffer = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);

        buffer.set_bit(0, true);
        buffer.set_bit(1, false);
        buffer.set_bit(2, true);
        buffer.set_bit(w + 0, false);
        buffer.set_bit(w + 1, true);
        buffer.set_bit(w + 2, false);

        let display = to_display_string(
            &buffer,
            Some(Size {
                width: 4,
                height: 2,
            }),
        );
        println!("{}", display);

        assert_eq!(
            String::from("\n") + &display,
            r#"
▀▄▀ 
"#
        );
    }

    #[test]
    fn test_render() {
        let size = Size {
            width: 90,
            height: 128,
        };
        let mut buffer = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);
        let mut display = FrameBuf::<BinaryColor, &mut BinaryFrameBuffer<BinaryColor>>::new(
            &mut buffer,
            size.width as usize,
            size.height as usize,
        );
        // let mut display = Display1in54::default();

        display.clear(BinaryColor::On).unwrap();
        let primitives = vec![Primitive::Text(TextItem {
            value: "Hello, World!".to_string(),
            position: Point { x: 0, y: 0 },
        })];

        let result = draw(&mut display, primitives);
        result.unwrap();

        let display = to_display_string(
            &buffer,
            Some(Size {
                width: 81,
                height: 12,
            }),
        );
        println!("{}", display);

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
}
