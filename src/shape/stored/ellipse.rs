use std::f64::consts::PI;
use crate::point::{Point, Vec2DWorld};
use crate::draw_commands::{DrawCommand, self};
use crate::geom::{
    Angle, foci_and_sum_from_center_and_radi,
};
use crate::shape::{ShapeStored, ShapeType};
use crate::style::Style;

#[derive(Debug)]
pub struct Ellipse {
    center: Vec2DWorld,
    semimajor: f64,
    semiminor: f64,
    angle: Angle,
    style: Style,
}

impl Ellipse {
    pub fn from_parts(center: Vec2DWorld, semimajor: f64, semiminor: f64, angle: Angle, style: Style) -> Ellipse {
        Ellipse {
            center, semimajor, semiminor, angle, style,
        }
    }
}

impl ShapeStored for Ellipse {
    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Ellipse(draw_commands::Ellipse {
            center: self.center,
            semimajor: self.semimajor,
            semiminor: self.semiminor,
            angle: self.angle,
            style: self.style,
        })
    }

    fn bbox(&self) -> [Vec2DWorld; 2] {
        let fun_t_x = |t: f64| {
            self.center.x + self.semimajor * t.cos() * self.angle.radians().cos() - self.semiminor * t.sin() * self.angle.radians().sin()
        };

        let fun_t_y = |t: f64| {
            self.center.y + self.semiminor * t.sin() * self.angle.radians().cos() + self.semimajor * t.cos() * self.angle.radians().sin()
        };

        let t = (-(self.semiminor / self.semimajor) * self.angle.radians().tan()).atan();
        let x1 = fun_t_x(t);
        let x2 = fun_t_x(t + PI);
        let [min_x, max_x] = [x1.min(x2), x1.max(x2)];

        let t = ((self.semiminor / self.semimajor) * (1.0 / self.angle.radians().tan())).atan();
        let y1 = fun_t_y(t);
        let y2 = fun_t_y(t + PI);
        let [min_y, max_y] = [y1.min(y2), y1.max(y2)];

        [
            Vec2DWorld::new(min_x, min_y),
            Vec2DWorld::new(max_x, max_y),
        ]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Ellipse
    }

    fn intersects_circle(&self, center: Vec2DWorld, radius: f64) -> bool {
        let ([f1, f2], sum) = foci_and_sum_from_center_and_radi(
            self.center, self.semimajor, self.semiminor, self.angle
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

        // axis aligned
        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, Angle::from_radians(0.0), Default::default());

        assert_eq!(ellipse.bbox(), [
            center - Vec2DWorld::new(semimajor, semiminor),
            center + Vec2DWorld::new(semimajor, semiminor),
        ]);

        // rotated
        let angle = Angle::from_radians(-0.7769764190469384);
        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, angle, Default::default());

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

        todo!("but only if it has stroke");
    }
}
