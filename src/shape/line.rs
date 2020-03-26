use graphics::math::Vec2d;
use crate::draw_commands::WireDrawCommand;
use super::{Shape, ShapeTrait};
use crate::color::Color;

pub struct Line {
    points: Vec<Vec2d>,
}

impl Line {
    pub fn new(color: Color) -> Shape {
        Shape {
            color,
            shape_impl: Box::new(Line {
                points: Vec::with_capacity(1000),
            }),
        }
    }
}

impl ShapeTrait for Line {
    fn handle(&mut self, val: Vec2d) {
        self.points.push(val);
    }

    fn draw_commands(&self) -> Vec<WireDrawCommand> {
        self.points.iter().zip(self.points.iter().skip(1)).map(|([x1, y1], [x2, y2])| {
            WireDrawCommand::Line {
                line: [*x1, *y1, *x2, *y2],
            }
        }).collect()
    }

    fn bbox(&self) -> [[f64; 2]; 2] {
        unimplemented!()
    }
}
