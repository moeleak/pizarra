use std::str::FromStr;

use crate::point::{Point, Vec2DDrawing};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Rad(f64);

impl Rad {
    pub fn degrees(self) -> f64 {
        self.0.to_degrees()
    }

    pub fn radians(self) -> f64 {
        self.0
    }
}

impl From<f64> for Rad {
    fn from(radians: f64) -> Rad {
        Rad(radians)
    }
}

pub fn bbox_from_points<I>(points: I) -> [Vec2DDrawing; 2]
    where I: Iterator<Item=Vec2DDrawing>
{
    let bbox = points.fold([
        Vec2DDrawing::new(std::f64::INFINITY, std::f64::INFINITY), // bottom-left corner
        Vec2DDrawing::new(std::f64::NEG_INFINITY, std::f64::NEG_INFINITY), // top right corner
    ], |acc, point| {
        [
            Vec2DDrawing::new(
                if point.x < acc[0].x { point.x } else { acc[0].x },
                if point.y < acc[0].y { point.y } else { acc[0].y },
            ),
            Vec2DDrawing::new(
                if point.x > acc[1].x { point.x } else { acc[1].x },
                if point.y > acc[1].y { point.y } else { acc[1].y },
            ),
        ]
    });

    assert_ne!(bbox[0].x, std::f64::INFINITY);
    assert_ne!(bbox[0].y, std::f64::INFINITY);
    assert_ne!(bbox[1].x, std::f64::NEG_INFINITY);
    assert_ne!(bbox[1].y, std::f64::NEG_INFINITY);

    bbox
}

pub fn project<P: Point>(point: P, [a, b]: [P; 2]) -> P {
    // https://en.wikibooks.org/wiki/Linear_Algebra/Orthogonal_Projection_Onto_a_Line

    // If the two points that define the segment are equal they the projection is any of them
    if a == b {
        return a;
    }

    let v = point - a;
    let s = b - a;

    s * (v.dot(s) / s.dot(s)) + a
}

pub fn is_within<P: Point>(point: P, a: P, b: P) -> bool {
    let min = a.min(b);
    let max = a.max(b);

    point.x() >= min.x() && point.x() <= max.x() &&
    point.y() >= min.y() && point.y() <= max.y()
}

pub fn segment_intersects_circle([a, b]: [Vec2DDrawing; 2], center: Vec2DDrawing, radius: f64) -> bool {
    let proj = project(center, [a, b]);

    proj.distance(center) < radius && is_within(proj, a, b)
}

pub fn middle_point<P: Point>(p1: P, p2: P) -> P {
    (p1 + p2) / 2.0
}

/// given the four corners that define an ellipse returns the length of the
/// larger side, a vector that defines it and the length of the shorter side
pub fn parts_of_ellipse<P: Point>(corners: [P; 4]) -> (f64, P, f64) {
    let ellipse_side_1 = corners[0] - corners[1];
    let ellipse_side_2 = corners[1] - corners[2];
    let ellipse_len_1 = ellipse_side_1.magnitude();
    let ellipse_len_2 = ellipse_side_2.magnitude();

    if ellipse_len_1 > ellipse_len_2 {
        (ellipse_len_1, ellipse_side_1, ellipse_len_2)
    } else {
        (ellipse_len_2, ellipse_side_2, ellipse_len_1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_intersects_circle() {
        assert!(segment_intersects_circle([Vec2DDrawing::new(0.0, 0.0), Vec2DDrawing::new(100.0, 0.0)], Vec2DDrawing::new(50.0, 9.0), 10.0));
    }

    #[test]
    fn test_single_point_segment_intersects_circle() {
        assert!(segment_intersects_circle([Vec2DDrawing::new(0.0, 0.0), Vec2DDrawing::new(0.0, 0.0)], Vec2DDrawing::new(0.0, 0.0), 10.0));
    }
}
