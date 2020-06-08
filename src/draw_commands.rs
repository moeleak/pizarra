use crate::point::Point;
use crate::color::Color;

/// The most primitive draw command: the one returned by the shape impl
pub enum WireDrawCommand {
    Line {
        line: Vec<Point>,
    },
    Rectangle {
        corner_1: Point,
        corner_2: Point,
    },
    Ellipse {
        corner_1: Point,
        corner_2: Point,
    },
}

/// The draw command as returned by the shape: adds the color 
pub enum ColoredDrawCommand {
    Line {
        color: Color,
        line: Vec<Point>,
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

/// A Full Draw command returned by the storage
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

impl DrawCommand {
    pub fn color(&self) -> Color {
        match self {
            DrawCommand::Line { color, .. } => *color,
            DrawCommand::Rectangle { color, .. } => *color,
            DrawCommand::Ellipse { color, .. } => *color,
        }
    }
}
