use graphics::math::Vec2d;
use crate::draw_commands::WireDrawCommand;
use super::{Shape, ShapeTrait, ShapeId};
use crate::color::Color;
use crate::ShapeType;

pub struct Circle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Circle {
    pub fn new(color: Color, [[x1, y1], [x2, y2]]: [[f64; 2]; 2], id: ShapeId) -> Shape {
        Shape {
            id,
            color,
            shape_impl: Box::new(Circle {
                x: x1,
                y: y1,
                width: x2 - x1,
                height: y2 - y1,
            }),
        }
    }
}

impl ShapeTrait for Circle {
    fn handle(&mut self, val: Vec2d) {
        self.width = val[0]-self.x;
        self.height = val[1]-self.y;
    }

    fn draw_commands(&self) -> Vec<WireDrawCommand> {
        vec![WireDrawCommand::Circle{
            rect: [self.x, self.y, self.width, self.height],
        }]
    }

    fn bbox(&self) -> [[f64; 2]; 2] {
        let x2 = self.x + self.width;
        let y2 = self.y + self.height;

        [
            [
                self.x.min(x2),
                self.y.min(y2),
            ],
            [
                self.x.max(x2),
                self.y.max(y2),
            ],
        ]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Circle
    }
}

#[cfg(test)]
mod tests {
    use super::Circle;
    use crate::color::Color;
    use crate::shape::ShapeId;

    #[test]
    fn test_bbox() {
        let shape1 = Circle::new(Color::Green, [[-1.0, -2.0], [5.0, 2.0]], ShapeId::from(1));
        let shape2 = Circle::new(Color::Green, [[-1.0, 2.0], [5.0, -2.0]], ShapeId::from(1));

        assert_eq!(shape1.shape_impl.bbox(), [[-1.0, -2.0], [5.0, 2.0]]);
        assert_eq!(shape2.shape_impl.bbox(), [[-1.0, -2.0], [5.0, 2.0]]);
    }

    #[test]
    fn test_bbox_extended() {
        let rectangles = vec![
            Circle::new(Color::Green, [[15.0, -48.0], [37.0, -27.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[15.0, -27.0], [37.0, -48.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[37.0, -48.0], [15.0, -27.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[37.0, -27.0], [15.0, -48.0]], ShapeId::from(1)),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[15.0, -48.0], [37.0, -27.0]]);
        }

        let rectangles = vec![
            Circle::new(Color::Green, [[15.0, -24.0], [60.0, 6.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[15.0, 6.0], [60.0, -24.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[60.0, -24.0], [15.0, 6.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[60.0, 6.0], [15.0, -24.0]], ShapeId::from(1)),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[15.0, -24.0], [60.0, 6.0]]);
        }

        let rectangles = vec![
            Circle::new(Color::Green, [[-8.0, -32.0], [16.0, 0.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[-8.0, 0.0], [16.0, -32.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[16.0, -32.0], [-8.0, 0.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[16.0, 0.0], [-8.0, -32.0]], ShapeId::from(1)),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[-8.0, -32.0], [16.0, 0.0]]);
        }

        let rectangles = vec![
            Circle::new(Color::Green, [[48.0, -19.0], [78.0, 22.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[48.0, 22.0], [78.0, -19.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[78.0, -19.0], [48.0, 22.0]], ShapeId::from(1)),
            Circle::new(Color::Green, [[78.0, 22.0], [48.0, -19.0]], ShapeId::from(1)),
        ];

        for rectangle in rectangles.into_iter() {
            assert_eq!(rectangle.shape_impl.bbox(), [[48.0, -19.0], [78.0, 22.0]]);
        }
    }
}
