use crate::shape::{Shape, ShapeId};

#[derive(Debug, Default)]
pub enum Action {
    Unfinished,

    #[default]
    Empty,

    Draw(Vec<ShapeId>),
    Erase(Vec<Shape>),
}
