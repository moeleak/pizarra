#![allow(non_snake_case)]
//! All sorts of geometrical functions, constants and procedures

use crate::point::{Vec2D, Unit, WorldUnit, Unittless};
use crate::transform::Transform;
use num_traits::NumCast;

mod angle;

pub use angle::Angle;

/// How many points will be used to approximate the bezier curve for the purpose
/// of erasing a shape.
///
/// Too few will yield poor results (shapes not being deleted when the user
/// expects them to or shapes being deleted event before being touched).
/// Too many will impact performance.
const BEZIER_PARTS: usize = 50;

pub(crate) fn bbox_from_points<I>(points: I) -> [Vec2D<WorldUnit>; 2]
    where I: Iterator<Item=Vec2D<WorldUnit>>
{
    let bbox = points.fold([
        None, // bottom-left corner
        None, // top right corner
    ], |acc: [Option<Vec2D<WorldUnit>>; 2], point| {
        [
            acc[0].map(|acc| {
                Vec2D::new(
                    if point.x < acc.x { point.x } else { acc.x },
                    if point.y < acc.y { point.y } else { acc.y },
                )
            }).or(Some(point)),
            acc[1].map(|acc | {
                Vec2D::new(
                    if point.x > acc.x { point.x } else { acc.x },
                    if point.y > acc.y { point.y } else { acc.y },
                )
            }).or(Some(point))
        ]
    });

    [bbox[0].unwrap(), bbox[1].unwrap()]
}

pub(crate) fn project<T: Unit>(point: Vec2D<T>, [a, b]: [Vec2D<T>; 2]) -> Vec2D<T> {
    // https://en.wikibooks.org/wiki/Linear_Algebra/Orthogonal_Projection_Onto_a_Line

    // If the two points that define the segment are equal they the projection is any of them
    if a == b {
        return a;
    }

    let v = point - a;
    let s = b - a;

    s * (v.dot(s) / s.dot(s)) + a
}

pub(crate) fn is_within<T: Unit>(point: Vec2D<T>, a: Vec2D<T>, b: Vec2D<T>) -> bool {
    let min = a.min(b);
    let max = a.max(b);

    point.x.val() >= min.x.val() && point.x.val() <= max.x.val() &&
    point.y.val() >= min.y.val() && point.y.val() <= max.y.val()
}

pub(crate) fn segment_intersects_circle<T: Unit>(from: Option<Vec2D<T>>, to: Vec2D<T>, center: Vec2D<T>, radius: T) -> bool {
    if let Some(from) = from {
        segment_intersects_circle_inner([from, to], center, radius)
    } else {
        false
    }
}

#[inline]
fn segment_intersects_circle_inner<T: Unit>([a, b]: [Vec2D<T>; 2], center: Vec2D<T>, radius: T) -> bool {
    let proj = project(center, [a, b]);

    proj.distance(center) < radius && is_within(proj, a, b)
}

pub(crate) fn bezier_intersects_circle<T: Unit>(from: Option<Vec2D<T>>, p1: Vec2D<T>, p2: Vec2D<T>, to: Vec2D<T>, center: Vec2D<T>, radius: T) -> bool {
    if let Some(from) = from {
        bezier_intersects_circle_inner(from, p1, p2, to, center, radius)
    } else {
        false
    }
}

#[inline]
fn combine<T: Unit>(a: Vec2D<T>, b: Vec2D<T>, t: f64) -> Vec2D<T> {
    a * (1.0 - t) + b * t
}

