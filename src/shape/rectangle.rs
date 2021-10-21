use crate::point::{Point, Vec2DWorld, Vec2DScreen};
use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::transform::Transform;

use super::{ShapeBuilder, ShapeFinished, path::{PathCommand, Path}};

#[derive(Debug)]
pub struct AxisAlignedRectangleBuilder {
    corners: [Vec2DWorld; 4],
    stroke: f64,
    color: Color,
}

impl AxisAlignedRectangleBuilder {
    pub fn start(color: Color, initial: Vec2DWorld, stroke: f64) -> AxisAlignedRectangleBuilder {
        AxisAlignedRectangleBuilder {
            corners: [initial; 4],
            stroke,
            color,
        }
    }

    pub fn all_four_corners(&self) -> [Vec2DWorld; 4] {
        self.corners
    }
}

fn draw_commands(corners: [Vec2DWorld; 4]) -> Vec<PathCommand> {
    let mut commands = Vec::with_capacity(5);
    let mut points_iter = corners.iter().cycle().take(5);

    commands.push(PathCommand::MoveTo(*points_iter.next().unwrap()));
    commands.extend(points_iter.map(|p| PathCommand::LineTo(*p)));

    commands
}

impl ShapeBuilder for AxisAlignedRectangleBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform) {
        let first_point = self.corners[0];
        let first_point_screen = t.to_screen_coordinates(self.corners[0]);

        self.corners = [
            first_point,
            t.to_world_coordinates(Vec2DScreen::new(pos.x, first_point_screen.y)),
            t.to_world_coordinates(pos),
            t.to_world_coordinates(Vec2DScreen::new(first_point_screen.x, pos.y)),
        ];
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform) { }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform) -> ShapeFinished {
        self.handle_mouse_moved(pos, t);

        ShapeFinished::Yes(Box::new(Path::from_parts(
            draw_commands(self.all_four_corners()), self.color, self.stroke,
        )))
    }

    fn draw_commands(&self) -> Vec<DrawCommand> {
        let corners = self.all_four_corners();

        vec![DrawCommand::Path {
            commands: draw_commands(corners),
            thickness: self.stroke,
            color: self.color,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finished_shape_has_all_sides() {
        let a: Vec2DScreen = (4.0, 5.0).into();
        let b: Vec2DScreen = (10.0, 10.0).into();
        let t: Transform = Default::default();

        let mut rect = AxisAlignedRectangleBuilder::start(Default::default(), t.to_world_coordinates(a), 1.0);

        rect.handle_mouse_moved(b, t);

        // while being drawn
        assert_eq!(rect.draw_commands()[0], DrawCommand::Path {
            color: Default::default(),
            thickness: 1.0,
            commands: vec![
                PathCommand::MoveTo(Vec2DWorld::new(4.0, 5.0)),
                PathCommand::LineTo(Vec2DWorld::new(10.0, 5.0)),
                PathCommand::LineTo(Vec2DWorld::new(10.0, 10.0)),
                PathCommand::LineTo(Vec2DWorld::new(4.0, 10.0)),
                PathCommand::LineTo(Vec2DWorld::new(4.0, 5.0)),
            ],
        });

        // while finished
        match rect.handle_button_released(b, t) {
            ShapeFinished::Yes(s) => {
                assert_eq!(s.draw_commands(), DrawCommand::Path {
                    color: Default::default(),
                    thickness: 1.0,
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
        }
    }
}
