use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::draw_commands::DrawCommand;
use crate::path_command::PathCommand::*;
use crate::point::{Point, Vec2DScreen, Vec2DWorld};
use crate::transform::Transform;
use crate::style::Style;
use crate::geom::rombus_to_rectangle;
use crate::shape::builder::grid::grid_from_cell_and_point;
use crate::shape::ShapeStored;
use crate::shape::stored::path::Path;

#[derive(Debug, Copy, Clone)]
enum State {
    Initial(Vec2DWorld),

    OneDimension {
        start: Vec2DWorld,
        free: Vec2DWorld
    },

    TwoDimension {
        start: Vec2DWorld,
        p1: Vec2DWorld,
        free: Vec2DWorld,
    },

    DefineGrid {
        start: Vec2DWorld,
        p1: Vec2DWorld,
        p2: Vec2DWorld,
        free: Vec2DWorld,

        /// A transform matrix that turns the parallelogram defined by start, p1
        /// and p2 into a square of size 1 in the first quadrant
        t: Transform,
    },
}

/// This tools builds a grid through a 5-step process. It might seem complicated
/// at first but it is flexible and easy to learn.
///
/// 1. First place a point, this will be a corner of the grid.
#[derive(Debug)]
pub struct FreeGrid {
    style: Style,
    state: State,
}

impl FreeGrid {
    pub fn start(initial: Vec2DWorld, style: Style) -> FreeGrid {
        FreeGrid {
            style,
            state: State::Initial(initial),
        }
    }
}

impl ShapeBuilder for FreeGrid {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) {
        let wpos = t.to_world_coordinates(pos);

        match self.state {
            State::Initial(_) => {
                self.state = State::Initial(wpos);
            }

            State::OneDimension { start, .. } => {
                self.state = State::OneDimension { start, free: wpos };
            }

            State::TwoDimension { start, p1, .. } => {
                self.state = State::TwoDimension { start, p1, free: wpos };
            }

            State::DefineGrid { start, p1, p2, t, .. } => {
                self.state = State::DefineGrid { start, p1, p2, t, free: wpos };
            }
        }
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform, _snap: f64) { }

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) -> ShapeFinished {
        let wpos = t.to_world_coordinates(pos);

        match self.state {
            State::Initial(p) => {
                self.state = State::OneDimension { start: p, free: wpos };

                ShapeFinished::No
            }

            State::OneDimension { start, free } => {
                self.state = State::TwoDimension { start, p1: free, free: wpos };

                ShapeFinished::No
            }

            State::TwoDimension { start, p1, free } => {
                let p2 = free;
                let t = rombus_to_rectangle(start.to_vec2d(), p1.to_vec2d(), p2.to_vec2d());

                self.state = State::DefineGrid { start, p1, p2, t, free: wpos };

                ShapeFinished::No
            }

            State::DefineGrid { start, p2, free, t, ..} => {
                let cell = (
                    t.to_screen_coordinates(start),
                    t.to_screen_coordinates(p2),
                );
                let free = t.to_screen_coordinates(free);
                let grid = grid_from_cell_and_point(cell, free);

                let mut shapes: Vec<Box<dyn ShapeStored>> = Vec::with_capacity(
                    grid.row_count() * (grid.col_count() - 1) +
                    grid.col_count() * (grid.row_count() - 1)
                );

                let rows = grid.rows();
                let cols = grid.cols();

                for row in rows.iter() {
                    for (&a, &b) in row.iter().zip(row.iter().skip(1)) {
                        shapes.push(Box::new(Path::from_parts(
                            vec![
                                MoveTo(t.to_world_coordinates(a)),
                                LineTo(t.to_world_coordinates(b)),
                            ],
                            self.style,
                        )));
                    }
                }

                for col in cols.iter() {
                    for (&a, &b) in col.iter().zip(col.iter().skip(1)) {
                        shapes.push(Box::new(Path::from_parts(
                            vec![
                                MoveTo(t.to_world_coordinates(a)),
                                LineTo(t.to_world_coordinates(b)),
                            ],
                            self.style,
                        )));
                    }
                }

                ShapeFinished::Yes(shapes)
            }
        }
    }

    fn draw_commands(&self, _t: Transform, _snap: f64) -> Vec<DrawCommand> {
        match self.state {
            State::Initial(p) => vec![DrawCommand::Path {
                commands: vec![
                    MoveTo(p),
                    LineTo(p),
                ],
                style: self.style,
            }],

            State::OneDimension { start, free } => vec![DrawCommand::Path {
                commands: vec![
                    MoveTo(start),
                    LineTo(free),
                ],
                style: self.style,
            }],

            State::TwoDimension { start, p1, free } => {
                vec![DrawCommand::Path {
                    commands: vec![
                        MoveTo(start),
                        LineTo(p1),
                        LineTo(free),
                        LineTo(start + (free - p1)),
                        LineTo(start),
                    ],
                    style: self.style,
                }]
            }

            State::DefineGrid { start, p1: _, p2, t, free } => {
                let cell = (
                    t.apply(start.to_vec2d()),
                    t.apply(p2.to_vec2d()),
                );
                let free = t.apply(free.to_vec2d());
                let inverse = t.invert();
                let grid = grid_from_cell_and_point(cell, free);

                let mut commands = Vec::with_capacity(grid.row_count() + grid.col_count());

                for row in grid.rows() {
                    commands.push(DrawCommand::Path {
                        commands: vec![
                            MoveTo(inverse.apply(row[0]).into()),
                            LineTo(inverse.apply(*row.last().unwrap()).into()),
                        ],
                        style: self.style,
                    });
                }

                for col in grid.cols() {
                    commands.push(DrawCommand::Path {
                        commands: vec![
                            MoveTo(inverse.apply(col[0]).into()),
                            LineTo(inverse.apply(*col.last().unwrap()).into()),
                        ],
                        style: self.style,
                    });
                }

                commands
            }
        }
    }
}
