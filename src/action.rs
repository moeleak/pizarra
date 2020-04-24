use crate::zoom::Zoom;
use crate::shape::{Shape, ShapeId};

pub enum Action {
    Skip,
    Draw(ShapeId),
    Delete((Shape, Zoom)),
}

impl Default for Action {
    fn default() -> Action {
        Action::Skip
    }
}
