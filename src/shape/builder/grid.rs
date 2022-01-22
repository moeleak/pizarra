use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::draw_commands::{DrawCommand, circle_helper, red_circle_helper};
use crate::path_command::PathCommand::{self, *};
use crate::point::{Point, Vec2DScreen, Vec2DWorld};
use crate::transform::Transform;
use crate::style::Style;

#[derive(Debug, Copy, Clone)]
enum State {
    Initial(Vec2DWorld),

    DefineCell(Vec2DWorld, Vec2DWorld),

    DefineGrid {
        cell: (Vec2DWorld, Vec2DWorld),
        free: Vec2DWorld,
    },
}

/// This tools builds a grid through a 5-step process. It might seem complicated
/// at first but it is flexible and easy to learn.
///
/// 1. First place a point, this will be a corner of the grid.
#[derive(Debug)]
pub struct Grid {
    style: Style,
    state: State,
}

impl Grid {
    pub fn start(initial: Vec2DWorld, style: Style) -> Grid {
        Grid {
            style,
            state: State::Initial(initial),
        }
    }
}

impl ShapeBuilder for Grid {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) {
        let wpos = t.to_world_coordinates(pos);

        match self.state {
            State::Initial(p) => {
                self.state = State::DefineCell(p, wpos);
            }

            State::DefineCell(p1, _) => {
                self.state = State::DefineCell(p1, wpos);
            }

            State::DefineGrid { cell, .. } => {
                self.state = State::DefineGrid { cell, free: wpos };
            }
        }
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform, _snap: f64) { }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform, snap: f64) -> ShapeFinished {
        let wpos = t.to_world_coordinates(pos);

        match self.state {
            State::Initial(_) => ShapeFinished::Cancelled,

            State::DefineCell(p1, p2) => {
                let p1s = t.to_screen_coordinates(p1);
                let p2s = t.to_screen_coordinates(p2);

                if p1s.distance(p2s) < snap {
                    ShapeFinished::Cancelled
                } else {
                    self.state = State::DefineGrid {
                        cell: (p1, p2),
                        free: wpos,
                    };

                    ShapeFinished::No
                }
            }

            State::DefineGrid { cell, free } => {
                ShapeFinished::Yes(todo!())
            }
        }
    }

    fn draw_commands(&self, t: Transform, snap: f64) -> Vec<DrawCommand> {
        match self.state {
            State::Initial(p) => vec![
                circle_helper(t.to_screen_coordinates(p), snap),
            ],

            State::DefineCell(p1, p2) => {
                let p1s = t.to_screen_coordinates(p1);
                let p2s = t.to_screen_coordinates(p2);

                vec![
                    if p1s.distance(p2s) < snap {
                        red_circle_helper(p1s, snap)
                    } else {
                        circle_helper(p1s, snap)
                    },

                    DrawCommand::Path {
                        commands: corners_to_path_commands(
                            corners_from_two_points(p1, p2, t)
                        ),
                        style: self.style,
                    },
                ]
            },

            State::DefineGrid { cell: (p1, p2), free } => {
                let p1s = t.to_screen_coordinates(p1);
                let p2s = t.to_screen_coordinates(p2);

                vec![]
            }
        }
    }
}

fn corners_from_two_points(p1: Vec2DWorld, p2: Vec2DWorld, t: Transform) -> [Vec2DWorld; 4] {
    let p1s = t.to_screen_coordinates(p1);
    let p2s = t.to_screen_coordinates(p2);
    let diagonal = p2s - p1s;

    [
        p1,
        t.to_world_coordinates(p1s + Vec2DScreen::new(diagonal.x, 0.0)),
        p2,
        t.to_world_coordinates(p1s + Vec2DScreen::new(0.0, diagonal.y)),
    ]
}

fn corners_to_path_commands(corners: [Vec2DWorld; 4]) -> Vec<PathCommand<Vec2DWorld>> {
    corners.into_iter().cycle().take(5).enumerate().map(|(i, p)| if i == 0 {
        MoveTo(p)
    } else {
        LineTo(p)
    }).collect()
}
