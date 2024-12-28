use embedded_graphics::prelude::Size;
use serde::{Deserialize, Serialize};

use super::Point;

#[derive(Debug, Copy, Clone, Ord, Serialize, Deserialize, PartialOrd, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum VerticalAlignment {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Copy, Clone, Ord, Serialize, Deserialize, PartialOrd, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

pub fn place_rectangle(
    s: Size,
    halign: Option<HorizontalAlignment>,
    valign: Option<VerticalAlignment>,
    pos: Point,
) -> Point {
    Point {
        x: match halign {
            None | Some(HorizontalAlignment::Left) => pos.x,
            Some(HorizontalAlignment::Center) => pos.x - (s.width / 2) as i32,
            Some(HorizontalAlignment::Right) => pos.x - s.width as i32,
        },
        y: match valign {
            None | Some(VerticalAlignment::Top) => pos.y,
            Some(VerticalAlignment::Middle) => pos.y - (s.height / 2) as i32,
            Some(VerticalAlignment::Bottom) => pos.y - s.height as i32,
        },
    }
}
