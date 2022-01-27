use crate::draw_commands::{DrawCommand, cancel_helper};
use crate::path_command::PathCommand::{self, *};
use crate::point::{Vec2D, WorldUnit, ScreenUnit};
use crate::shape::{stored::path::Path, ShapeBuilder, ShapeFinished};
use crate::style::Style;
use crate::transform::Transform;

#[derive(Debug)]
pub struct AxisAlignedRectangleBuilder {
    corners: [Vec2D<WorldUnit>; 4],
    style: Style<WorldUnit>,
}

impl AxisAlignedRectangleBuilder {
    pub fn start(initial: Vec2D<WorldUnit>, style: Style<WorldUnit>) -> AxisAlignedRectangleBuilder {
        AxisAlignedRectangleBuilder {
            corners: [initial; 4],
            style,
        }
    }

    pub fn all_four_corners(&self) -> [Vec2D<WorldUnit>; 4] {
        self.corners
    }
}

fn draw_commands(corners: [Vec2D<WorldUnit>; 4]) -> Vec<PathCommand<WorldUnit>> {
    let mut commands = Vec::with_capacity(5);
    let mut points_iter = corners.iter().cycle().take(5);

    commands.push(MoveTo(*points_iter.next().unwrap()));
    commands.extend(points_iter.map(|p| LineTo(*p)));

    commands
}

impl ShapeBuilder for AxisAlignedRectangleBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2D<ScreenUnit>, t: Transform, _snap: ScreenUnit) {
        let first_point = self.corners[0];
        let first_point_screen = t.to_screen_coordinates(self.corners[0]);

        self.corners = [
            first_point,
            t.to_world_coordinates(Vec2D::new(pos.x, first_point_screen.y)),
            t.to_world_coordinates(pos),
            t.to_world_coordinates(Vec2D::new(first_point_screen.x, pos.y)),
        ];
    }

    fn handle_button_pressed(&mut self, _pos: Vec2D<ScreenUnit>, _t: Transform, _snap: ScreenUnit) { }

    fn handle_button_released(&mut self, pos: Vec2D<ScreenUnit>, t: Transform, snap: ScreenUnit) -> ShapeFinished {
        self.handle_mouse_moved(pos, t, snap);

        let first_point = self.corners[0];

        if t.to_screen_coordinates(first_point).distance(pos) < snap {
            return ShapeFinished::Cancelled;
        }

        ShapeFinished::Yes(vec![Box::new(Path::from_parts(
            draw_commands(self.all_four_corners()), self.style,
        ))])
    }

    fn draw_commands(&self, t: Transform, snap: ScreenUnit) -> Vec<DrawCommand> {
        let corners = self.all_four_corners();

        vec![
            DrawCommand::Path {
                commands: draw_commands(corners),
                style: self.style,
            },

            cancel_helper(corners[0], corners[2], t, snap),
        ]
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const SNAP: ScreenUnit = ScreenUnit::from_float(1.0);

    #[test]
    fn finished_shape_has_all_sides() {
        let a: Vec2D<ScreenUnit> = (4.0, 5.0).into();
        let b: Vec2D<ScreenUnit> = (10.0, 10.0).into();
        let t: Transform = Default::default();

        let mut rect = AxisAlignedRectangleBuilder::start(t.to_world_coordinates(a), Default::default());

        rect.handle_mouse_moved(b, t, SNAP);

        // while being drawn
        assert_eq!(rect.draw_commands(t, SNAP)[0], DrawCommand::Path {
            style: Default::default(),
            commands: vec![
                MoveTo(Vec2D::new_world(4.0, 5.0)),
                LineTo(Vec2D::new_world(10.0, 5.0)),
                LineTo(Vec2D::new_world(10.0, 10.0)),
                LineTo(Vec2D::new_world(4.0, 10.0)),
                LineTo(Vec2D::new_world(4.0, 5.0)),
            ],
        });

        // while finished
        match rect.handle_button_released(b, t, SNAP) {
            ShapeFinished::Yes(s) => {
                assert_eq!(s[0].draw_commands(), DrawCommand::Path {
                    style: Default::default(),
                    commands: vec![
                        MoveTo(Vec2D::new_world(4.0, 5.0)),
                        LineTo(Vec2D::new_world(10.0, 5.0)),
                        LineTo(Vec2D::new_world(10.0, 10.0)),
                        LineTo(Vec2D::new_world(4.0, 10.0)),
                        LineTo(Vec2D::new_world(4.0, 5.0)),
                    ],
                });
            }
            _ => panic!(),
        }
    }

    #[test]
    fn very_tall_and_skinny_rectangle_doesnt_paint_helper_red() {
        let t = Default::default();
        let mut rect = AxisAlignedRectangleBuilder::start((0.0, 0.0).into(), Default::default());

        rect.handle_mouse_moved((0.0, 50.0).into(), t, SNAP);

        assert_eq!(rect.draw_commands(t, SNAP), vec![
            DrawCommand::Path {
                commands: vec![
                    MoveTo((0.0, 0.0).into()),
                    LineTo((0.0, 0.0).into()),
                    LineTo((0.0, 50.0).into()),
                    LineTo((0.0, 50.0).into()),
                    LineTo((0.0, 0.0).into()),
                ],
                style: Default::default(),
            },
            DrawCommand::ScreenCircle {
                center: (0.0, 0.0).into(),
                radius: SNAP,
                style: Style::circle_helper(),
            },
        ]);
    }
}
