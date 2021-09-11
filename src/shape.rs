use std::fmt;
use std::str::FromStr;

use crate::point::{Vec2DDrawing, Vec2DScreen};
use crate::transform::Transform;

pub mod path;
mod rectangle;
mod polygon;
mod circle;
mod ellipse;

use super::color::Color;
pub use crate::draw_commands::DrawCommand;

pub use self::path::Path;
pub use self::rectangle::Rectangle;
pub use self::polygon::Polygon;
pub use self::circle::Circle;
pub use self::ellipse::Ellipse;

/// Shapes handle input from the user, and they're responsible of notifying when
/// they're finished, for example in the case of a polygon
#[derive(Debug, PartialEq, Eq)]
pub enum ShapeFinished {
    Yes,
    No,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShapeType {
    Path,
    Polygon,
    Rectangle,
    Circle,
    Ellipse,
}

impl ShapeType {
    pub fn start(&self, color: Color, initial: Vec2DDrawing, stroke: f64) -> Box<dyn ShapeTrait> {
        match *self {
            ShapeType::Path => Box::new(Path::new(color, initial, stroke)),
            ShapeType::Rectangle => Box::new(Rectangle::new(color, initial, stroke)),
            ShapeType::Polygon => Box::new(Polygon::new(color, initial, stroke)),
            ShapeType::Circle => Box::new(Circle::new(color, initial, stroke)),
            ShapeType::Ellipse => Box::new(Ellipse::new(color, initial, stroke)),
        }
    }
}

impl FromStr for ShapeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "path" => Ok(ShapeType::Path),
            "polygon" => Ok(ShapeType::Polygon),
            "rectangle" => Ok(ShapeType::Rectangle),
            "circle" => Ok(ShapeType::Circle),
            "ellipse" => Ok(ShapeType::Ellipse),
            _ => Err(()),
        }
    }
}

pub trait ShapeTrait {
    /// Must handle new coordinates given to this shape. If this method is
    /// called it means that the shape is being modified (thus this is the most
    /// recently added shape
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, transform: Transform);

    fn handle_button_pressed(&mut self, pos: Vec2DScreen, transform: Transform);

    fn handle_button_released(&mut self, pos: Vec2DScreen, transform: Transform) -> ShapeFinished;

    /// Must return the necessary commands to display this shape on the screen
    fn draw_commands(&self) -> DrawCommand;

    /// Must know its bbox
    fn bbox(&self) -> [Vec2DDrawing; 2];

    /// Returns the current shape type
    fn shape_type(&self) -> ShapeType;

    /// does this circle intersect this shape? Used by the eraser
    fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool;

    /// returns the color of this shape
    fn color(&self) -> Color;
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
    id: ShapeId,
    z_index: usize,
    shape_impl: Box<dyn ShapeTrait>,
}

impl Shape {
    pub fn from_shape(shape: Box<dyn ShapeTrait>, id: ShapeId, z_index: usize) -> Shape {
        Shape {
            id,
            shape_impl: shape,
            z_index,
        }
    }

    pub fn draw_commands(&self) -> DrawCommand {
        self.shape_impl.draw_commands()
    }

    pub fn shape_type(&self) -> ShapeType {
        self.shape_impl.shape_type()
    }

    pub fn id(&self) -> ShapeId {
        self.id
    }

    pub fn z_index(&self) -> usize {
        self.z_index
    }

    pub fn color(&self) -> Color {
        self.shape_impl.color()
    }

    pub fn bbox(&self) -> [Vec2DDrawing; 2] {
        self.shape_impl.bbox()
    }

    pub fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool {
        self.shape_impl.intersects_circle(center, radius)
    }

    pub fn handle_mouse_moved(&mut self, val: Vec2DScreen, transform: Transform) {
        self.shape_impl.handle_mouse_moved(val, transform);
    }

    pub fn handle_button_pressed(&mut self, point: Vec2DScreen, transform: Transform) {
        self.shape_impl.handle_button_pressed(point, transform);
    }

    pub fn handle_button_released(&mut self, point: Vec2DScreen, transform: Transform) -> ShapeFinished {
        self.shape_impl.handle_button_released(point, transform)
    }
}

impl fmt::Debug for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Shape({:?}, {}, {})", self.shape_impl.shape_type(), self.id, self.color())
    }
}
