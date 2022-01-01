use crate::color::Color;
use crate::point::{Point, Vec2DWorld, Vec2DScreen};
use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::shape::ellipse::Ellipse;
use crate::draw_commands::{self, DrawCommand, path_helper, circle_helper};
use crate::transform::Transform;
use crate::geom::Angle;

#[derive(Debug)]
pub struct CircleBuilder {
    center: Vec2DWorld,
    point: Vec2DWorld,
    thickness: f64,
    color: Color,
}

impl CircleBuilder {
    pub fn start(color: Color, initial: Vec2DWorld, thickness: f64) -> CircleBuilder {
        CircleBuilder {
            center: initial,
            thickness,
            color,
            point: initial,
        }
    }
}

impl ShapeBuilder for CircleBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) {
        self.point = t.to_world_coordinates(pos);
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform, _snap: f64) {}

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform, snap: f64) -> ShapeFinished {
        self.handle_mouse_moved(pos, t, snap);

        let radius = self.point.distance(self.center);

        ShapeFinished::Yes(Box::new(
            Ellipse::from_parts(
                self.center,
                radius,
                radius,
                Angle::from_radians(0.0),
                Some(self.color),
                None,
                self.thickness,
            )
        ))
    }

    fn draw_commands(&self, t: Transform, snap: f64) -> Vec<DrawCommand> {
        let radius = self.center.distance(self.point);
        vec![
            DrawCommand::Ellipse(draw_commands::Ellipse {
                center: self.center,
                semimajor: radius,
                semiminor: radius,
                angle: Angle::from_radians(0.0),
                thickness: self.thickness,
                color: Some(self.color),
                fill: None,
            }),

            path_helper(vec![
                t.to_screen_coordinates(self.center),
                t.to_screen_coordinates(self.point),
            ]),

            circle_helper(t.to_screen_coordinates(self.center), snap),
        ]
    }
}
