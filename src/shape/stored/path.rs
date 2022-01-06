use crate::draw_commands::DrawCommand;
use crate::point::Vec2DWorld;
use crate::geom::bbox_from_points;
use crate::path_command::PathCommand;
use crate::shape::{ShapeStored, ShapeType};
use crate::style::Style;
use crate::point::Point;

#[derive(Debug)]
pub struct Path {
    commands: Vec<PathCommand<Vec2DWorld>>,
    style: Style,
}

impl Path {
    pub fn from_parts(commands: Vec<PathCommand<Vec2DWorld>>, style: Style) -> Path {
        Path {
            commands, style,
        }
    }
}

impl ShapeStored for Path {
    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Path {
            commands: self.commands.clone(),
            style: self.style,
        }
    }

    // The bounding box of the path needs to consider the beizer handles
    fn bbox(&self) -> [Vec2DWorld; 2] {
        let points = self.commands.iter().map(|c| c.points()).flatten();
        let [topleft, bottomright] = bbox_from_points(points);
        let (topleftpad, bottomrightpad) = if let Some(stroke) = self.style.stroke {
            let size = stroke.size;

            (Vec2DWorld::new(-size, -size), Vec2DWorld::new(size, size))
        } else {
            Default::default()
        };

        [
            topleft + topleftpad,
            bottomright + bottomrightpad,
        ]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Path
    }

    fn intersects_circle(&self, center: Vec2DWorld, radius: f64) -> bool {
        let mut last_point = None;
        let radius = if let Some(stroke) = self.style.stroke {
            radius + stroke.size / 2.0
        } else {
            radius
        };

        for c in self.commands.iter() {
            if c.intersects_circle(center, radius, last_point) {
                return true;
            }

            last_point = Some(c.to());
        }

        false
    }

    fn style(&self) -> Style {
        self.style
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::point::{Point, Vec2DWorld};
    use crate::path_command::{PathCommand::*, CubicBezierCurve};

    use super::*;

    #[test]
    fn test_bbox() {
        let line = Path::from_parts(vec![
            MoveTo(Vec2DWorld::new(1.0, 0.0)),
            LineTo(Vec2DWorld::new(0.0, 1.0)),
        ], Style {
            stroke: None,
            fill: None,
        });

        assert_eq!(line.bbox(), [Vec2DWorld::new(0.0, 0.0), Vec2DWorld::new(1.0, 1.0)]);
    }

    #[test]
    fn test_bbox_twisted_line() {
        let line = Path::from_parts(vec![
            MoveTo(Vec2DWorld::new(-12.0, -1.0)),
            LineTo(Vec2DWorld::new(-5.0, 0.0)),
            LineTo(Vec2DWorld::new(-2.0, 7.0)),
            LineTo(Vec2DWorld::new(2.0, -8.0)),
        ], Style {
            stroke: None,
            fill: None,
        });

        assert_eq!(line.bbox(), [Vec2DWorld::new(-12.0, -8.0), Vec2DWorld::new(2.0, 7.0)]);
    }

    #[test]
    fn can_delete_single_point_paths() {
        let poly = Path::from_parts(vec![
            MoveTo(Vec2DWorld::new(0.0, 0.0)),
        ], Default::default());

        assert!(poly.intersects_circle(Vec2DWorld::new(0.0, 0.0), 10.0));
    }

    #[test]
    fn can_delete_beizer_curves() {
        let path = Path::from_parts(vec![
            MoveTo(Vec2DWorld::new(33.0, 135.0)),
            CurveTo(CubicBezierCurve {
                pt1: Vec2DWorld::new(50.0, 200.0),
                pt2: Vec2DWorld::new(171.0, 70.0),
                to: Vec2DWorld::new(196.0, 113.0),
            }),
        ], Default::default());

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
            MoveTo(Vec2DWorld::new(33.0, 135.0)),
            LineTo(Vec2DWorld::new(196.0, 113.0)),
        ], Default::default());

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
        let path = Path::from_parts(vec![
            MoveTo(Vec2DWorld::new(0.0, 0.0)),
            LineTo(Vec2DWorld::new(30.0, 30.0)),
            CurveTo(CubicBezierCurve {
                pt1: Vec2DWorld::new(30.0, 60.0),
                pt2: Vec2DWorld::new(60.0, 60.0),
                to: Vec2DWorld::new(60.0, 30.0),
            }),
        ], Default::default());

        let stroke = Style::default().stroke.unwrap().size;

        assert_eq!(path.bbox(), [
            Vec2DWorld::new(0.0 - stroke, 0.0 - stroke),
            Vec2DWorld::new(60.0 + stroke, 60.0 + stroke),
        ]);
    }

    #[test]
    fn erase_on_touch_stroke() {
        todo!()
    }

    #[test]
    fn erase_on_touch_interior_if_fill() {
        todo!()
    }
}
