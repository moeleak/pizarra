use crate::color::Color;
use crate::point::{Point, Vec2DDrawing, Vec2DScreen};
use crate::shape::{ShapeTrait, ShapeFinished, ShapeType, path::PathCommand};
use crate::consts::TOUCH_RADIUS;
use crate::draw_commands::DrawCommand;
use crate::geom::{bbox_from_points, segment_intersects_circle};
use crate::transform::Transform;

pub struct Polygon {
    points: Vec<Vec2DDrawing>,
    tip: Option<Vec2DDrawing>,
    thickness: f64,
    color: Color,
}

impl Polygon {
    pub fn new(color: Color, initial: Vec2DDrawing, thickness: f64) -> Polygon {
        let init_vec = Vec::with_capacity(8);

        Polygon {
            points: init_vec,
            tip: Some(initial),
            thickness,
            color,
        }
    }

    pub fn with_params(color: Color, points: Vec<Vec2DDrawing>, thickness: f64) -> Polygon {
        Polygon {
            tip: None,
            points, color, thickness,
        }
    }

    fn touches_prev_point(&self, point: Vec2DDrawing) -> Option<Vec2DDrawing> {
        self.points.iter().filter(|p| p.distance(point) <= TOUCH_RADIUS).map(|p| *p).next()
    }
}

impl ShapeTrait for Polygon {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform) {
        self.tip = Some(t.to_storage_coordinates(pos));
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, t: Transform) {}

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform) -> ShapeFinished {
        let drawing_pos = t.to_storage_coordinates(pos);

        if self.points.len() == 0 {
            self.points.push(drawing_pos);
            return ShapeFinished::No;
        }

        if let Some(point) = self.touches_prev_point(drawing_pos) {
            self.points.push(point);
            self.tip = None;

            ShapeFinished::Yes
        } else {
            self.points.push(drawing_pos);

            ShapeFinished::No
        }
    }

    fn draw_commands(&self) -> DrawCommand {
        let mut points_iter = self.points.iter();
        let mut commands = Vec::with_capacity(self.points.len());

        if let Some(p) = points_iter.next() {
            commands.push(PathCommand::MoveTo(*p));
            commands.extend(points_iter.map(|p| PathCommand::LineTo(*p)));
        }

        if let Some(p) = self.tip {
            commands.push(if commands.len() == 0 {
                PathCommand::MoveTo(p)
            } else {
                PathCommand::LineTo(p)
            });
        }

        DrawCommand::Path {
            commands,
            thickness: self.thickness,
            color: self.color,
        }
    }

    fn bbox(&self) -> [Vec2DDrawing; 2] {
        bbox_from_points(self.points.iter().map(|p| *p))
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Polygon
    }

    fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool {
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
    use super::*;

    #[test]
    fn it_behaves() {
        let mut poly = Polygon::new(Default::default(), Vec2DDrawing::new(0.0, 0.0), 4.0);
        let t = Default::default();

        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t);

        poly.handle_mouse_moved(Vec2DScreen::new(20.0, 0.0), t);
        assert_eq!(poly.handle_button_released(Vec2DScreen::new(20.0, 0.0), t), ShapeFinished::No);

        poly.handle_mouse_moved(Vec2DScreen::new(20.0, 20.0), t);
        assert_eq!(poly.handle_button_released(Vec2DScreen::new(20.0, 20.0), t), ShapeFinished::No);

        poly.handle_mouse_moved(Vec2DScreen::new(0.0, 0.0), t);
        assert_eq!(poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t), ShapeFinished::Yes);
    }

    #[test]
    fn no_clunky_path_ends() {
        let mut poly = Polygon::new(Default::default(), Vec2DDrawing::new(1.0, 0.0), 4.0);
        let t = Default::default();

        poly.handle_mouse_moved(Vec2DScreen::new(1.0, 1.0), t);
        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t);

        poly.handle_mouse_moved(Vec2DScreen::new(31.0, 0.0), t);
        poly.handle_button_released(Vec2DScreen::new(30.0, 0.0), t);

        poly.handle_mouse_moved(Vec2DScreen::new(29.0, 0.0), t);
        assert_eq!(poly.handle_button_released(Vec2DScreen::new(33.0, 0.0), t), ShapeFinished::Yes);

        assert_eq!(poly.draw_commands(), DrawCommand::Path {
            color: Default::default(),
            commands: vec![
                PathCommand::MoveTo(Vec2DDrawing::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2DDrawing::new(30.0, 0.0)),
                PathCommand::LineTo(Vec2DDrawing::new(30.0, 0.0)),
            ],
            thickness: 4.0,
        });
    }

    #[test]
    fn can_delete_single_point_polygons() {
        let mut poly = Polygon::new(Default::default(), Vec2DDrawing::new(0.0, 0.0), 4.0);
        let t = Default::default();

        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t);

        if let ShapeFinished::No = poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t) {
            panic!("Unfinished shape");
        }

        assert!(poly.intersects_circle(Vec2DDrawing::new(0.0, 0.0), 10.0));
    }

    #[test]
    fn can_render_first_point() {
        let poly = Polygon::new(Default::default(), Vec2DDrawing::new(0.0, 0.0), 3.0);

        assert_eq!(poly.draw_commands(), DrawCommand::Path {
            color: Default::default(),
            commands: vec![
                PathCommand::MoveTo(Vec2DDrawing::new(0.0, 0.0)),
            ],
            thickness: 3.0,
        });
    }
}
