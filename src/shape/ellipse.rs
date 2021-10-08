use std::f64::consts::PI;
use crate::point::{Point, Vec2DDrawing, Vec2DScreen};
use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::ShapeType;
use crate::transform::Transform;
use crate::geom::{
    Radians, ellipse_from_foci_and_point, foci_and_sum_from_center_and_radi,
};
use crate::shape::path::PathCommand;

use super::{ShapeBuilder, ShapeStored, ShapeFinished};

#[derive(Debug, Copy, Clone)]
enum ThreePointEllipseState {
    One(Vec2DDrawing),
    Foci([Vec2DDrawing; 2]),
    FociAndPoint {
        foci: [Vec2DDrawing; 2],
        point: Vec2DDrawing,
    }
}

pub struct ThreePointEllipseBuilder {
    state: ThreePointEllipseState,
    thickness: f64,
    color: Color,
}

impl ThreePointEllipseBuilder {
    pub fn start(color: Color, initial: Vec2DDrawing, thickness: f64) -> ThreePointEllipseBuilder {
        ThreePointEllipseBuilder {
            state: ThreePointEllipseState::One(initial),
            thickness,
            color,
        }
    }
}

impl ShapeBuilder for ThreePointEllipseBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform) {
        match self.state {
            ThreePointEllipseState::One(_) => {
                self.state = ThreePointEllipseState::One(t.to_storage_coordinates(pos));
            }
            ThreePointEllipseState::Foci([f1, _]) => {
                self.state = ThreePointEllipseState::Foci([f1, t.to_storage_coordinates(pos)]);
            }
            ThreePointEllipseState::FociAndPoint {
                foci,
                point: _,
            } => {
                self.state = ThreePointEllipseState::FociAndPoint {
                    foci, point: t.to_storage_coordinates(pos),
                }
            }
        }
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _: Transform) { }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform) -> ShapeFinished {
        match self.state {
            ThreePointEllipseState::One(f1) => {
                self.state = ThreePointEllipseState::Foci([f1, t.to_storage_coordinates(pos)]);

                ShapeFinished::No
            }
            ThreePointEllipseState::Foci(foci) => {
                self.state = ThreePointEllipseState::FociAndPoint {
                    foci,
                    point: t.to_storage_coordinates(pos),
                };

                ShapeFinished::No
            }
            ThreePointEllipseState::FociAndPoint {
                foci, ..
            } => {
                let (center, semimajor, semiminor, angle) = ellipse_from_foci_and_point(foci, t.to_storage_coordinates(pos));

                ShapeFinished::Yes(Box::new(Ellipse::from_parts(
                    center, semimajor, semiminor, angle, self.color, self.thickness,
                )))
            }
        }
    }

    fn draw_commands(&self) -> Vec<DrawCommand> {
        match self.state {
            ThreePointEllipseState::One(f1) => vec![DrawCommand::Path {
                color: self.color,
                thickness: self.thickness,
                commands: vec![PathCommand::MoveTo(f1)],
            }],
            ThreePointEllipseState::Foci([f1, f2]) => vec![DrawCommand::Path {
                color: self.color,
                thickness: self.thickness,
                commands: vec![PathCommand::MoveTo(f1), PathCommand::LineTo(f2)],
            }],
            ThreePointEllipseState::FociAndPoint {
                foci, point,
            } => {
                let (center, semimajor, semiminor, angle) = ellipse_from_foci_and_point(foci, point);

                vec![DrawCommand::Ellipse {
                    thickness: self.thickness,
                    color: self.color,
                    center, semimajor, semiminor, angle,
                }]
            }
        }
    }
}

#[derive(Debug)]
pub struct Ellipse {
    center: Vec2DDrawing,
    semimajor: f64,
    semiminor: f64,
    angle: Radians,
    color: Color,
    thickness: f64,
}

impl Ellipse {
    pub fn from_parts(center: Vec2DDrawing, semimajor: f64, semiminor: f64, angle: Radians, color: Color, thickness: f64) -> Ellipse {
        Ellipse {
            center, semimajor, semiminor, angle, color, thickness
        }
    }
}

