use crate::draw_commands::{cancel_helper, path_helper, DrawCommand};
use crate::geom::{self, Angle};
use crate::point::{Vec2D, ScreenUnit, WorldUnit};
use crate::shape::stored::ellipse::Ellipse;
use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::style::Style;
use crate::transform::Transform;

#[derive(Debug)]
pub struct CenterAndRadiusCircle {
    center: Vec2D<WorldUnit>,
    point: Vec2D<WorldUnit>,
    style: Style,
}

impl CenterAndRadiusCircle {
    pub fn start(initial: Vec2D<WorldUnit>, style: Style) -> CenterAndRadiusCircle {
        CenterAndRadiusCircle {
            center: initial,
            point: initial,
            style,
        }
    }
}

impl ShapeBuilder for CenterAndRadiusCircle {
    fn handle_mouse_moved(&mut self, pos: Vec2D<ScreenUnit>, t: Transform, _snap: ScreenUnit) {
        self.point = t.to_world_coordinates(pos);
    }

    fn handle_button_pressed(&mut self, _pos: Vec2D<ScreenUnit>, _t: Transform, _snap: ScreenUnit) {}

    fn handle_button_released(&mut self, pos: Vec2D<ScreenUnit>, t: Transform, snap: ScreenUnit) -> ShapeFinished {
        self.handle_mouse_moved(pos, t, snap);

        let radius = self.point.distance(self.center);

        if radius < snap {
            return ShapeFinished::Cancelled;
        }

        ShapeFinished::Yes(vec![Box::new(
            Ellipse::from_parts(
                self.center,
                radius,
                radius,
                Angle::from_radians(0.0),
                self.style,
            )
        )])
    }

    fn draw_commands(&self, t: Transform, snap: ScreenUnit) -> Vec<DrawCommand> {
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

            cancel_helper(self.center, self.point, t, snap),
        ]
    }
}
