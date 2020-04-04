/// A wireframe-only draw command returned by the underlying shape impl
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

/// A wireframe plus color draw command returned by the Shape struct
pub enum ColoredDrawCommand {
    Line{
        line: [f64; 4],
        color: [f32; 4],
    },
    Rectangle{
        rect: [f64; 4],
        color: [f32; 4],
    },
    Circle{
        rect: [f64; 4],
        color: [f32; 4],
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
