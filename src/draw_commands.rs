use std::rc::Rc;

use crate::point::Point;
use crate::color::Color;

/// The draw command as returned by the shape: adds the color
#[derive(Debug, PartialEq, Clone)]
pub enum DrawCommand {
    Line {
        color: Color,
        line: Rc<Vec<Point>>,
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
