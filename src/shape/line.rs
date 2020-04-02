use graphics::math::Vec2d;
use crate::draw_commands::WireDrawCommand;
use super::{Shape, ShapeTrait, ShapeId};
use crate::color::Color;
use crate::ShapeType;

pub struct Line {
    points: Vec<Vec2d>,
}

impl Line {
    pub fn new(color: Color, initial: [[f64; 2]; 2], id: ShapeId) -> Shape {
        let mut init_vec = Vec::with_capacity(1000);

        init_vec.push(initial[0]);
        init_vec.push(initial[1]);

        Shape {
            id,
            color,
            shape_impl: Box::new(Line {
                points: init_vec,
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
        let bbox = self.points.iter().fold([
            [std::f64::INFINITY, std::f64::INFINITY], // bottom-left corner
            [std::f64::NEG_INFINITY, std::f64::NEG_INFINITY], // top right corner
        ], |acc, point| {
            [
                [
                    if point[0] < acc[0][0] { point[0] } else { acc[0][0] },
                    if point[1] < acc[0][1] { point[1] } else { acc[0][1] },
                ],
                [
                    if point[0] > acc[1][0] { point[0] } else { acc[1][0] },
                    if point[1] > acc[1][1] { point[1] } else { acc[1][1] },
                ],
            ]
        });

        assert_ne!(bbox[0][0], std::f64::INFINITY);
        assert_ne!(bbox[0][1], std::f64::INFINITY);
        assert_ne!(bbox[1][0], std::f64::NEG_INFINITY);
        assert_ne!(bbox[1][1], std::f64::NEG_INFINITY);

        bbox
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Line
    }
}

#[cfg(test)]
mod tests {
    use super::Line;
    use crate::shape::ShapeTrait;
    use crate::color::Color;

    #[test]
    fn test_bbox() {
        let mut line = Line {
            points: Vec::new(),
        };

        line.handle([1.0, 0.0]);
        line.handle([0.0, 1.0]);

        assert_eq!(line.bbox(), [[0.0, 0.0], [1.0, 1.0]]);
    }
}