impl ShapeStored for Ellipse {
    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Ellipse {
            center: self.center,
            semimajor: self.semimajor,
            semiminor: self.semiminor,
            angle: self.angle,
            color: self.color,
            thickness: self.thickness,
        }
    }

    fn bbox(&self) -> [Vec2DDrawing; 2] {
        let ([f1, f2], _) = foci_and_sum_from_center_and_radi(
            self.center,
            self.semimajor,
            self.semiminor,
            self.angle
        );

        // half the distance between the foci
        let c = f1.distance(f2) / 2.0;
        let rx = self.semiminor.powi(2) / (self.semimajor + c * self.angle.radians().cos());
        let ry = self.semiminor.powi(2) / (self.semimajor + c * (self.angle.radians() + PI/2.0).cos());

        let v = Vec2DDrawing::new(dbg!(rx), dbg!(ry));

        [
            self.center - v,
            self.center + v,
        ]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::ThreePointEllipse
    }

    fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool {
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

        let (center, semimajor, semiminor, angle) = ellipse_from_foci_and_sum([f1, f2], sum);

        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, angle, Default::default(), 1.0);

        let cases = [
            ((-18.23673, -15.70531), true),
            ((2.1475, -11.18778), true),
            ((-5.65939, -2.20061), false),
            ((7.90327, -3.12798), false),
        ];

        for (coords, output) in cases {
            assert!(ellipse.intersects_circle(Vec2DDrawing::from(coords), 4.0) == output);
        }
    }

    #[test]
    fn three_point_ellipse_builder() {
        let a0: Vec2DScreen = (-5.69132,5.13608).into();
        let a1: Vec2DScreen = (-6.41006,4.44396).into();
        let a: Vec2DScreen = (-5.61146,3.80508).into();
        let b: Vec2DScreen = (-4.36,0.52).into();
        let c: Vec2DScreen = (-0.44,-1.28).into();
        let d: Vec2DScreen = (2.2,0.56).into();
        let e: Vec2DScreen = (4.3,-0.4).into();

        let t: Transform = Default::default();

        // start the ellipse somewhere by pressing the pen
        let mut builder = ThreePointEllipseBuilder::start(Default::default(), t.to_storage_coordinates(a0), 1.0);

        // move the pen without releasing
        builder.handle_mouse_moved(a1, t);

        // assert that the first point moves
        assert_eq!(builder.draw_commands()[0], DrawCommand::Path {
            color: Default::default(),
            thickness: 1.0,
            commands: vec![
                PathCommand::MoveTo(t.to_storage_coordinates(a1)),
            ],
        });

        // release the pen at a point A, this will be the first focus
        builder.handle_mouse_moved(a, t);
        builder.handle_button_released(a, t);

        // a line is drawn connecting A and A
        assert_eq!(builder.draw_commands()[0], DrawCommand::Path {
            color: Default::default(),
            thickness: 1.0,
            commands: vec![
                PathCommand::MoveTo(t.to_storage_coordinates(a)),
                PathCommand::LineTo(t.to_storage_coordinates(a)),
            ],
        });

        // move the pen to a point B
        builder.handle_mouse_moved(b, t);

        // a line is drawn connecting A and B
        assert_eq!(builder.draw_commands()[0], DrawCommand::Path {
            color: Default::default(),
            thickness: 1.0,
            commands: vec![
                PathCommand::MoveTo(t.to_storage_coordinates(a)),
                PathCommand::LineTo(t.to_storage_coordinates(b)),
            ],
        });

        // release pen at point C
        builder.handle_mouse_moved(c, t);
        builder.handle_button_released(c, t);

        // now move pen to point D
        builder.handle_mouse_moved(d, t);

        // and check that a first ellipse is built
        assert_eq!(builder.draw_commands()[0], DrawCommand::Ellipse {
            color: Default::default(),
            thickness: 1.0,
            center: Vec2DDrawing::new(-3.0257300000000003, 1.26254),
            semimajor: 5.838320272867801,
            semiminor: 4.575529950080007,
            angle: Radians::from_radians(-0.7769764190469384),
        });

        // finish the ellipse at a point E
        builder.handle_mouse_moved(e, t);

        match builder.handle_button_released(e, t) {
            ShapeFinished::Yes(shape) => {
                assert_eq!(shape.draw_commands(), DrawCommand::Ellipse {
                    color: Default::default(),
                    thickness: 1.0,
                    center: Vec2DDrawing::new(-3.0257300000000003, 1.26254),
                    semimajor: 7.793799303722533,
                    semiminor: 6.898753387548061,
                    angle: Radians::from_radians(-0.7769764190469384),
                });
            },
            _ => panic!(),
        }
    }

    #[test]
    fn the_bbox_of_a_rotated_ellipse() {
        let center = Vec2DDrawing::new(-3.0257300000000003, 1.26254);
        let semimajor = 7.793799303722533;
        let semiminor = 6.898753387548061;

        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, Radians::from_radians(0.0), Default::default(), 1.0);

        assert_eq!(ellipse.bbox(), [
            center - Vec2DDrawing::new(semimajor, semiminor),
            center + Vec2DDrawing::new(semimajor, semiminor),
        ]);

        let angle = Radians::from_radians(-0.7769764190469384);
        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, angle, Default::default(), 1.0);

        assert_eq!(ellipse.bbox(), [
            Vec2DDrawing::new(-10.4, -6.1),
            Vec2DDrawing::new(4.33, 8.6),
        ]);
    }
}
