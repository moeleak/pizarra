use crate::point::{Vec2DWorld, Vec2DScreen};
use crate::color::Color;
use crate::path_command::PathCommand::{self, *};
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
            DrawCommand::Ellipse(e) => e.color,
            DrawCommand::ScreenPath { color, .. } => *color,
            DrawCommand::ScreenCircle { color, .. } => *color,
        }
    }

    pub fn fill(&self) -> Option<Color> {
        match self {
            DrawCommand::Path { fill, .. } => *fill,
            DrawCommand::Ellipse(e) => e.fill,
            DrawCommand::ScreenPath { fill, .. } => *fill,
            DrawCommand::ScreenCircle { fill, .. } => *fill,
        }
    }
}

/// Returns a circle that can be used as visual helper for a tool that is being
/// built
pub fn circle_helper(center: Vec2DScreen, radius: f64) -> DrawCommand {
    DrawCommand::ScreenCircle {
        color: None,
        fill: Some(Color::gray().half_transparent()),
        center,
        radius,
        thickness: 0.0,
    }
}

pub fn path_helper(points: Vec<Vec2DScreen>) -> DrawCommand {
    DrawCommand::ScreenPath {
        commands: points.into_iter().enumerate().map(|(i, p)| {
            if i == 0 {
                MoveTo(p)
            } else {
                LineTo(p)
            }
        }).collect(),
        color: Some(Color::gray()),
        fill: None,
        thickness: 1.0,
    }
}
