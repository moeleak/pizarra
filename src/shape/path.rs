#![allow(non_snake_case)]
use std::fmt;

use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::ShapeType;
use crate::point::{Point, Vec2DDrawing, Vec2DScreen};
use crate::transform::Transform;

use super::{ShapeBuilder, ShapeStored, ShapeFinished};

/// Given two points return the points in their 1/3 and 2/3 of the segment from
/// a to b.
fn thirds(a: Vec2DDrawing, b: Vec2DDrawing) -> (Vec2DDrawing, Vec2DDrawing) {
    let v = b - a;

    (v*1.0/3.0 + a, v*2.0/3.0 + a)
}

/// return a vector of magnitude 1 that has the slope of the external bisector
/// of the angle described by the three given points.
///
/// B is the vertex of the angle. The returned vector points towads C
fn bisector(A: Vec2DDrawing, B: Vec2DDrawing, C: Vec2DDrawing) -> Vec2DDrawing {
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
fn add_smooth(sec: Option<PathCommand>, last: PathCommand, pos: Vec2DDrawing) -> (PathCommand, Option<PathCommand>) {
    match last {
        PathCommand::MoveTo(p) | PathCommand::LineTo(p) => {
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

fn add_point(commands: &mut Vec<PathCommand>, pos: Vec2DDrawing) {
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
    pub pt1: Vec2DDrawing,
    /// The (x, y) coordinates of the second control point.
    pub pt2: Vec2DDrawing,
    /// The (x, y) coordinates of the end point of this path segment.
    pub to: Vec2DDrawing,
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
    MoveTo(Vec2DDrawing),
    LineTo(Vec2DDrawing),
    CurveTo(CubicBezierCurve),
}

impl PathCommand {
    pub fn is_nan(&self) -> bool {
        match self {
            PathCommand::MoveTo(p) => p.is_nan(),
            PathCommand::LineTo(p) => p.is_nan(),
            PathCommand::CurveTo(c) => c.is_nan(),
        }
    }

    fn min_x(&self) -> f64 {
        match self {
            PathCommand::MoveTo(p) => p.x,
            PathCommand::LineTo(p) => p.x,
            PathCommand::CurveTo(c) => c.pt1.x.min(c.pt2.x).min(c.to.x),
        }
    }

    fn min_y(&self) -> f64 {
        match self {
            PathCommand::MoveTo(p) => p.y,
            PathCommand::LineTo(p) => p.y,
            PathCommand::CurveTo(c) => c.pt1.y.min(c.pt2.y).min(c.to.y),
        }
    }

    fn max_x(&self) -> f64 {
        match self {
            PathCommand::MoveTo(p) => p.x,
            PathCommand::LineTo(p) => p.x,
            PathCommand::CurveTo(c) => c.pt1.x.max(c.pt2.x).max(c.to.x),
        }
    }

    fn max_y(&self) -> f64 {
        match self {
            PathCommand::MoveTo(p) => p.y,
            PathCommand::LineTo(p) => p.y,
            PathCommand::CurveTo(c) => c.pt1.y.max(c.pt2.y).max(c.to.y),
        }
    }

    fn distance(&self, p: Vec2DDrawing) -> f64 {
        self.point().distance(p)
    }

    pub fn point(&self) -> Vec2DDrawing {
        match *self {
            PathCommand::MoveTo(p) => p,
            PathCommand::LineTo(p) => p,
            PathCommand::CurveTo(c) => c.to,
        }
    }

    pub fn move_to(&self) -> Option<Vec2DDrawing> {
        match self {
            PathCommand::MoveTo(p) => Some(*p),
            PathCommand::LineTo(p) => Some(*p),
            PathCommand::CurveTo(_) => None,
        }
    }

    pub fn curve_to(&self) -> Option<CubicBezierCurve> {
        match self {
            PathCommand::MoveTo(_) => None,
            PathCommand::LineTo(_) => None,
            PathCommand::CurveTo(c) => Some(*c),
        }
    }
}

impl fmt::Display for PathCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathCommand::MoveTo(p) => write!(f, "M {} ", p),
            PathCommand::LineTo(p) => write!(f, "L {} ", p),
            PathCommand::CurveTo(c) => write!(f, "C {}, {}, {} ", c.pt1, c.pt2, c.to),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PathBuilder {
    commands: Vec<PathCommand>,
    thickness: f64,
    color: Color,
}

impl PathBuilder {
    pub fn start(color: Color, initial: Vec2DDrawing, thickness: f64) -> PathBuilder {
        let mut commands = Vec::with_capacity(1000);

        commands.push(PathCommand::MoveTo(initial));

        PathBuilder {
            commands,
            thickness,
            color,
        }
    }

    pub fn with_params(color: Color, commands: Vec<PathCommand>, thickness: f64) -> PathBuilder {
        PathBuilder {
            commands, color, thickness,
        }
    }
}

impl ShapeBuilder for PathBuilder {
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
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform) {
        add_point(&mut self.commands, t.to_storage_coordinates(pos));
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform) { }

    fn handle_button_released(&mut self, _pos: Vec2DScreen, _t: Transform) -> ShapeFinished {
        ShapeFinished::Yes(Box::new(Path::from_parts(
            self.commands.clone(), self.color, self.thickness,
        )))
    }

    fn draw_commands(&self) -> Vec<DrawCommand> {
        vec![DrawCommand::Path {
            commands: self.commands.clone(),
            thickness: self.thickness,
            color: self.color,
        }]
    }
}

#[derive(Debug)]
pub struct Path {
    commands: Vec<PathCommand>,
    color: Color,
    thickness: f64,
}

impl Path {
    #[cfg(test)]
    pub fn new() -> Path {
        Path {
            commands: Vec::new(),
            color: Default::default(),
            thickness: 1.0,
        }
    }

    pub fn from_parts(commands: Vec<PathCommand>, color: Color, thickness: f64) -> Path {
        Path {
            commands, color, thickness,
        }
    }
}

impl ShapeStored for Path {
    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Path {
            commands: self.commands.clone(),
            thickness: self.thickness,
            color: self.color,
        }
    }

    // The bounding box of the path needs to consider the beizer handles
    fn bbox(&self) -> [Vec2DDrawing; 2] {
        let bbox = self.commands.iter().fold([
            Vec2DDrawing::new(std::f64::INFINITY, std::f64::INFINITY), // bottom-left corner
            Vec2DDrawing::new(std::f64::NEG_INFINITY, std::f64::NEG_INFINITY), // top right corner
        ], |acc, cmd| {
            [
                Vec2DDrawing::new(
                    cmd.min_x().min(acc[0].x),
                    cmd.min_y().min(acc[0].y),
                ),
                Vec2DDrawing::new(
                    cmd.max_x().max(acc[1].x),
                    cmd.max_y().max(acc[1].y),
                ),
            ]
        });

        assert_ne!(bbox[0].x, std::f64::INFINITY);
        assert_ne!(bbox[0].y, std::f64::INFINITY);
        assert_ne!(bbox[1].x, std::f64::NEG_INFINITY);
        assert_ne!(bbox[1].y, std::f64::NEG_INFINITY);

        bbox
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Path
    }

    fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool {
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
    use super::*;

    use crate::color::Color;
    use crate::point::Vec2DDrawing;

    #[test]
    fn test_thirds() {
        assert_eq!(thirds(Vec2DDrawing::new(3.0, 0.0), Vec2DDrawing::new(6.0, 0.0)), (Vec2DDrawing::new(4.0, 0.0), Vec2DDrawing::new(5.0, 0.0)));
        assert_eq!(thirds(Vec2DDrawing::new(0.0, 0.0), Vec2DDrawing::new(6.0, 6.0)), (Vec2DDrawing::new(2.0, 2.0), Vec2DDrawing::new(4.0, 4.0)));
        assert_eq!(thirds(Vec2DDrawing::new(-15.0, 30.0), Vec2DDrawing::new(-18.0, 27.0)), (Vec2DDrawing::new(-16.0, 29.0), Vec2DDrawing::new(-17.0, 28.0)));
    }

    #[test]
    fn test_bisector() {
        let a = (-2.0, 3.0).into();
        let b = (1.0, 6.0).into();
        let c = (5.0, 2.0).into();
        let expected_bisector = Vec2DDrawing::new(1.0, 0.0);
        let given_bisector = bisector(a, b, c);

        assert!(expected_bisector.x - given_bisector.x.abs() < 0.001);
        assert!(expected_bisector.y - given_bisector.y.abs() < 0.001);
    }

    #[test]
    fn test_bbox() {
        let line = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DDrawing::new(1.0, 0.0)),
            PathCommand::LineTo(Vec2DDrawing::new(0.0, 1.0)),
        ], Color::green(), 4.0);

        assert_eq!(line.bbox(), [Vec2DDrawing::new(0.0, 0.0), Vec2DDrawing::new(1.0, 1.0)]);
    }

    #[test]
    fn test_bbox_twisted_line() {
        let line = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DDrawing::new(-12.0, -1.0)),
            PathCommand::LineTo(Vec2DDrawing::new(-5.0, 0.0)),
            PathCommand::LineTo(Vec2DDrawing::new(-2.0, 7.0)),
            PathCommand::LineTo(Vec2DDrawing::new(2.0, -8.0)),
        ], Color::green(), 1.0);

        assert_eq!(line.bbox(), [Vec2DDrawing::new(-12.0, -8.0), Vec2DDrawing::new(3.161263886380917, 7.182987048373279)]);
    }
}
