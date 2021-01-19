#![allow(non_snake_case)]
use std::fmt;

use super::{ShapeTrait, ShapeFinished};
use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::ShapeType;
use crate::point::Point;

/// Given two points return the points in their 1/3 and 2/3 of the segment from
/// a to b.
fn thirds(a: Point, b: Point) -> (Point, Point) {
    let v = b - a;

    (v*1.0/3.0 + a, v*2.0/3.0 + a)
}

/// return a vector of magnitude 1 that has the slope of the external bisector
/// of the angle described by the three given points.
///
/// B is the vertex of the angle. The returned vector points towads C
fn bisector(A: Point, B: Point, C: Point) -> Point {
    let Ap = (A - B) * -1.0 + B;
    let (a, b, c) = (B.distance(C), A.distance(C), A.distance(B));
    let incenter = (Ap * a + B * b + C * c) / (a + b + c);

    let vec = incenter - B;

    vec / vec.magnitude()
}

/// Given the last two commands of a path and a new point given as input compute
/// what the last two commands of the path should be.
///
/// This means that the command at the last position before the operation must
/// be replaced with the one provided first by this function.
fn add_smooth(sec: Option<PathCommand>, last: PathCommand, pos: Point) -> (PathCommand, Option<PathCommand>) {
    match last {
        PathCommand::MoveTo(p) => {
            // Second point, put pt1 and pt2 at 1/3 and 2/3 of the
            // segment, set `to` to p.
            if p == pos {
                (last, None)
            } else {
                let (pt1, pt2) = thirds(p, pos);

                (last, Some(PathCommand::CurveTo(CubicBezierCurve {
                    pt1,
                    pt2,
                    to: pos,
                })))
            }
        }
        PathCommand::CurveTo(c) => {
            if c.to == pos {
                (last, None)
            } else {
                // calcula la bisectriz entre el punto anteanterior, el
                // anterior y este. Calcula la perpendicular a esa bisectriz
                // que pasa por el punto anterior. Pon el manejador de la
                // curva anterior y el primer manejador de esta curva en esa
                // recta.
                //
                // El Segundo manejador va en el segmento que conecta los
                // dos últimos puntos
                let prev = sec.unwrap().point();
                let bis = bisector(prev, c.to, pos);

                let fixed_prev_handle = bis * (c.to - prev).magnitude()/-3.0 + c.to;
                let new_pt1 = bis * (pos - c.to).magnitude()/3.0 + c.to;
                let new_pt2 = (pos - c.to) * 2.0/3.0 + c.to;

                if fixed_prev_handle.is_nan() || new_pt1.is_nan() || new_pt2.is_nan() {
                    (last, None)
                } else {
                    (PathCommand::CurveTo(CubicBezierCurve {
                        pt2: fixed_prev_handle,
                        ..c
                    }), Some(PathCommand::CurveTo(CubicBezierCurve {
                        pt1: new_pt1,
                        pt2: new_pt2,
                        to: pos,
                    })))
                }
            }
        }
    }
}

