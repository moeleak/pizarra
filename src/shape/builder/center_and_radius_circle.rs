use crate::draw_commands::{circle_helper, red_circle_helper, path_helper, DrawCommand};
use crate::geom::{self, Angle};
use crate::point::{Point, Vec2DScreen, Vec2DWorld};
use crate::shape::stored::ellipse::Ellipse;
use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::style::Style;
use crate::transform::Transform;

#[derive(Debug)]
pub struct CenterAndRadiusCircle {
    center: Vec2DWorld,
    point: Vec2DWorld,
    style: Style,
}

impl CenterAndRadiusCircle {
    pub fn start(initial: Vec2DWorld, style: Style) -> CenterAndRadiusCircle {
        CenterAndRadiusCircle {
            center: initial,
            point: initial,
            style,
        }
    }
}

impl ShapeBuilder for CenterAndRadiusCircle {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) {
        self.point = t.to_world_coordinates(pos);
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform, _snap: f64) {}

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform, snap: f64) -> ShapeFinished {
        self.handle_mouse_moved(pos, t, snap);

        let radius = self.point.distance(self.center);

        if radius < snap {
            return ShapeFinished::Cancelled;
        }

        ShapeFinished::Yes(Box::new(
            Ellipse::from_parts(
                self.center,
                radius,
                radius,
                Angle::from_radians(0.0),
                self.style,
            )
        ))
    }

    fn draw_commands(&self, t: Transform, snap: f64) -> Vec<DrawCommand> {
        let radius = self.center.distance(self.point);
        vec![
            DrawCommand::Ellipse {
                ellipse: geom::Ellipse {
                    center: self.center,
                    semimajor: radius,
                    semiminor: radius,
                    angle: Angle::from_radians(0.0),
                },
                style: self.style,
            },

            path_helper(vec![
                t.to_screen_coordinates(self.center),
                t.to_screen_coordinates(self.point),
            ]),

            if radius < snap {
                red_circle_helper(t.to_screen_coordinates(self.center), snap)
            } else {
                circle_helper(t.to_screen_coordinates(self.center), snap)
            }
        ]
    }
}
