mod alignment;
mod drawing_error;
mod image;
mod positioning;
mod progress;
mod qrcode;
mod text;

use std::fmt::Debug;

use crate::binary_framebuffer::{BinarisedColor, BinaryFrameBuffer};
use crate::error::Error;
use drawing_error::IntoDrawingError;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use image::{draw_image, Image};
use progress::Progress;
use qrcode::{draw_qrcode, QRCode};
use serde::{Deserialize, Serialize};
use text::{draw_text, TextItem};
use yaml_merge_keys::serde_yaml;

pub trait ColorFromTemplate {
    fn resolve(color: &Option<String>) -> Self;
    fn invert(&self) -> Self;
}

// EPD specific implem.
impl ColorFromTemplate for BinaryColor {
    fn resolve(color: &Option<String>) -> Self {
        match color.as_ref().map(|s| s.as_str()) {
            Some("black") => BinaryColor::Off,
            Some("0") => BinaryColor::Off,
            Some("white") => BinaryColor::On,
            Some("1") => BinaryColor::On,
            _ => BinaryColor::Off,
        }
    }

    fn invert(&self) -> Self {
        match self {
            BinaryColor::Off => BinaryColor::On,
            BinaryColor::On => BinaryColor::Off,
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
pub struct Dummy {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Primitive {
    #[serde(rename = "x-ignore")]
    Dummy(Dummy),
    Text(TextItem),
    QRCode(QRCode),
    Image(Image),
    Progress(Progress),
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

pub fn draw<D, TargetColor>(display: &mut D, primitives: &Vec<Primitive>) -> Result<(), Error>
where
    D: DrawTarget<Color = TargetColor, Error: IntoDrawingError>,
    TargetColor: PixelColor + ColorFromTemplate,
{
    let mut result: Result<(), Error> = Ok(());
    for primitive in primitives {
        println!("Rendering {:?}", primitive);
        let problem = match primitive {
            Primitive::Dummy(_) => Ok(()),
            Primitive::Text(text) => draw_text::<D, TargetColor>(display, text),
            Primitive::Image(image) => draw_image::<D, TargetColor>(display, image),
            Primitive::QRCode(qr) => draw_qrcode::<D, TargetColor>(display, qr),
            Primitive::Progress(progress) => {
                progress::draw_progress::<D, TargetColor>(display, progress)
            }
        };
        if let Err(err) = problem {
            println!("Error rendering {:?}: {:?}", primitive, &err);
            if result.is_ok() {
                result = Err(Error::DrawingError(err.into()));
            }
        }
    }
    result
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
    bounds: Option<Rectangle>, // Bounds must be even for y
) -> String {
    let mut result = String::new();
    let width = buffer.width();

    let bounds = bounds.unwrap_or(Rectangle::new(
        embedded_graphics::prelude::Point { x: 0, y: 0 },
        Size {
            width: buffer.width(),
            height: buffer.height(),
        },
    ));

    let get_pixel = |x: i32, y: i32| -> u8 {
        let index = (y as u32 * width + x as u32) as usize;

        buffer.get_bit(index) as u8
    };

    let minx = bounds.top_left.x;
    let miny = bounds.top_left.y;
    let maxx = minx + bounds.size.width as i32;
    let maxy = miny + bounds.size.height as i32;

    // Iterator over two lines
    for y in miny / 2..maxy / 2 {
        for x in minx..maxx {
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
    use assert_ok::assert_ok;
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
        assert_ok!(&result);

        let result = result.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Primitive::Dummy(Dummy {}));
        assert_eq!(
            result[1],
            Primitive::Text(TextItem {
                value: "Hello, World!".to_string(),
                position: Point { x: 0, y: 0 },
                align: None,
                font: None,
                color: None
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
            Some(Rectangle::new(
                embedded_graphics::prelude::Point { x: 0, y: 0 },
                Size {
                    width: 4,
                    height: 2,
                },
            )),
        );
        println!("{}", display);

        assert_eq!(
            String::from("\n") + &display,
            r#"
▀▄▀ 
"#
        );
    }

    pub fn render(size: Size, primitives: Vec<Primitive>, bounds: Option<Rectangle>) -> String {
        let mut buffer = BinaryFrameBuffer::<BinaryColor>::new(size.width, size.height);
        let mut display = FrameBuf::<BinaryColor, &mut BinaryFrameBuffer<BinaryColor>>::new(
            &mut buffer,
            size.width as usize,
            size.height as usize,
        );
        // let mut display = Display1in54::default();

        display.clear(BinaryColor::On).unwrap();

        let result = draw(&mut display, &primitives);
        result.unwrap();

        let display = to_display_string(&buffer, bounds);
        println!("{}", display);

        display
    }
}
