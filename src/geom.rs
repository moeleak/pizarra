#![allow(non_snake_case)]
//! All sorts of geometrical functions, constants and procedures
use std::ops::Mul;

use crate::point::{Point, Vec2DWorld, Vec2D};

/// How many points will be used to approximate the bezier curve for the purpose
/// of erasing a shape.
///
/// Too few will yield poor results (shapes not being deleted when the user
/// expects them to or shapes being deleted event before being touched).
/// Too many will impact performance.
const BEZIER_PARTS: usize = 50;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Angle(f64);

impl Angle {
    pub fn degrees(self) -> f64 {
        self.0.to_degrees()
    }

    pub fn radians(self) -> f64 {
        self.0
    }

    pub fn from_radians(radians: f64) -> Angle {
        Angle(radians)
    }

    pub fn from_degrees(degrees: f64) -> Angle {
        Angle(degrees.to_radians())
    }

    pub fn between<P: Point>(a: P, b: P) -> Angle {
        if a == b {
            return Angle::from_radians(0.0);
        }

        let angle = (dot(a, b)/(a.magnitude() * b.magnitude())).acos();

        Self::from_radians(if a.x() * b.y() - a.y() * b.x() < 0.0 {
            -angle
        } else {
            angle
        })
    }
}

impl Mul<f64> for Angle {
    type Output = Self;

    fn mul(self, s: f64) -> Angle {
        Angle::from_radians(self.radians() *  s)
    }
}

fn dot<P: Point>(a: P, b: P) -> f64 {
    a.x() * b.x() + a.y() * b.y()
}

