use graphics::math::Vec2d;
use rstar::{RTreeObject, AABB};

pub mod line;
pub mod rectangle;
pub mod circle;

use super::color::Color;
use crate::draw_commands::{WireDrawCommand, ColoredDrawCommand};

pub use self::line::Line;
pub use self::rectangle::Rectangle;
pub use self::circle::Circle;

pub enum Tool {
    Line,
    Rectangle,
    Circle,
}

impl Tool {
    pub fn make(&self, color: Color) -> Shape {
        match *self {
            Tool::Line => Line::new(color),
            Tool::Rectangle => Rectangle::new(color),
            Tool::Circle => Circle::new(color),
        }
    }
}


pub trait ShapeTrait {
    /// Must handle new coordinates given to this shape. If this method is
    /// called it means that the shape is being modified (thus this is the most
    /// recently added shape
    fn handle(&mut self, val: Vec2d);

    /// Must return the necessary commands to display this shape on the screen
    fn draw_commands(&self) -> Vec<WireDrawCommand>;

    /// Must know its bbox
    fn bbox(&self) -> [[f64; 2]; 2];
}

#[derive(Debug, PartialEq)]
pub struct ShapeId(usize);

impl From<usize> for ShapeId {
    fn from(data: usize) -> ShapeId {
        ShapeId(data)
    }
}

pub struct Shape {
    color: Color,
    shape_impl: Box<dyn ShapeTrait>,
}

impl Shape {
    pub fn handle(&mut self, val: Vec2d) {
        unimplemented!()
    }

    fn draw_commands(&self) -> Vec<ColoredDrawCommand> {
        unimplemented!()
    }
}

impl RTreeObject for Shape {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_points(self.shape_impl.bbox().iter())
    }
}
