use crate::point::Point;
use crate::color::Color;

/// The draw command as returned by the shape: adds the color 
#[derive(Debug, PartialEq)]
pub enum DrawCommand {
    Line {
        color: Color,
        line: Vec<Point>,
        thickness: f64,
    },
    Rectangle {
        color: Color,
        corner_1: Point,
        corner_2: Point,
    },
    Ellipse {
        color: Color,
        corner_1: Point,
        corner_2: Point,
    },
}
