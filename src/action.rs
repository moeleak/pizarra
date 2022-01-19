use crate::shape::{Shape, ShapeId};

#[derive(Debug)]
pub enum Action {
    Unfinished,
    Empty,

    Draw(Vec<ShapeId>),
    Erase(Vec<Shape>),
}

impl Default for Action {
    fn default() -> Action {
        Action::Empty
    }
}
