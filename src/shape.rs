use std::fmt;

use crate::point::Point;
use rstar::{RTreeObject, AABB};

pub mod line;
pub mod rectangle;
pub mod ellipse;

use super::color::Color;
use crate::draw_commands::{WireDrawCommand, DrawCommand};

pub use self::line::Line;
pub use self::rectangle::Rectangle;
pub use self::ellipse::Ellipse;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShapeType {
    Line,
    Rectangle,
    Ellipse,
}

impl ShapeType {
    pub fn make(&self, color: Color, initial: [Point; 2], id: ShapeId) -> Shape {
        match *self {
            ShapeType::Line => Line::new(color, initial, id),
            ShapeType::Rectangle => Rectangle::new(color, initial, id),
            ShapeType::Ellipse => Ellipse::new(color, initial, id),
        }
    }
}

pub trait ShapeTrait {
    /// Must handle new coordinates given to this shape. If this method is
    /// called it means that the shape is being modified (thus this is the most
    /// recently added shape
    fn handle(&mut self, val: Point);

    /// Must return the necessary commands to display this shape on the screen
    fn draw_commands(&self) -> WireDrawCommand;

    /// Must know its bbox
    fn bbox(&self) -> [[f64; 2]; 2];

    /// Returns the current shape type
    fn shape_type(&self) -> ShapeType;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ShapeId(usize);

impl ShapeId {
    pub fn incr(&mut self) {
        self.0 += 1;
    }
}

impl From<usize> for ShapeId {
    fn from(data: usize) -> ShapeId {
        ShapeId(data)
    }
}

impl fmt::Display for ShapeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "shape{}", self.0)
    }
}

pub struct Shape {
    color: Color,
    id: ShapeId,
    shape_impl: Box<dyn ShapeTrait>,
}

impl Shape {
    pub fn handle(&mut self, val: Point) {
        self.shape_impl.handle(val);
    }

    pub fn draw_commands(&self) -> DrawCommand {
        let cmd = self.shape_impl.draw_commands();

        match cmd {
            WireDrawCommand::Ellipse{ corner_1, corner_2 } => DrawCommand::Ellipse{ corner_1, corner_2, color: self.color },
            WireDrawCommand::Rectangle{ corner_1, corner_2 } => DrawCommand::Rectangle{ corner_1, corner_2, color: self.color },
            WireDrawCommand::Line{ line, thickness } => DrawCommand::Line{ line, color: self.color, thickness },
        }
    }

    pub fn shape_type(&self) -> ShapeType {
        self.shape_impl.shape_type()
    }

    pub fn id(&self) -> ShapeId {
        self.id
    }

    pub fn color(&self) -> Color {
        self.color
    }
}

impl RTreeObject for Shape {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let [p1, p2] = self.shape_impl.bbox();

        AABB::from_corners(p1, p2)
    }
}

pub fn corners_to_props(_corner_1: Point, _corner_2: Point) -> (f64, f64, f64, f64) {
    unimplemented!()
}
