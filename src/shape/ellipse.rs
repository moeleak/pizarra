use crate::point::Vec2DDrawing;
use crate::draw_commands::DrawCommand;
use super::{ShapeTrait, ShapeFinished};
use crate::color::Color;
use crate::ShapeType;

pub struct Ellipse {
    bbox: [Vec2DDrawing; 2],
    thickness: f64,
    color: Color,
}

impl Ellipse {
    pub fn new(color: Color, initial: Vec2DDrawing, thickness: f64) -> Ellipse {
        Ellipse {
            bbox: [initial; 2],
            thickness,
            color,
        }
    }

    #[cfg(test)]
    pub fn from_corners([corner_1, corner_2]: [Vec2DDrawing; 2]) -> Ellipse {
        Ellipse {
            bbox: [corner_1, corner_2],
            thickness: 4.0,
            color: Color::green(),
        }
    }

    pub fn with_params(bbox: [Vec2DDrawing; 2], color: Color, thickness: f64) -> Ellipse {
        Ellipse {
            bbox, color, thickness,
        }
    }
}

impl ShapeTrait for Ellipse {
    fn handle_mouse_moved(&mut self, pos: Vec2DDrawing) {
        self.bbox[1] = pos;
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DDrawing) { }

    fn handle_button_released(&mut self, pos: Vec2DDrawing) -> ShapeFinished {
        self.bbox[1] = pos;
        ShapeFinished::Yes
    }

    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Ellipse {
            thickness: self.thickness,
            color: self.color,
            bbox: self.bbox,
        }
    }

        [
            self.bbox[0].min(self.bbox[1]).to_a(),
            self.bbox[0].max(self.bbox[1]).to_a(),
        ]
    fn bbox(&self) -> [Vec2DDrawing; 2] {
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Ellipse
    }

    fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool {
        let min = self.bbox[0].min(self.bbox[1]);
        let max = self.bbox[0].max(self.bbox[1]);

        let dimensions = max - min;
        let ellipse_center = dimensions/2.0 + min;
        let bigside = dimensions.x.max(dimensions.y);
        let smallside = dimensions.x.min(dimensions.y);

        let d = (bigside.powi(2) - smallside.powi(2)).sqrt() / 2.0;
        let (center1, center2) = if dimensions.x > dimensions.y {
            (ellipse_center + Vec2DDrawing::new(d, 0.0), ellipse_center + Vec2DDrawing::new(-d, 0.0))
        } else {
            (ellipse_center + Vec2DDrawing::new(0.0, d), ellipse_center + Vec2DDrawing::new(0.0, -d))
        };

        let ellipse_sum_of_distances = bigside;
        let sum_of_distances = center1.distance(center) + center2.distance(center);

        sum_of_distances <= ellipse_sum_of_distances + 2.0*radius &&
        sum_of_distances >= ellipse_sum_of_distances - 2.0*radius
    }

    fn color(&self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use super::Ellipse;
    use crate::point::Vec2DDrawing;
    use crate::shape::ShapeTrait;

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

        for (coords, output) in cases.iter().map(|t| *t) {
            assert!(ellipse.intersects_circle(Vec2DDrawing::from(coords) + Vec2DDrawing::new(8.0, 8.0), 8.0) == output);
        }
    }
}
