use crate::draw_commands::{cancel_helper, circle_helper, path_helper, DrawCommand};
use crate::geom::{self, ellipse_from_foci_and_point};
use crate::point::{ScreenUnit, Vec2D, WorldUnit};
use crate::shape::stored::ellipse::Ellipse;
use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::style::Style;
use crate::transform::Transform;

#[derive(Debug, Copy, Clone)]
enum State {
    One(Vec2D<WorldUnit>),
    Foci([Vec2D<WorldUnit>; 2]),
    FociAndPoint {
        foci: [Vec2D<WorldUnit>; 2],
        point: Vec2D<WorldUnit>,
    },
}

#[derive(Debug)]
pub struct FociAndPointEllipse {
    state: State,
    style: Style<WorldUnit>,
}

impl FociAndPointEllipse {
    pub fn start(initial: Vec2D<WorldUnit>, style: Style<WorldUnit>) -> FociAndPointEllipse {
        FociAndPointEllipse {
            state: State::One(initial),
            style,
        }
    }
}

impl ShapeBuilder for FociAndPointEllipse {
    fn handle_mouse_moved(&mut self, pos: Vec2D<ScreenUnit>, t: Transform, _snap: ScreenUnit) {
        match self.state {
            State::One(_) => {
                self.state = State::One(t.to_world_coordinates(pos));
            }
            State::Foci([f1, _]) => {
                self.state = State::Foci([f1, t.to_world_coordinates(pos)]);
            }
            State::FociAndPoint { foci, point: _ } => {
                self.state = State::FociAndPoint {
                    foci,
                    point: t.to_world_coordinates(pos),
                }
            }
        }
    }

    fn handle_button_pressed(&mut self, _pos: Vec2D<ScreenUnit>, _: Transform, _snap: ScreenUnit) {}

    fn handle_button_released(
        &mut self,
        pos: Vec2D<ScreenUnit>,
        t: Transform,
        snap: ScreenUnit,
    ) -> ShapeFinished {
        match self.state {
            State::One(f1) => {
                self.state = State::Foci([f1, t.to_world_coordinates(pos)]);

                ShapeFinished::No
            }
            State::Foci(foci) => {
                if t.to_screen_coordinates(foci[0]).distance(pos) < snap {
                    return ShapeFinished::Cancelled;
                }

                self.state = State::FociAndPoint {
                    foci,
                    point: t.to_world_coordinates(pos),
                };

                ShapeFinished::No
            }
            State::FociAndPoint { foci, .. } => {
                let (center, semimajor, semiminor, angle) =
                    ellipse_from_foci_and_point(foci, t.to_world_coordinates(pos));

                ShapeFinished::Yes(vec![Box::new(Ellipse::from_parts(
                    center, semimajor, semiminor, angle, self.style,
                ))])
            }
        }
    }

