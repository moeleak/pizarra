/// The most primitive draw command: the one returned by the shape impl
pub enum WireDrawCommand {
    Line{
        line: [f64; 4],
    },
    Rectangle{
        rect: [f64; 4],
    },
    Circle{
        rect: [f64; 4],
    },
}

/// The draw command as returned by the shape: adds the color 
pub enum ColoredDrawCommand {
    Line{
        color: [f32; 4],
        line: [f64; 4],
    },
    Rectangle{
        color: [f32; 4],
        rect: [f64; 4],
    },
    Circle{
        color: [f32; 4],
        rect: [f64; 4],
    },
}

/// A Full Draw command returned by the storage
#[derive(Debug, PartialEq)]
pub enum DrawCommand {
    Line{
        color: [f32; 4],
        line: [f64; 4],
        thickness: f64,
    },
    Rectangle{
        color: [f32; 4],
        rect: [f64; 4],
    },
    Circle{
        color: [f32; 4],
        rect: [f64; 4],
    },
}

impl DrawCommand {
    pub fn color(&self) -> [f32; 4] {
        match self {
            DrawCommand::Line { color, .. } => *color,
            DrawCommand::Rectangle { color, .. } => *color,
            DrawCommand::Circle { color, .. } => *color,
        }
    }
}
