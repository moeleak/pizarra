use std::fmt;
use std::str::FromStr;

use crate::point::{Vec2DDrawing, Vec2DScreen};
use crate::transform::Transform;
use crate::draw_commands::DrawCommand;

pub mod path;
pub mod rectangle;
pub mod polygon;
pub mod circle;
pub mod ellipse;

use super::color::Color;

use self::path::PathBuilder;
use self::ellipse::ThreePointEllipseBuilder;
use self::polygon::PolygonBuilder;
use self::circle::CircleBuilder;
use self::rectangle::AxisAlignedRectangleBuilder;

/// Shapes handle input from the user, and they're responsible of notifying when
/// they're finished, for example in the case of a polygon
#[derive(Debug)]
pub enum ShapeFinished {
    Yes(Box<dyn ShapeStored>),
    No,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShapeType {
    Path,
    Polygon,
    Rectangle,
    Circle,
    ThreePointEllipse,
}

impl ShapeType {
    pub fn start(&self, color: Color, initial: Vec2DDrawing, stroke: f64) -> Box<dyn ShapeBuilder> {
        match *self {
            ShapeType::Path => Box::new(PathBuilder::start(color, initial, stroke)),
            ShapeType::Rectangle => Box::new(AxisAlignedRectangleBuilder::start(color, initial, stroke)),
            ShapeType::Polygon => Box::new(PolygonBuilder::start(color, initial, stroke)),
            ShapeType::Circle => Box::new(CircleBuilder::start(color, initial, stroke)),
            ShapeType::ThreePointEllipse => Box::new(ThreePointEllipseBuilder::start(color, initial, stroke)),
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
            "ellipse" => Ok(ShapeType::ThreePointEllipse),
            _ => Err(()),
        }
    }
}

pub trait ShapeBuilder {
    /// Must handle new coordinates given to this shape. If this method is
    /// called it means that the shape is being modified (thus this is the most
    /// recently added shape
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, transform: Transform);

    fn handle_button_pressed(&mut self, pos: Vec2DScreen, transform: Transform);

    fn handle_button_released(&mut self, pos: Vec2DScreen, transform: Transform) -> ShapeFinished;

    /// Must return the necessary commands to display this shape on the screen
    /// when building it
    fn draw_commands(&self) -> Vec<DrawCommand>;
}

pub trait ShapeStored: fmt::Debug {
    /// return the commands needed to draw this shape
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
    shape_impl: Box<dyn ShapeStored>,
}

impl Shape {
    pub fn from_shape(shape: Box<dyn ShapeStored>, id: ShapeId, z_index: usize) -> Shape {
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
}

impl fmt::Debug for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Shape({:?}, {}, {})", self.shape_impl.shape_type(), self.id, self.color())
    }
}
