use crate::geom::{bezier_intersects_circle, segment_intersects_circle};
use crate::point::{Unit, Vec2D};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CubicBezierCurve<T: Unit> {
    /// The (x, y) coordinates of the first control point.
    pub pt1: Vec2D<T>,
    /// The (x, y) coordinates of the second control point.
    pub pt2: Vec2D<T>,
    /// The (x, y) coordinates of the end point of this path segment.
    pub to: Vec2D<T>,
}

impl<T: Unit> CubicBezierCurve<T> {
    pub fn is_nan(&self) -> bool {
        self.pt1.is_nan() || self.pt2.is_nan() || self.to.is_nan()
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PathCommand<T: Unit> {
    MoveTo(Vec2D<T>),
    LineTo(Vec2D<T>),
    CurveTo(CubicBezierCurve<T>),
}

impl<T: Unit + 'static> PathCommand<T> {
    pub fn is_nan(&self) -> bool {
        match self {
            PathCommand::MoveTo(p) => p.is_nan(),
            PathCommand::LineTo(p) => p.is_nan(),
            PathCommand::CurveTo(c) => c.is_nan(),
        }
    }

    /// returns the point this path segment leads to
    pub fn to(&self) -> Vec2D<T> {
        match *self {
            PathCommand::MoveTo(p) => p,
            PathCommand::LineTo(p) => p,
            PathCommand::CurveTo(c) => c.to,
        }
    }

    pub fn intersects_circle(&self, center: Vec2D<T>, radius: T, prev: Option<Vec2D<T>>) -> bool {
        match *self {
            PathCommand::MoveTo(p) => p.distance(center) < radius,
            PathCommand::LineTo(p) => {
                p.distance(center) < radius || segment_intersects_circle(prev, p, center, radius)
            }
            PathCommand::CurveTo(p) => {
                p.to.distance(center) < radius
                    || bezier_intersects_circle(prev, p.pt1, p.pt2, p.to, center, radius)
            }
        }
    }

    pub fn points(&self) -> Box<dyn Iterator<Item = Vec2D<T>>> {
        match *self {
            PathCommand::MoveTo(p) => Box::new(IntoIterator::into_iter([p])),
            PathCommand::LineTo(p) => Box::new(IntoIterator::into_iter([p])),
            PathCommand::CurveTo(CubicBezierCurve { pt1, pt2, to }) => {
                Box::new(IntoIterator::into_iter([pt1, pt2, to]))
            }
        }
    }
}
