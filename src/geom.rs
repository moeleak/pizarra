//! All sorts of geometrical functions, constants and procedures

use crate::point::{Point, Vec2DDrawing};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Radians(f64);

impl Radians {
    pub fn degrees(self) -> f64 {
        self.0.to_degrees()
    }

    pub fn radians(self) -> f64 {
        self.0
    }

    pub fn from_radians(radians: f64) -> Radians {
        Radians(radians)
    }

    pub fn from_degrees(degrees: f64) -> Radians {
        Radians(degrees.to_radians())
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
pub fn parts_of_ellipse_from_corners<P: Point>(corners: [P; 4]) -> (f64, P, f64) {
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

/// given the foci and the sum of distances that define the ellipse return its
/// center, larger radius and shorter radius
pub fn ellipse_from_foci_and_sum<P: Point>(foci: [P; 2], sum: f64) -> (P, f64, f64, Radians) {
    // half the distance between the foci
    let d = foci[0].distance(foci[1]) / 2.0;
    let bigside = sum / 2.0;
    let smallside = (bigside.powi(2) - d.powi(2)).sqrt();
    let v = foci[1] - foci[0];

    return ((foci[0] + foci[1]) / 2.0, bigside, smallside, v.angle());
}

pub fn foci_and_sum_from_center_and_radi<P: Point>(center: P, bigside: f64, smallside: f64, angle: Radians) -> ([P; 2], f64) {
    // this is the magnitude of a vector that goes from the center of the
    // ellipse to either of the focis
    let x = (bigside.powi(2) - smallside.powi(2)).sqrt();

    // this is a vector parallel to the line that connects the two focis of
    // the ellipse
    let v = P::new(angle.radians().cos(), angle.radians().sin());

    // now we can compute the centers of the ellipse
    let center1 = center + v * x;
    let center2 = center - v * x;

    // this variable represents the (constant) sum of the distance from any
    // point of the ellipse to both of its centers
    let sum = bigside * 2.0;

    ([center1, center2], sum)
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

    #[test]
    fn the_expected_ellipse_is_built() {
        let f1: Vec2DDrawing = (-20.22222, -13.24752).into();
        let f2: Vec2DDrawing = (-2.88, 1.28).into();
        let sum = 14.29 + 17.67;

        let (center, bigside, smallside, angle) = ellipse_from_foci_and_sum([f1, f2], sum);

        assert!(center.distance((-11.5, -5.98).into()) < 0.06);
        assert!((bigside - sum / 2.0).abs() < 0.01);
        assert!((smallside - 11.28).abs() < 0.01);
        assert!((angle.degrees() - 39.94).abs() < 0.02);
    }

    #[test]
    fn the_expected_ellipse_is_built_back() {
        let center: Vec2DDrawing = (-11.5, -5.98).into();
        let bigside = (14.29 + 17.67) / 2.0;
        let smallside = 11.28;
        let angle = Radians::from_degrees(39.94);

        let ([f1, f2], sum) = foci_and_sum_from_center_and_radi(center, bigside, smallside, angle);

        assert!(f2.distance((-20.22222, -13.24752).into()) < 0.1);
        assert!(f1.distance((-2.88, 1.28).into()) < 0.1);
        assert!((sum - (14.29 + 17.67)).abs() < 0.1);
    }
}
