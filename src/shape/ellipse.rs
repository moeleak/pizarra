use crate::point::{Point, Vec2DDrawing, Vec2DScreen};
use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::ShapeType;
use crate::transform::Transform;
use crate::geom::{middle_point, parts_of_ellipse, bbox_from_points};

use super::{ShapeBuilder, ShapeFinished};

pub struct Ellipse {
    corners: [Vec2DDrawing; 4],
    thickness: f64,
    color: Color,
}

impl Ellipse {
    pub fn start(color: Color, initial: Vec2DDrawing, thickness: f64) -> Ellipse {
        Ellipse {
            corners: [initial; 4],
            thickness,
            color,
        }
    }

    pub fn with_params(corners: [Vec2DDrawing; 4], color: Color, thickness: f64) -> Ellipse {
        Ellipse {
            corners, thickness, color
        }
    }

    #[cfg(test)]
    pub fn from_corners([corner_1, corner_2]: [Vec2DDrawing; 2]) -> Ellipse {
        Ellipse {
            corners: [
                corner_1,
                Vec2DDrawing::new(corner_2.x, corner_1.y),
                corner_2,
                Vec2DDrawing::new(corner_1.x, corner_2.y),
            ],
            thickness: 4.0,
            color: Color::green(),
        }
    }
}

impl ShapeBuilder for Ellipse {
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

        ShapeFinished::Yes
    }

    fn draw_commands(&self) -> DrawCommand {
        let (bigside, bigvector, smallside) = parts_of_ellipse(self.corners);
        let angle = bigvector.angle();

        DrawCommand::Ellipse {
            thickness: self.thickness,
            color: self.color,
            center: middle_point(self.corners[0], self.corners[2]),
            bigside, smallside, angle,
        }
    }

    fn bbox(&self) -> [Vec2DDrawing; 2] {
        bbox_from_points(self.corners.iter().copied())
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Ellipse
    }

    fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool {
        let corners = self.corners;

        let (bigside, bigvector, smallside) = parts_of_ellipse(corners);

        let ellipse_center = middle_point(corners[0], corners[2]);
        let d = (bigside.powi(2) - smallside.powi(2)).sqrt() / 2.0;

        let center1 = ellipse_center + bigvector.unit() * d;
        let center2 = ellipse_center + bigvector.unit() * (d * -1.0);

        // this variable represents the (constant) sum of the distance from any
        // point of the ellipse to both of its centers
        let ellipse_sum_of_distances = bigside;
        // the sum of the distances from the center of the circle to both of the
        // centers of the ellipse
        let sum_of_distances = center1.distance(center) + center2.distance(center);

        // if the sum of the distances from the center of the circle is within
        // an acceptable margin then we consider that the ellipse and the circle
        // are touching. Not true but close.
        sum_of_distances <= ellipse_sum_of_distances + 2.0*radius &&
        sum_of_distances >= ellipse_sum_of_distances - 2.0*radius
    }

    fn color(&self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersects_circle() {
        let ellipse = Ellipse::from_corners([Vec2DDrawing::new(0.0, 0.0), Vec2DDrawing::new(150.0, 100.0)]);

        let cases = [
            ((126.0, 6.0), true),
            ((5.0, 18.0), true),
            ((9.0, 72.0), true),
            ((25.0, 40.0), false),
            ((134.0, 84.0), false),
        ];

        for (coords, output) in cases {
            assert!(ellipse.intersects_circle(Vec2DDrawing::from(coords) + Vec2DDrawing::new(8.0, 8.0), 8.0) == output);
        }
    }
}
