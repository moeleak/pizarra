use crate::point::{Point, Vec2DDrawing, Vec2DScreen};
use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::ShapeType;
use crate::transform::Transform;
use crate::geom::{
    Radians, middle_point, parts_of_ellipse_from_corners,
    foci_and_sum_from_center_and_radi
};

use super::{ShapeBuilder, ShapeStored, ShapeFinished};

pub struct EllipseBuilder {
    corners: [Vec2DDrawing; 4],
    thickness: f64,
    color: Color,
}

impl EllipseBuilder {
    pub fn start(color: Color, initial: Vec2DDrawing, thickness: f64) -> EllipseBuilder {
        EllipseBuilder {
            corners: [initial; 4],
            thickness,
            color,
        }
    }

    pub fn with_params(corners: [Vec2DDrawing; 4], color: Color, thickness: f64) -> EllipseBuilder {
        EllipseBuilder {
            corners, thickness, color
        }
    }
}

impl ShapeBuilder for EllipseBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform) {
        let first_point = self.corners[0];
        let first_point_screen = t.to_screen_coordinates(self.corners[0]);

        self.corners = [
            first_point,
            t.to_storage_coordinates(Vec2DScreen::new(pos.x, first_point_screen.y)),
            t.to_storage_coordinates(pos),
            t.to_storage_coordinates(Vec2DScreen::new(first_point_screen.x, pos.y)),
        ];
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _: Transform) { }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform) -> ShapeFinished {
        self.handle_mouse_moved(pos, t);

        todo!()
    }

    fn draw_commands(&self) -> Vec<DrawCommand> {
        let (bigside, bigvector, smallside) = parts_of_ellipse_from_corners(self.corners);
        let angle = bigvector.angle();

        vec![DrawCommand::Ellipse {
            thickness: self.thickness,
            color: self.color,
            center: middle_point(self.corners[0], self.corners[2]),
            bigside, smallside, angle,
        }]
    }
}

#[derive(Debug)]
pub struct Ellipse {
    center: Vec2DDrawing,
    larger_radius: f64,
    shorter_radius: f64,
    angle: Radians,
    color: Color,
    thickness: f64,
}

impl Ellipse {
    pub fn from_parts(center: Vec2DDrawing, larger_radius: f64, shorter_radius: f64, angle: Radians, color: Color, thickness: f64) -> Ellipse {
        Ellipse {
            center, larger_radius, shorter_radius, angle, color, thickness
        }
    }
}

impl ShapeStored for Ellipse {
    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Ellipse {
            center: self.center,
            bigside: self.larger_radius,
            smallside: self.shorter_radius,
            angle: self.angle,
            color: self.color,
            thickness: self.thickness,
        }
    }

    fn bbox(&self) -> [Vec2DDrawing; 2] {
        todo!()
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Ellipse
    }

    fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool {
        let ([f1, f2], sum) = foci_and_sum_from_center_and_radi(
            self.center, self.larger_radius, self.shorter_radius, self.angle
        );
        // the sum of the distances from the center of the circle to both of the
        // centers of the ellipse
        let sum_of_distances = f1.distance(center) + f2.distance(center);

        // if the sum of the distances from the center of the circle is within
        // an acceptable margin then we consider that the ellipse and the circle
        // are touching. Not true but close.
        sum_of_distances <= sum + 2.0*radius &&
        sum_of_distances >= sum - 2.0*radius
    }

    fn color(&self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::ellipse_from_foci_and_sum;

    use super::*;

    #[test]
    fn test_intersects_circle() {
        let f1: Vec2DDrawing = (-20.22222, -13.24752).into();
        let f2: Vec2DDrawing = (-2.88, 1.28).into();
        let sum = 14.29 + 17.67;

        let (center, bigside, smallside, angle) = ellipse_from_foci_and_sum([f1, f2], sum);

        let ellipse = Ellipse::from_parts(center, bigside, smallside, angle, Default::default(), 1.0);

        let cases = [
            ((-18.23673, -15.70531), true),
            ((2.1475, -11.18778), true),
            ((-5.65939, -2.20061), false),
            ((7.90327, -3.12798), false),
        ];

        for (coords, output) in cases {
            assert!(dbg!(ellipse.intersects_circle(Vec2DDrawing::from(dbg!(coords)), 4.0)) == dbg!(output));
        }
    }
}
