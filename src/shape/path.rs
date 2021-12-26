use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::point::{Point, Vec2DWorld, Vec2DScreen};
use crate::transform::Transform;
use crate::geom::{bbox_from_points, thirds, bisector};
use crate::path_command::{PathCommand, CubicBezierCurve};

use super::{ShapeBuilder, ShapeStored, ShapeFinished, ShapeType};

/// Given the last two commands of a path and a new point given as input compute
/// what the last two commands of the path should be.
///
/// This means that the command at the last position before the operation must
/// be replaced with the one provided first by this function.
fn add_smooth<T: 'static + Point>(sec: Option<PathCommand<T>>, last: PathCommand<T>, pos: T) -> (PathCommand<T>, Option<PathCommand<T>>) {
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
                let prev = sec.unwrap().to();
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

fn add_point<T: 'static + Point>(commands: &mut Vec<PathCommand<T>>, pos: T) {
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

#[derive(Debug, PartialEq)]
pub struct PathBuilder {
    commands: Vec<PathCommand<Vec2DWorld>>,
    thickness: f64,
    color: Color,
}

impl PathBuilder {
    pub fn start(color: Color, initial: Vec2DWorld, thickness: f64) -> PathBuilder {
        let mut commands = Vec::with_capacity(1000);

        commands.push(PathCommand::MoveTo(initial));

        PathBuilder {
            commands,
            thickness,
            color,
        }
    }

    pub fn with_params(color: Color, commands: Vec<PathCommand<Vec2DWorld>>, thickness: f64) -> PathBuilder {
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
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) {
        add_point(&mut self.commands, t.to_world_coordinates(pos));
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform, _snap: f64) { }

    fn handle_button_released(&mut self, _pos: Vec2DScreen, _t: Transform, _snap: f64) -> ShapeFinished {
        ShapeFinished::Yes(Box::new(Path::from_parts(
            self.commands.clone(), Some(self.color), None, self.thickness,
        )))
    }

    fn draw_commands(&self, _t: Transform, _snap: f64) -> Vec<DrawCommand> {
        vec![DrawCommand::Path {
            commands: self.commands.clone(),
            thickness: self.thickness,
            color: Some(self.color),
            fill: None,
        }]
    }
}

#[derive(Debug)]
pub struct Path {
    commands: Vec<PathCommand<Vec2DWorld>>,
    color: Option<Color>,
    fill: Option<Color>,
    thickness: f64,
}

impl Path {
    pub fn from_parts(commands: Vec<PathCommand<Vec2DWorld>>, color: Option<Color>, fill: Option<Color>, thickness: f64) -> Path {
        Path {
            commands, color, fill, thickness,
        }
    }
}

impl ShapeStored for Path {
    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Path {
            commands: self.commands.clone(),
            thickness: self.thickness,
            color: self.color,
            fill: None,
        }
    }

    // The bounding box of the path needs to consider the beizer handles
    fn bbox(&self) -> [Vec2DWorld; 2] {
        let points = self.commands.iter().map(|c| c.points()).flatten();

        bbox_from_points(points)
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Path
    }

    fn intersects_circle(&self, center: Vec2DWorld, radius: f64) -> bool {
        let mut last_point = None;

        for c in self.commands.iter() {
            if c.intersects_circle(center, radius, last_point) {
                return true;
            }

            last_point = Some(c.to());
        }

        false
    }

    fn color(&self) -> Option<Color> {
        self.color
    }

    fn fill(&self) -> Option<Color> {
        self.fill
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::color::Color;
    use crate::point::Vec2DWorld;

    #[test]
    fn test_bbox() {
        let line = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(1.0, 0.0)),
            PathCommand::LineTo(Vec2DWorld::new(0.0, 1.0)),
        ], Some(Color::green()), None, 4.0);

        assert_eq!(line.bbox(), [Vec2DWorld::new(0.0, 0.0), Vec2DWorld::new(1.0, 1.0)]);
    }

    #[test]
    fn test_bbox_twisted_line() {
        let line = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(-12.0, -1.0)),
            PathCommand::LineTo(Vec2DWorld::new(-5.0, 0.0)),
            PathCommand::LineTo(Vec2DWorld::new(-2.0, 7.0)),
            PathCommand::LineTo(Vec2DWorld::new(2.0, -8.0)),
        ], Some(Color::green()), None, 1.0);

        assert_eq!(line.bbox(), [Vec2DWorld::new(-12.0, -8.0), Vec2DWorld::new(2.0, 7.0)]);
    }

    #[test]
    fn can_delete_single_point_paths() {
        let poly = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(0.0, 0.0)),
        ], Some(Color::green()), None, 1.0);

        assert!(poly.intersects_circle(Vec2DWorld::new(0.0, 0.0), 10.0));
    }

    #[test]
    fn can_delete_beizer_curves() {
        let path = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(33.0, 135.0)),
            PathCommand::CurveTo(CubicBezierCurve {
                pt1: Vec2DWorld::new(50.0, 200.0),
                pt2: Vec2DWorld::new(171.0, 70.0),
                to: Vec2DWorld::new(196.0, 113.0),
            }),
        ], Some(Color::green()), None, 1.0);

        let cases = [
             (Vec2DWorld::new(37.0, 129.0), true),
             (Vec2DWorld::new(81.0, 154.0), true),
             (Vec2DWorld::new(127.0, 109.0), true),
             (Vec2DWorld::new(74.0, 90.0), false),
             (Vec2DWorld::new(147.0, 165.0), false),
             (Vec2DWorld::new(177.0, 121.0), false),
        ];

        for (center, result) in cases {
            assert_eq!(path.intersects_circle(center, 15.0), result);
        }
    }

    #[test]
    fn can_delete_straight_segments() {
        let path = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(33.0, 135.0)),
            PathCommand::LineTo(Vec2DWorld::new(196.0, 113.0)),
        ], Default::default(), None, 1.0);

        let cases = [
             (Vec2DWorld::new(37.0, 129.0), true),
             (Vec2DWorld::new(81.0, 154.0), false),
             (Vec2DWorld::new(127.0, 109.0), true),
             (Vec2DWorld::new(74.0, 90.0), false),
             (Vec2DWorld::new(147.0, 165.0), false),
             (Vec2DWorld::new(177.0, 121.0), true),
        ];

        for (center, result) in cases {
            assert_eq!(path.intersects_circle(center, 15.0), result);
        }
    }
}
