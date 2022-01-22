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
                let diagonal = p2s - p1s;
                let freep = t.to_screen_coordinates(free) - p1s;

                let base = diagonal.x;
                let height = diagonal.y;

                let x_cells = freep.x / base;
                let y_cells = freep.y / height;

                // integer amount of cells that will be created
                let num_x_cells = x_cells.abs().ceil();
                let num_y_cells = y_cells.abs().ceil();

                // lines that divide the space (the grid separators)
                let x_separators: Vec<_> = (1..(num_x_cells as i8)).map(|n| n * x_cells.signum() as i8).collect();
                let y_separators: Vec<_> = (1..(num_y_cells as i8)).map(|n| n * y_cells.signum() as i8).collect();

                // the boundaries of the grid in the transformed space
                let x_max = num_x_cells * x_cells.signum() * base;
                let y_max = num_y_cells * y_cells.signum() * height;

                // the box boundaries in screen coordinates
                let b1 = p1s + Vec2DScreen::new(x_max, 0.0);
                let b2 = p1s + Vec2DScreen::new(x_max, y_max);
                let b3 = p1s + Vec2DScreen::new(0.0, y_max);

                let world_b1 = t.to_world_coordinates(b1).into();
                let world_b2 = t.to_world_coordinates(b2).into();
                let world_b3 = t.to_world_coordinates(b3).into();

                let world_x_seps: Vec<_> = x_separators.into_iter().map(|n| {
                    DrawCommand::Path {
                        commands: vec![
                            MoveTo(t.to_world_coordinates(Vec2DScreen::new(n as f64 * base, 0.0)).into()),
                            LineTo(t.to_world_coordinates(Vec2DScreen::new(n as f64 * base, y_max)).into()),
                        ],
                        style: self.style,
                    }
                }).collect();
                let world_y_seps: Vec<_> = y_separators.into_iter().map(|n| {
                    DrawCommand::Path {
                        commands: vec![
                            MoveTo(t.to_world_coordinates(Vec2DScreen::new(0.0, n as f64 * height)).into()),
                            LineTo(t.to_world_coordinates(Vec2DScreen::new(x_max, n as f64 * height)).into()),
                        ],
                        style: self.style,
                    }
                }).collect();

                let mut commands = Vec::with_capacity(1 + world_x_seps.len() + world_y_seps.len());

                // the big rectangle
                commands.push(DrawCommand::Path {
                    commands: vec![
                        MoveTo(p1),
                        LineTo(world_b1),
                        LineTo(world_b2),
                        LineTo(world_b3),
                        LineTo(p1),
                    ],
                    style: self.style,
                });

                // the separators that make it a grid
                commands.extend(world_x_seps);
                commands.extend(world_y_seps);

                commands
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
