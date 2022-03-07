use crate::draw_commands::DrawCommand;
use crate::point::{Vec2D, WorldUnit};
use crate::geom::bbox_from_points;
use crate::path_command::PathCommand;
use crate::shape::{ShapeStored, ShapeType};
use crate::style::Style;

#[derive(Debug)]
pub struct Path {
    commands: Vec<PathCommand<WorldUnit>>,
    style: Style<WorldUnit>,
}

impl Path {
    pub fn from_parts(commands: Vec<PathCommand<WorldUnit>>, style: Style<WorldUnit>) -> Path {
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
    fn bbox(&self) -> [Vec2D<WorldUnit>; 2] {
        let points = self.commands.iter().flat_map(|c| c.points());
        let [topleft, bottomright] = bbox_from_points(points);
        let (topleftpad, bottomrightpad) = if let Some(stroke) = self.style.stroke {
            let size = stroke.size;

            (Vec2D::new(-size, -size), Vec2D::new(size, size))
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

    fn intersects_circle(&self, center: Vec2D<WorldUnit>, radius: WorldUnit) -> bool {
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

    fn style(&self) -> Style<WorldUnit> {
        self.style
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::path_command::{PathCommand::*, CubicBezierCurve};
    use crate::style::Stroke;

    use super::*;

    #[test]
    fn test_bbox() {
        let line = Path::from_parts(vec![
            MoveTo(Vec2D::new_world(1.0, 0.0)),
            LineTo(Vec2D::new_world(0.0, 1.0)),
        ], Style {
            stroke: None,
            fill: None,
        });

        assert_eq!(line.bbox(), [Vec2D::new_world(0.0, 0.0), Vec2D::new_world(1.0, 1.0)]);
    }

    #[test]
    fn test_bbox_twisted_line() {
        let line = Path::from_parts(vec![
            MoveTo(Vec2D::new_world(-12.0, -1.0)),
            LineTo(Vec2D::new_world(-5.0, 0.0)),
            LineTo(Vec2D::new_world(-2.0, 7.0)),
            LineTo(Vec2D::new_world(2.0, -8.0)),
        ], Style {
            stroke: None,
            fill: None,
        });

        assert_eq!(line.bbox(), [Vec2D::new_world(-12.0, -8.0), Vec2D::new_world(2.0, 7.0)]);
    }

    #[test]
    fn can_delete_single_point_paths() {
        let poly = Path::from_parts(vec![
            MoveTo(Vec2D::new_world(0.0, 0.0)),
        ], Default::default());

        assert!(poly.intersects_circle(Vec2D::new_world(0.0, 0.0), 10.0.into()));
    }

    #[test]
    fn can_delete_beizer_curves() {
        let path = Path::from_parts(vec![
            MoveTo(Vec2D::new_world(33.0, 135.0)),
            CurveTo(CubicBezierCurve {
                pt1: Vec2D::new_world(50.0, 200.0),
                pt2: Vec2D::new_world(171.0, 70.0),
                to: Vec2D::new_world(196.0, 113.0),
            }),
        ], Default::default());

        let cases = [
             (Vec2D::new_world(37.0, 129.0), true),
             (Vec2D::new_world(81.0, 154.0), true),
             (Vec2D::new_world(127.0, 109.0), true),
             (Vec2D::new_world(74.0, 90.0), false),
             (Vec2D::new_world(147.0, 165.0), false),
             (Vec2D::new_world(177.0, 121.0), false),
        ];

        for (center, result) in cases {
            assert_eq!(path.intersects_circle(center, 15.0.into()), result);
        }
    }

    #[test]
    fn can_delete_straight_segments() {
        let path = Path::from_parts(vec![
            MoveTo(Vec2D::new_world(33.0, 135.0)),
            LineTo(Vec2D::new_world(196.0, 113.0)),
        ], Default::default());

        let cases = [
             (Vec2D::new_world(37.0, 129.0), true),
             (Vec2D::new_world(81.0, 154.0), false),
             (Vec2D::new_world(127.0, 109.0), true),
             (Vec2D::new_world(74.0, 90.0), false),
             (Vec2D::new_world(147.0, 165.0), false),
             (Vec2D::new_world(177.0, 121.0), true),
        ];

        for (center, result) in cases {
            assert_eq!(path.intersects_circle(center, 15.0.into()), result);
        }
    }

    #[test]
    fn line_thickness_affects_bbox() {
        let path = Path::from_parts(vec![
            MoveTo(Vec2D::new_world(0.0, 0.0)),
            LineTo(Vec2D::new_world(30.0, 30.0)),
            CurveTo(CubicBezierCurve {
                pt1: Vec2D::new_world(30.0, 60.0),
                pt2: Vec2D::new_world(60.0, 60.0),
                to: Vec2D::new_world(60.0, 30.0),
            }),
        ], Default::default());

        let stroke: WorldUnit = Style::default().stroke.unwrap().size;

        assert_eq!(path.bbox(), [
            Vec2D::new_world(0.0, 0.0) - Vec2D::new(stroke, stroke),
            Vec2D::new_world(60.0, 60.0) + Vec2D::new(stroke, stroke),
        ]);
    }

    #[test]
    fn erase_on_touch_stroke() {
        let start = Vec2D::new_world(27.410791, 119.70396);
        let path = Path::from_parts(vec![
            MoveTo(start),
            CurveTo(CubicBezierCurve {
                pt1: start + Vec2D::new_world(0.06519,-10.40955),
                pt2: start + Vec2D::new_world(11.676337,-10.01372),
                to: start + Vec2D::new_world(11.966013,0.48172),
            }),
        ], Style {
            stroke: Some(Stroke {
                color: Default::default(),
                size: 2.1166.into(),
            }),
            fill: None,
        });

        // these must touch
        assert!(path.intersects_circle(Vec2D::new_world(29.841375, 116.64932), 1.3229166.into()));
        assert!(path.intersects_circle(Vec2D::new_world(33.242252, 110.06812), 1.3229166.into()));
        assert!(path.intersects_circle(Vec2D::new_world(37.421051, 120.87376), 1.3229166.into()));

        // these must touchn't
        assert!(!path.intersects_circle(Vec2D::new_world(39.535831, 112.48697), 1.3229166.into()));
        assert!(!path.intersects_circle(Vec2D::new_world(34.404907, 115.59615), 1.3229166.into()));
        assert!(!path.intersects_circle(Vec2D::new_world(27.536837, 110.98792), 1.3229166.into()));
    }

    #[test]
    #[ignore]
    fn erase_on_touch_interior_if_fill() {
        todo!()
    }
}
