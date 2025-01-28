use crate::{
    binary_framebuffer::{BinarisedColor, BinaryFrameBuffer},
    error::DrawingError,
};

use super::{ColorFromTemplate, Point};
use embedded_graphics::pixelcolor::raw::RawU1;
use embedded_graphics::primitives::Rectangle;

use embedded_graphics::prelude::*;

use qrcode::render::{Canvas, Pixel};
use serde::{Deserialize, Serialize};

mod eclevel {
    use qrcode::EcLevel;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn default() -> Option<EcLevel> {
        None
    }

    pub fn serialize<S>(v: &Option<EcLevel>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if v.is_none() {
            return Option::<String>::None.serialize(s);
        }

        let v: &str = match v.unwrap() {
            EcLevel::L => "L",
            EcLevel::M => "M",
            EcLevel::Q => "Q",
            EcLevel::H => "H",
        };

        v.to_string().serialize(s)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<EcLevel>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Option::<String>::deserialize(d)?.map(|e| e.to_lowercase());

        if v.is_none() {
            return Ok(None);
        } else {
            match v.unwrap().as_str() {
                "l" => Ok(Some(EcLevel::L)),
                "m" => Ok(Some(EcLevel::M)),
                "q" => Ok(Some(EcLevel::Q)),
                "h" => Ok(Some(EcLevel::H)),
                v => {
                    println!("Invalid alignment : {:?}", v);
                    Ok(None)
                }
            }
        }
    }
}

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QRCode {
    pub value: String,
    pub position: Point,
    pub color: Option<String>,
    pub background: Option<String>,
    #[serde(with = "eclevel", default = "eclevel::default")]
    pub ec_level: Option<qrcode::EcLevel>,
    pub width: u32,
    pub height: u32,
    // #[serde(default = "baseline::default", with = "baseline")]
    // pub baseline: Option<Baseline>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct DummyPixel(u8);

impl BinarisedColor for DummyPixel {
    fn to_binary_color(&self) -> bool {
        self.0 != 0
    }
    fn from_binary_color(value: bool) -> Self {
        DummyPixel(if value { 1 } else { 0 })
    }
}

impl PixelColor for DummyPixel {
    type Raw = RawU1;
}

struct DummyCanvas {
    dark_pixel: DummyPixel,
    buffer: BinaryFrameBuffer<DummyPixel>,
}

impl Canvas for DummyCanvas {
    type Pixel = DummyPixel;
    type Image = BinaryFrameBuffer<DummyPixel>;

    /// Constructs a new canvas of the given dimensions.
    fn new(width: u32, height: u32, dark_pixel: Self::Pixel, light_pixel: Self::Pixel) -> Self {
        let mut buffer = BinaryFrameBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                buffer.set_pixel(x, y, light_pixel);
            }
        }
        Self { buffer, dark_pixel }
    }

    /// Draws a single dark pixel at the (x, y) coordinate.s
    fn draw_dark_pixel(&mut self, x: u32, y: u32) {
        self.buffer.set_pixel(x, y, self.dark_pixel);
    }

    fn draw_dark_rect(&mut self, left: u32, top: u32, width: u32, height: u32) {
        for y in top..(top + height) {
            for x in left..(left + width) {
                self.draw_dark_pixel(x, y);
            }
        }
    }

    /// Finalize the canvas to a real image.
    fn into_image(self) -> Self::Image {
        println!("into_image from {:?}", self.buffer);
        self.buffer
    }
}

impl Pixel for DummyPixel {
    type Image = BinaryFrameBuffer<DummyPixel>;

    /// The type that stores an intermediate buffer before finalizing to a
    /// concrete image
    type Canvas = DummyCanvas;

    /// Obtains the default module size. The result must be at least 1×1.
    fn default_unit_size() -> (u32, u32) {
        (1, 1)
    }

    /// Obtains the default pixel color when a module is dark or light.
    fn default_color(color: qrcode::Color) -> Self {
        match color {
            qrcode::Color::Dark => DummyPixel(0),
            qrcode::Color::Light => DummyPixel(1),
        }
    }
}

