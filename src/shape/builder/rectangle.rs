use crate::draw_commands::{circle_helper, DrawCommand};
use crate::geom::middle_point;
use crate::path_command::PathCommand;
use crate::point::{Point, Vec2DScreen, Vec2DWorld};
use crate::shape::{stored::path::Path, ShapeBuilder, ShapeFinished};
use crate::style::Style;
use crate::transform::Transform;

#[derive(Debug)]
pub struct AxisAlignedRectangleBuilder {
    corners: [Vec2DWorld; 4],
    style: Style,
}

impl AxisAlignedRectangleBuilder {
    pub fn start(initial: Vec2DWorld, style: Style) -> AxisAlignedRectangleBuilder {
        AxisAlignedRectangleBuilder {
            corners: [initial; 4],
            style,
        }
    }

    pub fn all_four_corners(&self) -> [Vec2DWorld; 4] {
        self.corners
    }
}

fn draw_commands(corners: [Vec2DWorld; 4]) -> Vec<PathCommand<Vec2DWorld>> {
    let mut commands = Vec::with_capacity(5);
    let mut points_iter = corners.iter().cycle().take(5);

    commands.push(PathCommand::MoveTo(*points_iter.next().unwrap()));
    commands.extend(points_iter.map(|p| PathCommand::LineTo(*p)));

    commands
}

impl ShapeBuilder for AxisAlignedRectangleBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) {
        let first_point = self.corners[0];
        let first_point_screen = t.to_screen_coordinates(self.corners[0]);

        self.corners = [
            first_point,
            t.to_world_coordinates(Vec2DScreen::new(pos.x, first_point_screen.y)),
            t.to_world_coordinates(pos),
            t.to_world_coordinates(Vec2DScreen::new(first_point_screen.x, pos.y)),
        ];
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform, _snap: f64) { }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform, snap: f64) -> ShapeFinished {
        self.handle_mouse_moved(pos, t, snap);

        let first_point = self.corners[0];
        let pos_new = t.to_world_coordinates(pos);
        if first_point.distance(pos_new) < snap {
            return ShapeFinished::Cancelled;
        }

        ShapeFinished::Yes(Box::new(Path::from_parts(
            draw_commands(self.all_four_corners()), self.style,
        )))
    }

    fn draw_commands(&self, t: Transform, snap: f64) -> Vec<DrawCommand> {
        let corners = self.all_four_corners();
        let center = middle_point(corners[0], corners[2]);

        vec![
            DrawCommand::Path {
                commands: draw_commands(corners),
                style: self.style,
            },

            circle_helper(t.to_screen_coordinates(center), snap),
        ]
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const SNAP: f64 = 10.0;

    #[test]
    fn finished_shape_has_all_sides() {
        let a: Vec2DScreen = (4.0, 5.0).into();
        let b: Vec2DScreen = (10.0, 10.0).into();
        let t: Transform = Default::default();

        let mut rect = AxisAlignedRectangleBuilder::start(t.to_world_coordinates(a), Default::default());

        rect.handle_mouse_moved(b, t, SNAP);

        // while being drawn
        assert_eq!(rect.draw_commands(t, SNAP)[0], DrawCommand::Path {
            style: Default::default(),
            commands: vec![
                PathCommand::MoveTo(Vec2DWorld::new(4.0, 5.0)),
                PathCommand::LineTo(Vec2DWorld::new(10.0, 5.0)),
                PathCommand::LineTo(Vec2DWorld::new(10.0, 10.0)),
                PathCommand::LineTo(Vec2DWorld::new(4.0, 10.0)),
                PathCommand::LineTo(Vec2DWorld::new(4.0, 5.0)),
            ],
        });

        // while finished
        match rect.handle_button_released(b, t, SNAP) {
            ShapeFinished::Yes(s) => {
                assert_eq!(s.draw_commands(), DrawCommand::Path {
                    style: Default::default(),
                    commands: vec![
                        PathCommand::MoveTo(Vec2DWorld::new(4.0, 5.0)),
                        PathCommand::LineTo(Vec2DWorld::new(10.0, 5.0)),
                        PathCommand::LineTo(Vec2DWorld::new(10.0, 10.0)),
                        PathCommand::LineTo(Vec2DWorld::new(4.0, 10.0)),
                        PathCommand::LineTo(Vec2DWorld::new(4.0, 5.0)),
                    ],
                });
            }
            ShapeFinished::No => panic!(),
            ShapeFinished::Cancelled => panic!(),
        }
    }
}
