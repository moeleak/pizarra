use graphics::math::Vec2d;
use crate::draw_commands::WireDrawCommand;
use super::{Shape, ShapeTrait, ShapeId};
use crate::color::Color;
use crate::ShapeType;

pub struct Circle {
    rect: [f64; 4],
}

impl Circle {
    pub fn new(color: Color, rect: [[f64; 2]; 2], id: ShapeId) -> Shape {
        Shape {
            id,
            color,
            shape_impl: Box::new(Circle {
                rect: [rect[0][0], rect[0][1], rect[1][0], rect[1][0]],
            })
        }
    }
}

impl ShapeTrait for Circle {
    fn handle(&mut self, val: Vec2d) {
        self.rect = [self.rect[0], self.rect[1], val[0]-self.rect[0], val[1]-self.rect[1]];
    }

    fn draw_commands(&self) -> Vec<WireDrawCommand> {
        vec![WireDrawCommand::Circle{
            rect: self.rect,
        }]
    }

    fn bbox(&self) -> [[f64; 2]; 2] {
        let rect = self.rect;

        [[rect[0], rect[1]], [rect[0]+rect[2], rect[1] + rect[3]]]
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
        let shape = Circle::new(Color::green(), [[1.0, 1.0], [1.0, 1.0]], ShapeId::from(1));

        assert_eq!(shape.shape_impl.bbox(), [[1.0, 1.0], [2.0, 2.0]]);
    }

    #[test]
    fn test_bbox_extended() {
        assert!(false, "a Circle with coordinates in every cuadrant both with top-left,bottom-right and bottom-left,top-right coordinates");
        assert!(false, "a Circle with one coordinate in every cuadrant, given its top-left and bottom-right coordinates");
        assert!(false, "a Circle with one coordinate in every cuadrant, given its bottom-left and top-right coordinates");
    }
}