pub fn draw_qrcode<D, TargetColor>(display: &mut D, qrcode: &QRCode) -> Result<(), DrawingError>
where
    D: DrawTarget<Color = TargetColor, Error: Into<DrawingError>>,
    TargetColor: PixelColor + ColorFromTemplate,
{
    let code = match qrcode.ec_level {
        Some(level) => qrcode::QrCode::with_error_correction_level(&qrcode.value, level),
        None => qrcode::QrCode::new(&qrcode.value),
    };
    if code.is_err() {
        println!("Error rendering QR code : {:?}", code.err());
        return Ok(());
    }

    let code = code.unwrap();

    let mut render = code.render::<DummyPixel>();

    let res = render
        .dark_color(DummyPixel(0))
        .light_color(DummyPixel(1))
        .min_dimensions(qrcode.width, qrcode.height)
        .max_dimensions(qrcode.width, qrcode.height)
        .quiet_zone(true)
        .build();

    let (back, front) = if qrcode.color.is_some() && qrcode.background.is_some() {
        (
            TargetColor::resolve(&qrcode.background),
            TargetColor::resolve(&qrcode.color),
        )
    } else if qrcode.background.is_some() {
        let back = TargetColor::resolve(&qrcode.background);
        let color = back.invert();
        (back, color)
    } else {
        let color = TargetColor::resolve(&qrcode.color);
        let back = color.invert();
        (back, color)
    };

    display
        .fill_solid(
            &Rectangle {
                top_left: qrcode.position.clone().into(),
                size: Size::new(qrcode.width, qrcode.height),
            },
            back,
        )
        .map_err(Into::into)?;

    let actual_size = res.dimensions();

    let mut shift: (i32, i32) = (
        if actual_size.width < qrcode.width {
            ((qrcode.width - actual_size.width) / 2) as i32
        } else {
            0
        },
        if actual_size.height < qrcode.height {
            ((qrcode.height - actual_size.height) / 2) as i32
        } else {
            0
        },
    );

    shift.0 += qrcode.position.x;
    shift.1 += qrcode.position.y;

    display
        .draw_iter(
            res.iter()
                .into_iter()
                .filter(|Pixel(_, c)| !c.to_binary_color())
                .map(move |Pixel(p, c)| {
                    let c = c.to_binary_color();
                    let new_p = embedded_graphics::geometry::Point {
                        x: p.x + shift.0,
                        y: p.y + shift.1,
                    };
                    embedded_graphics::Pixel(new_p, if c { back } else { front })
                }),
        )
        .map_err(Into::into)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::super::Primitive;
    use super::super::*;
    use super::*;

    #[test]
    fn test_color_1() {
        let display = render(
            Size {
                width: 32,
                height: 32,
            },
            vec![Primitive::QRCode(QRCode {
                value: "Hello, World!".to_string(),
                position: Point { x: 0, y: 0 },
                color: Some("1".to_string()),
                background: None,
                ec_level: None,
                width: 32,
                height: 32,
            })],
            None,
        );

        assert_eq!(
            String::from("\n") + &display,
            r#"
                                
                                
     ▄▄▄▄▄▄▄ ▄  ▄  ▄▄▄▄▄▄▄      
     █ ▄▄▄ █ ▀█ ▄  █ ▄▄▄ █      
     █ ███ █ ▀█▄█  █ ███ █      
     █▄▄▄▄▄█ ▄▀▄▀█ █▄▄▄▄▄█      
     ▄ ▄▄ ▄▄▄▀█ ██ ▄  ▄ ▄▄      
     ▄▄▀ █▀▄▀▀█▄▄▀█ ▄ ▀▀▄█      
     █ █▄ ▄▄ ▀▀█▀▄ ▄▄▀▀ ▀▄      
     ▄▄▄▄▄▄▄ █▄█▀▀▀▄ █ ▀ ▀      
     █ ▄▄▄ █ ▀██ ▀█ █▄███       
     █ ███ █ █  ▄█▀▄▄ ▀█▀       
     █▄▄▄▄▄█ ▄ ██▄███▀ ▄ ▀      
                                
                                
                                
"#
        );
    }

    #[test]
    fn test_color_0() {
        let display = render(
            Size {
                width: 32,
                height: 32,
            },
            vec![Primitive::QRCode(QRCode {
                value: "Hello, World!".to_string(),
                position: Point { x: 0, y: 0 },
                color: Some("0".to_string()),
                background: None,
                ec_level: None,
                width: 32,
                height: 32,
            })],
            None,
        );

        assert_eq!(
            String::from("\n") + &display,
            r#"
████████████████████████████████
████████████████████████████████
█████▀▀▀▀▀▀▀█▀██▀██▀▀▀▀▀▀▀██████
█████ █▀▀▀█ █▄ █▀██ █▀▀▀█ ██████
█████ █   █ █▄ ▀ ██ █   █ ██████
█████ ▀▀▀▀▀ █▀▄▀▄ █ ▀▀▀▀▀ ██████
█████▀█▀▀█▀▀▀▄ █  █▀██▀█▀▀██████
█████▀▀▄█ ▄▀▄▄ ▀▀▄ █▀█▄▄▀ ██████
█████ █ ▀█▀▀█▄▄ ▄▀█▀▀▄▄█▄▀██████
█████▀▀▀▀▀▀▀█ ▀ ▄▄▄▀█ █▄█▄██████
█████ █▀▀▀█ █▄  █▄ █ ▀   ███████
█████ █   █ █ ██▀ ▄▀▀█▄ ▄███████
█████ ▀▀▀▀▀ █▀█  ▀   ▄█▀█▄██████
████████████████████████████████
████████████████████████████████
████████████████████████████████
"#
        );
    }

    #[test]
    fn test_render_overflow() {
        let display = render(
            Size {
                width: 32,
                height: 32,
            },
            vec![Primitive::QRCode(QRCode {
                value: "Hello, World!".to_string(),
                position: Point { x: 16, y: 16 },
                color: Some("0".to_string()),
                background: None,
                ec_level: None,
                width: 32,
                height: 32,
            })],
            Some(Rectangle::new(
                embedded_graphics::prelude::Point { x: 0, y: 0 },
                Size {
                    width: 32,
                    height: 32,
                },
            )),
        );

        assert_eq!(
            String::from("\n") + &display,
            r#"
████████████████████████████████
████████████████████████████████
████████████████████████████████
████████████████████████████████
████████████████████████████████
████████████████████████████████
████████████████████████████████
████████████████████████████████
████████████████████████████████
████████████████████████████████
█████████████████████▀▀▀▀▀▀▀█▀██
█████████████████████ █▀▀▀█ █▄ █
█████████████████████ █   █ █▄ ▀
█████████████████████ ▀▀▀▀▀ █▀▄▀
█████████████████████▀█▀▀█▀▀▀▄ █
█████████████████████▀▀▄█ ▄▀▄▄ ▀
"#
        );
    }

    #[test]
    fn test_render_fill() {
        let display = render(
            Size {
                width: 90,
                height: 90,
            },
            vec![Primitive::QRCode(QRCode {
                value: "lés bon amis amös L3s €".to_string(),
                position: Point { x: 4, y: 6 },
                color: Some("1".to_string()),
                background: None,
                ec_level: Some(::qrcode::EcLevel::L),
                width: 82,
                height: 72,
            })],
            None,
        );

        assert_eq!(
            String::from("\n") + &display,
            r#"
██████████████████████████████████████████████████████████████████████████████████████████
██████████████████████████████████████████████████████████████████████████████████████████
██████████████████████████████████████████████████████████████████████████████████████████
████                                                                                  ████
████                                                                                  ████
████                                                                                  ████
████                                                                                  ████
████                                                                                  ████
████                ▄▄▄▄▄▄▄▄▄▄▄▄▄▄    ▄▄    ▄▄    ▄▄▄▄  ▄▄▄▄▄▄▄▄▄▄▄▄▄▄                ████
████                ██▀▀▀▀▀▀▀▀▀▀██  ▄▄▀▀  ▄▄██▄▄▄▄▀▀▀▀  ██▀▀▀▀▀▀▀▀▀▀██                ████
████                ██  ▄▄▄▄▄▄  ██  ██▄▄▄▄████▀▀▀▀▄▄▄▄  ██  ▄▄▄▄▄▄  ██                ████
████                ██  ██████  ██  ▀▀██████▀▀  ▄▄██▀▀  ██  ██████  ██                ████
████                ██  ██████  ██  ▄▄▀▀▀▀▀▀  ▄▄▀▀▀▀▄▄  ██  ██████  ██                ████
████                ██  ▀▀▀▀▀▀  ██  ██▄▄  ▄▄  ▀▀▄▄  ██  ██  ▀▀▀▀▀▀  ██                ████
████                ██▄▄▄▄▄▄▄▄▄▄██  ██▀▀▄▄▀▀▄▄  ██  ██  ██▄▄▄▄▄▄▄▄▄▄██                ████
████                ▀▀▀▀▀▀▀▀▀▀▀▀▀▀  ██▄▄▀▀  ██  ██▄▄▀▀  ▀▀▀▀▀▀▀▀▀▀▀▀▀▀                ████
████                ▄▄▄▄  ▄▄    ▄▄▄▄▀▀▀▀▄▄  ██▄▄▀▀██▄▄  ▄▄▄▄▄▄  ▄▄▄▄                  ████
████                ██▀▀▄▄▀▀▄▄▄▄▀▀██▄▄  ██▄▄▀▀██  ▀▀██▄▄██▀▀▀▀▄▄████▄▄                ████
████                ██  ██  ▀▀██▄▄▀▀▀▀▄▄██▀▀▄▄██▄▄  ██▀▀██▄▄▄▄██▀▀▀▀▀▀                ████
████                ▀▀▄▄▀▀▄▄▄▄██▀▀▄▄  ▀▀▀▀  ██████▄▄▀▀▄▄▀▀████▀▀  ▄▄▄▄                ████
████                  ██▄▄▀▀████▄▄██▄▄    ▄▄██▀▀▀▀▀▀▄▄██▄▄▀▀▀▀▄▄  ▀▀██                ████
████                  ▀▀██▄▄▀▀██▀▀████▄▄▄▄▀▀▀▀▄▄    ▀▀████    ██▄▄▄▄▀▀                ████
████                ▄▄  ██▀▀▄▄▀▀▄▄██▀▀▀▀▀▀  ▄▄▀▀▄▄  ▄▄▀▀▀▀  ▄▄██▀▀▀▀                  ████
████                ▀▀▄▄▀▀  ██▄▄▀▀▀▀▄▄      ▀▀▄▄▀▀▄▄▀▀      ████  ▄▄                  ████
████                ▄▄██▄▄▄▄████▄▄  ██  ▄▄  ▄▄▀▀▄▄▀▀▄▄▄▄▄▄▄▄██▀▀▄▄██                  ████
████                ▀▀▀▀▀▀▀▀▀▀▀▀▀▀  ██  ██  ▀▀▄▄▀▀▄▄██▀▀▀▀▀▀██▄▄▀▀▀▀▄▄                ████
████                ▄▄▄▄▄▄▄▄▄▄▄▄▄▄  ██  ██▄▄▄▄██▄▄████  ▄▄  ██▀▀▄▄▄▄██                ████
████                ██▀▀▀▀▀▀▀▀▀▀██  ▀▀  ██▀▀▀▀▀▀██████  ▀▀  ██  ██▀▀██                ████
████                ██  ▄▄▄▄▄▄  ██      ▀▀▄▄▄▄▄▄██▀▀██▄▄▄▄▄▄██  ▀▀▄▄▀▀                ████
████                ██  ██████  ██  ▄▄    ▀▀██████  ██▀▀▀▀████▄▄  ▀▀                  ████
████                ██  ██████  ██  ▀▀▄▄▄▄▄▄██▀▀██  ▀▀    ▀▀██▀▀▄▄  ▄▄                ████
████                ██  ▀▀▀▀▀▀  ██  ▄▄▀▀██▀▀▀▀▄▄▀▀  ▄▄  ▄▄  ▀▀  ▀▀  ▀▀                ████
████                ██▄▄▄▄▄▄▄▄▄▄██  ██  ██  ▄▄██▄▄▄▄██▄▄██▄▄      ▄▄▄▄                ████
████                ▀▀▀▀▀▀▀▀▀▀▀▀▀▀  ▀▀  ▀▀  ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀      ▀▀▀▀                ████
████                                                                                  ████
████                                                                                  ████
████                                                                                  ████
████                                                                                  ████
████                                                                                  ████
██████████████████████████████████████████████████████████████████████████████████████████
██████████████████████████████████████████████████████████████████████████████████████████
██████████████████████████████████████████████████████████████████████████████████████████
██████████████████████████████████████████████████████████████████████████████████████████
██████████████████████████████████████████████████████████████████████████████████████████
██████████████████████████████████████████████████████████████████████████████████████████
"#
        );
    }
}
