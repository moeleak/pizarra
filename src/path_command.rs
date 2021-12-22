use crate::point::Point;
use crate::geom::{
    segment_intersects_circle, bezier_intersects_circle,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CubicBezierCurve<T> {
    /// The (x, y) coordinates of the first control point.
    pub pt1: T,
    /// The (x, y) coordinates of the second control point.
    pub pt2: T,
    /// The (x, y) coordinates of the end point of this path segment.
    pub to: T,
}

impl<T: Point> CubicBezierCurve<T> {
    pub fn is_nan(&self) -> bool {
        self.pt1.is_nan() || self.pt2.is_nan() || self.to.is_nan()
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PathCommand<T> {
    MoveTo(T),
    LineTo(T),
    CurveTo(CubicBezierCurve<T>),
}

impl<T: 'static + Point> PathCommand<T> {
    pub fn is_nan(&self) -> bool {
        match self {
            PathCommand::MoveTo(p) => p.is_nan(),
            PathCommand::LineTo(p) => p.is_nan(),
            PathCommand::CurveTo(c) => c.is_nan(),
        }
    }

    /// returns the point this path segment leads to
    pub fn to(&self) -> T {
        match *self {
            PathCommand::MoveTo(p) => p,
            PathCommand::LineTo(p) => p,
            PathCommand::CurveTo(c) => c.to,
        }
    }

    pub fn intersects_circle(&self, center: T, radius: f64, prev: Option<T>) -> bool {
        match *self {
            PathCommand::MoveTo(p) => p.distance(center) < radius,
            PathCommand::LineTo(p) => {
                p.distance(center) < radius || segment_intersects_circle(prev, p, center, radius)
            }
            PathCommand::CurveTo(p) => {
                p.to.distance(center) < radius || bezier_intersects_circle(prev, p.pt1, p.pt2, p.to, center, radius)
            }
        }
    }

    pub fn points(&self) -> Box<dyn Iterator<Item=T>> {
        match *self {
            PathCommand::MoveTo(p) => Box::new(IntoIterator::into_iter([p])),
            PathCommand::LineTo(p) => Box::new(IntoIterator::into_iter([p])),
            PathCommand::CurveTo(CubicBezierCurve { pt1, pt2, to }) => Box::new(IntoIterator::into_iter([pt1, pt2, to])),
        }
    }
}
