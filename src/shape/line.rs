use std::rc::Rc;

use crate::draw_commands::DrawCommand;
use super::{ShapeTrait, ShapeFinished};
use crate::color::Color;
use crate::ShapeType;
use crate::point::Point;

pub struct Line {
    points: Vec<Point>,
    thickness: f64,
    color: Color,
}

impl Line {
    pub fn new(color: Color, initial: Point, thickness: f64) -> Line {
        let mut init_vec = Vec::with_capacity(1000);

        init_vec.push(initial);

        Line {
            points: init_vec,
            thickness,
            color,
        }
    }

    pub fn with_params(color: Color, points: Vec<Point>, thickness: f64) -> Line {
        Line {
            points, color, thickness,
        }
    }
}

impl ShapeTrait for Line {
    fn handle_mouse_moved(&mut self, pos: Point) {
        self.points.push(pos);
    }

    fn handle_button_pressed(&mut self, _pos: Point) {
    }

    fn handle_button_released(&mut self, pos: Point) -> ShapeFinished {
        self.points.push(pos);
        ShapeFinished::Yes
    }

    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Line {
            line: Rc::new(self.points.iter().map(|p| p.clone()).collect()),
            thickness: self.thickness,
            color: self.color,
        }
    }

    fn bbox(&self) -> [[f64; 2]; 2] {
        let bbox = self.points.iter().fold([
            [std::f64::INFINITY, std::f64::INFINITY], // bottom-left corner
            [std::f64::NEG_INFINITY, std::f64::NEG_INFINITY], // top right corner
        ], |acc, point| {
            [
                [
                    if point.x < acc[0][0] { point.x } else { acc[0][0] },
                    if point.y < acc[0][1] { point.y } else { acc[0][1] },
                ],
                [
                    if point.x > acc[1][0] { point.x } else { acc[1][0] },
                    if point.y > acc[1][1] { point.y } else { acc[1][1] },
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

    fn intersects_circle(&self, center: Point, radius: f64) -> bool {
        for p in self.points.iter() {
            if p.distance(center) <= radius {
                return true;
            }
        }

        false
    }

    fn color(&self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::Line;
    use crate::shape::ShapeTrait;
    use crate::color::Color;
    use crate::point::Point;

    #[test]
    fn test_bbox() {
        let mut line = Line {
            points: Vec::new(),
            thickness: 4.0,
            color: Color::green(),
        };

        line.handle_mouse_moved(Point::new(1.0, 0.0));
        line.handle_mouse_moved(Point::new(0.0, 1.0));

        assert_eq!(line.bbox(), [[0.0, 0.0], [1.0, 1.0]]);
    }

    #[test]
    fn test_bbox_twisted_line() {
        let mut line = Line::new(Color::green(), Point::new(-12.0, -1.0), 1.0);

        line.handle_mouse_moved(Point::new(-5.0, 0.0));
        line.handle_mouse_moved(Point::new(-2.0, 7.0));
        line.handle_mouse_moved(Point::new(2.0, -8.0));

        assert_eq!(line.bbox(), [[-12.0, -8.0], [2.0, 7.0]]);
    }
}
