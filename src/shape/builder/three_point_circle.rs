use crate::point::{Vec2D, WorldUnit, ScreenUnit};
use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::shape::stored::ellipse::Ellipse;
use crate::draw_commands::{DrawCommand, circle_helper, cancel_helper};
use crate::transform::Transform;
use crate::geom::{self, Angle, circle_through_three_points};
use crate::style::Style;

#[derive(Debug, Copy, Clone)]
enum State {
    One(Vec2D<WorldUnit>),
    Two(Vec2D<WorldUnit>, Vec2D<WorldUnit>),
    Three(Vec2D<WorldUnit>, Vec2D<WorldUnit>, Vec2D<WorldUnit>),
}

#[derive(Debug)]
pub struct CircleThroughThreePointsBuilder {
    state: State,
    style: Style,
}

impl CircleThroughThreePointsBuilder {
    pub fn start(initial: Vec2D<WorldUnit>, style: Style) -> CircleThroughThreePointsBuilder {
        CircleThroughThreePointsBuilder {
            state: State::One(initial),
            style,
        }
    }
}

impl ShapeBuilder for CircleThroughThreePointsBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2D<ScreenUnit>, t: Transform, _snap: ScreenUnit) {
        match self.state {
            State::One(_) => {
                self.state = State::One(t.to_world_coordinates(pos));
            }
            State::Two(p1, _) => {
                self.state = State::Two(p1, t.to_world_coordinates(pos));
            }
            State::Three(p1, p2, _) => {
                self.state = State::Three(p1, p2, t.to_world_coordinates(pos));
            }
        }
    }

    fn handle_button_pressed(&mut self, _pos: Vec2D<ScreenUnit>, _t: Transform, _snap: ScreenUnit) {}

    fn handle_button_released(&mut self, pos: Vec2D<ScreenUnit>, t: Transform, snap: ScreenUnit) -> ShapeFinished {
        self.handle_mouse_moved(pos, t, snap);

        let pos_new = t.to_world_coordinates(pos);
        match self.state {
            State::One(p1) => {
                self.state = State::Two(p1, t.to_world_coordinates(pos));

                ShapeFinished::No
            }
            State::Two(p1, p2) => {
                if p1.distance(pos_new) < snap {
                    return ShapeFinished::Cancelled;
                }
                self.state = State::Three(p1, p2, t.to_world_coordinates(pos));

                ShapeFinished::No
            }
            State::Three(p1, p2, p3) => {
                let (center, radius) = circle_through_three_points(p1, p2, p3);

                ShapeFinished::Yes(vec![Box::new(
                    Ellipse::from_parts(
                        center,
                        radius,
                        radius,
                        Angle::from_radians(0.0),
                        self.style,
                    )
                )])
            }
        }
    }

    fn draw_commands(&self, t: Transform, snap: ScreenUnit) -> Vec<DrawCommand> {
        match self.state {
            State::One(p1) => vec![
                circle_helper(t.to_screen_coordinates(p1), snap),
            ],
            State::Two(p1, p2) => vec![
                cancel_helper(p1, p2, t, snap),
            ],
            State::Three(p1, p2, p3) => {
                let (center, radius) = circle_through_three_points(p1, p2, p3);

                vec![
                    DrawCommand::Ellipse {
                        ellipse: geom::Ellipse {
                            center,
                            semimajor: radius,
                            semiminor: radius,
                            angle: Angle::from_radians(0.0),
                        },
                        style: self.style,
                    },

                    circle_helper(t.to_screen_coordinates(p1), snap),
                    circle_helper(t.to_screen_coordinates(p2), snap),
                    circle_helper(t.to_screen_coordinates(p3), snap),
                ]
            }
        }
    }
}