#[inline]
fn bezier_intersects_circle_inner<T: Unit>(from: Vec2D<T>, p1: Vec2D<T>, p2: Vec2D<T>, to: Vec2D<T>, center: Vec2D<T>, radius: T) -> bool {
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

pub(crate) fn middle_point<T: Unit>(p1: Vec2D<T>, p2: Vec2D<T>) -> Vec2D<T> {
    (p1 + p2) / 2.0
}

/// given the four corners that define an ellipse returns the length of the
/// semimajor axis, a vector that defines it and the length of the semiminor
/// axis
pub(crate) fn parts_of_ellipse_from_corners<T: Unit>(corners: [Vec2D<T>; 4]) -> (T, Vec2D<T>, T) {
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
pub(crate) fn ellipse_from_foci_and_point<T: Unit>(foci: [Vec2D<T>; 2], point: Vec2D<T>) -> (Vec2D<T>, T, T, Angle) {
    let sum = point.distance(foci[0]) + point.distance(foci[1]);

    ellipse_from_foci_and_sum(foci, sum)
}

/// given the foci and the sum of distances that define the ellipse return its
/// center, semimajor and semiminor axis
pub(crate) fn ellipse_from_foci_and_sum<T: Unit>(foci: [Vec2D<T>; 2], sum: T) -> (Vec2D<T>, T, T, Angle) {
    // half the distance between the foci
    let d = foci[0].distance(foci[1]) / 2.0;
    let semimajor = sum / 2.0;
    let semiminor = (semimajor.powi(2) - d.powi(2)).sqrt();
    let v = foci[1] - foci[0];

    let center = (foci[0] + foci[1]) / 2.0;
    let angle = v.angle();

    let semimajor = if semimajor == 0.0.into() { 1.0.into() } else { semimajor };
    let semiminor = if semiminor == 0.0.into() { 1.0.into() } else { semiminor };

    (center, semimajor, semiminor, angle)
}

pub(crate) fn foci_and_sum_from_center_and_radi<T: Unit>(center: Vec2D<T>, semimajor: f64, semiminor: f64, angle: Angle) -> ([Vec2D<T>; 2], f64) {
    // this is the magnitude of a vector that goes from the center of the
    // ellipse to either of the focis
    let x = (semimajor.powi(2) - semiminor.powi(2)).sqrt();

    // this is a vector parallel to the line that connects the two focis of
    // the ellipse
    let v = Vec2D::new(
        angle.radians().cos().into(),
        angle.radians().sin().into(),
    );

    // now we can compute the centers of the ellipse
    let center1 = center + v * x;
    let center2 = center - v * x;

    // this variable represents the (constant) sum of the distance from any
    // point of the ellipse to both of its centers
    let sum = semimajor * 2.0;

    ([center1, center2], sum)
}

#[allow(clippy::many_single_char_names)]
pub(crate) fn circle_through_three_points<T: Unit>(p1: Vec2D<T>, p2: Vec2D<T>, p3: Vec2D<T>) -> (Vec2D<T>, T) {
    let s1 = p1.x.powi(2) + p1.y.powi(2);
    let s2 = p2.x.powi(2) + p2.y.powi(2);
    let s3 = p3.x.powi(2) + p3.y.powi(2);

    let a = p1.x*(p2.y - p3.y) - p1.y * (p2.x - p3.x) + p2.x*p3.y - p3.x*p2.y;
    let b = s1 * (p3.y - p2.y) + s2 * (p1.y - p3.y) + s3 * (p2.y - p1.y);
    let c = s1 * (p2.x - p3.x) + s2 * (p3.x - p1.x) + s3 * (p1.x - p2.x);
    let d = s1 * (p3.x * p2.y - p2.x * p3.y) + s2 * (p1.x * p3.y - p3.x * p1.y) + s3 * (p2.x * p1.y - p1.x * p2.y);

    if a == 0.0.into() {
        return ((p1 + p2) / 2.0, 1.0.into());
    }

    let two = <T as NumCast>::from(2.0).unwrap();
    let x = -b / (two * a);
    let y = -c / (two * a);

    let four = <T as NumCast>::from(4.0).unwrap();
    let r = ((b.powi(2) + c.powi(2) - four * a * d) / (four * a.powi(2))).sqrt();

    (Vec2D::new(x, y), r)
}

/// Given two points return the points in their 1/3 and 2/3 of the segment from
/// a to b.
pub(crate) fn thirds<T: Unit>(a: Vec2D<T>, b: Vec2D<T>) -> (Vec2D<T>, Vec2D<T>) {
    let v = b - a;

    (v*1.0/3.0 + a, v*2.0/3.0 + a)
}

/// return a vector of magnitude 1 that has the slope of the external bisector
/// of the angle described by the three given points.
///
/// B is the vertex of the angle. The returned vector points towads C
pub(crate) fn bisector<T: Unit>(A: Vec2D<T>, B: Vec2D<T>, C: Vec2D<T>) -> Vec2D<T> {
    let Ap = (A - B) * -1.0 + B;
    let (a, b, c) = (B.distance(C), A.distance(C), A.distance(B));
    let incenter = (Ap * a + B * b + C * c) / (a + b + c);

    let vec = incenter - B;

    vec / vec.magnitude()
}

/// Mirrors the first point given through the second one
pub(crate) fn mirror<T: Unit>(point: Vec2D<T>, through: Vec2D<T>) -> Vec2D<T> {
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

const MAX_ITERATIONS: usize = 1074;

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

        g = ratio0.powi(2) + ratio1.powi(2) - 1.0;

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
    assert!(y0 >= 0.0);
    assert!(y1 >= 0.0);
    assert!(e0 >= e1);

    if y1 > 0.0 {
        if y0 > 0.0 {
            let z0 = y0 / e0;
            let z1 = y1 / e1;
            // the ellipse equation
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
                // the given point satisfies the ellipse equation, then the
                // distance is 0.0
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

/// An ellipse for geometrical purposes
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ellipse<T: Unit> {
    pub center: Vec2D<T>,
    pub semimajor: T,
    pub semiminor: T,
    pub angle: Angle,
}

pub(crate) fn distance_from_point_to_ellipse<T: Unit>(point: Vec2D<T>, ellipse: Ellipse<T>) -> T {
    let point = Transform::default()
        .translate((-ellipse.center).to_vec2d())
        .rotate(-ellipse.angle)
        .apply(point.to_vec2d());

    // now make sure the point is in the first quadrant. This is a
    // requirement of the algorithm and it is easy to see through symetries
    // that it doesn't affect the result
    let point = point.abs();
    let (_closest_point, distance) = distance_point_ellipse(
        ellipse.semimajor.val(),
        ellipse.semiminor.val(),
        point.x.val(),
        point.y.val(),
    );

    distance.into()
}

/// Given the three points that define a rombus in some space, return the
/// matrix that transform its space into one where that very rombus is rectangle
/// in the first quadrant.
///
/// The three points are considered to be laid out like this, modulo some
/// rotations, scales and mirrors:
///
/// ```text
///                 p2
///                 ^
///                /
///               /
/// origin ---> p1
/// ```
///
/// so that when the space is transformed they will ve the two unit vectors that
/// define the space î and ĵ
pub(crate) fn rombus_to_rectangle(origin: Vec2D<Unittless>, p1: Vec2D<Unittless>, p2: Vec2D<Unittless>) -> Transform {
    // first vector. In the final transformed space this will point to the
    // positive X axis and be perfectly horizontal
    let a = p1 - origin;

    let t = Transform::new_translate(-origin).rotate(-a.angle());

    // Ensure that the second point is in the first quadrant by reflecting along
    // the X axis only of the Y coordinate of a transformed point is negative.
    let t = if t.apply(p2).y.val() < 0.0 { t.scale_y(-1.0) } else { t };

    let p1p = t.apply(p1);
    let p2p = t.apply(p2);
    let bp = p2p - p1p;
    let shear_factor = bp.x.val() / bp.y.val();

    t.shear_x(-shear_factor)
}

#[cfg(test)]
mod tests {
    use num_traits::Float;

    use crate::point::Vec2D;

    use super::*;

    #[test]
    fn test_segment_intersects_circle() {
        assert!(segment_intersects_circle_inner([Vec2D::new_world(0.0, 0.0), Vec2D::new_world(100.0, 0.0)], Vec2D::new_world(50.0, 9.0), 10.0.into()));
    }

    #[test]
    fn test_single_point_segment_intersects_circle() {
        assert!(segment_intersects_circle_inner([Vec2D::new_world(0.0, 0.0), Vec2D::new_world(0.0, 0.0)], Vec2D::new_world(0.0, 0.0), 10.0.into()));
    }

    #[test]
    fn the_expected_ellipse_is_built() {
        let f1: Vec2D<WorldUnit> = (-20.22222, -13.24752).into();
        let f2: Vec2D<WorldUnit> = (-2.88, 1.28).into();
        let sum: WorldUnit = (14.29 + 17.67).into();

        let (center, semimajor, semiminor, angle) = ellipse_from_foci_and_sum([f1, f2], sum);

        assert!(center.distance((-11.5, -5.98).into()) < 0.06.into());
        assert!((semimajor - sum / 2.0).abs() < 0.01.into());
        assert!((semiminor - 11.28.into()).abs() < 0.01.into());
        assert!((angle.degrees() - 39.94).abs() < 0.02.into());
    }

    #[test]
    fn the_expected_ellipse_is_built_back() {
        let center: Vec2D<WorldUnit> = (-11.5, -5.98).into();
        let semimajor = (14.29 + 17.67) / 2.0;
        let semiminor = 11.28;
        let angle = Angle::from_degrees(39.94);

        let ([f1, f2], sum) = foci_and_sum_from_center_and_radi(center, semimajor, semiminor, angle);

        assert!(f2.distance((-20.22222, -13.24752).into()) < 0.1.into());
        assert!(f1.distance((-2.88, 1.28).into()) < 0.1.into());
        assert!((sum - (14.29 + 17.67)).abs() < 0.1);
    }

    #[test]
    fn bezier_curve_properly_intersects_circles() {
        let from = Some(Vec2D::new_world(33.0, 135.0));
        let pt1 = Vec2D::new_world(50.0, 200.0);
        let pt2 = Vec2D::new_world(171.0, 70.0);
        let to = Vec2D::new_world(196.0, 113.0);

        let cases = [
             (Vec2D::new_world(81.0, 154.0), true),
             (Vec2D::new_world(127.0, 109.0), true),
             (Vec2D::new_world(74.0, 90.0), false),
             (Vec2D::new_world(147.0, 165.0), false),
             (Vec2D::new_world(177.0, 121.0), false),
        ];

        for (center, result) in cases {
            assert_eq!(bezier_intersects_circle(from, pt1, pt2, to, center, 15.0.into()), result);
        }
    }

    #[test]
    fn middle_point_behaves() {
        let a = Vec2D::new_world(-5.61146,3.80508);
        let b = Vec2D::new_world(-0.44,-1.28);

        assert_eq!(middle_point(a, b), (-3.0257300000000003, 1.26254).into());
    }

    #[test]
    fn angle_between_two_vectors() {
        let a: Vec2D<WorldUnit> = (-2.68,5.12).into();
        let b: Vec2D<WorldUnit> = (-4.4,2.12).into();

        assert!((Angle::between(a, b).radians() - Angle::from_degrees(36.65).radians()).abs() < 0.001);

        let a: Vec2D<WorldUnit> = (-2.68,5.12).into();
        let b: Vec2D<WorldUnit> = (-4.4,2.12).into();

        assert!((Angle::between(b, a).radians() - Angle::from_degrees(-36.65).radians()).abs() < 0.001);
    }

    #[test]
    fn angle_between_two_identical_points() {
        let a = Vec2D::new_screen(
            -38.89215087890625,
            145.26171875,
        );
        let b = Vec2D::new_screen(
            -38.89215087890625,
            145.26171875,
        );

        assert_eq!(Angle::between(a, b), Angle::from_radians(0.0));
    }

    #[test]
    fn circle_through_three_points_works() {
        let a: Vec2D<WorldUnit> = (-11.0, 4.0).into();
        let b: Vec2D<WorldUnit> = (-6.5, -1.6).into();
        let c: Vec2D<WorldUnit> = (2.44, 2.56).into();

        let (center, radius) = circle_through_three_points(a, b, c);

        assert_eq!(center, Vec2D::new_world( -4.102742498255409, 4.93440334961619 ));
        assert!((radius - 6.96.into()).abs() < 0.01.into());
    }

    #[test]
    fn ellipse_from_foci_and_point_cannot_give_flat_or_null_ellipses() {
        let p = Vec2D::new_world( -4.102742498255409, 4.93440334961619 );

        let (_, semimajor, semiminor, _) = ellipse_from_foci_and_point([p, p], p);

        assert_ne!(semimajor, 0.0.into());
        assert_ne!(semiminor, 0.0.into());

        let (_, semimajor, semiminor, _) = ellipse_from_foci_and_sum([p, p], 0.0.into());

        assert_ne!(semimajor, 0.0.into());
        assert_ne!(semiminor, 0.0.into());
    }

    #[test]
    fn adimensional_circle_doesnt_explode() {
        let p = Vec2D::new_world( -4.102742498255409, 4.93440334961619 );

        let (center, radius) = circle_through_three_points(p, p, p);

        assert_eq!(center, p);
        assert_eq!(radius, 1.0.into());
    }

    #[test]
    fn test_thirds() {
        assert_eq!(thirds(Vec2D::new_world(3.0, 0.0), Vec2D::new_world(6.0, 0.0)), (Vec2D::new_world(4.0, 0.0), Vec2D::new_world(5.0, 0.0)));
        assert_eq!(thirds(Vec2D::new_world(0.0, 0.0), Vec2D::new_world(6.0, 6.0)), (Vec2D::new_world(2.0, 2.0), Vec2D::new_world(4.0, 4.0)));
        assert_eq!(thirds(Vec2D::new_world(-15.0, 30.0), Vec2D::new_world(-18.0, 27.0)), (Vec2D::new_world(-16.0, 29.0), Vec2D::new_world(-17.0, 28.0)));
    }

    #[test]
    fn test_bisector() {
        let a: Vec2D<WorldUnit> = (-2.0, 3.0).into();
        let b: Vec2D<WorldUnit> = (1.0, 6.0).into();
        let c: Vec2D<WorldUnit> = (5.0, 2.0).into();
        let expected_bisector = Vec2D::new_world(1.0, 0.0);
        let given_bisector = bisector(a, b, c);

        assert!(expected_bisector.x - given_bisector.x.abs() < 0.001.into());
        assert!(expected_bisector.y - given_bisector.y.abs() < 0.001.into());
    }

    #[test]
    fn mirror_mirrors() {
        let j: Vec2D<WorldUnit> = (180.0, 120.0).into();
        let i: Vec2D<WorldUnit> = (160.0, 100.0).into();

        assert_eq!(mirror(j, i), (140.0, 80.0).into());
    }

    #[test]
    fn distance_to_ellipse() {
        let ellipse = Ellipse {
            center: Vec2D::new_world(-4.08, 3.07),
            semimajor: 3.05.into(),
            semiminor: 2.03.into(),
            angle: Angle::from_degrees(24.09),
        };

        let points = [
            ('G', (0.0, 3.0), 1.27),
            ('H', (-9.0, 3.0), 2.06),
            ('I', (-4.0, 6.0), 0.77),
            ('J', (-4.0, -4.0), 4.89),
            ('K', (-3.0, 3.0), 1.37),
            ('L', (-6.0, 3.0), 0.8),
            ('M', (-4.06, 1.34), 0.39),
            ('N', (-3.34, 4.9), 0.4),
        ];

        for (name, coords, expected) in points {
            let distance = distance_from_point_to_ellipse(coords.into(), ellipse);

            assert!(
                (distance - expected.into()).abs() < 0.01.into(),
                "Failed for point {}. Distance: {}, expected: {}",
                name, distance, expected,
            );
        }
    }

    #[test]
    fn can_turn_a_rombus_into_a_square() {
        let a = Vec2D::new_unitless(1.64,1.06);
        let b = Vec2D::new_unitless(6.2,1.9);
        let c = Vec2D::new_unitless(8.72,-0.82);

        // this is the tolerable error for this test
        let epsilon: Unittless = 0.004.into();

        let t1 = rombus_to_rectangle(a, b, c);
        assert!(dbg!(t1.apply(a).distance(Vec2D::new_unitless(0.0, 0.0))) < epsilon);
        assert!(dbg!(t1.apply(b).distance(Vec2D::new_unitless(4.64, 0.0))) < epsilon);
        assert!(dbg!(t1.apply(c).distance(Vec2D::new_unitless(4.64, 3.13))) < epsilon);
    }
}
