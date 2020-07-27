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
    Circle {
        center: Point,
        radius: f64,
        color: Color,
        thickness: f64,
    },
    Ellipse {
        bbox: [Point; 2],
        thickness: f64,
        color: Color,
    },
}
