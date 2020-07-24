use crate::shape::{Shape, ShapeId};

#[derive(Debug)]
pub enum Action {
    Unfinished,
    Empty,

    Draw(ShapeId),
    Delete(Shape),

    Erase(Vec<Shape>),
    Redraw(Vec<ShapeId>),
}

impl Default for Action {
    fn default() -> Action {
        Action::Empty
    }
}
