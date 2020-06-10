use crate::draw_commands::WireDrawCommand;
use super::{Shape, ShapeTrait, ShapeId};
use crate::color::Color;
use crate::ShapeType;
use crate::point::Point;

pub struct Ellipse {
    corner_1: Point,
    corner_2: Point,
}

impl Ellipse {
    pub fn new(color: Color, [corner_1, corner_2]: [Point; 2], id: ShapeId) -> Shape {
        Shape {
            id,
            color,
            shape_impl: Box::new(Ellipse {
                corner_1, corner_2,
            }),
        }
    }
}

impl ShapeTrait for Ellipse {
    fn handle(&mut self, val: Point) {
        self.corner_2 = val;
    }

    fn draw_commands(&self) -> WireDrawCommand {
        WireDrawCommand::Ellipse{
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
        ShapeType::Ellipse
    }
}

#[cfg(test)]
mod tests {
    use super::Ellipse;
    use crate::color::Color;
    use crate::shape::ShapeId;
    use crate::point::Point;

    #[test]
    fn test_bbox() {
        let shape1 = Ellipse::new(Color::green(), [Point::new(-1.0, -2.0), Point::new(5.0, 2.0)], ShapeId::from(1));
        let shape2 = Ellipse::new(Color::green(), [Point::new(-1.0, 2.0), Point::new(5.0, -2.0)], ShapeId::from(1));

        assert_eq!(shape1.shape_impl.bbox(), [[-1.0, -2.0], [5.0, 2.0]]);
        assert_eq!(shape2.shape_impl.bbox(), [[-1.0, -2.0], [5.0, 2.0]]);
    }

    #[test]
    fn test_bbox_extended() {
        let rectangles = vec![
            Ellipse::new(Color::green(), [Point::new(15.0, -48.0), Point::new(37.0, -27.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(15.0, -27.0), Point::new(37.0, -48.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(37.0, -48.0), Point::new(15.0, -27.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(37.0, -27.0), Point::new(15.0, -48.0)], ShapeId::from(1)),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[15.0, -48.0], [37.0, -27.0]]);
        }

        let rectangles = vec![
            Ellipse::new(Color::green(), [Point::new(15.0, -24.0), Point::new(60.0, 6.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(15.0, 6.0), Point::new(60.0, -24.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(60.0, -24.0), Point::new(15.0, 6.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(60.0, 6.0), Point::new(15.0, -24.0)], ShapeId::from(1)),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[15.0, -24.0], [60.0, 6.0]]);
        }

        let rectangles = vec![
            Ellipse::new(Color::green(), [Point::new(-8.0, -32.0), Point::new(16.0, 0.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(-8.0, 0.0), Point::new(16.0, -32.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(16.0, -32.0), Point::new(-8.0, 0.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(16.0, 0.0), Point::new(-8.0, -32.0)], ShapeId::from(1)),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[-8.0, -32.0], [16.0, 0.0]]);
        }

        let rectangles = vec![
            Ellipse::new(Color::green(), [Point::new(48.0, -19.0), Point::new(78.0, 22.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(48.0, 22.0), Point::new(78.0, -19.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(78.0, -19.0), Point::new(48.0, 22.0)], ShapeId::from(1)),
            Ellipse::new(Color::green(), [Point::new(78.0, 22.0), Point::new(48.0, -19.0)], ShapeId::from(1)),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[48.0, -19.0], [78.0, 22.0]]);
        }
    }
}