pub fn bbox_from_points<I>(points: I) -> [Vec2DWorld; 2]
    where I: Iterator<Item=Vec2DWorld>
{
    let bbox = points.fold([
        None, // bottom-left corner
        None, // top right corner
    ], |acc: [Option<Vec2DWorld>; 2], point| {
        [
            acc[0].map(|acc| {
                Vec2DWorld::new(
                    if point.x < acc.x { point.x } else { acc.x },
                    if point.y < acc.y { point.y } else { acc.y },
                )
            }).or(Some(point)),
            acc[1].map(|acc | {
                Vec2DWorld::new(
                    if point.x > acc.x { point.x } else { acc.x },
                    if point.y > acc.y { point.y } else { acc.y },
                )
            }).or(Some(point))
        ]
    });

    [bbox[0].unwrap(), bbox[1].unwrap()]
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

pub fn segment_intersects_circle<P: Point>(from: Option<P>, to: P, center: P, radius: f64) -> bool {
    if let Some(from) = from {
        segment_intersects_circle_inner([from, to], center, radius)
    } else {
        false
    }
}

#[inline]
fn segment_intersects_circle_inner<P: Point>([a, b]: [P; 2], center: P, radius: f64) -> bool {
    let proj = project(center, [a, b]);

    proj.distance(center) < radius && is_within(proj, a, b)
}

pub fn bezier_intersects_circle<P: Point>(from: Option<P>, p1: P, p2: P, to: P, center: P, radius: f64) -> bool {
    if let Some(from) = from {
        bezier_intersects_circle_inner(from, p1, p2, to, center, radius)
    } else {
        false
    }
}

#[inline]
fn combine<P: Point>(a: P, b: P, t: f64) -> P {
    a * (1.0 - t) + b * t
}

#[inline]
fn bezier_intersects_circle_inner<P: Point>(from: P, p1: P, p2: P, to: P, center: P, radius: f64) -> bool {
    if to.distance(center) < radius {
        return true;
    }

    let piece = 1.0 / BEZIER_PARTS as f64;

    // here we compute 10 curve's points. This should be a good enough
    // aproximation
    let points: Vec<_> = (0..BEZIER_PARTS).map(|p| {
        p as f64 * piece
    }).map(|t| {
        let b1 = combine(from, p1, t);
        let b2 = combine(p1, p2, t);
        let b3 = combine(p2, to, t);

        let c1 = combine(b1, b2, t);
        let c2 = combine(b2, b3, t);

        combine(c1, c2, t)
    }).collect();

    for (&p1, &p2) in points.iter().zip(points.iter().skip(1)) {
        if segment_intersects_circle_inner([p1, p2], center, radius) {
            return true;
        }
    }

    false
}

pub fn middle_point<P: Point>(p1: P, p2: P) -> P {
    (p1 + p2) / 2.0
}

/// given the four corners that define an ellipse returns the length of the
/// semimajor axis, a vector that defines it and the length of the semiminor
/// axis
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

/// given the foci and a point of the ellipse return its center, semimajor axis,
/// semiminor axis and angle
pub fn ellipse_from_foci_and_point<P: Point>(foci: [P; 2], point: P) -> (P, f64, f64, Angle) {
    let sum = point.distance(foci[0]) + point.distance(foci[1]);

    ellipse_from_foci_and_sum(foci, sum)
}

/// given the foci and the sum of distances that define the ellipse return its
/// center, semimajor and semiminor axis
pub fn ellipse_from_foci_and_sum<P: Point>(foci: [P; 2], sum: f64) -> (P, f64, f64, Angle) {
    // half the distance between the foci
    let d = foci[0].distance(foci[1]) / 2.0;
    let semimajor = sum / 2.0;
    let semiminor = (semimajor.powi(2) - d.powi(2)).sqrt();
    let v = foci[1] - foci[0];

    let center = (foci[0] + foci[1]) / 2.0;
    let angle = v.angle();

    let semimajor = if semimajor == 0.0 { 1.0 } else { semimajor };
    let semiminor = if semiminor == 0.0 { 1.0 } else { semiminor };

    (center, semimajor, semiminor, angle)
}

pub fn foci_and_sum_from_center_and_radi<P: Point>(center: P, semimajor: f64, semiminor: f64, angle: Angle) -> ([P; 2], f64) {
    // this is the magnitude of a vector that goes from the center of the
    // ellipse to either of the focis
    let x = (semimajor.powi(2) - semiminor.powi(2)).sqrt();

    // this is a vector parallel to the line that connects the two focis of
    // the ellipse
    let v = P::new(angle.radians().cos(), angle.radians().sin());

    // now we can compute the centers of the ellipse
    let center1 = center + v * x;
    let center2 = center - v * x;

    // this variable represents the (constant) sum of the distance from any
    // point of the ellipse to both of its centers
    let sum = semimajor * 2.0;

    ([center1, center2], sum)
}

#[allow(clippy::many_single_char_names)]
pub fn circle_through_three_points<P: Point>(p1: P, p2: P, p3: P) -> (P, f64) {
    let s1 = p1.x().powi(2) + p1.y().powi(2);
    let s2 = p2.x().powi(2) + p2.y().powi(2);
    let s3 = p3.x().powi(2) + p3.y().powi(2);

    let a = p1.x()*(p2.y() - p3.y()) - p1.y() * (p2.x() - p3.x()) + p2.x()*p3.y() - p3.x()*p2.y();
    let b = s1 * (p3.y() - p2.y()) + s2 * (p1.y() - p3.y()) + s3 * (p2.y() - p1.y());
    let c = s1 * (p2.x() - p3.x()) + s2 * (p3.x() - p1.x()) + s3 * (p1.x() - p2.x());
    let d = s1 * (p3.x() * p2.y() - p2.x() * p3.y()) + s2 * (p1.x() * p3.y() - p3.x() * p1.y()) + s3 * (p2.x() * p1.y() - p1.x() * p2.y());

    if a == 0.0 {
        return ((p1 + p2) / 2.0, 1.0);
    }

    let x = -b / (2.0 * a);
    let y = -c / (2.0 * a);

    let r = ((b.powi(2) + c.powi(2) - 4.0 * a * d) / (4.0 * a.powi(2))).sqrt();

    (P::new(x, y), r)
}

/// Given two points return the points in their 1/3 and 2/3 of the segment from
/// a to b.
pub fn thirds<P: Point>(a: P, b: P) -> (P, P) {
    let v = b - a;

    (v*1.0/3.0 + a, v*2.0/3.0 + a)
}

/// return a vector of magnitude 1 that has the slope of the external bisector
/// of the angle described by the three given points.
///
/// B is the vertex of the angle. The returned vector points towads C
pub fn bisector<P: Point>(A: P, B: P, C: P) -> P {
    let Ap = (A - B) * -1.0 + B;
    let (a, b, c) = (B.distance(C), A.distance(C), A.distance(B));
    let incenter = (Ap * a + B * b + C * c) / (a + b + c);

    let vec = incenter - B;

    vec / vec.magnitude()
}

/// Mirrors the first point given through the second one
pub fn mirror<P: Point>(point: P, through: P) -> P {
    let c = through - point;

    through + c
}

fn robust_length(a: f64, b: f64) -> f64 {
    if a.abs() > b.abs() {
        a.abs() * (1.0 + (b/a).powi(2)).sqrt()
    } else {
        b.abs() * (1.0 + (a/b).powi(2)).sqrt()
    }
}

const MAX_ITERATIONS: usize = 1000;

/// from https://www.geometrictools.com/Documentation/DistancePointEllipseEllipsoid.pdf
/// David Eberly, Geometric Tools, Redmond WA 98052
fn get_root(r0: f64, z0: f64, z1: f64, mut g: f64) -> f64 {
    let n0 = r0 * z0;
    let mut s0 = z1 - 1.0;
    let mut s1 = if g < 0.0 { 0.0 } else { robust_length(n0, z1) - 1.0 };
    let mut s = 0.0;

    for _ in 0..MAX_ITERATIONS {
        s = ( s0 + s1 ) / 2.0;

        if s == s0 || s == s1 {
            break;
        }

        let ratio0 = n0 / ( s + r0 );
        let ratio1 = z1 / ( s + 1.0 );

        g = ratio0.sqrt() + ratio1.sqrt() - 1.0;

        if g > 0.0 {
            s0 = s;
        } else if g < 0.0 {
            s1 = s;
        } else {
            break;
        }
    }

    s
}

/// from https://www.geometrictools.com/Documentation/DistancePointEllipseEllipsoid.pdf
/// David Eberly, Geometric Tools, Redmond WA 98052
///
/// computes the distance from an ellipse to the given point. The ellipse is
/// centered in the origin. The point's coordinates must be in the 1st cuadrant
/// i.e. y0, y1 >= 0. The ellipse's longer axis e0 is along the x axis. It is
/// assumed that e0 >= e1.
fn distance_point_ellipse(e0: f64, e1: f64, y0: f64, y1: f64) -> ((f64, f64), f64) {
    if y1 > 0.0 {
        if y0 > 0.0 {
            let z0 = y0 / e0;
            let z1 = y1 / e1;
            let g = z0.powi(2) + z1.powi(2) - 1.0;

            if g != 0.0 {
                let r0 = ( e0 / e1 ).powi(2);
                let sbar = get_root(r0, z0, z1, g);
                let x0 = r0 * y0 / ( sbar + r0 );
                let x1 = y1 / (sbar + 1.0) ;
                (
                    (x0, x1),
                    ((x0 - y0).powi(2) + (x1 - y1).powi(2)).sqrt()
                )
            } else {
                let x0 = y0;
                let x1 = y1;
                ((x0, x1), 0.0)
            }
        } else { // y0 == 0
            let x0 = 0.0;
            let x1 = e1;
            ((x0, x1), (y1 - e1).abs())
        }
    } else { // y1 == 0
        let numer0 = e0 * y0;
        let denom0 = ( e0 ).powi(2) - ( e1 ).powi(2);

        if numer0 < denom0 {
            let xde0 = numer0 / denom0;
            let x0 = e0 * xde0;
            let x1 = e1 * (1.0 - xde0 * xde0).sqrt();
            ((x0, x1), ((x0 - y0).powi(2) + x1.powi(2)).sqrt())
        } else {
            let x0 = e0;
            let x1 = 0.0;
            ((x0, x1), (y0 - e0).abs())
        }
    }
}

pub struct Ellipse<P> {
    pub center: P,
    pub semimajor: f64,
    pub semiminor: f64,
    pub angle: Angle,
}

/// Compute the distance from a point to an ellipse. Accepts the center, the
/// two semiaxes (horizontal and vertical respectively) and the point.
///
/// It is assumed that e0 >= e1, this means that the point might need to be
/// transformed in the following situations:
///
/// * if the ellipse is rotated
/// * if the larger axis of the ellipse is not parallel to the x axis
pub fn distance_from_point_to_ellipse<P: Point>(point: P, ellipse: Ellipse<P>) -> f64 {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::point::Vec2DScreen;

    use super::*;

    #[test]
    fn test_segment_intersects_circle() {
        assert!(segment_intersects_circle_inner([Vec2DWorld::new(0.0, 0.0), Vec2DWorld::new(100.0, 0.0)], Vec2DWorld::new(50.0, 9.0), 10.0));
    }

    #[test]
    fn test_single_point_segment_intersects_circle() {
        assert!(segment_intersects_circle_inner([Vec2DWorld::new(0.0, 0.0), Vec2DWorld::new(0.0, 0.0)], Vec2DWorld::new(0.0, 0.0), 10.0));
    }

    #[test]
    fn the_expected_ellipse_is_built() {
        let f1: Vec2DWorld = (-20.22222, -13.24752).into();
        let f2: Vec2DWorld = (-2.88, 1.28).into();
        let sum = 14.29 + 17.67;

        let (center, semimajor, semiminor, angle) = ellipse_from_foci_and_sum([f1, f2], sum);

        assert!(center.distance((-11.5, -5.98).into()) < 0.06);
        assert!((semimajor - sum / 2.0).abs() < 0.01);
        assert!((semiminor - 11.28).abs() < 0.01);
        assert!((angle.degrees() - 39.94).abs() < 0.02);
    }

    #[test]
    fn the_expected_ellipse_is_built_back() {
        let center: Vec2DWorld = (-11.5, -5.98).into();
        let semimajor = (14.29 + 17.67) / 2.0;
        let semiminor = 11.28;
        let angle = Angle::from_degrees(39.94);

        let ([f1, f2], sum) = foci_and_sum_from_center_and_radi(center, semimajor, semiminor, angle);

        assert!(f2.distance((-20.22222, -13.24752).into()) < 0.1);
        assert!(f1.distance((-2.88, 1.28).into()) < 0.1);
        assert!((sum - (14.29 + 17.67)).abs() < 0.1);
    }

    #[test]
    fn bezier_curve_properly_intersects_circles() {
        let from = Some(Vec2DWorld::new(33.0, 135.0));
        let pt1 = Vec2DWorld::new(50.0, 200.0);
        let pt2 = Vec2DWorld::new(171.0, 70.0);
        let to = Vec2DWorld::new(196.0, 113.0);

        let cases = [
             (Vec2DWorld::new(81.0, 154.0), true),
             (Vec2DWorld::new(127.0, 109.0), true),
             (Vec2DWorld::new(74.0, 90.0), false),
             (Vec2DWorld::new(147.0, 165.0), false),
             (Vec2DWorld::new(177.0, 121.0), false),
        ];

        for (center, result) in cases {
            assert_eq!(bezier_intersects_circle(from, pt1, pt2, to, center, 15.0), result);
        }
    }

    #[test]
    fn middle_point_behaves() {
        let a = Vec2DWorld::new(-5.61146,3.80508);
        let b = Vec2DWorld::new(-0.44,-1.28);

        assert_eq!(middle_point(a, b), (-3.0257300000000003, 1.26254).into());
    }

    #[test]
    fn angle_between_two_vectors() {
        let a: Vec2DWorld = (-2.68,5.12).into();
        let b: Vec2DWorld = (-4.4,2.12).into();

        assert!((Angle::between(a, b).radians() - Angle::from_degrees(36.65).radians()).abs() < 0.001);

        let a: Vec2DWorld = (-2.68,5.12).into();
        let b: Vec2DWorld = (-4.4,2.12).into();

        assert!((Angle::between(b, a).radians() - Angle::from_degrees(-36.65).radians()).abs() < 0.001);
    }

    #[test]
    fn angle_between_two_identical_points() {
        let a = Vec2DScreen {
            x: -38.89215087890625,
            y: 145.26171875,
        };
        let b = Vec2DScreen {
            x: -38.89215087890625,
            y: 145.26171875,
        };

        assert_eq!(Angle::between(a, b), Angle::from_radians(0.0));
    }

    #[test]
    fn circle_through_three_points_works() {
        let a: Vec2DWorld = (-11.0, 4.0).into();
        let b: Vec2DWorld = (-6.5, -1.6).into();
        let c: Vec2DWorld = (2.44, 2.56).into();

        let (center, radius) = circle_through_three_points(a, b, c);

        assert_eq!(center, Vec2DWorld { x: -4.102742498255409, y: 4.93440334961619 });
        assert!((radius - 6.96).abs() < 0.01);
    }

    #[test]
    fn ellipse_from_foci_and_point_cannot_give_flat_or_null_ellipses() {
        let p = Vec2DWorld { x: -4.102742498255409, y: 4.93440334961619 };

        let (_, semimajor, semiminor, _) = ellipse_from_foci_and_point([p, p], p);

        assert_ne!(semimajor, 0.0);
        assert_ne!(semiminor, 0.0);

        let (_, semimajor, semiminor, _) = ellipse_from_foci_and_sum([p, p], 0.0);

        assert_ne!(semimajor, 0.0);
        assert_ne!(semiminor, 0.0);
    }

    #[test]
    fn adimensional_circle_doesnt_explode() {
        let p = Vec2DWorld { x: -4.102742498255409, y: 4.93440334961619 };

        let (center, radius) = circle_through_three_points(p, p, p);

        assert_eq!(center, p);
        assert_eq!(radius, 1.0);
    }

    #[test]
    fn test_thirds() {
        assert_eq!(thirds(Vec2DWorld::new(3.0, 0.0), Vec2DWorld::new(6.0, 0.0)), (Vec2DWorld::new(4.0, 0.0), Vec2DWorld::new(5.0, 0.0)));
        assert_eq!(thirds(Vec2DWorld::new(0.0, 0.0), Vec2DWorld::new(6.0, 6.0)), (Vec2DWorld::new(2.0, 2.0), Vec2DWorld::new(4.0, 4.0)));
        assert_eq!(thirds(Vec2DWorld::new(-15.0, 30.0), Vec2DWorld::new(-18.0, 27.0)), (Vec2DWorld::new(-16.0, 29.0), Vec2DWorld::new(-17.0, 28.0)));
    }

    #[test]
    fn test_bisector() {
        let a: Vec2DWorld = (-2.0, 3.0).into();
        let b: Vec2DWorld = (1.0, 6.0).into();
        let c: Vec2DWorld = (5.0, 2.0).into();
        let expected_bisector = Vec2DWorld::new(1.0, 0.0);
        let given_bisector = bisector(a, b, c);

        assert!(expected_bisector.x - given_bisector.x.abs() < 0.001);
        assert!(expected_bisector.y - given_bisector.y.abs() < 0.001);
    }

    #[test]
    fn mirror_mirrors() {
        let j: Vec2DWorld = (180.0, 120.0).into();
        let i: Vec2DWorld = (160.0, 100.0).into();

        assert_eq!(mirror(j, i), (140.0, 80.0).into());
    }
}
