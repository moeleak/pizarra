use crate::draw_commands::DrawCommand;
use super::{ShapeTrait, ShapeFinished};
use crate::color::Color;
use crate::ShapeType;
use crate::point::Point;

pub struct Ellipse {
    corner_1: Point,
    corner_2: Point,
    color: Color,
}

impl Ellipse {
    pub fn new(color: Color, initial: Point) -> Ellipse {
        Ellipse {
            corner_1: initial,
            corner_2: initial,
            color,
        }
    }

    pub fn from_corners([corner_1, corner_2]: [Point; 2]) -> Ellipse {
        Ellipse {
            corner_1, corner_2,
            color: Color::green(),
        }
    }
}

impl ShapeTrait for Ellipse {
    fn handle_mouse_moved(&mut self, pos: Point) {
        self.corner_2 = pos;
    }

    fn handle_button_pressed(&mut self, _pos: Point) {
    }

    fn handle_button_released(&mut self, pos: Point) -> ShapeFinished {
        self.corner_2 = pos;
        ShapeFinished::Yes
    }

    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Ellipse{
            corner_1: self.corner_1,
            corner_2: self.corner_2,
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
        ShapeType::Ellipse
    }

    fn intersects_circle(&self, _center: Point, _radius: f64) -> bool {
        false
    }

    fn color(&self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::Ellipse;
    use crate::point::Point;
    use crate::shape::ShapeTrait;

    #[test]
    fn test_bbox() {
        let shape1 = Ellipse::from_corners([Point::new(-1.0, -2.0), Point::new(5.0, 2.0)]);
        let shape2 = Ellipse::from_corners([Point::new(-1.0, 2.0), Point::new(5.0, -2.0)]);

        assert_eq!(shape1.bbox(), [[-1.0, -2.0], [5.0, 2.0]]);
        assert_eq!(shape2.bbox(), [[-1.0, -2.0], [5.0, 2.0]]);
    }

    #[test]
    fn test_bbox_extended() {
        let rectangles = vec![
            Ellipse::from_corners([Point::new(15.0, -48.0), Point::new(37.0, -27.0)]),
            Ellipse::from_corners([Point::new(15.0, -27.0), Point::new(37.0, -48.0)]),
            Ellipse::from_corners([Point::new(37.0, -48.0), Point::new(15.0, -27.0)]),
            Ellipse::from_corners([Point::new(37.0, -27.0), Point::new(15.0, -48.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [[15.0, -48.0], [37.0, -27.0]]);
        }

        let rectangles = vec![
            Ellipse::from_corners([Point::new(15.0, -24.0), Point::new(60.0, 6.0)]),
            Ellipse::from_corners([Point::new(15.0, 6.0), Point::new(60.0, -24.0)]),
            Ellipse::from_corners([Point::new(60.0, -24.0), Point::new(15.0, 6.0)]),
            Ellipse::from_corners([Point::new(60.0, 6.0), Point::new(15.0, -24.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [[15.0, -24.0], [60.0, 6.0]]);
        }

        let rectangles = vec![
            Ellipse::from_corners([Point::new(-8.0, -32.0), Point::new(16.0, 0.0)]),
            Ellipse::from_corners([Point::new(-8.0, 0.0), Point::new(16.0, -32.0)]),
            Ellipse::from_corners([Point::new(16.0, -32.0), Point::new(-8.0, 0.0)]),
            Ellipse::from_corners([Point::new(16.0, 0.0), Point::new(-8.0, -32.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [[-8.0, -32.0], [16.0, 0.0]]);
        }

        let rectangles = vec![
            Ellipse::from_corners([Point::new(48.0, -19.0), Point::new(78.0, 22.0)]),
            Ellipse::from_corners([Point::new(48.0, 22.0), Point::new(78.0, -19.0)]),
            Ellipse::from_corners([Point::new(78.0, -19.0), Point::new(48.0, 22.0)]),
            Ellipse::from_corners([Point::new(78.0, 22.0), Point::new(48.0, -19.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.bbox(), [[48.0, -19.0], [78.0, 22.0]]);
        }
    }

    #[test]
    #[ignore]
    fn test_intersects() {
        assert!(false, "intersects is implemented");
    }
}
