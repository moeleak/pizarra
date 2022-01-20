use std::fmt;

use crate::point::{Vec2DWorld, Vec2DScreen};
use crate::transform::Transform;
use crate::draw_commands::DrawCommand;
use crate::style::Style;

pub mod builder;
pub mod stored;

use self::builder::free_hand_line::FreeHandLine;
use self::builder::foci_and_point_ellipse::FociAndPointEllipse;
use self::builder::polygon::PolygonBuilder;
use self::builder::center_and_radius_circle::CenterAndRadiusCircle;
use self::builder::rectangle::AxisAlignedRectangleBuilder;
use self::builder::three_point_circle::CircleThroughThreePointsBuilder;

/// Shapes handle input from the user, and they're responsible of notifying when
/// they're finished, for example in the case of a polygon
#[derive(Debug)]
pub enum ShapeFinished {
    Yes(Box<dyn ShapeStored>),
    No,
    Cancelled,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShapeType {
    Path,
    Circle,
    Ellipse,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ShapeTool {
    Path,
    Polygon,
    Rectangle,
    CircleByCenterAndPoint,
    CircleThroughThreePoints,
    ThreePointEllipse,
}

impl ShapeTool {
    pub fn start(&self, initial: Vec2DWorld, style: Style) -> Box<dyn ShapeBuilder> {
        match *self {
            ShapeTool::Path => Box::new(FreeHandLine::start(initial, style)),
            ShapeTool::Rectangle => Box::new(AxisAlignedRectangleBuilder::start(initial, style)),
            ShapeTool::Polygon => Box::new(PolygonBuilder::start(initial, style)),
            ShapeTool::CircleByCenterAndPoint => Box::new(CenterAndRadiusCircle::start(initial, style)),
            ShapeTool::CircleThroughThreePoints => Box::new(CircleThroughThreePointsBuilder::start(initial, style)),
            ShapeTool::ThreePointEllipse => Box::new(FociAndPointEllipse::start(initial, style)),
        }
    }
}

pub trait ShapeBuilder: std::fmt::Debug {
    /// Must handle new coordinates given to this shape. If this method is
    /// called it means that the shape is being modified (thus this is the most
    /// recently added shape
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, transform: Transform, snap: f64);

    fn handle_button_pressed(&mut self, pos: Vec2DScreen, transform: Transform, snap: f64);

    fn handle_button_released(&mut self, pos: Vec2DScreen, transform: Transform, snap: f64) -> ShapeFinished;

    /// Must return the necessary commands to display this shape on the screen
    /// when building it
    fn draw_commands(&self, transform: Transform, snap: f64) -> Vec<DrawCommand>;
}

pub trait ShapeStored: fmt::Debug {
    /// return the commands needed to draw this shape
    fn draw_commands(&self) -> DrawCommand;

    /// Must know its bbox
    fn bbox(&self) -> [Vec2DWorld; 2];

    /// Returns the current shape type
    fn shape_type(&self) -> ShapeType;

    /// does this circle intersect this shape? Used by the eraser
    fn intersects_circle(&self, center: Vec2DWorld, radius: f64) -> bool;

    /// returns this shape's style
    fn style(&self) -> Style;
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

    pub fn style(&self) -> Style {
        self.shape_impl.style()
    }

    pub fn bbox(&self) -> [Vec2DWorld; 2] {
        self.shape_impl.bbox()
    }

    pub fn intersects_circle(&self, center: Vec2DWorld, radius: f64) -> bool {
        self.shape_impl.intersects_circle(center, radius)
    }
}

impl fmt::Debug for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Shape({:?}, {}, {:?})", self.shape_impl.shape_type(), self.id, self.style())
    }
}
