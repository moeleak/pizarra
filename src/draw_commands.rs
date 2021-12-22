use crate::point::{Vec2DWorld, Vec2DScreen};
use crate::color::Color;
use crate::path_command::PathCommand;
use crate::geom::Angle;

/// This enum is returned by the shapes (both stored and the one being drawn).
/// It is the interface between the backend (libpizarra) and the frontend
/// (pizarra), it is the responsibility of the latter to render those commands
/// on the screen.
#[derive(Debug, PartialEq, Clone)]
pub enum DrawCommand {
    /// A string of points and curves without filling
    Path {
        color: Color,
        commands: Vec<PathCommand<Vec2DWorld>>,
        thickness: f64,
    },

    /// A circle without filling
    Circle {
        center: Vec2DWorld,
        radius: f64,
        color: Color,
        thickness: f64,
    },

    /// An ellipse without filling
    Ellipse {
        center: Vec2DWorld,
        semimajor: f64,
        semiminor: f64,
        angle: Angle,
        thickness: f64,
        color: Color,
    },

    /// A path in screen coordinates without filling. Used to render handles and
    /// other visual aids
    ScreenPath {
        color: Color,
        commands: Vec<PathCommand<Vec2DWorld>>,
        thickness: f64,
    },

    /// A circle in screen coordinates without filling. Used to render handles
    /// and visual aids
    ScreenCircle {
        center: Vec2DScreen,
        radius: f64,
        color: Color,
        thickness: f64,
    },
}

impl DrawCommand {
    pub fn color(&self) -> Color {
        match self {
            DrawCommand::Path { color, .. } => *color,
            DrawCommand::Circle { color, .. } => *color,
            DrawCommand::Ellipse { color, .. } => *color,
            DrawCommand::ScreenPath { color, .. } => *color,
            DrawCommand::ScreenCircle { color, .. } => *color,
        }
    }
}
