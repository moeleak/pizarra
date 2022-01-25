use crate::point::{Vec2D, WorldUnit, ScreenUnit};
use crate::color::Color;
use crate::path_command::PathCommand::{self, *};
use crate::geom::Ellipse;
use crate::style::Style;
use crate::transform::Transform;

/// This enum is returned by the shapes (both stored and the one being drawn).
/// It is the interface between the backend (libpizarra) and the frontend
/// (pizarra), it is the responsibility of the latter to render those commands
/// on the screen.
#[derive(Debug, PartialEq, Clone)]
pub enum DrawCommand {
    /// A string of points and curves without filling
    Path {
        commands: Vec<PathCommand<WorldUnit>>,
        style: Style,
    },

    /// An ellipse without filling
    Ellipse {
        ellipse: Ellipse<WorldUnit>,
        style: Style,
    },

    /// A path in screen coordinates without filling. Used to render handles and
    /// other visual aids
    ScreenPath {
        commands: Vec<PathCommand<ScreenUnit>>,
        style: Style,
    },

    /// A circle in screen coordinates without filling. Used to render handles
    /// and visual aids
    ScreenCircle {
        center: Vec2D<ScreenUnit>,
        radius: ScreenUnit,
        style: Style,
    },
}

impl DrawCommand {
    pub fn color(&self) -> Option<Color> {
        match self {
            DrawCommand::Path { style, .. } => style.stroke.map(|s| s.color),
            DrawCommand::Ellipse { style, .. } => style.stroke.map(|s| s.color),
            DrawCommand::ScreenPath { style, .. } => style.stroke.map(|s| s.color),
            DrawCommand::ScreenCircle { style, .. } => style.stroke.map(|s| s.color),
        }
    }

    pub fn fill(&self) -> Option<Color> {
        match self {
            DrawCommand::Path { style, .. } => style.fill,
            DrawCommand::Ellipse { style, .. } => style.fill,
            DrawCommand::ScreenPath { style, .. } => style.fill,
            DrawCommand::ScreenCircle { style, .. } => style.fill,
        }
    }
}

/// Returns a circle that can be used as visual helper for a tool that is being
/// built
pub fn circle_helper(center: Vec2D<ScreenUnit>, radius: ScreenUnit) -> DrawCommand {
    DrawCommand::ScreenCircle {
        center,
        radius,
        style: Style::circle_helper(),
    }
}

fn red_circle_helper(center: Vec2D<ScreenUnit>, radius: ScreenUnit) -> DrawCommand {
    DrawCommand::ScreenCircle {
        center,
        radius,
        style: Style::red_circle_helper(),
    }
}

pub fn cancel_helper(prev: Vec2D<WorldUnit>, new: Vec2D<WorldUnit>, t: Transform, snap: ScreenUnit) -> DrawCommand {
    if t.to_screen_coordinates(new).distance(t.to_screen_coordinates(prev)) < snap {
        red_circle_helper(t.to_screen_coordinates(prev), snap)
    } else {
        circle_helper(t.to_screen_coordinates(prev), snap)
    }
}

pub fn path_helper(points: Vec<Vec2D<ScreenUnit>>) -> DrawCommand {
    DrawCommand::ScreenPath {
        commands: points.into_iter().enumerate().map(|(i, p)| {
            if i == 0 {
                MoveTo(p)
            } else {
                LineTo(p)
            }
        }).collect(),
        style: Style::path_helper(),
    }
}
