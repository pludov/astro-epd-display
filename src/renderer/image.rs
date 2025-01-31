use std::fs::File;

use crate::{error::DrawingError, renderer::positioning::place_rectangle};

use super::{
    positioning::{HorizontalAlignment, VerticalAlignment},
    ColorFromTemplate, Point,
};
use embedded_graphics::prelude::{DrawTarget, PixelColor};
use png::{BitDepth, Transformations};
use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Image {
    pub path: String,
    pub position: Point,
    pub align: Option<HorizontalAlignment>,
    pub vertical_align: Option<VerticalAlignment>,
    pub invert: Option<bool>,
}

pub fn draw_image<D, TargetColor>(display: &mut D, image: &Image) -> Result<(), DrawingError>
where
    D: DrawTarget<Color = TargetColor, Error: Into<DrawingError>>,
    TargetColor: PixelColor + ColorFromTemplate,
{
    let mut decoder = png::Decoder::new(
        File::open(&image.path).map_err(|e| DrawingError::ResourceError(image.path.clone(), e))?,
    );
    decoder.set_transformations(Transformations::normalize_to_color8());
    let mut reader = decoder
        .read_info()
        .map_err(|e| DrawingError::ImageError(image.path.clone(), e))?;
    // Allocate the output buffer.
    let mut buf = vec![0; reader.output_buffer_size()];
    // Read the next frame. An APNG might contain multiple frames.
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| DrawingError::ImageError(image.path.clone(), e))?;
    // Grab the bytes of the image.
    let bytes = &buf[..info.buffer_size()];

    if info.bit_depth != BitDepth::Eight {
        println!("Unsupported bit depth: {:?}", info.bit_depth);
        return Ok(());
    }

    let (mut back, mut front) = (
        TargetColor::resolve(&Some("0".to_string())),
        TargetColor::resolve(&Some("1".to_string())),
    );
    if image.invert.unwrap_or(false) {
        std::mem::swap(&mut back, &mut front);
    }

    let origin = place_rectangle(
        embedded_graphics::geometry::Size {
            width: info.width,
            height: info.height,
        },
        image.align,
        image.vertical_align,
        image.position.clone(),
    );

    let mut pixels = Vec::with_capacity(256);

    for y in 0..info.height {
        let mut pos = y as usize * info.line_size;
        for x in 0..info.width {
            let color = match info.color_type {
                png::ColorType::Grayscale => Some(bytes[pos] >= 128),
                png::ColorType::GrayscaleAlpha => {
                    if bytes[pos + 1] < 128 {
                        None
                    } else {
                        Some(bytes[pos] >= 128)
                    }
                }
                png::ColorType::Indexed => Some(bytes[pos] != 0),
                png::ColorType::Rgb => {
                    Some(bytes[pos] >= 128 || bytes[pos + 1] >= 128 || bytes[pos + 2] >= 128)
                }
                png::ColorType::Rgba => {
                    if bytes[pos + 3] < 128 {
                        None
                    } else {
                        Some(bytes[pos] >= 128 || bytes[pos + 1] >= 128 || bytes[pos + 2] >= 128)
                    }
                }
            };

            if color.is_some() {
                pixels.push(embedded_graphics::Pixel(
                    embedded_graphics::geometry::Point {
                        x: origin.x + x as i32,
                        y: origin.y + y as i32,
                    },
                    if color.unwrap() { front } else { back },
                ));
                if pixels.len() >= 256 {
                    display.draw_iter(pixels).map_err(|e| e.into())?;
                    pixels = Vec::with_capacity(256);
                }
            }

            pos += info.color_type.samples();
        }
    }

    display.draw_iter(pixels).map_err(|e| e.into())?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::super::tests::*;
    use super::super::Primitive;
    use super::super::*;
    use super::*;

    #[test]
    fn test_png() {
        let display = render(
            embedded_graphics::prelude::Size {
                width: 32,
                height: 32,
            },
            vec![Primitive::Image(Image {
                path: "resources/wifi-small.png".to_string(),
                position: Point { x: 16, y: 16 },
                align: Some(HorizontalAlignment::Center),
                vertical_align: Some(VerticalAlignment::Middle),
                invert: None,
            })],
            None,
        );

        assert_eq!(
            String::from("\n") + &display,
            r#"
████████████████████████████████
████████████████████████████████
████████████▀▀▀▀▀▀▀▀████████████
███████▀▀              ▀▀███████
█████▀   ▄▄▄████████▄▄▄   ▀█████
███▀   ▄████▀▀▀▀▀▀▀▀████▄   ▀███
███▄▄▄███▀            ▀███▄▄▄███
███████▀   ▄████████▄   ▀███████
███████▄▄▄██▀▀▀▀▀▀▀▀██▄▄▄███████
███████████    ▄▄    ███████████
███████████▄▄██████▄▄███████████
██████████████▀  ▀██████████████
█████████████      █████████████
██████████████    ██████████████
████████████████████████████████
████████████████████████████████
"#
        );
    }
}
