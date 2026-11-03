use crate::draw_commands::DrawCommand;
use crate::geom::{bisector, thirds};
use crate::path_command::{CubicBezierCurve, PathCommand};
use crate::point::{ScreenUnit, Unit, Vec2D, WorldUnit};
use crate::shape::stored::path::Path;
use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::style::Style;
use crate::transform::Transform;

/// Given the last two commands of a path and a new point given as input compute
/// what the last two commands of the path should be.
///
/// This means that the command at the last position before the operation must
/// be replaced with the one provided first by this function.
fn add_smooth<T: Unit + 'static>(
    sec: Option<PathCommand<T>>,
    last: PathCommand<T>,
    pos: Vec2D<T>,
) -> (PathCommand<T>, Option<PathCommand<T>>) {
    match last {
        PathCommand::MoveTo(p) | PathCommand::LineTo(p) => {
            // Second point, put pt1 and pt2 at 1/3 and 2/3 of the
            // segment, set `to` to p.
            if p == pos {
                (last, None)
            } else {
                let (pt1, pt2) = thirds(p, pos);

                (
                    last,
                    Some(PathCommand::CurveTo(CubicBezierCurve { pt1, pt2, to: pos })),
                )
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

                let fixed_prev_handle = bis * (c.to - prev).magnitude() / -3.0 + c.to;
                let new_pt1 = bis * (pos - c.to).magnitude() / 3.0 + c.to;
                let new_pt2 = (pos - c.to) * 2.0 / 3.0 + c.to;

                if fixed_prev_handle.is_nan() || new_pt1.is_nan() || new_pt2.is_nan() {
                    (last, None)
                } else {
                    (
                        PathCommand::CurveTo(CubicBezierCurve {
                            pt2: fixed_prev_handle,
                            ..c
                        }),
                        Some(PathCommand::CurveTo(CubicBezierCurve {
                            pt1: new_pt1,
                            pt2: new_pt2,
                            to: pos,
                        })),
                    )
                }
            }
        }
    }
}

fn add_point<T: Unit + 'static>(commands: &mut Vec<PathCommand<T>>, pos: Vec2D<T>) {
    let sub = commands
        .len()
        .checked_sub(2)
        .and_then(|index| commands.get(index))
        .copied();
    let last = commands.pop().unwrap();

    let (last, new) = add_smooth(sub, last, pos);

    commands.push(last);

    if let Some(new) = new {
        commands.push(new);
    }
}

#[derive(Debug, PartialEq)]
pub struct FreeHandLine {
    commands: Vec<PathCommand<WorldUnit>>,
    style: Style<WorldUnit>,
}

impl FreeHandLine {
    pub fn start(initial: Vec2D<WorldUnit>, style: Style<WorldUnit>) -> FreeHandLine {
        let mut commands = Vec::with_capacity(1000);

        commands.push(PathCommand::MoveTo(initial));

        FreeHandLine { commands, style }
    }

    pub fn with_params(
        commands: Vec<PathCommand<WorldUnit>>,
        style: Style<WorldUnit>,
    ) -> FreeHandLine {
        FreeHandLine { commands, style }
    }
}

impl ShapeBuilder for FreeHandLine {
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
    fn handle_mouse_moved(&mut self, pos: Vec2D<ScreenUnit>, t: Transform, _snap: ScreenUnit) {
        add_point(&mut self.commands, t.to_world_coordinates(pos));
    }

    fn handle_button_pressed(&mut self, _pos: Vec2D<ScreenUnit>, _t: Transform, _snap: ScreenUnit) {
    }

    fn handle_button_released(
        &mut self,
        _pos: Vec2D<ScreenUnit>,
        _t: Transform,
        _snap: ScreenUnit,
    ) -> ShapeFinished {
        ShapeFinished::Yes(vec![Box::new(Path::from_parts(
            self.commands.clone(),
            self.style,
        ))])
    }

    fn draw_commands(&self, _t: Transform, _snap: ScreenUnit) -> Vec<DrawCommand> {
        vec![DrawCommand::Path {
            commands: self.commands.clone(),
            style: self.style,
        }]
    }
}
