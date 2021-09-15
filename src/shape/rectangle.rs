use crate::point::{Point, Vec2DDrawing, Vec2DScreen};
use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::ShapeType;
use crate::transform::Transform;
use crate::geom::{bbox_from_points, project};

use super::{ShapeBuilder, ShapeStored, ShapeFinished, path::PathCommand};

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

        ShapeFinished::Yes(Box::new(Rectangle::from_parts(
            self.corners, self.color, self.stroke,
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

#[derive(Debug)]
pub struct Rectangle {
    corners: [Vec2DDrawing; 4],
    color: Color,
    thickness: f64,
}

impl Rectangle {
    pub fn from_parts(corners: [Vec2DDrawing; 4], color: Color, thickness: f64) -> Rectangle {
        Rectangle {
            corners, color, thickness,
        }
    }
}

impl ShapeStored for Rectangle {
    fn draw_commands(&self) -> DrawCommand {
        let corners = self.corners;
        let mut points_iter = corners.iter().cycle().take(5);
        let mut commands = Vec::with_capacity(4);

        commands.push(PathCommand::MoveTo(*points_iter.next().unwrap()));
        commands.extend(points_iter.map(|p| PathCommand::LineTo(*p)));

        DrawCommand::Path {
            commands,
            thickness: self.thickness,
            color: self.color,
        }
    }

    fn bbox(&self) -> [Vec2DDrawing; 2] {
        bbox_from_points(self.corners.iter().copied())
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Rectangle
    }

    fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool {
        let intersects_corners = self.corners
            .iter()
            .map(|p| p.distance(center) < radius)
            .filter(|a| *a)
            .count() > 0;

        let intersects_sides = self.corners.iter().zip(self.corners.iter().cycle())
            .any(|(&p1, &p2)| {
                let projection = project(center, [p1, p2]);

                projection.distance(center) < radius
            });

        intersects_sides || intersects_corners
    }

    fn color(&self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox() {
        assert!(false, "test the bbox of an axis aligned rectangle and a twisted one");
    }
}
