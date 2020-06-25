use std::fmt;

use crate::point::Point;

pub mod line;
pub mod rectangle;
pub mod ellipse;

use super::color::Color;
use crate::draw_commands::{WireDrawCommand, DrawCommand};

pub use self::line::Line;
pub use self::rectangle::Rectangle;
pub use self::ellipse::Ellipse;

/// Shapes handle input from the user, and they're responsible of notifying when
/// they're finished, for example in the case of a polygon
pub enum ShapeFinished {
    Yes,
    No,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShapeType {
    Line,
    Rectangle,
    Ellipse,
}

impl ShapeType {
    pub fn start(&self, color: Color, initial: Point, id: ShapeId, stroke: f64) -> Shape {
        match *self {
            ShapeType::Line => Line::new(color, initial, id, stroke),
            ShapeType::Rectangle => Rectangle::new(color, initial, id),
            ShapeType::Ellipse => Ellipse::new(color, initial, id),
        }
    }
}

pub trait ShapeTrait {
    /// Must handle new coordinates given to this shape. If this method is
    /// called it means that the shape is being modified (thus this is the most
    /// recently added shape
    fn handle_mouse_moved(&mut self, pos: Point);

    fn handle_button_pressed(&mut self, pos: Point);

    fn handle_button_released(&mut self, pos: Point) -> ShapeFinished;

    /// Must return the necessary commands to display this shape on the screen
    fn draw_commands(&self) -> WireDrawCommand;

    /// Must know its bbox
    fn bbox(&self) -> [[f64; 2]; 2];

    /// Returns the current shape type
    fn shape_type(&self) -> ShapeType;

    /// does this circle intersect this shape? Used by the eraser
    fn intersects_circle(&self, center: Point, radius: f64) -> bool;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ShapeId(usize);

impl ShapeId {
    pub fn next(self) -> ShapeId {
        ShapeId(self.0 + 1)
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

    pub fn bbox(&self) -> [[f64; 2]; 2] {
        self.shape_impl.bbox()
    }

    pub fn intersects_circle(&self, center: Point, radius: f64) -> bool {
        self.shape_impl.intersects_circle(center, radius)
    }

    pub fn handle_mouse_moved(&mut self, val: Point) {
        self.shape_impl.handle_mouse_moved(val);
    }

    pub fn handle_button_pressed(&mut self, point: Point) {
        self.shape_impl.handle_button_pressed(point);
    }

    pub fn handle_button_released(&mut self, point: Point) -> ShapeFinished {
        self.shape_impl.handle_button_released(point)
    }
}

pub fn corners_to_props(_corner_1: Point, _corner_2: Point) -> (f64, f64, f64, f64) {
    unimplemented!()
}
