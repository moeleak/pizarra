use std::f64::consts::PI;
use crate::point::{Point, Vec2DWorld};
use crate::draw_commands::DrawCommand;
use crate::geom::{
    self, Angle, distance_from_point_to_ellipse,
};
use crate::shape::{ShapeStored, ShapeType};
use crate::style::Style;

#[derive(Debug)]
pub struct Ellipse {
    ellipse: geom::Ellipse<Vec2DWorld>,
    style: Style,
}

impl Ellipse {
    pub fn from_parts(center: Vec2DWorld, semimajor: f64, semiminor: f64, angle: Angle, style: Style) -> Ellipse {
        Ellipse {
            ellipse: geom::Ellipse {
                center, semimajor, semiminor, angle,
            },
            style,
        }
    }
}

impl ShapeStored for Ellipse {
    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Ellipse {
            ellipse: self.ellipse,
            style: self.style,
        }
    }

    fn bbox(&self) -> [Vec2DWorld; 2] {
        let geom::Ellipse { center, semimajor, semiminor, angle } = self.ellipse;

        let fun_t_x = |t: f64| {
            center.x + semimajor * t.cos() * angle.radians().cos() - semiminor * t.sin() * angle.radians().sin()
        };

        let fun_t_y = |t: f64| {
            center.y + semiminor * t.sin() * angle.radians().cos() + semimajor * t.cos() * angle.radians().sin()
        };

        let t = (-(semiminor / semimajor) * angle.radians().tan()).atan();
        let x1 = fun_t_x(t);
        let x2 = fun_t_x(t + PI);
        let [min_x, max_x] = [x1.min(x2), x1.max(x2)];

        let t = ((semiminor / semimajor) * (1.0 / angle.radians().tan())).atan();
        let y1 = fun_t_y(t);
        let y2 = fun_t_y(t + PI);
        let [min_y, max_y] = [y1.min(y2), y1.max(y2)];

        let (topleft, bottomright) = if let Some(stroke) = self.style.stroke {
            let size = stroke.size / 2.0;

            (Vec2DWorld::new(-size, -size), Vec2DWorld::new(size, size))
        } else {
            Default::default()
        };

        [
            Vec2DWorld::new(min_x, min_y) + topleft,
            Vec2DWorld::new(max_x, max_y) + bottomright,
        ]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Ellipse
    }

    fn intersects_circle(&self, center: Vec2DWorld, radius: f64) -> bool {
        distance_from_point_to_ellipse(center, self.ellipse) <= radius + if let Some(s) = self.style.stroke {
            s.size / 2.0
        } else {
            0.0
        }
    }

    fn style(&self) -> Style {
        self.style
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::geom::ellipse_from_foci_and_sum;
    use crate::style::{Style, Stroke};

    use super::*;

    #[test]
    fn test_intersects_circle() {
        let f1: Vec2DWorld = (-20.22222, -13.24752).into();
        let f2: Vec2DWorld = (-2.88, 1.28).into();
        let sum = 14.29 + 17.67;

        let (center, semimajor, semiminor, angle) = ellipse_from_foci_and_sum([f1, f2], sum);

        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, angle, Default::default());

        let cases = [
            ((-18.23673, -15.70531), true),
            ((2.1475, -11.18778), true),
            ((-5.65939, -2.20061), false),
            ((7.90327, -3.12798), false),
        ];

        for (coords, output) in cases {
            assert!(ellipse.intersects_circle(Vec2DWorld::from(coords), 4.0) == output);
        }
    }

    #[test]
    fn the_bbox_of_an_ellipse() {
        let center = Vec2DWorld::new(-3.0257300000000003, 1.26254);
        let semimajor = 7.793799303722533;
        let semiminor = 6.898753387548061;
        let style = Style {
            fill: None,
            stroke: Some(Stroke {
                size: 0.0,
                color: Default::default(),
            }),
        };

        // axis aligned
        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, Angle::from_radians(0.0), style);

        assert_eq!(ellipse.bbox(), [
            center - Vec2DWorld::new(semimajor, semiminor),
            center + Vec2DWorld::new(semimajor, semiminor),
        ]);

        // rotated
        let angle = Angle::from_radians(-0.7769764190469384);
        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, angle, style);

        assert_eq!(ellipse.bbox(), [
            Vec2DWorld { x: -10.39314460317701, y: -6.089827506715496 },
            Vec2DWorld { x: 4.3416846031770095, y: 8.614907506715497 },
        ]);
    }

    #[test]
    fn line_thickness_affects_bbox() {
        let center = Vec2DWorld::new(0.0, 0.0);
        let semimajor = 73.0;
        let semiminor = 36.0;
        let angle = Angle::from_degrees(0.0);
        let style = Style {
            fill: None,
            stroke: Some(Stroke {
                size: 8.0,
                color: Default::default(),
            }),
        };

        let ellipse = Ellipse::from_parts(
            center,
            semimajor,
            semiminor,
            angle,
            style,
        );

        let topleft = Vec2DWorld::new(-77.0, -40.0);
        let bottomright = Vec2DWorld::new(77.0, 40.0);

        assert_eq!(ellipse.bbox(), [
            topleft,
            bottomright,
        ]);

        let style = Style {
            fill: None,
            stroke: None,
        };
        let ellipse = Ellipse::from_parts(
            center,
            semimajor,
            semiminor,
            angle,
            style,
        );

        let topleft = Vec2DWorld::new(-73.0, -36.0);
        let bottomright = Vec2DWorld::new(73.0, 36.0);

        assert_eq!(ellipse.bbox(), [
            topleft,
            bottomright,
        ]);

        // there is no need to take into account the zoom level here since the
        // returned bbox is in world units. Meaning that when it is queried it
        // will also be queried in world units
    }

    #[test]
    fn erase_on_touch_stroke() {
        let ellipse = Ellipse::from_parts(
            Vec2DWorld::new(109.04235, 67.418289),
            77.040901,
            41.799728,
            Angle::from_radians(0.0),
            Style {
                stroke: Some(Stroke {
                    color: Default::default(),
                    size: 8.0,
                }),
                fill: None,
            },
        );

        assert!(ellipse.intersects_circle(Vec2DWorld::new(53.297844, 30.258219), 5.0));
        assert!(ellipse.intersects_circle(Vec2DWorld::new(107.63097, 110.3745), 5.0));
        assert!(ellipse.intersects_circle(Vec2DWorld::new(179.2052, 65.739304), 5.0));

        assert!(!ellipse.intersects_circle(Vec2DWorld::new(43.053429, 66.872536), 5.0));
        assert!(!ellipse.intersects_circle(Vec2DWorld::new(113.06088, 14.42239), 5.0));
        assert!(!ellipse.intersects_circle(Vec2DWorld::new(196.29651, 67.056252), 5.0));
        assert!(!ellipse.intersects_circle(Vec2DWorld::new(109.35854, 98.329758), 5.0));
        assert!(!ellipse.intersects_circle(Vec2DWorld::new(108.36674, 121.74726), 5.0));
    }

    #[test]
    fn erase_on_touch_stroke_rotated() {
        let ellipse = Ellipse::from_parts(
            Vec2DWorld::new(109.26, 66.20),
            77.040901,
            41.799728,
            Angle::from_degrees(-22.534699),
            Style {
                stroke: Some(Stroke {
                    color: Default::default(),
                    size: 8.0,
                }),
                fill: None,
            },
        );

        assert!(ellipse.intersects_circle(Vec2DWorld::new(61.263462, 37.067692), 5.0));
        assert!(ellipse.intersects_circle(Vec2DWorld::new(79.996864, 114.73833), 5.0));
        assert!(ellipse.intersects_circle(Vec2DWorld::new(171.72203, 37.919937), 5.0));

        assert!(!ellipse.intersects_circle(Vec2DWorld::new(49.445847, 91.485023), 5.0));
        assert!(!ellipse.intersects_circle(Vec2DWorld::new(111.19404, 35.147041), 5.0));
        assert!(!ellipse.intersects_circle(Vec2DWorld::new(158.52556, 99.19162), 5.0));
    }

    #[test]
    #[ignore]
    fn erase_on_touch_interior_if_fill() {
        todo!()
    }
}
