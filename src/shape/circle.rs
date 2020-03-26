use graphics::math::Vec2d;
use crate::draw_commands::WireDrawCommand;
use super::{Shape, ShapeTrait};
use crate::color::Color;

pub struct Circle {
    borders: Option<[f64; 4]>,
}

impl Circle {
    pub fn new(color: Color) -> Shape {
        Shape {
            color,
            shape_impl: Box::new(Circle {
                borders: None,
            })
        }
    }
}

impl ShapeTrait for Circle {
    fn handle(&mut self, val: Vec2d) {
        match self.borders.as_mut() {
            Some(bb) => {
                self.borders = Some([bb[0], bb[1], val[0]-bb[0], val[1]-bb[1]]);
            },
            None => {
                self.borders = Some([val[0], val[1], 0.0, 0.0]);
            },
        }
    }

    fn draw_commands(&self) -> Vec<WireDrawCommand> {
        match self.borders {
            Some(bb) => vec![WireDrawCommand::Circle{
                rect: bb,
            }],
            None => Vec::new(),
        }
    }

    fn bbox(&self) -> [[f64; 2]; 2] {
        unimplemented!()
    }
}
