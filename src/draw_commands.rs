use crate::point::Vec2DDrawing;
use crate::color::Color;
use crate::shape::path::PathCommand;

/// The draw command as returned by the shape: adds the color
#[derive(Debug, PartialEq, Clone)]
pub enum DrawCommand {
    Path {
        color: Color,
        commands: Vec<PathCommand>,
        thickness: f64,
    },
    Circle {
        center: Vec2DDrawing,
        radius: f64,
        color: Color,
        thickness: f64,
    },
    Ellipse {
        bbox: [Vec2DDrawing; 2],
        thickness: f64,
        color: Color,
    },
}

impl DrawCommand {
    pub fn color(&self) -> Color {
        match self {
            DrawCommand::Path { color, .. } => *color,
            DrawCommand::Circle { color, .. } => *color,
            DrawCommand::Ellipse { color, .. } => *color,
        }
    }
}
