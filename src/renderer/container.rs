use crate::error::DrawingError;

use super::{drawing_error::IntoDrawingError, ColorFromTemplate, Point, Primitive, Size};
use embedded_graphics::{
    prelude::{Dimensions, DrawTarget, PixelColor},
    primitives::Rectangle,
    Pixel,
};
use serde::{Deserialize, Serialize};

#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Container {
    pub position: Point,
    pub size: Size,
    pub rotate: Option<i32>,

    pub content: Option<Vec<Primitive>>,
}

pub trait ShiftableDisplay<D, TargetColor, ErrorType>
where
    D: DrawTarget<Color = TargetColor, Error = ErrorType>,
    TargetColor: PixelColor + ColorFromTemplate,
    ErrorType: Into<DrawingError>,
{
    fn shift<'a>(
        &'a mut self,
        bounds: Rectangle,
        rotate: i32,
    ) -> ShiftedDisplay<'a, D, TargetColor, ErrorType>;
}

pub struct ShiftedDisplay<'a, D, TargetColor, ErrorType>
where
    D: DrawTarget<Color = TargetColor, Error = ErrorType>,
    TargetColor: PixelColor + ColorFromTemplate,
    ErrorType: Into<DrawingError>,
{
    display: &'a mut D,
    inner_bounds: Rectangle,
    outer_bounds: Rectangle,
    rotate: i32,
}

impl<'b, D, TargetColor, ErrorType> ShiftableDisplay<D, TargetColor, ErrorType>
    for ShiftedDisplay<'b, D, TargetColor, ErrorType>
where
    D: DrawTarget<Color = TargetColor, Error = ErrorType>,
    TargetColor: PixelColor + ColorFromTemplate,
    ErrorType: Into<DrawingError>,
{
    fn shift<'a>(
        &'a mut self,
        bounds: Rectangle,
        rotate: i32,
    ) -> ShiftedDisplay<'a, D, TargetColor, ErrorType> {
        let inner_bounds = Rectangle {
            top_left: embedded_graphics::prelude::Point { x: 0, y: 0 },
            size: match rotate % 2 {
                0 => bounds.size.clone().into(),
                _ => embedded_graphics::prelude::Size {
                    width: bounds.size.height,
                    height: bounds.size.width,
                },
            },
        };
        let outer_bounds = Rectangle {
            top_left: embedded_graphics::prelude::Point {
                x: self.outer_bounds.top_left.x + bounds.top_left.x,
                y: self.outer_bounds.top_left.y + bounds.top_left.y,
            },
            size: bounds.size.clone().into(),
        };

        ShiftedDisplay {
            display: self.display,
            inner_bounds,
            outer_bounds,
            rotate: (self.rotate + rotate) % 4,
        }
    }
}

impl<'b, D, TargetColor, ErrorType> ShiftedDisplay<'b, D, TargetColor, ErrorType>
where
    D: DrawTarget<Color = TargetColor, Error = ErrorType>,
    TargetColor: PixelColor + ColorFromTemplate,
    ErrorType: Into<DrawingError>,
{
    pub fn from(display: &'b mut D) -> ShiftedDisplay<'b, D, TargetColor, ErrorType> {
        let outer_bounds = display.bounding_box();
        let inner_bounds = Rectangle {
            top_left: embedded_graphics::prelude::Point { x: 0, y: 0 },
            size: outer_bounds.size.clone().into(),
        };

        ShiftedDisplay {
            display,
            inner_bounds,
            outer_bounds,
            rotate: 0,
        }
    }
}
/*{
    fn shift<'a>(&'a mut self, bounds: Rectangle) -> ShiftedDisplay<'a, D, TargetColor, ErrorType> {
        let inner_bounds = Rectangle {
            top_left: embedded_graphics::prelude::Point { x: 0, y: 0 },
            size: bounds.size.clone().into(),
        };
        let outer_bounds = Rectangle {
            top_left: embedded_graphics::prelude::Point { x: 0, y: 0 },
            size: bounds.size.clone().into(),
        };

        ShiftedDisplay::<'a, _, _, _> {
            display: self,
            inner_bounds,
            outer_bounds,
        }
    }
}*/

impl<'a, TargetColor, ErrorType, D> Dimensions for ShiftedDisplay<'_, D, TargetColor, ErrorType>
where
    D: DrawTarget<Color = TargetColor, Error = ErrorType>,
    TargetColor: PixelColor + ColorFromTemplate,
    ErrorType: Into<DrawingError>,
{
    fn bounding_box(&self) -> Rectangle {
        self.inner_bounds.clone()
    }
}

impl<'a, TargetColor, ErrorType, D> DrawTarget for ShiftedDisplay<'a, D, TargetColor, ErrorType>
where
    D: DrawTarget<Color = TargetColor, Error = ErrorType>,
    TargetColor: PixelColor + ColorFromTemplate,
    ErrorType: Into<DrawingError>,
{
    type Color = TargetColor;
    type Error = ErrorType;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.display
            .draw_iter(pixels.into_iter().filter_map(|pixel| {
                if pixel.0.x < 0 || pixel.0.y < 0 {
                    return None;
                }
                if pixel.0.x >= self.inner_bounds.size.width as i32
                    || pixel.0.y >= self.inner_bounds.size.height as i32
                {
                    return None;
                }
                match self.rotate {
                    0 => Some(Pixel(
                        embedded_graphics::prelude::Point {
                            x: pixel.0.x + self.outer_bounds.top_left.x,
                            y: pixel.0.y + self.outer_bounds.top_left.y,
                        },
                        pixel.1,
                    )),
                    1 => Some(Pixel(
                        embedded_graphics::prelude::Point {
                            x: pixel.0.y + self.outer_bounds.top_left.x,
                            y: self.inner_bounds.size.width as i32 - pixel.0.x
                                + self.outer_bounds.top_left.y,
                        },
                        pixel.1,
                    )),
                    2 => Some(Pixel(
                        embedded_graphics::prelude::Point {
                            x: self.inner_bounds.size.width as i32 - pixel.0.x
                                + self.outer_bounds.top_left.x,
                            y: self.inner_bounds.size.height as i32 - pixel.0.y
                                + self.outer_bounds.top_left.y,
                        },
                        pixel.1,
                    )),
                    3 => Some(Pixel(
                        embedded_graphics::prelude::Point {
                            x: self.inner_bounds.size.height as i32 - pixel.0.y
                                + self.outer_bounds.top_left.x,
                            y: pixel.0.x + self.outer_bounds.top_left.y,
                        },
                        pixel.1,
                    )),
                    _ => None,
                }
            }))?;

        Ok(())
    }
}