    fn draw_commands(&self, t: Transform, snap: ScreenUnit) -> Vec<DrawCommand> {
        match self.state {
            State::One(f1) => vec![circle_helper(t.to_screen_coordinates(f1), snap)],
            State::Foci([f1, f2]) => vec![
                cancel_helper(f1, f2, t, snap),
                cancel_helper(f2, f1, t, snap),
            ],
            State::FociAndPoint { foci, point } => {
                let (center, semimajor, semiminor, angle) =
                    ellipse_from_foci_and_point(foci, point);

                vec![
                    DrawCommand::Ellipse {
                        ellipse: geom::Ellipse {
                            center,
                            semimajor,
                            semiminor,
                            angle,
                        },
                        style: self.style,
                    },
                    path_helper(vec![
                        t.to_screen_coordinates(foci[0]),
                        t.to_screen_coordinates(point),
                        t.to_screen_coordinates(foci[1]),
                    ]),
                    circle_helper(t.to_screen_coordinates(foci[0]), snap),
                    circle_helper(t.to_screen_coordinates(foci[1]), snap),
                ]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::color::Color;
    use crate::geom::Angle;

    use super::*;

    const SNAP: ScreenUnit = ScreenUnit::from_float(1.0);

    #[test]
    fn three_point_ellipse_builder() {
        let a0: Vec2D<ScreenUnit> = (-5.69132, 5.13608).into();
        let a1: Vec2D<ScreenUnit> = (-6.41006, 4.44396).into();
        let a: Vec2D<ScreenUnit> = (-5.61146, 3.80508).into();
        let b: Vec2D<ScreenUnit> = (-4.36, 0.52).into();
        let c: Vec2D<ScreenUnit> = (-0.44, -1.28).into();
        let d: Vec2D<ScreenUnit> = (2.2, 0.56).into();
        let e: Vec2D<ScreenUnit> = (4.3, -0.4).into();

        let t: Transform = Default::default();

        // start the ellipse somewhere by pressing the pen
        let mut builder =
            FociAndPointEllipse::start(t.to_world_coordinates(a0), Default::default());

        // move the pen without releasing
        builder.handle_mouse_moved(a1, t, SNAP);

        // assert that the first point moves
        assert_eq!(
            builder.draw_commands(t, SNAP),
            vec![DrawCommand::ScreenCircle {
                style: Style {
                    stroke: None,
                    fill: Some(Color::gray().half_transparent()),
                },
                center: a1,
                radius: SNAP,
            },]
        );

        // release the pen at a point A, this will be the first focus
        builder.handle_mouse_moved(a, t, SNAP);
        builder.handle_button_released(a, t, SNAP);

        // circles are drawn around each foci
        assert_eq!(
            builder.draw_commands(t, SNAP),
            vec![
                DrawCommand::ScreenCircle {
                    style: Style {
                        stroke: None,
                        fill: Some(Color::red().half_transparent()),
                    },
                    center: a,
                    radius: SNAP,
                },
                DrawCommand::ScreenCircle {
                    style: Style {
                        stroke: None,
                        fill: Some(Color::red().half_transparent()),
                    },
                    center: a,
                    radius: SNAP,
                },
            ]
        );

        // move the pen to a point B
        builder.handle_mouse_moved(b, t, SNAP);

        // a line is drawn connecting A and B
        assert_eq!(
            builder.draw_commands(t, SNAP),
            vec![
                DrawCommand::ScreenCircle {
                    style: Style {
                        stroke: None,
                        fill: Some(Color::gray().half_transparent()),
                    },
                    center: a,
                    radius: SNAP,
                },
                DrawCommand::ScreenCircle {
                    style: Style {
                        stroke: None,
                        fill: Some(Color::gray().half_transparent()),
                    },
                    center: b,
                    radius: SNAP,
                },
            ]
        );

        // release pen at point C
        builder.handle_mouse_moved(c, t, SNAP);
        builder.handle_button_released(c, t, SNAP);

        // now move pen to point D
        builder.handle_mouse_moved(d, t, SNAP);

        // and check that a first ellipse is built
        assert_eq!(
            builder.draw_commands(t, SNAP)[0],
            DrawCommand::Ellipse {
                style: Default::default(),
                ellipse: geom::Ellipse {
                    center: Vec2D::new_world(-3.0257300000000003, 1.26254),
                    semimajor: 5.838320272867801.into(),
                    semiminor: 4.575529950080007.into(),
                    angle: Angle::from_radians(-0.7769764190469384),
                },
            }
        );

        // finish the ellipse at a point E
        builder.handle_mouse_moved(e, t, SNAP);

        match builder.handle_button_released(e, t, SNAP) {
            ShapeFinished::Yes(shape) => {
                assert_eq!(
                    shape[0].draw_commands(),
                    DrawCommand::Ellipse {
                        style: Default::default(),
                        ellipse: geom::Ellipse {
                            center: Vec2D::new_world(-3.0257300000000003, 1.26254),
                            semimajor: 7.793799303722533.into(),
                            semiminor: 6.898753387548061.into(),
                            angle: Angle::from_radians(-0.7769764190469384),
                        },
                    }
                );
            }
            _ => panic!(),
        }
    }
}
