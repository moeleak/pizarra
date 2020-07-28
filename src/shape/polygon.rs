use std::rc::Rc;

use crate::color::Color;
use crate::point::Point;
use crate::shape::{ShapeTrait, ShapeFinished, ShapeType};
use crate::consts::TOUCH_RADIUS;
use crate::draw_commands::DrawCommand;
use crate::geom::{bbox_from_points, segment_intersects_circle};

pub struct Polygon {
    points: Vec<Point>,
    tip: Option<Point>,
    thickness: f64,
    color: Color,
}

impl Polygon {
    pub fn new(color: Color, initial: Point, thickness: f64) -> Polygon {
        let init_vec = Vec::with_capacity(8);

        Polygon {
            points: init_vec,
            tip: Some(initial),
            thickness,
            color,
        }
    }

    pub fn with_params(color: Color, points: Vec<Point>, thickness: f64) -> Polygon {
        Polygon {
            tip: None,
            points, color, thickness,
        }
    }

    fn touches_prev_point(&self, point: Point) -> Option<Point> {
        self.points.iter().filter(|p| p.distance(point) <= TOUCH_RADIUS).map(|p| *p).next()
    }
}

impl ShapeTrait for Polygon {
    fn handle_mouse_moved(&mut self, pos: Point) {
        self.tip = Some(pos);
    }

    fn handle_button_pressed(&mut self, _pos: Point) {}

    fn handle_button_released(&mut self, pos: Point) -> ShapeFinished {
        if self.points.len() == 0 {
            self.points.push(pos);
            return ShapeFinished::No;
        }

        if let Some(point) = self.touches_prev_point(pos) {
            self.points.push(point);
            self.tip = None;

            ShapeFinished::Yes
        } else {
            self.points.push(pos);

            ShapeFinished::No
        }
    }

    fn draw_commands(&self) -> DrawCommand {
        let mut points: Vec<_> = self.points.iter().map(|p| p.clone()).collect();

        if let Some(p) = self.tip {
            points.push(p);
        }

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

    use std::rc::Rc;

    use crate::point::Point;
    use crate::shape::{ShapeTrait, ShapeFinished};
    use crate::draw_commands::DrawCommand;

    #[test]
    fn it_behaves() {
        let mut poly = Polygon::new(Default::default(), Point::new(0.0, 0.0), 4.0);
        poly.handle_button_released(Point::new(0.0, 0.0));

        poly.handle_mouse_moved(Point::new(20.0, 0.0));
        assert_eq!(poly.handle_button_released(Point::new(20.0, 0.0)), ShapeFinished::No);

        poly.handle_mouse_moved(Point::new(20.0, 20.0));
        assert_eq!(poly.handle_button_released(Point::new(20.0, 20.0)), ShapeFinished::No);

        poly.handle_mouse_moved(Point::new(0.0, 0.0));
        assert_eq!(poly.handle_button_released(Point::new(0.0, 0.0)), ShapeFinished::Yes);
    }

    #[test]
    fn no_clunky_path_ends() {
        let mut poly = Polygon::new(Default::default(), Point::new(1.0, 0.0), 4.0);

        poly.handle_mouse_moved(Point::new(1.0, 1.0));
        poly.handle_button_released(Point::new(0.0, 0.0));

        poly.handle_mouse_moved(Point::new(31.0, 0.0));
        poly.handle_button_released(Point::new(30.0, 0.0));

        poly.handle_mouse_moved(Point::new(29.0, 0.0));
        assert_eq!(poly.handle_button_released(Point::new(33.0, 0.0)), ShapeFinished::Yes);

        assert_eq!(poly.draw_commands(), DrawCommand::Line {
            color: Default::default(),
            line: Rc::new(vec![Point::new(0.0, 0.0), Point::new(30.0, 0.0), Point::new(30.0, 0.0)]),
            thickness: 4.0,
        });
    }
}
