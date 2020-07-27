use crate::point::Point;
use crate::draw_commands::DrawCommand;
use super::{ShapeTrait, ShapeFinished};
use crate::color::Color;
use crate::ShapeType;

pub struct Ellipse {
    bbox: [Point; 2],
    stroke: f64,
    color: Color,
}

impl Ellipse {
    pub fn new(color: Color, initial: Point, stroke: f64) -> Ellipse {
        Ellipse {
            bbox: [initial; 2],
            stroke,
            color,
        }
    }

    /// Only used for testing
    pub fn from_corners([corner_1, corner_2]: [Point; 2]) -> Ellipse {
        Ellipse {
            bbox: [corner_1, corner_2],
            stroke: 4.0,
            color: Color::green(),
        }
    }
}

impl ShapeTrait for Ellipse {
    fn handle_mouse_moved(&mut self, pos: Point) {
        self.bbox[1] = pos;
    }

    fn handle_button_pressed(&mut self, _pos: Point) { }

    fn handle_button_released(&mut self, pos: Point) -> ShapeFinished {
        self.bbox[1] = pos;
        ShapeFinished::Yes
    }

    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Ellipse {
            thickness: self.stroke,
            color: self.color,
            bbox: self.bbox,
        }
    }

    fn bbox(&self) -> [[f64; 2]; 2] {
        [
            self.bbox[0].min(self.bbox[1]).to_a(),
            self.bbox[0].max(self.bbox[1]).to_a(),
        ]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Ellipse
    }

    fn intersects_circle(&self, center: Point, radius: f64) -> bool {
        let min = self.bbox[0].min(self.bbox[1]);
        let max = self.bbox[0].max(self.bbox[1]);

        let dimensions = max - min;
        let ellipse_center = dimensions/2.0 + min;
        let bigside = dimensions.x.max(dimensions.y);
        let smallside = dimensions.x.min(dimensions.y);

        let d = (bigside.powi(2) - smallside.powi(2)).sqrt() / 2.0;
        let (center1, center2) = if dimensions.x > dimensions.y {
            (ellipse_center + Point::new(d, 0.0), ellipse_center + Point::new(-d, 0.0))
        } else {
            (ellipse_center + Point::new(0.0, d), ellipse_center + Point::new(0.0, -d))
        };

        dbg!((center1, center2));

        let ellipse_sum_of_distances = bigside;
        dbg!(ellipse_sum_of_distances);
        let sum_of_distances = center1.distance(center) + center2.distance(center);
        dbg!(sum_of_distances);

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
    use crate::point::Point;
    use crate::shape::ShapeTrait;

    #[test]
    fn test_intersects_circle() {
        let ellipse = Ellipse::from_corners([Point::new(0.0, 0.0), Point::new(150.0, 100.0)]);

        let cases = [
            ((126.0, 6.0), true),
            ((5.0, 18.0), true),
            ((9.0, 72.0), true),
            ((25.0, 40.0), false),
            ((134.0, 84.0), false),
        ];

        for (coords, output) in cases.iter().map(|t| *t) {
            assert!(ellipse.intersects_circle(dbg!(Point::from(coords)) + Point::new(8.0, 8.0), 8.0) == dbg!(output));
        }
    }
}
