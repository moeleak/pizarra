use crate::point::{Point, Vec2DDrawing, Vec2DScreen};
use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::transform::Transform;

use super::{ShapeBuilder, ShapeFinished, path::{PathCommand, Path}};

pub struct RectangleBuilder {
    corners: [Vec2DDrawing; 4],
    stroke: f64,
    color: Color,
}

impl RectangleBuilder {
    pub fn start(color: Color, initial: Vec2DDrawing, stroke: f64) -> RectangleBuilder {
        RectangleBuilder {
            corners: [initial; 4],
            stroke,
            color,
        }
    }

    pub fn all_four_corners(&self) -> [Vec2DDrawing; 4] {
        self.corners
    }
}

impl ShapeBuilder for RectangleBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform) {
        let first_point = self.corners[0];
        let first_point_screen = t.to_screen_coordinates(self.corners[0]);

        self.corners = [
            first_point,
            t.to_storage_coordinates(Vec2DScreen::new(pos.x, first_point_screen.y)),
            t.to_storage_coordinates(pos),
            t.to_storage_coordinates(Vec2DScreen::new(first_point_screen.x, pos.y)),
        ];
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform) { }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform) -> ShapeFinished {
        self.handle_mouse_moved(pos, t);

        let commands = self.corners.iter().enumerate().map(|(i, &c)| {
            if i == 0 {
                PathCommand::MoveTo(c)
            } else {
                PathCommand::LineTo(c)
            }
        }).collect();

        ShapeFinished::Yes(Box::new(Path::from_parts(
            commands, self.color, self.stroke,
        )))
    }

    fn draw_commands(&self) -> Vec<DrawCommand> {
        let corners = self.all_four_corners();
        let mut points_iter = corners.iter().cycle().take(5);
        let mut commands = Vec::with_capacity(4);

        commands.push(PathCommand::MoveTo(*points_iter.next().unwrap()));
        commands.extend(points_iter.map(|p| PathCommand::LineTo(*p)));

        vec![DrawCommand::Path {
            commands,
            thickness: self.stroke,
            color: self.color,
        }]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn three_point_rectangle_builder() {
        assert!(false, "test whatever way of building a rectangle I decide");
    }
}