pub fn draw_container<D, TargetColor>(
    display: &mut ShiftedDisplay<D, TargetColor, D::Error>,
    container: &Container,
) -> Result<(), DrawingError>
where
    D: DrawTarget<Color = TargetColor, Error: IntoDrawingError>,
    TargetColor: PixelColor + ColorFromTemplate,
{
    let outer_bounds = Rectangle {
        top_left: container.position.clone().into(),
        size: container.size.clone().into(),
    };

    let mut shifted_display = display.shift(outer_bounds.clone(), container.rotate.unwrap_or(0));
    super::draw(
        &mut shifted_display,
        &container.content.clone().unwrap_or_default(),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use embedded_graphics::text::Alignment;

    use super::super::tests::*;
    use super::super::Primitive;
    use super::super::*;
    use super::*;

    #[test]
    fn test_yaml_parsing() {
        let example = Primitive::Container(Container {
            size: Size {
                width: 60,
                height: 28,
            },
            position: Point { x: 2, y: 2 },
            rotate: Some(1),
            content: Some(vec![Primitive::Text(TextItem {
                value: format!("Hi{}!", 1).to_string(),
                position: Point { x: 0, y: 0 },
                font: Some("4x6".to_string()),
                color: Some("0".to_string()),
                align: Some(Alignment::Left),
            })]),
        });

        let yaml = serde_yaml::to_string(&example).unwrap();

        assert_eq!(
            yaml,
            r#"
!container
position:
  x: 2
  y: 2
size:
  width: 60
  height: 28
rotate: 1
content:
- !text
  value: Hi1!
  position:
    x: 0
    y: 0
  font: 4x6
  color: '0'
  align: left
"#
        );
    }

    #[test]
    fn test_container() {
        let display = render(
            embedded_graphics::prelude::Size {
                width: 64,
                height: 32,
            },
            (0..4)
                .map(|r| {
                    Primitive::Container(Container {
                        size: Size {
                            width: 60,
                            height: 28,
                        },
                        position: Point { x: 2, y: 2 },
                        rotate: Some(r),
                        content: Some(vec![Primitive::Text(TextItem {
                            value: format!("Hi{}!", r).to_string(),
                            position: Point { x: 0, y: 0 },
                            font: Some("4x6".to_string()),
                            color: Some("0".to_string()),
                            align: Some(Alignment::Left),
                        })]),
                    })
                })
                .collect(),
            None,
        );

        assert_eq!(
            String::from("\n") + &display,
            r#"
████████████████████████████████████████████████████████████████
██ █ ██▄██▀▄▀██ ██████████████████████████████████████████▄▄ ▄▄█
██ ▄ █▄ ██ ▄ ██▄██████████████████████████████████████████▄▄▄▄▄█
██▄█▄█▄▄▄██▄███▄██████████████████████████████████████████ ▀ █▀█
██████████████████████████████████████████████████████████▄█████
██████████████████████████████████████████████████████████ █▀█ █
███████████████████████████████████████████████████████████▄█▄▄█
██████████████████████████████████████████████████████████▀█▀▀▀█
██▀▀▀█▀█████████████████████████████████████████████████████████
████████████████████████████████████████████████████████████████
██▀▀▀▀ █████████████████████████████████████████████████████████
███▄██▄█████████████████████████████████████████████████████████
██▀█▀▀ █████████████████████████████████████████████████████████
████▄█▄██████████████████████████████████████████▄██▄ ▄█▄ ▄█ █ █
██▄▄ ▄▄██████████████████████████████████████████ ██ █▀██▄▄█ ▄ █
██▄▄▄▄▄██████████████████████████████████████████▄███▄███▄██▄█▄█
"#
        );
    }
}
