use crate::color::Color;
use crate::point::{Point, Vec2DWorld, Vec2DScreen};
use crate::shape::{ShapeBuilder, ShapeFinished, path::{Path, PathCommand}};
use crate::consts::TOUCH_RADIUS;
use crate::draw_commands::DrawCommand;
use crate::transform::Transform;

#[derive(Debug)]
pub struct PolygonBuilder {
    points: Vec<Vec2DWorld>,
    tip: Option<Vec2DWorld>,
    thickness: f64,
    color: Color,
}

impl PolygonBuilder {
    pub fn start(color: Color, initial: Vec2DWorld, thickness: f64) -> PolygonBuilder {
        let init_vec = Vec::with_capacity(8);

        PolygonBuilder {
            points: init_vec,
            tip: Some(initial),
            thickness,
            color,
        }
    }

    pub fn with_params(color: Color, points: Vec<Vec2DWorld>, thickness: f64) -> PolygonBuilder {
        PolygonBuilder {
            tip: None,
            points, color, thickness,
        }
    }

    fn touches_prev_point(&self, point: Vec2DWorld, t: Transform) -> Option<Vec2DWorld> {
        let point = t.to_screen_coordinates(point);

        self.points.iter().filter(|&&p| t.to_screen_coordinates(p).distance(point) <= TOUCH_RADIUS).copied().next()
    }
}

impl ShapeBuilder for PolygonBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform) {
        self.tip = Some(t.to_world_coordinates(pos));
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform) {}

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform) -> ShapeFinished {
        let drawing_pos = t.to_world_coordinates(pos);

        if self.points.is_empty() {
            self.points.push(drawing_pos);
            return ShapeFinished::No;
        }

        if let Some(point) = self.touches_prev_point(drawing_pos, t) {
            self.points.push(point);
            self.tip = None;

            let commands = self.points.iter().enumerate().map(|(i, &p)| {
                if i == 0 {
                    PathCommand::MoveTo(p)
                } else {
                    PathCommand::LineTo(p)
                }
            }).collect();

            ShapeFinished::Yes(Box::new(Path::from_parts(
                commands,
                self.color,
                self.thickness,
            )))
        } else {
            self.points.push(drawing_pos);

            ShapeFinished::No
        }
    }

    fn draw_commands(&self) -> Vec<DrawCommand> {
        let mut points_iter = self.points.iter();
        let mut commands = Vec::with_capacity(self.points.len());

        if let Some(p) = points_iter.next() {
            commands.push(PathCommand::MoveTo(*p));
            commands.extend(points_iter.map(|p| PathCommand::LineTo(*p)));
        }

        if let Some(p) = self.tip {
            commands.push(if commands.is_empty() {
                PathCommand::MoveTo(p)
            } else {
                PathCommand::LineTo(p)
            });
        }

        vec![DrawCommand::Path {
            commands,
            thickness: self.thickness,
            color: self.color,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_behaves() {
        let mut poly = PolygonBuilder::start(Default::default(), Vec2DWorld::new(0.0, 0.0), 4.0);
        let t = Default::default();

        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t);

        poly.handle_mouse_moved(Vec2DScreen::new(20.0, 0.0), t);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(Vec2DScreen::new(20.0, 0.0), t) {
            panic!()
        }

        poly.handle_mouse_moved(Vec2DScreen::new(20.0, 20.0), t);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(Vec2DScreen::new(20.0, 20.0), t) {
            panic!()
        }

        poly.handle_mouse_moved(Vec2DScreen::new(0.0, 0.0), t);
        if let ShapeFinished::No = poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t) {
            panic!()
        }
    }

    #[test]
    fn no_clunky_path_ends() {
        let mut poly = PolygonBuilder::start(Default::default(), Vec2DWorld::new(1.0, 0.0), 4.0);
        let t = Default::default();

        poly.handle_mouse_moved(Vec2DScreen::new(1.0, 1.0), t);
        poly.handle_button_released(Vec2DScreen::new(0.0, 0.0), t);

        poly.handle_mouse_moved(Vec2DScreen::new(31.0, 0.0), t);
        poly.handle_button_released(Vec2DScreen::new(30.0, 0.0), t);

        poly.handle_mouse_moved(Vec2DScreen::new(29.0, 0.0), t);
        if let ShapeFinished::No = poly.handle_button_released(Vec2DScreen::new(33.0, 0.0), t) {
            panic!()
        }

        assert_eq!(poly.draw_commands()[0], DrawCommand::Path {
            color: Default::default(),
            commands: vec![
                PathCommand::MoveTo(Vec2DWorld::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2DWorld::new(30.0, 0.0)),
                PathCommand::LineTo(Vec2DWorld::new(30.0, 0.0)),
            ],
            thickness: 4.0,
        });
    }

    #[test]
    fn can_render_first_point_when_building() {
        let poly = PolygonBuilder::start(Default::default(), Vec2DWorld::new(0.0, 0.0), 3.0);

        assert_eq!(poly.draw_commands()[0], DrawCommand::Path {
            color: Default::default(),
            commands: vec![
                PathCommand::MoveTo(Vec2DWorld::new(0.0, 0.0)),
            ],
            thickness: 3.0,
        });
    }

    fn make_this_poly(t: Transform) -> PolygonBuilder {
        let center = Vec2DWorld::new(0.0, 0.0);
        let pf = Vec2DWorld::new(0.0, -100.0);
        let mut poly = PolygonBuilder::start(Default::default(), center, 4.0);

        poly.handle_button_released(t.to_screen_coordinates(center), t);
        poly.handle_mouse_moved(t.to_screen_coordinates(pf), t);
        poly.handle_button_released(t.to_screen_coordinates(pf), t);

        poly
    }

    #[test]
    fn the_polygon_closes_itself_in_screen_units() {
        // at home scale
        let screen = Vec2DScreen::new(800.0, 600.0);
        let center = screen / 2.0;

        // No matter the zoom level the first point should finish a shape but
        // the second shouldn't at the default 10 px radius
        let p1 = center + Vec2DScreen::new(9.9, 0.0);
        let p2 = center + Vec2DScreen::new(10.1, 0.0);

        let t0 = Transform::default_for_viewport(screen);

        // closes polygon at initial zoom
        let mut poly = make_this_poly(t0);
        let t = t0;
        poly.handle_mouse_moved(p1, t);
        if let ShapeFinished::No = poly.handle_button_released(p1, t) {
            panic!();
        }
        poly.handle_mouse_moved(p2, t);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(p2, t) {
            panic!();
        }

        // at zoomed in scale closes at shorter distance in world coordinates
        let mut poly = make_this_poly(t0);
        let t = t0.zoom(2.0, center);
        poly.handle_mouse_moved(p1, t);
        if let ShapeFinished::No = poly.handle_button_released(p1, t) {
            panic!();
        }
        poly.handle_mouse_moved(p2, t);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(p2, t) {
            panic!();
        }

        // at zoomed out scale closes at greater distance in world coordinates
        let mut poly = make_this_poly(t0);
        let t = t0.zoom(0.5, center);
        poly.handle_mouse_moved(p1, t);
        if let ShapeFinished::No = poly.handle_button_released(p1, t) {
            panic!();
        }
        poly.handle_mouse_moved(p2, t);
        if let ShapeFinished::Yes(_) = poly.handle_button_released(p2, t) {
            panic!();
        }
    }
}
