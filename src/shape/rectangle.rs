use crate::point::Vec2DDrawing;
use crate::draw_commands::DrawCommand;
use super::{ShapeTrait, ShapeFinished, path::PathCommand};
use crate::color::Color;
use crate::ShapeType;

pub struct Rectangle {
    corner_1: Vec2DDrawing,
    corner_2: Vec2DDrawing,
    stroke: f64,
    color: Color,
}

impl Rectangle {
    pub fn new(color: Color, initial: Vec2DDrawing, stroke: f64) -> Rectangle {
        Rectangle {
            corner_1: initial,
            corner_2: initial,
            stroke,
            color,
        }
    }

    /// Only used for testing
    pub fn from_corners([corner_1, corner_2]: [Vec2DDrawing; 2]) -> Rectangle {
        Rectangle {
            corner_1, corner_2,
            stroke: 4.0,
            color: Color::green(),
        }
    }

    pub fn all_four_corners(&self) -> [Vec2DDrawing; 4] {
        [
            Vec2DDrawing::new(self.corner_1.x, self.corner_1.y),
            Vec2DDrawing::new(self.corner_1.x, self.corner_2.y),
            Vec2DDrawing::new(self.corner_2.x, self.corner_2.y),
            Vec2DDrawing::new(self.corner_2.x, self.corner_1.y),
        ]
    }
}

impl ShapeTrait for Rectangle {
    fn handle_mouse_moved(&mut self, pos: Vec2DDrawing) {
        self.corner_2 = pos;
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DDrawing) {
    }

    fn handle_button_released(&mut self, pos: Vec2DDrawing) -> ShapeFinished {
        self.corner_2 = pos;
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

    fn bbox(&self) -> [[f64; 2]; 2] {
        let x2 = self.corner_2.x;
        let y2 = self.corner_2.y;

        [
            [
                self.corner_1.x.min(x2),
                self.corner_1.y.min(y2),
            ],
            [
                self.corner_1.x.max(x2),
                self.corner_1.y.max(y2),
            ],
        ]
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

        let min_y = self.corner_1.y.min(self.corner_2.y);
        let min_x = self.corner_1.x.min(self.corner_2.x);
        let max_y = self.corner_1.y.max(self.corner_2.y);
        let max_x = self.corner_1.x.max(self.corner_2.x);

        let intersects_sides =
            ((center.x - self.corner_1.x).abs() < radius && center.y >= min_y && center.y <= max_y) ||
            ((center.x - self.corner_2.x).abs() < radius && center.y >= min_y && center.y <= max_y) ||
            ((center.y - self.corner_1.y).abs() < radius && center.x >= min_x && center.x <= max_x) ||
            ((center.y - self.corner_2.y).abs() < radius && center.x >= min_x && center.x <= max_x);

        intersects_sides || intersects_corners
    }

    fn color(&self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::Rectangle;
    use crate::point::Vec2DDrawing;
    use crate::shape::ShapeTrait;

    #[test]
    fn test_bbox() {
        let shape1 = Rectangle::from_corners([Vec2DDrawing::new(-1.0, -2.0), Vec2DDrawing::new(5.0, 2.0)]);
        let shape2 = Rectangle::from_corners([Vec2DDrawing::new(-1.0, 2.0), Vec2DDrawing::new(5.0, -2.0)]);

        assert_eq!(shape1.bbox(), [[-1.0, -2.0], [5.0, 2.0]]);
        assert_eq!(shape2.bbox(), [[-1.0, -2.0], [5.0, 2.0]]);
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
            assert_eq!(rectangle.bbox(), [[15.0, -48.0], [37.0, -27.0]]);
        }

        let rectangles = vec![
            Rectangle::from_corners([Vec2DDrawing::new(15.0, -24.0), Vec2DDrawing::new(60.0, 6.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(15.0, 6.0), Vec2DDrawing::new(60.0, -24.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(60.0, -24.0), Vec2DDrawing::new(15.0, 6.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(60.0, 6.0), Vec2DDrawing::new(15.0, -24.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [[15.0, -24.0], [60.0, 6.0]]);
        }

        let rectangles = vec![
            Rectangle::from_corners([Vec2DDrawing::new(-8.0, -32.0), Vec2DDrawing::new(16.0, 0.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(-8.0, 0.0), Vec2DDrawing::new(16.0, -32.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(16.0, -32.0), Vec2DDrawing::new(-8.0, 0.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(16.0, 0.0), Vec2DDrawing::new(-8.0, -32.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [[-8.0, -32.0], [16.0, 0.0]]);
        }

        let rectangles = vec![
            Rectangle::from_corners([Vec2DDrawing::new(48.0, -19.0), Vec2DDrawing::new(78.0, 22.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(48.0, 22.0), Vec2DDrawing::new(78.0, -19.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(78.0, -19.0), Vec2DDrawing::new(48.0, 22.0)]),
            Rectangle::from_corners([Vec2DDrawing::new(78.0, 22.0), Vec2DDrawing::new(48.0, -19.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [[48.0, -19.0], [78.0, 22.0]]);
        }
    }
}
