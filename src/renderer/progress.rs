use crate::{error::DrawingError, renderer::positioning::place_rectangle};

use super::{
    positioning::{Direction, HorizontalAlignment, VerticalAlignment},
    ColorFromTemplate, Point,
};
use embedded_graphics::prelude::{DrawTarget, PixelColor};
use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Progress {
    pub position: Point,
    pub width: u32,
    pub height: u32,

    pub align: Option<HorizontalAlignment>,
    pub vertical_align: Option<VerticalAlignment>,

    pub direction: Option<Direction>,

    /// modulo default to width for direction horizontal and height for direction vertical
    pub modulo: Option<u32>,

    /// Threshold default to 0 (all black)
    pub threshold: Option<u32>,

    /// Base default to 0 (no offset)
    pub base: Option<u32>,
}

pub fn draw_progress<D, TargetColor>(
    display: &mut D,
    progress: &Progress,
) -> Result<(), DrawingError>
where
    D: DrawTarget<Color = TargetColor, Error: Into<DrawingError>>,
    TargetColor: PixelColor + ColorFromTemplate,
{
    let (back, front) = (
        TargetColor::resolve(&Some("0".to_string())),
        TargetColor::resolve(&Some("1".to_string())),
    );

    let origin = place_rectangle(
        embedded_graphics::geometry::Size {
            width: progress.width,
            height: progress.height,
        },
        progress.align,
        progress.vertical_align,
        progress.position.clone(),
    );
    let mut pixels = Vec::with_capacity(256);

    for y in 0..progress.height {
        for x in 0..progress.width {
            let id = match progress.direction {
                None | Some(Direction::Horizontal) => x + y * progress.width,
                Some(Direction::Vertical) => y + x * progress.height,
            };

            let v = (progress.base.unwrap_or(0) + id)
                % progress.modulo.unwrap_or(match progress.direction {
                    None | Some(Direction::Horizontal) => progress.width,
                    Some(Direction::Vertical) => progress.height,
                });

            pixels.push(embedded_graphics::Pixel(
                embedded_graphics::geometry::Point {
                    x: origin.x + x as i32,
                    y: origin.y + y as i32,
                },
                if v < progress.threshold.unwrap_or(0) {
                    front
                } else {
                    back
                },
            ));
            if pixels.len() >= 256 {
                display.draw_iter(pixels).map_err(Into::into)?;
                pixels = Vec::with_capacity(256);
            }
        }
    }

    display.draw_iter(pixels).map_err(Into::into)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::super::tests::*;
    use super::super::Primitive;
    use super::super::*;
    use super::*;

    #[test]
    fn test_horizontal() {
        let display = render(
            embedded_graphics::prelude::Size {
                width: 32,
                height: 32,
            },
            vec![
                Primitive::Progress(Progress {
                    position: Point { x: 16, y: 3 },
                    width: 20,
                    height: 4,
                    align: Some(HorizontalAlignment::Center),
                    vertical_align: Some(VerticalAlignment::Middle),
                    direction: Some(Direction::Horizontal),
                    modulo: Some(20),
                    threshold: Some(0),
                    base: None,
                }),
                Primitive::Progress(Progress {
                    position: Point { x: 16, y: 8 },
                    width: 20,
                    height: 4,
                    align: Some(HorizontalAlignment::Center),
                    vertical_align: Some(VerticalAlignment::Middle),
                    direction: Some(Direction::Horizontal),
                    modulo: Some(20),
                    threshold: Some(5),
                    base: None,
                }),
                Primitive::Progress(Progress {
                    position: Point { x: 16, y: 13 },
                    width: 20,
                    height: 4,
                    align: Some(HorizontalAlignment::Center),
                    vertical_align: Some(VerticalAlignment::Middle),
                    direction: Some(Direction::Horizontal),
                    modulo: Some(20),
                    threshold: Some(15),
                    base: None,
                }),
                Primitive::Progress(Progress {
                    position: Point { x: 16, y: 18 },
                    width: 21,
                    height: 4,
                    align: Some(HorizontalAlignment::Center),
                    vertical_align: Some(VerticalAlignment::Middle),
                    direction: Some(Direction::Horizontal),
                    modulo: Some(7),
                    threshold: Some(4),
                    base: Some(0),
                }),
                Primitive::Progress(Progress {
                    position: Point { x: 16, y: 23 },
                    width: 21,
                    height: 4,
                    align: Some(HorizontalAlignment::Center),
                    vertical_align: Some(VerticalAlignment::Middle),
                    direction: Some(Direction::Horizontal),
                    modulo: Some(7),
                    threshold: Some(4),
                    base: Some(5),
                }),
                Primitive::Progress(Progress {
                    position: Point { x: 16, y: 28 },
                    width: 21,
                    height: 4,
                    align: Some(HorizontalAlignment::Center),
                    vertical_align: Some(VerticalAlignment::Middle),
                    direction: Some(Direction::Horizontal),
                    modulo: Some(7),
                    threshold: Some(4),
                    base: Some(3),
                }),
            ],
            None,
        );

        assert_eq!(
            String::from("\n") + &display,
            r#"
██████▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀██████
██████                    ██████
██████▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄██████
███████████               ██████
███████████               ██████
█████████████████████▀▀▀▀▀██████
█████████████████████     ██████
█████████████████████▄▄▄▄▄██████
██████████   ████   ████   █████
██████████   ████   ████   █████
██████▀▀████▀▀▀████▀▀▀████▀█████
██████  ████   ████   ████ █████
██████▄▄████▄▄▄████▄▄▄████▄█████
███████   ████   ████   ████████
███████   ████   ████   ████████
████████████████████████████████
"#
        );
    }
}