fn add_point(commands: &mut Vec<PathCommand>, pos: Point) {
    let sub = commands
        .len()
        .checked_sub(2)
        .map(|index| commands.get(index))
        .flatten()
        .copied();
    let last = commands.pop().unwrap();

    let (last, new) = add_smooth(sub, last, pos);

    commands.push(last);

    if let Some(new) = new {
        commands.push(new);
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct CubicBezierCurve {
    /// The (x, y) coordinates of the first control point.
    pub pt1: Point,
    /// The (x, y) coordinates of the second control point.
    pub pt2: Point,
    /// The (x, y) coordinates of the end point of this path segment.
    pub to: Point,
}

impl CubicBezierCurve {
    pub fn is_nan(&self) -> bool {
        self.pt1.is_nan() || self.pt2.is_nan() || self.to.is_nan()
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Sweep {
    Negative,
    Positive,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PathCommand {
    MoveTo(Point),
    CurveTo(CubicBezierCurve),
}

impl PathCommand {
    pub fn is_nan(&self) -> bool {
        match self {
            PathCommand::MoveTo(p) => p.is_nan(),
            PathCommand::CurveTo(c) => c.is_nan(),
        }
    }

    fn min_x(&self) -> f64 {
        match self {
            PathCommand::MoveTo(p) => p.x,
            PathCommand::CurveTo(c) => c.pt1.x.min(c.pt2.x).min(c.to.x),
        }
    }

    fn min_y(&self) -> f64 {
        match self {
            PathCommand::MoveTo(p) => p.y,
            PathCommand::CurveTo(c) => c.pt1.y.min(c.pt2.y).min(c.to.y),
        }
    }

    fn max_x(&self) -> f64 {
        match self {
            PathCommand::MoveTo(p) => p.x,
            PathCommand::CurveTo(c) => c.pt1.x.max(c.pt2.x).max(c.to.x),
        }
    }

    fn max_y(&self) -> f64 {
        match self {
            PathCommand::MoveTo(p) => p.y,
            PathCommand::CurveTo(c) => c.pt1.y.max(c.pt2.y).max(c.to.y),
        }
    }

    fn distance(&self, p: Point) -> f64 {
        self.point().distance(p)
    }

    pub fn point(&self) -> Point {
        match *self {
            PathCommand::MoveTo(p) => p,
            PathCommand::CurveTo(c) => c.to,
        }
    }

    pub fn move_to(&self) -> Option<Point> {
        match self {
            PathCommand::MoveTo(p) => Some(*p),
            PathCommand::CurveTo(_) => None,
        }
    }

    pub fn curve_to(&self) -> Option<CubicBezierCurve> {
        match self {
            PathCommand::MoveTo(_) => None,
            PathCommand::CurveTo(c) => Some(*c),
        }
    }
}

impl fmt::Display for PathCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathCommand::MoveTo(p) => write!(f, "M {} ", p),
            PathCommand::CurveTo(c) => write!(f, "C {}, {}, {} ", c.pt1, c.pt2, c.to),
        }
    }
}

pub struct Path {
    commands: Vec<PathCommand>,
    thickness: f64,
    color: Color,
}

impl Path {
    pub fn new(color: Color, initial: Point, thickness: f64) -> Path {
        let mut init_vec = Vec::with_capacity(1000);

        init_vec.push(PathCommand::MoveTo(initial));

        Path {
            commands: init_vec,
            thickness,
            color,
        }
    }
}

impl ShapeTrait for Path {
    // 1. The first point is an absolute move.
    //
    // 2. The second point is a flat curve, with the handles put on the line
    //    that joins it with the previous one and its lengths set to 1/3 of the
    //    distance between the two points.
    //
    // 3. Every new point fixes the previous one, setting the slope of its last
    //    control point respecting its length. It sets correctly the length of
    //    its two control points but the last one sits in the segment between
    //    the last two points.
    fn handle_mouse_moved(&mut self, pos: Point) {
        add_point(&mut self.commands, pos);
    }

    fn handle_button_pressed(&mut self, _pos: Point) { }

    fn handle_button_released(&mut self, _pos: Point) -> ShapeFinished {
        ShapeFinished::Yes
    }

    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Path {
            line: self.commands.clone(),
            thickness: self.thickness,
            color: self.color,
        }
    }

    // The bounding box of the path needs to consider the beizer handles
    fn bbox(&self) -> [[f64; 2]; 2] {
        let bbox = self.commands.iter().fold([
            [std::f64::INFINITY, std::f64::INFINITY], // bottom-left corner
            [std::f64::NEG_INFINITY, std::f64::NEG_INFINITY], // top right corner
        ], |acc, cmd| {
            [
                [
                    cmd.min_x().min(acc[0][0]),
                    cmd.min_y().min(acc[0][1]),
                ],
                [
                    cmd.max_x().max(acc[1][0]),
                    cmd.max_y().max(acc[1][1]),
                ],
            ]
        });

        assert_ne!(bbox[0][0], std::f64::INFINITY);
        assert_ne!(bbox[0][1], std::f64::INFINITY);
        assert_ne!(bbox[1][0], std::f64::NEG_INFINITY);
        assert_ne!(bbox[1][1], std::f64::NEG_INFINITY);

        bbox
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Path
    }

    fn intersects_circle(&self, center: Point, radius: f64) -> bool {
        for c in self.commands.iter() {
            if c.distance(center) <= radius {
                return true;
            }
        }

        false
    }

    fn color(&self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::{thirds, bisector, Path, PathCommand, CubicBezierCurve};
    use crate::point::Point;
    use crate::shape::ShapeTrait;
    use crate::color::Color;
    use crate::draw_commands::DrawCommand;

    const ERROR: f64 = 0.001;

    fn is_same_variant(p1: PathCommand, p2: PathCommand) -> bool {
        use PathCommand::*;

        match (p1, p2) {
            (MoveTo(_), MoveTo(_)) => true,
            (CurveTo(_), CurveTo(_)) => true,
            _ => false,
        }
    }

    fn error(c1: CubicBezierCurve, c2: CubicBezierCurve) -> f64 {
        c1.pt1.distance(c2.pt1)
            .max(c1.pt2.distance(c2.pt2)
                 .max(c1.to.distance(c2.to)))
    }

    #[test]
    fn test_thirds() {
        assert_eq!(thirds(Point::new(3.0, 0.0), Point::new(6.0, 0.0)), (Point::new(4.0, 0.0), Point::new(5.0, 0.0)));
        assert_eq!(thirds(Point::new(0.0, 0.0), Point::new(6.0, 6.0)), (Point::new(2.0, 2.0), Point::new(4.0, 4.0)));
        assert_eq!(thirds(Point::new(-15.0, 30.0), Point::new(-18.0, 27.0)), (Point::new(-16.0, 29.0), Point::new(-17.0, 28.0)));
    }

    #[test]
    fn test_bisector() {
        let a = (-2.0, 3.0).into();
        let b = (1.0, 6.0).into();
        let c = (5.0, 2.0).into();
        let expected_bisector = Point::new(1.0, 0.0);
        let given_bisector = bisector(a, b, c);

        assert!(expected_bisector.x - given_bisector.x.abs() < 0.001);
        assert!(expected_bisector.y - given_bisector.y.abs() < 0.001);
    }

    #[test]
    fn test_path() {
        let mut path = Path::new(Color::default(), Point::new(-0.73504638671875, -107.16925048828125), 3.0);

        path.handle_mouse_moved(Point::new(-0.73504638671875, -107.16925048828125));
        path.handle_mouse_moved(Point::new(-0.73504638671875, -107.16925048828125));
        path.handle_mouse_moved(Point::new(-2.35595703125, -107.51025390625));
        path.handle_mouse_moved(Point::new(-5.59783935546875, -107.73760986328125));
        path.handle_mouse_moved(Point::new(-9.3028564453125, -107.62393188476563));
        path.handle_mouse_moved(Point::new(-15.32354736328125, -106.14620971679688));
        path.handle_mouse_moved(Point::new(-22.73358154296875, -102.84970092773438));
        path.handle_mouse_moved(Point::new(-31.06988525390625, -98.07546997070313));
        path.handle_mouse_moved(Point::new(-36.85894775390625, -92.8465576171875));
        path.handle_mouse_moved(Point::new(-38.71148681640625, -85.23049926757813));
        path.handle_mouse_moved(Point::new(-34.77490234375, -80.68362426757813));
        path.handle_mouse_moved(Point::new(-28.52264404296875, -77.15975952148438));
        path.handle_mouse_moved(Point::new(-17.176025390625, -74.65899658203125));
        path.handle_mouse_moved(Point::new(-9.534423828125, -74.65899658203125));
        path.handle_mouse_moved(Point::new(-8.37664794921875, -75.22735595703125));
        path.handle_button_released(Point::new(-8.37664794921875, -75.22735595703125));

        let commands = path.draw_commands();

        let start = Point::new(-0.73504639,-107.16925);
        let expected_commands = vec![PathCommand::MoveTo(start)];
        let relative = vec![
            (0.0,0.0, -1.07408131,-0.26467, -1.62091061,-0.341),
            (-1.0728808,-0.14975, -2.1588103,-0.20605, -3.2418824,-0.22736),
            (-1.2353478,-0.0243, -2.4807853,-0.0534, -3.705017,0.11368),
            (-2.0474706,0.27951, -4.0654836,0.80882, -6.0206906,1.47772),
            (-2.557858,0.87508, -4.998962,2.07376, -7.410035,3.29651),
            (-2.855938,1.44836, -5.743849,2.894518, -8.336303,4.77423),
            (-2.105172,1.526398, -4.457949,2.995131, -5.789063,5.228912),
            (-1.337459,2.244428, -2.46721,5.076683, -1.852539,7.616059),
            (0.471638,1.948469, 2.384362,3.278183, 3.936585,4.546875),
            (1.852311,1.513966, 4.017614,2.669746, 6.252258,3.523864),
            (3.617726,1.382755, 7.496401,2.081505, 11.346619,2.500763),
            (2.532232,0.27574, 5.160421,0.576172, 7.6416012,0.0),
            (0.4187765,-0.09725, 1.1577759,-0.568359, 1.1577759,-0.568359),
        ];

        let absolute = relative.into_iter().fold(expected_commands, |mut acc, curr| {
            let last_coord = match acc.last().unwrap() {
                PathCommand::MoveTo(p) => *p,
                PathCommand::CurveTo(c) => c.to,
            };

            acc.push(PathCommand::CurveTo(CubicBezierCurve {
                pt1: last_coord + Point::new(curr.0, curr.1),
                pt2: last_coord + Point::new(curr.2, curr.3),
                to: last_coord + Point::new(curr.4, curr.5),
            }));

            acc
        });

        if let DrawCommand::Path { line, .. } = commands {
            for (i, (given, wanted)) in line.iter().zip(absolute.iter()).enumerate() {
                assert!(is_same_variant(*given, *wanted));

                match given {
                    PathCommand::MoveTo(p) => {
                        let wanted = wanted.move_to().unwrap();
                        let error = p.distance(wanted);

                        assert!(
                            error < ERROR,
                            "Error: {} | {}. expected: {} wanted: {}",
                            error, i, p, wanted
                        )
                    },
                    PathCommand::CurveTo(c) => {
                        let wanted = wanted.curve_to().unwrap();
                        let err = error(*c, wanted);

                        assert!(
                            err < ERROR,
                            "Error: {} | {}. \ng: {:?}\nw: {:?}",
                            err, i, c, wanted,
                        );
                    },
                }
            }

            assert_eq!(line.len(), absolute.len());
        } else {
            panic!("Didn't return a path");
        }
    }
}
