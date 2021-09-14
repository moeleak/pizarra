use crate::point::{Point, Vec2DDrawing, Vec2DScreen};
use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::ShapeType;
use crate::transform::Transform;
use crate::geom::{bbox_from_points, project};

use super::{ShapeTrait, ShapeFinished, path::PathCommand};

pub struct Rectangle {
    corners: [Vec2DDrawing; 4],
    stroke: f64,
    color: Color,
}

impl Rectangle {
    pub fn start(color: Color, initial: Vec2DDrawing, stroke: f64) -> Rectangle {
        Rectangle {
            corners: [initial; 4],
            stroke,
            color,
        }
    }

    /// Only used for testing
    #[cfg(test)]
    pub fn from_corners([corner_1, corner_2]: [Vec2DDrawing; 2]) -> Rectangle {
        Rectangle {
            corners: [
                corner_1,
                Vec2DDrawing::new(corner_2.x, corner_1.y),
                corner_2,
                Vec2DDrawing::new(corner_1.x, corner_2.y),
            ],
            stroke: 4.0,
            color: Color::green(),
        }
    }

    pub fn all_four_corners(&self) -> [Vec2DDrawing; 4] {
        self.corners
    }
}

impl ShapeTrait for Rectangle {
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

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, t: Transform) { }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform) -> ShapeFinished {
        self.handle_mouse_moved(pos, t);

        ShapeFinished::Yes
    }

    fn draw_commands(&self) -> DrawCommand {
        let corners = self.all_four_corners();
        let mut points_iter = corners.iter().cycle().take(5);
        let mut commands = Vec::with_capacity(4);

        commands.push(PathCommand::MoveTo(*points_iter.next().unwrap()));
        commands.extend(points_iter.map(|p| PathCommand::LineTo(*p)));

        DrawCommand::Path {
            commands,
            thickness: self.stroke,
            color: self.color,
        }
    }

    fn bbox(&self) -> [Vec2DDrawing; 2] {
        bbox_from_points(self.all_four_corners().into_iter().map(|p| *p))
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Rectangle
    }

    fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool {
        let intersects_corners = self
            .all_four_corners()
            .iter()
            .map(|p| p.distance(center) < radius)
            .filter(|a| *a)
            .count() > 0;

        let intersects_sides = self.all_four_corners().iter().zip(self.all_four_corners().iter().cycle())
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
        let shape1 = Rectangle::from_corners([Vec2DDrawing::new(-1.0, -2.0), Vec2DDrawing::new(5.0, 2.0)]);
        let shape2 = Rectangle::from_corners([Vec2DDrawing::new(-1.0, 2.0), Vec2DDrawing::new(5.0, -2.0)]);

        assert_eq!(shape1.bbox(), [Vec2DDrawing::new(-1.0, -2.0), Vec2DDrawing::new(5.0, 2.0)]);
        assert_eq!(shape2.bbox(), [Vec2DDrawing::new(-1.0, -2.0), Vec2DDrawing::new(5.0, 2.0)]);
    }

    #[test]
    fn test_bbox_extended() {
        let rectangles = vec![
            Rectangle::from_corners([Vec2DDrawing::new(15.0, -48.0), Vec2DDrawing::new(37.0, -27.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(15.0, -27.0), Vec2DDrawing::new(37.0, -48.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(37.0, -48.0), Vec2DDrawing::new(15.0, -27.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(37.0, -27.0), Vec2DDrawing::new(15.0, -48.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [Vec2DDrawing::new(15.0, -48.0), Vec2DDrawing::new(37.0, -27.0)]);
        }

        let rectangles = vec![
            Rectangle::from_corners([Vec2DDrawing::new(15.0, -24.0), Vec2DDrawing::new(60.0, 6.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(15.0, 6.0), Vec2DDrawing::new(60.0, -24.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(60.0, -24.0), Vec2DDrawing::new(15.0, 6.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(60.0, 6.0), Vec2DDrawing::new(15.0, -24.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [Vec2DDrawing::new(15.0, -24.0), Vec2DDrawing::new(60.0, 6.0)]);
        }

        let rectangles = vec![
            Rectangle::from_corners([Vec2DDrawing::new(-8.0, -32.0), Vec2DDrawing::new(16.0, 0.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(-8.0, 0.0), Vec2DDrawing::new(16.0, -32.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(16.0, -32.0), Vec2DDrawing::new(-8.0, 0.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(16.0, 0.0), Vec2DDrawing::new(-8.0, -32.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [Vec2DDrawing::new(-8.0, -32.0), Vec2DDrawing::new(16.0, 0.0)]);
        }

        let rectangles = vec![
            Rectangle::from_corners([Vec2DDrawing::new(48.0, -19.0), Vec2DDrawing::new(78.0, 22.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(48.0, 22.0), Vec2DDrawing::new(78.0, -19.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(78.0, -19.0), Vec2DDrawing::new(48.0, 22.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(78.0, 22.0), Vec2DDrawing::new(48.0, -19.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [Vec2DDrawing::new(48.0, -19.0), Vec2DDrawing::new(78.0, 22.0)]);
        }
    }
}
