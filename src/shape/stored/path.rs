use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::point::Vec2DWorld;
use crate::geom::bbox_from_points;
use crate::path_command::PathCommand;
use crate::shape::{ShapeStored, ShapeType};

#[derive(Debug)]
pub struct Path {
    commands: Vec<PathCommand<Vec2DWorld>>,
    color: Option<Color>,
    fill: Option<Color>,
    thickness: f64,
}

impl Path {
    pub fn from_parts(commands: Vec<PathCommand<Vec2DWorld>>, color: Option<Color>, fill: Option<Color>, thickness: f64) -> Path {
        Path {
            commands, color, fill, thickness,
        }
    }
}

impl ShapeStored for Path {
    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Path {
            commands: self.commands.clone(),
            thickness: self.thickness,
            color: self.color,
            fill: None,
        }
    }

    // The bounding box of the path needs to consider the beizer handles
    fn bbox(&self) -> [Vec2DWorld; 2] {
        let points = self.commands.iter().map(|c| c.points()).flatten();

        bbox_from_points(points)
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Path
    }

    fn intersects_circle(&self, center: Vec2DWorld, radius: f64) -> bool {
        let mut last_point = None;

        for c in self.commands.iter() {
            if c.intersects_circle(center, radius, last_point) {
                return true;
            }

            last_point = Some(c.to());
        }

        false
    }

    fn color(&self) -> Option<Color> {
        self.color
    }

    fn fill(&self) -> Option<Color> {
        self.fill
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::color::Color;
    use crate::point::{Point, Vec2DWorld};
    use crate::path_command::CubicBezierCurve;

    #[test]
    fn test_bbox() {
        let line = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(1.0, 0.0)),
            PathCommand::LineTo(Vec2DWorld::new(0.0, 1.0)),
        ], Some(Color::green()), None, 4.0);

        assert_eq!(line.bbox(), [Vec2DWorld::new(0.0, 0.0), Vec2DWorld::new(1.0, 1.0)]);
    }

    #[test]
    fn test_bbox_twisted_line() {
        let line = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(-12.0, -1.0)),
            PathCommand::LineTo(Vec2DWorld::new(-5.0, 0.0)),
            PathCommand::LineTo(Vec2DWorld::new(-2.0, 7.0)),
            PathCommand::LineTo(Vec2DWorld::new(2.0, -8.0)),
        ], Some(Color::green()), None, 1.0);

        assert_eq!(line.bbox(), [Vec2DWorld::new(-12.0, -8.0), Vec2DWorld::new(2.0, 7.0)]);
    }

    #[test]
    fn can_delete_single_point_paths() {
        let poly = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(0.0, 0.0)),
        ], Some(Color::green()), None, 1.0);

        assert!(poly.intersects_circle(Vec2DWorld::new(0.0, 0.0), 10.0));
    }

    #[test]
    fn can_delete_beizer_curves() {
        let path = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(33.0, 135.0)),
            PathCommand::CurveTo(CubicBezierCurve {
                pt1: Vec2DWorld::new(50.0, 200.0),
                pt2: Vec2DWorld::new(171.0, 70.0),
                to: Vec2DWorld::new(196.0, 113.0),
            }),
        ], Some(Color::green()), None, 1.0);

        let cases = [
             (Vec2DWorld::new(37.0, 129.0), true),
             (Vec2DWorld::new(81.0, 154.0), true),
             (Vec2DWorld::new(127.0, 109.0), true),
             (Vec2DWorld::new(74.0, 90.0), false),
             (Vec2DWorld::new(147.0, 165.0), false),
             (Vec2DWorld::new(177.0, 121.0), false),
        ];

        for (center, result) in cases {
            assert_eq!(path.intersects_circle(center, 15.0), result);
        }
    }

    #[test]
    fn can_delete_straight_segments() {
        let path = Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(33.0, 135.0)),
            PathCommand::LineTo(Vec2DWorld::new(196.0, 113.0)),
        ], Default::default(), None, 1.0);

        let cases = [
             (Vec2DWorld::new(37.0, 129.0), true),
             (Vec2DWorld::new(81.0, 154.0), false),
             (Vec2DWorld::new(127.0, 109.0), true),
             (Vec2DWorld::new(74.0, 90.0), false),
             (Vec2DWorld::new(147.0, 165.0), false),
             (Vec2DWorld::new(177.0, 121.0), true),
        ];

        for (center, result) in cases {
            assert_eq!(path.intersects_circle(center, 15.0), result);
        }
    }

    #[test]
    fn line_thickness_affects_bbox() {
        todo!()
    }
}
