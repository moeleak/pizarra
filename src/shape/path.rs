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
fn add_smooth(sec: Option<PathCommand>, last: PathCommand, pos: Point) -> (PathCommand, PathCommand) {
    match last {
        PathCommand::MoveTo(p) => {
            // Second point, put pt1 and pt2 at 1/3 and 2/3 of the
            // segment, set `to` to p.
            let (pt1, pt2) = thirds(p, pos);

            (last, PathCommand::CurveTo(CubicBezierCurve {
                pt1,
                pt2,
                to: pos,
            }))
        }
        PathCommand::CurveTo(c) => {
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

            if bis.is_nan() {
                dbg!(prev, c.to, pos);
            }

            let fixed_prev_handle = bis * (c.to - prev).magnitude()/-3.0 + prev;
            let new_pt1 = bis * (pos - c.to).magnitude()/3.0 + prev;

            (PathCommand::CurveTo(CubicBezierCurve {
                pt2: fixed_prev_handle,
                ..c
            }), PathCommand::CurveTo(CubicBezierCurve {
                pt1: new_pt1,
                pt2: (pos - c.to) * 2.0/3.0 + c.to,
                to: pos,
            }))
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
    commands.push(new);
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
    use super::{thirds, bisector};
    use crate::point::Point;

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
}
