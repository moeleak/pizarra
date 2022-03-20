use std::f64::consts::PI;

use xml::writer::{EventWriter, XmlEvent};

use crate::point::{Vec2D, Unit, WorldUnit};
use crate::draw_commands::DrawCommand;
use crate::geom::{
    self, Angle, distance_from_point_to_ellipse,
};
use crate::shape::{ShapeStored, ShapeType};
use crate::style::Style;

#[derive(Debug)]
pub struct Ellipse {
    ellipse: geom::Ellipse<WorldUnit>,
    style: Style<WorldUnit>,
}

impl Ellipse {
    pub fn from_parts(center: Vec2D<WorldUnit>, semimajor: WorldUnit, semiminor: WorldUnit, angle: Angle, style: Style<WorldUnit>) -> Ellipse {
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

    fn bbox(&self) -> [Vec2D<WorldUnit>; 2] {
        let geom::Ellipse { center, semimajor, semiminor, angle } = self.ellipse;

        let center_x = center.x.val();
        let center_y = center.y.val();
        let semimajor = semimajor.val();
        let semiminor = semiminor.val();

        let fun_t_x = |t: f64| {
            center_x + semimajor * t.cos() * angle.radians().cos() - semiminor * t.sin() * angle.radians().sin()
        };

        let fun_t_y = |t: f64| {
            center_y + semiminor * t.sin() * angle.radians().cos() + semimajor * t.cos() * angle.radians().sin()
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

            (Vec2D::new(-size, -size), Vec2D::new(size, size))
        } else {
            Default::default()
        };

        [
            Vec2D::new_world(min_x, min_y) + topleft,
            Vec2D::new_world(max_x, max_y) + bottomright,
        ]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Ellipse
    }

    fn intersects_circle(&self, center: Vec2D<WorldUnit>, radius: WorldUnit) -> bool {
        distance_from_point_to_ellipse(center, self.ellipse) <= radius + if let Some(s) = self.style.stroke {
            s.size / 2.0
        } else {
            0.0.into()
        }
    }

    fn style(&self) -> Style<WorldUnit> {
        self.style
    }

    fn serialize(&self, writer: &mut EventWriter<&mut Vec<u8>>) {
        writer.write(XmlEvent::start_element("ellipse")
            .attr("cx", &self.ellipse.center.x.to_string())
            .attr("cy", &self.ellipse.center.y.to_string())
            .attr("rx", &self.ellipse.semimajor.to_string())
            .attr("ry", &self.ellipse.semiminor.to_string())
            .attr("transform", &format!(
                    "rotate({angle} {x} {y})",
                    angle = self.ellipse.angle.degrees(),
                    x = self.ellipse.center.x,
                    y = self.ellipse.center.y,
            ))
            .attr("style", &format!("\
                    fill:{fill};\
                    stroke-width:{stroke};\
                    stroke:{color};\
                    stroke-opacity:{alpha};\
                    stroke-miterlimit:10;",
                stroke = self.style.stroke.map(|s| s.size).unwrap_or_else(|| 0.0.into()),
                color = self.style.stroke.map(|s| s.color.css()).unwrap_or_else(|| "none".into()),
                alpha = self.style.stroke.map(|s| s.color.float_alpha()).unwrap_or(1.0),
                fill = self.style.fill.map(|c| c.css()).unwrap_or_else(|| "none".into()),
            ))
        ).unwrap();
        writer.write(XmlEvent::end_element()).unwrap();
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
        let f1: Vec2D<WorldUnit> = (-20.22222, -13.24752).into();
        let f2: Vec2D<WorldUnit> = (-2.88, 1.28).into();
        let sum = 14.29 + 17.67;

        let (center, semimajor, semiminor, angle) = ellipse_from_foci_and_sum([f1, f2], sum.into());

        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, angle, Default::default());

        let cases = [
            ((-18.23673, -15.70531), true),
            ((2.1475, -11.18778), true),
            ((-5.65939, -2.20061), false),
            ((7.90327, -3.12798), false),
        ];

        for (coords, output) in cases {
            assert!(ellipse.intersects_circle(Vec2D::from(coords), 4.0.into()) == output);
        }
    }

