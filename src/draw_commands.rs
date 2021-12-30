use crate::point::{Vec2DWorld, Vec2DScreen};
use crate::color::Color;
use crate::path_command::PathCommand;
use crate::geom::Angle;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Ellipse {
    pub center: Vec2DWorld,
    pub semimajor: f64,
    pub semiminor: f64,
    pub angle: Angle,
    pub thickness: f64,
    pub color: Option<Color>,
    pub fill: Option<Color>,
}

/// This enum is returned by the shapes (both stored and the one being drawn).
/// It is the interface between the backend (libpizarra) and the frontend
/// (pizarra), it is the responsibility of the latter to render those commands
/// on the screen.
#[derive(Debug, PartialEq, Clone)]
pub enum DrawCommand {
    /// A string of points and curves without filling
    Path {
        color: Option<Color>,
        fill: Option<Color>,
        commands: Vec<PathCommand<Vec2DWorld>>,
        thickness: f64,
    },

    /// A circle without filling
    Circle {
        center: Vec2DWorld,
        radius: f64,
        color: Option<Color>,
        fill: Option<Color>,
        thickness: f64,
    },

    /// An ellipse without filling
    Ellipse(Ellipse),

    /// A path in screen coordinates without filling. Used to render handles and
    /// other visual aids
    ScreenPath {
        color: Option<Color>,
        fill: Option<Color>,
        commands: Vec<PathCommand<Vec2DScreen>>,
        thickness: f64,
    },

    /// A circle in screen coordinates without filling. Used to render handles
    /// and visual aids
    ScreenCircle {
        center: Vec2DScreen,
        radius: f64,
        color: Option<Color>,
        fill: Option<Color>,
        thickness: f64,
    },
}

impl DrawCommand {
    pub fn color(&self) -> Option<Color> {
        match self {
            DrawCommand::Path { color, .. } => *color,
            DrawCommand::Circle { color, .. } => *color,
            DrawCommand::Ellipse(e) => e.color,
            DrawCommand::ScreenPath { color, .. } => *color,
            DrawCommand::ScreenCircle { color, .. } => *color,
        }
    }

    pub fn fill(&self) -> Option<Color> {
        match self {
            DrawCommand::Path { fill, .. } => *fill,
            DrawCommand::Circle { fill, .. } => *fill,
            DrawCommand::Ellipse(e) => e.fill,
            DrawCommand::ScreenPath { fill, .. } => *fill,
            DrawCommand::ScreenCircle { fill, .. } => *fill,
        }
    }
}
