use graphics::math::Vec2d;
use crate::draw_commands::WireDrawCommand;
use super::{Shape, ShapeTrait};
use crate::color::Color;
use crate::ShapeType;

pub struct Circle {
    rect: [f64; 4],
}

impl Circle {
    pub fn new(color: Color, rect: [[f64; 2]; 2]) -> Shape {
        Shape {
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