    #[test]
    fn the_bbox_of_an_ellipse() {
        let center = Vec2D::new_world(-3.0257300000000003, 1.26254);
        let semimajor = 7.793799303722533.into();
        let semiminor = 6.898753387548061.into();
        let style = Style {
            fill: None,
            stroke: Some(Stroke {
                size: 0.0.into(),
                color: Default::default(),
            }),
        };

        // axis aligned
        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, Angle::from_radians(0.0), style);

        assert_eq!(ellipse.bbox(), [
            center - Vec2D::new(semimajor, semiminor),
            center + Vec2D::new(semimajor, semiminor),
        ]);

        // rotated
        let angle = Angle::from_radians(-0.7769764190469384);
        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, angle, style);

        assert_eq!(ellipse.bbox(), [
            Vec2D::new_world( -10.39314460317701, -6.089827506715496 ),
            Vec2D::new_world( 4.3416846031770095, 8.614907506715497),
        ]);
    }

    #[test]
    fn line_thickness_affects_bbox() {
        let center = Vec2D::new_world(0.0, 0.0);
        let semimajor = 73.0.into();
        let semiminor = 36.0.into();
        let angle = Angle::from_degrees(0.0);
        let style = Style {
            fill: None,
            stroke: Some(Stroke {
                size: 8.0.into(),
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

        let topleft = Vec2D::new_world(-77.0, -40.0);
        let bottomright = Vec2D::new_world(77.0, 40.0);

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

        let topleft = Vec2D::new_world(-73.0, -36.0);
        let bottomright = Vec2D::new_world(73.0, 36.0);

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
            Vec2D::new_world(109.04235, 67.418289),
            77.040901.into(),
            41.799728.into(),
            Angle::from_radians(0.0),
            Style {
                stroke: Some(Stroke {
                    color: Default::default(),
                    size: 8.0.into(),
                }),
                fill: None,
            },
        );

        assert!(ellipse.intersects_circle(Vec2D::new_world(53.297844, 30.258219), 5.0.into()));
        assert!(ellipse.intersects_circle(Vec2D::new_world(107.63097, 110.3745), 5.0.into()));
        assert!(ellipse.intersects_circle(Vec2D::new_world(179.2052, 65.739304), 5.0.into()));

        assert!(!ellipse.intersects_circle(Vec2D::new_world(43.053429, 66.872536), 5.0.into()));
        assert!(!ellipse.intersects_circle(Vec2D::new_world(113.06088, 14.42239), 5.0.into()));
        assert!(!ellipse.intersects_circle(Vec2D::new_world(196.29651, 67.056252), 5.0.into()));
        assert!(!ellipse.intersects_circle(Vec2D::new_world(109.35854, 98.329758), 5.0.into()));
        assert!(!ellipse.intersects_circle(Vec2D::new_world(108.36674, 121.74726), 5.0.into()));
    }

    #[test]
    fn erase_on_touch_stroke_rotated() {
        let ellipse = Ellipse::from_parts(
            Vec2D::new_world(109.26, 66.20),
            77.040901.into(),
            41.799728.into(),
            Angle::from_degrees(-22.534699),
            Style {
                stroke: Some(Stroke {
                    color: Default::default(),
                    size: 8.0.into(),
                }),
                fill: None,
            },
        );

        assert!(ellipse.intersects_circle(Vec2D::new_world(61.263462, 37.067692), 5.0.into()));
        assert!(ellipse.intersects_circle(Vec2D::new_world(79.996864, 114.73833), 5.0.into()));
        assert!(ellipse.intersects_circle(Vec2D::new_world(171.72203, 37.919937), 5.0.into()));

        assert!(!ellipse.intersects_circle(Vec2D::new_world(49.445847, 91.485023), 5.0.into()));
        assert!(!ellipse.intersects_circle(Vec2D::new_world(111.19404, 35.147041), 5.0.into()));
        assert!(!ellipse.intersects_circle(Vec2D::new_world(158.52556, 99.19162), 5.0.into()));
    }

    #[test]
    #[ignore]
    fn erase_on_touch_interior_if_fill() {
        todo!()
    }
}
