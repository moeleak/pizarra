use std::borrow::Borrow;
use std::fmt;

use xml::writer::EventWriter;

use crate::draw_commands::DrawCommand;
use crate::point::{ScreenUnit, Vec2D, WorldUnit};
use crate::style::Style;
use crate::transform::Transform;

pub mod builder;
pub mod stored;

use self::builder::center_and_radius_circle::CenterAndRadiusCircle;
use self::builder::foci_and_point_ellipse::FociAndPointEllipse;
use self::builder::free_grid::FreeGrid;
use self::builder::free_hand_line::FreeHandLine;
use self::builder::grid::Grid;
use self::builder::polygon::PolygonBuilder;
use self::builder::rectangle::AxisAlignedRectangleBuilder;
use self::builder::three_point_circle::CircleThroughThreePointsBuilder;

/// Shapes handle input from the user, and they're responsible of notifying when
/// they're finished, for example in the case of a polygon
#[derive(Debug)]
pub enum ShapeFinished {
    Yes(Vec<Box<dyn ShapeStored>>),
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
    Grid,
    FreeGrid,
}

impl ShapeTool {
    pub fn start(
        &self,
        initial: Vec2D<WorldUnit>,
        style: Style<WorldUnit>,
    ) -> Box<dyn ShapeBuilder> {
        self.start_with_pressure(initial, style, 1.0)
    }

    /// Starts a shape with the pressure of the input device that began it.
    ///
    /// Only free-hand paths vary their width along the stroke; the other tools
    /// keep the configured width.
    pub fn start_with_pressure(
        &self,
        initial: Vec2D<WorldUnit>,
        style: Style<WorldUnit>,
        pressure: f64,
    ) -> Box<dyn ShapeBuilder> {
        match *self {
            ShapeTool::Path => Box::new(FreeHandLine::start(initial, style, pressure)),
            ShapeTool::Rectangle => Box::new(AxisAlignedRectangleBuilder::start(initial, style)),
            ShapeTool::Polygon => Box::new(PolygonBuilder::start(initial, style)),
            ShapeTool::CircleByCenterAndPoint => {
                Box::new(CenterAndRadiusCircle::start(initial, style))
            }
            ShapeTool::CircleThroughThreePoints => {
                Box::new(CircleThroughThreePointsBuilder::start(initial, style))
            }
            ShapeTool::ThreePointEllipse => Box::new(FociAndPointEllipse::start(initial, style)),
            ShapeTool::Grid => Box::new(Grid::start(initial, style)),
            ShapeTool::FreeGrid => Box::new(FreeGrid::start(initial, style)),
        }
    }
}

pub trait ShapeBuilder: fmt::Debug {
    /// Must handle new coordinates given to this shape. If this method is
    /// called it means that the shape is being modified (thus this is the most
    /// recently added shape
    fn handle_mouse_moved(
        &mut self,
        pos: Vec2D<ScreenUnit>,
        transform: Transform,
        snap: ScreenUnit,
    );

    fn handle_button_pressed(
        &mut self,
        pos: Vec2D<ScreenUnit>,
        transform: Transform,
        snap: ScreenUnit,
    );

    fn handle_button_released(
        &mut self,
        pos: Vec2D<ScreenUnit>,
        transform: Transform,
        snap: ScreenUnit,
    ) -> ShapeFinished;

    /// Variants that also receive the pressure of a pen (0.0 to 1.0).
    ///
    /// Shapes that do not care about pressure inherit the default
    /// implementations.
    fn handle_mouse_moved_with_pressure(
        &mut self,
        pos: Vec2D<ScreenUnit>,
        _pressure: f64,
        transform: Transform,
        snap: ScreenUnit,
    ) {
        self.handle_mouse_moved(pos, transform, snap);
    }

    fn handle_button_pressed_with_pressure(
        &mut self,
        pos: Vec2D<ScreenUnit>,
        _pressure: f64,
        transform: Transform,
        snap: ScreenUnit,
    ) {
        self.handle_button_pressed(pos, transform, snap);
    }

    fn handle_button_released_with_pressure(
        &mut self,
        pos: Vec2D<ScreenUnit>,
        _pressure: f64,
        transform: Transform,
        snap: ScreenUnit,
    ) -> ShapeFinished {
        self.handle_button_released(pos, transform, snap)
    }

    /// Must return the necessary commands to display this shape on the screen
    /// when building it
    fn draw_commands(&self, transform: Transform, snap: ScreenUnit) -> Vec<DrawCommand>;
}

pub trait ShapeStored: fmt::Debug {
    /// return the commands needed to draw this shape. Most shapes need one;
    /// a pressure stroke needs one per segment.
    fn draw_commands(&self) -> Vec<DrawCommand>;

    /// Must know its bbox
    fn bbox(&self) -> [Vec2D<WorldUnit>; 2];

    /// Returns the current shape type
    fn shape_type(&self) -> ShapeType;

    /// does this circle intersect this shape? Used by the eraser
    fn intersects_circle(&self, center: Vec2D<WorldUnit>, radius: WorldUnit) -> bool;

    /// returns this shape's style
    fn style(&self) -> Style<WorldUnit>;

    /// Serialize the shape to svg
    fn serialize(&self, writer: &mut EventWriter<&mut Vec<u8>>);
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

    pub fn draw_commands(&self) -> Vec<DrawCommand> {
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

    pub fn style(&self) -> Style<WorldUnit> {
        self.shape_impl.style()
    }

    pub fn bbox(&self) -> [Vec2D<WorldUnit>; 2] {
        self.shape_impl.bbox()
    }

    pub fn intersects_circle(&self, center: Vec2D<WorldUnit>, radius: WorldUnit) -> bool {
        self.shape_impl.intersects_circle(center, radius)
    }

    pub fn shape_impl(&self) -> &dyn ShapeStored {
        self.shape_impl.borrow()
    }
}

impl fmt::Debug for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Shape({:?}, {}, {:?})",
            self.shape_impl.shape_type(),
            self.id,
            self.style()
        )
    }
}
