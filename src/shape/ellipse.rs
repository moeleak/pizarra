use std::f64::consts::PI;
use crate::point::{Point, Vec2DWorld, Vec2DScreen};
use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::transform::Transform;
use crate::geom::{
    Angle, ellipse_from_foci_and_point, foci_and_sum_from_center_and_radi,
};
use crate::shape::path::PathCommand;

use super::{ShapeBuilder, ShapeStored, ShapeFinished, ShapeType};

#[derive(Debug, Copy, Clone)]
enum State {
    One(Vec2DWorld),
    Foci([Vec2DWorld; 2]),
    FociAndPoint {
        foci: [Vec2DWorld; 2],
        point: Vec2DWorld,
    }
}

#[derive(Debug)]
pub struct ThreePointEllipseBuilder {
    state: State,
    thickness: f64,
    color: Color,
}

impl ThreePointEllipseBuilder {
    pub fn start(color: Color, initial: Vec2DWorld, thickness: f64) -> ThreePointEllipseBuilder {
        ThreePointEllipseBuilder {
            state: State::One(initial),
            thickness,
            color,
        }
    }
}

impl ShapeBuilder for ThreePointEllipseBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) {
        match self.state {
            State::One(_) => {
                self.state = State::One(t.to_world_coordinates(pos));
            }
            State::Foci([f1, _]) => {
                self.state = State::Foci([f1, t.to_world_coordinates(pos)]);
            }
            State::FociAndPoint {
                foci,
                point: _,
            } => {
                self.state = State::FociAndPoint {
                    foci, point: t.to_world_coordinates(pos),
                }
            }
        }
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _: Transform, _snap: f64) { }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) -> ShapeFinished {
        match self.state {
            State::One(f1) => {
                self.state = State::Foci([f1, t.to_world_coordinates(pos)]);

                ShapeFinished::No
            }
            State::Foci(foci) => {
                self.state = State::FociAndPoint {
                    foci,
                    point: t.to_world_coordinates(pos),
                };

                ShapeFinished::No
            }
            State::FociAndPoint {
                foci, ..
            } => {
                let (center, semimajor, semiminor, angle) = ellipse_from_foci_and_point(foci, t.to_world_coordinates(pos));

                ShapeFinished::Yes(Box::new(Ellipse::from_parts(
                    center, semimajor, semiminor, angle, self.color, self.thickness,
                )))
            }
        }
    }

    fn draw_commands(&self) -> Vec<DrawCommand> {
        match self.state {
            State::One(f1) => vec![DrawCommand::Path {
                color: self.color,
                thickness: self.thickness,
                commands: vec![PathCommand::MoveTo(f1)],
            }],
            State::Foci([f1, f2]) => vec![DrawCommand::Path {
                color: self.color,
                thickness: self.thickness,
                commands: vec![PathCommand::MoveTo(f1), PathCommand::LineTo(f2)],
            }],
            State::FociAndPoint {
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
    center: Vec2DWorld,
    semimajor: f64,
    semiminor: f64,
    angle: Angle,
    color: Color,
    thickness: f64,
}

impl Ellipse {
    pub fn from_parts(center: Vec2DWorld, semimajor: f64, semiminor: f64, angle: Angle, color: Color, thickness: f64) -> Ellipse {
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

    fn color(&self) -> Color {
        self.color
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::ellipse_from_foci_and_sum;

    use super::*;

    const SNAP: f64 = 10.0;

    #[test]
    fn test_intersects_circle() {
        let f1: Vec2DWorld = (-20.22222, -13.24752).into();
        let f2: Vec2DWorld = (-2.88, 1.28).into();
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
            assert!(ellipse.intersects_circle(Vec2DWorld::from(coords), 4.0) == output);
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
        let mut builder = ThreePointEllipseBuilder::start(Default::default(), t.to_world_coordinates(a0), 1.0);

        // move the pen without releasing
        builder.handle_mouse_moved(a1, t, SNAP);

        // assert that the first point moves
        assert_eq!(builder.draw_commands()[0], DrawCommand::Path {
            color: Default::default(),
            thickness: 1.0,
            commands: vec![
                PathCommand::MoveTo(t.to_world_coordinates(a1)),
            ],
        });

        // release the pen at a point A, this will be the first focus
        builder.handle_mouse_moved(a, t, SNAP);
        builder.handle_button_released(a, t, SNAP);

        // a line is drawn connecting A and A
        assert_eq!(builder.draw_commands()[0], DrawCommand::Path {
            color: Default::default(),
            thickness: 1.0,
            commands: vec![
                PathCommand::MoveTo(t.to_world_coordinates(a)),
                PathCommand::LineTo(t.to_world_coordinates(a)),
            ],
        });

        // move the pen to a point B
        builder.handle_mouse_moved(b, t, SNAP);

        // a line is drawn connecting A and B
        assert_eq!(builder.draw_commands()[0], DrawCommand::Path {
            color: Default::default(),
            thickness: 1.0,
            commands: vec![
                PathCommand::MoveTo(t.to_world_coordinates(a)),
                PathCommand::LineTo(t.to_world_coordinates(b)),
            ],
        });

        // release pen at point C
        builder.handle_mouse_moved(c, t, SNAP);
        builder.handle_button_released(c, t, SNAP);

        // now move pen to point D
        builder.handle_mouse_moved(d, t, SNAP);

        // and check that a first ellipse is built
        assert_eq!(builder.draw_commands()[0], DrawCommand::Ellipse {
            color: Default::default(),
            thickness: 1.0,
            center: Vec2DWorld::new(-3.0257300000000003, 1.26254),
            semimajor: 5.838320272867801,
            semiminor: 4.575529950080007,
            angle: Angle::from_radians(-0.7769764190469384),
        });

        // finish the ellipse at a point E
        builder.handle_mouse_moved(e, t, SNAP);

        match builder.handle_button_released(e, t, SNAP) {
            ShapeFinished::Yes(shape) => {
                assert_eq!(shape.draw_commands(), DrawCommand::Ellipse {
                    color: Default::default(),
                    thickness: 1.0,
                    center: Vec2DWorld::new(-3.0257300000000003, 1.26254),
                    semimajor: 7.793799303722533,
                    semiminor: 6.898753387548061,
                    angle: Angle::from_radians(-0.7769764190469384),
                });
            },
            _ => panic!(),
        }
    }

    #[test]
    fn the_bbox_of_an_ellipse() {
        let center = Vec2DWorld::new(-3.0257300000000003, 1.26254);
        let semimajor = 7.793799303722533;
        let semiminor = 6.898753387548061;

        // axis aligned
        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, Angle::from_radians(0.0), Default::default(), 1.0);

        assert_eq!(ellipse.bbox(), [
            center - Vec2DWorld::new(semimajor, semiminor),
            center + Vec2DWorld::new(semimajor, semiminor),
        ]);

        // rotated
        let angle = Angle::from_radians(-0.7769764190469384);
        let ellipse = Ellipse::from_parts(center, semimajor, semiminor, angle, Default::default(), 1.0);

        assert_eq!(ellipse.bbox(), [
            Vec2DWorld { x: -10.39314460317701, y: -6.089827506715496 },
            Vec2DWorld { x: 4.3416846031770095, y: 8.614907506715497 },
        ]);
    }
}
