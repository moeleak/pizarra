use crate::color::Color;
use crate::point::{Vec2DWorld, Vec2DScreen};
use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::draw_commands::DrawCommand;
use crate::transform::Transform;
use crate::geom::circle_by_three_points;

use super::circle::Circle;
use super::path::PathCommand;

#[derive(Debug, Copy, Clone)]
enum State {
    One(Vec2DWorld),
    Two(Vec2DWorld, Vec2DWorld),
    Three(Vec2DWorld, Vec2DWorld, Vec2DWorld),
}

pub struct CircleByThreePointsBuilder {
    state: State,
    thickness: f64,
    color: Color,
}

impl CircleByThreePointsBuilder {
    pub fn start(color: Color, initial: Vec2DWorld, thickness: f64) -> CircleByThreePointsBuilder {
        CircleByThreePointsBuilder {
            state: State::One(initial),
            thickness,
            color,
        }
    }
}

impl ShapeBuilder for CircleByThreePointsBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform) {
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

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform) {}

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform) -> ShapeFinished {
        self.handle_mouse_moved(pos, t);

        match self.state {
            State::One(p1) => {
                self.state = State::Two(p1, t.to_world_coordinates(pos));

                ShapeFinished::No
            }
            State::Two(p1, p2) => {
                self.state = State::Three(p1, p2, t.to_world_coordinates(pos));

                ShapeFinished::No
            }
            State::Three(p1, p2, p3) => {
                let (center, radius) = circle_by_three_points(p1, p2, p3);

                ShapeFinished::Yes(Box::new(
                    Circle::from_parts(
                        center,
                        radius,
                        self.thickness,
                        self.color,
                    )
                ))
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
            State::Two(p1, p2) => vec![DrawCommand::Path {
                color: self.color,
                thickness: self.thickness,
                commands: vec![PathCommand::MoveTo(p1), PathCommand::LineTo(p2)],
            }],
            State::Three(p1, p2, p3) => {
                let (center, radius) = circle_by_three_points(p1, p2, p3);

                vec![DrawCommand::Circle {
                    center: center,
                    radius: radius,
                    thickness: self.thickness,
                    color: self.color,
                }]
            }
        }
    }
}
