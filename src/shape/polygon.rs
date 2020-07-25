use std::rc::Rc;

use crate::color::Color;
use crate::point::Point;
use crate::shape::{ShapeTrait, ShapeFinished, ShapeType};
use crate::consts::TOUCH_RADIUS;
use crate::draw_commands::DrawCommand;
use crate::geom::{bbox_from_points, segment_intersects_circle};

pub struct Polygon {
    points: Vec<Point>,
    tip: Point,
    thickness: f64,
    color: Color,
}

impl Polygon {
    pub fn new(color: Color, initial: Point, thickness: f64) -> Polygon {
        let mut init_vec = Vec::with_capacity(5);

        init_vec.push(initial);

        Polygon {
            points: init_vec,
            tip: initial,
            thickness,
            color,
        }
    }

    pub fn with_params(color: Color, points: Vec<Point>, thickness: f64) -> Polygon {
        Polygon {
            tip: *points.last().unwrap(),
            points, color, thickness,
        }
    }

    fn touches_prev_point(&self, point: Point) -> bool {
        self.points.iter().map(|p| p.distance(point)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() <= TOUCH_RADIUS
    }
}

impl ShapeTrait for Polygon {
    fn handle_mouse_moved(&mut self, pos: Point) {
        self.tip = pos;
    }

    fn handle_button_pressed(&mut self, _pos: Point) {}

    fn handle_button_released(&mut self, pos: Point) -> ShapeFinished {
        let finished = if self.points.len() > 1 && self.touches_prev_point(pos) { ShapeFinished::Yes } else { ShapeFinished::No };

        self.points.push(pos);

        finished
    }

    fn draw_commands(&self) -> DrawCommand {
        let mut points: Vec<_> = self.points.iter().map(|p| p.clone()).collect();

        points.push(self.tip);

        DrawCommand::Line {
            line: Rc::new(points),
            thickness: self.thickness,
            color: self.color,
        }
    }

    fn bbox(&self) -> [[f64; 2]; 2] {
        bbox_from_points(self.points.iter().map(|p| *p))
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Polygon
    }

    fn intersects_circle(&self, center: Point, radius: f64) -> bool {
        for (a, b) in self.points.iter().zip(self.points.iter().skip(1)) {
            if segment_intersects_circle([*a, *b], center, radius) {
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
    use super::Polygon;
    use crate::point::Point;
    use crate::shape::{ShapeTrait, ShapeFinished};

    #[test]
    fn it_behaves() {
        let mut poly = Polygon::new(Default::default(), Point::new(0.0, 0.0), 4.0);

        poly.handle_mouse_moved(Point::new(20.0, 0.0));
        assert_eq!(poly.handle_button_released(Point::new(20.0, 0.0)), ShapeFinished::No);

        poly.handle_mouse_moved(Point::new(20.0, 20.0));
        assert_eq!(poly.handle_button_released(Point::new(20.0, 20.0)), ShapeFinished::No);

        poly.handle_mouse_moved(Point::new(0.0, 0.0));
        assert_eq!(poly.handle_button_released(Point::new(0.0, 0.0)), ShapeFinished::Yes);
    }
}
