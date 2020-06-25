use crate::point::Point;
use crate::draw_commands::WireDrawCommand;
use super::{Shape, ShapeTrait, ShapeId, ShapeFinished};
use crate::color::Color;
use crate::ShapeType;

pub struct Rectangle {
    corner_1: Point,
    corner_2: Point,
}

impl Rectangle {
    pub fn new(color: Color, initial: Point, id: ShapeId) -> Shape {
        Shape {
            id,
            color,
            shape_impl: Box::new(Rectangle {
                corner_1: initial,
                corner_2: initial,
            }),
        }
    }

    /// Only used for testing
    pub fn from_corners([corner_1, corner_2]: [Point; 2]) -> Shape {
        Shape {
            id: ShapeId::from(1),
            color: Color::green(),
            shape_impl: Box::new(Rectangle {
                corner_1, corner_2,
            }),
        }
    }
}

impl ShapeTrait for Rectangle {
    fn handle_mouse_moved(&mut self, pos: Point) {
        self.corner_2 = pos;
    }

    fn handle_button_pressed(&mut self, _pos: Point) {
        unreachable!()
    }

    fn handle_button_released(&mut self, pos: Point) -> ShapeFinished {
        self.corner_2 = pos;
        ShapeFinished::Yes
    }

    fn draw_commands(&self) -> WireDrawCommand {
        WireDrawCommand::Rectangle {
            corner_1: self.corner_1,
            corner_2: self.corner_2,
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

    fn intersects_circle(&self, center: Point, radius: f64) -> bool {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::Rectangle;
    use crate::point::Point;

    #[test]
    fn test_bbox() {
        let shape1 = Rectangle::from_corners([Point::new(-1.0, -2.0), Point::new(5.0, 2.0)]);
        let shape2 = Rectangle::from_corners([Point::new(-1.0, 2.0), Point::new(5.0, -2.0)]);

        assert_eq!(shape1.shape_impl.bbox(), [[-1.0, -2.0], [5.0, 2.0]]);
        assert_eq!(shape2.shape_impl.bbox(), [[-1.0, -2.0], [5.0, 2.0]]);
    }

    #[test]
    fn test_bbox_extended() {
        let rectangles = vec![
            Rectangle::from_corners([Point::new(15.0, -48.0), Point::new(37.0, -27.0)]),
            Rectangle::from_corners([Point::new(15.0, -27.0), Point::new(37.0, -48.0)]),
            Rectangle::from_corners([Point::new(37.0, -48.0), Point::new(15.0, -27.0)]),
            Rectangle::from_corners([Point::new(37.0, -27.0), Point::new(15.0, -48.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[15.0, -48.0], [37.0, -27.0]]);
        }

        let rectangles = vec![
            Rectangle::from_corners([Point::new(15.0, -24.0), Point::new(60.0, 6.0)]),
            Rectangle::from_corners([Point::new(15.0, 6.0), Point::new(60.0, -24.0)]),
            Rectangle::from_corners([Point::new(60.0, -24.0), Point::new(15.0, 6.0)]),
            Rectangle::from_corners([Point::new(60.0, 6.0), Point::new(15.0, -24.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[15.0, -24.0], [60.0, 6.0]]);
        }

        let rectangles = vec![
            Rectangle::from_corners([Point::new(-8.0, -32.0), Point::new(16.0, 0.0)]),
            Rectangle::from_corners([Point::new(-8.0, 0.0), Point::new(16.0, -32.0)]),
            Rectangle::from_corners([Point::new(16.0, -32.0), Point::new(-8.0, 0.0)]),
            Rectangle::from_corners([Point::new(16.0, 0.0), Point::new(-8.0, -32.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[-8.0, -32.0], [16.0, 0.0]]);
        }

        let rectangles = vec![
            Rectangle::from_corners([Point::new(48.0, -19.0), Point::new(78.0, 22.0)]),
            Rectangle::from_corners([Point::new(48.0, 22.0), Point::new(78.0, -19.0)]),
            Rectangle::from_corners([Point::new(78.0, -19.0), Point::new(48.0, 22.0)]),
            Rectangle::from_corners([Point::new(78.0, 22.0), Point::new(48.0, -19.0)]),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[48.0, -19.0], [78.0, 22.0]]);
        }
    }
}
