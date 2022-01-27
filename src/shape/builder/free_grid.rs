use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::draw_commands::{DrawCommand, circle_helper, cancel_helper};
use crate::path_command::PathCommand::*;
use crate::point::{Vec2D, WorldUnit, ScreenUnit};
use crate::transform::Transform;
use crate::style::Style;
use crate::geom::{rombus_to_rectangle, project};
use crate::shape::builder::grid::grid_from_cell_and_point;
use crate::shape::ShapeStored;
use crate::shape::stored::path::Path;

#[derive(Debug, Copy, Clone)]
enum State {
    Initial(Vec2D<WorldUnit>),

    OneDimension {
        start: Vec2D<WorldUnit>,
        free: Vec2D<WorldUnit>
    },

    TwoDimension {
        start: Vec2D<WorldUnit>,
        p1: Vec2D<WorldUnit>,
        free: Vec2D<WorldUnit>,
    },

    DefineGrid {
        start: Vec2D<WorldUnit>,
        p1: Vec2D<WorldUnit>,
        p2: Vec2D<WorldUnit>,
        free: Vec2D<WorldUnit>,

        /// A transform matrix that turns the parallelogram defined by start, p1
        /// and p2 into a square of size 1 in the first quadrant
        t: Transform,
    },
}

/// This tools builds a grid through a 5-step process. It might seem complicated
/// at first but it is flexible and easy to learn.
///
/// 1. First place a point, this will be a corner of the grid.
#[derive(Debug, Copy, Clone)]
pub struct FreeGrid {
    style: Style<WorldUnit>,
    state: State,
}

impl FreeGrid {
    pub fn start(initial: Vec2D<WorldUnit>, style: Style<WorldUnit>) -> FreeGrid {
        FreeGrid {
            style,
            state: State::Initial(initial),
        }
    }
}

impl ShapeBuilder for FreeGrid {
    fn handle_mouse_moved(&mut self, pos: Vec2D<ScreenUnit>, t: Transform, _snap: ScreenUnit) {
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

    fn handle_button_pressed(&mut self, _pos: Vec2D<ScreenUnit>, _t: Transform, _snap: ScreenUnit) { }

    fn handle_button_released(&mut self, pos: Vec2D<ScreenUnit>, t: Transform, snap: ScreenUnit) -> ShapeFinished {
        let wpos = t.to_world_coordinates(pos);

        match self.state {
            State::Initial(p) => {
                self.state = State::OneDimension { start: p, free: wpos };

                ShapeFinished::No
            }

            State::OneDimension { start, free } => {
                self.state = State::TwoDimension { start, p1: free, free: wpos };

                if t.to_screen_coordinates(start).distance(pos) < snap {
                    ShapeFinished::Cancelled
                } else {
                    ShapeFinished::No
                }
            }

            State::TwoDimension { start, p1, free } => {
                let p2 = free;

                self.state = State::DefineGrid {
                    start, p1, p2,
                    t: rombus_to_rectangle(start.to_vec2d(), p1.to_vec2d(), p2.to_vec2d()),
                    free: wpos,
                };

                let sstart = t.to_screen_coordinates(start);
                let sp1 = t.to_screen_coordinates(p1);
                let proj = t.to_screen_coordinates(project(free, [start, p1]));
                let proj2 = t.to_screen_coordinates(project(start, [p1, free]));

                if sstart.distance(pos) < snap || sp1.distance(pos) < snap || proj.distance(pos) < snap || proj2.distance(sstart) < snap {
                    ShapeFinished::Cancelled
                } else {
                    ShapeFinished::No
                }
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

    fn draw_commands(&self, t: Transform, snap: ScreenUnit) -> Vec<DrawCommand> {
        match self.state {
            State::Initial(p) => vec![
                circle_helper(t.to_screen_coordinates(p), snap),
            ],

            State::OneDimension { start, free } => vec![
                DrawCommand::Path {
                    commands: vec![
                        MoveTo(start),
                        LineTo(free),
                    ],
                    style: self.style,
                },
                cancel_helper(start, free, t, snap),
            ],

            State::TwoDimension { start, p1, free } => vec![
                DrawCommand::Path {
                    commands: vec![
                        MoveTo(start),
                        LineTo(p1),
                        LineTo(free),
                        LineTo(start + (free - p1)),
                        LineTo(start),
                    ],
                    style: self.style,
                },

                cancel_helper(start, free, t, snap),
                cancel_helper(p1, free, t, snap),
                // the projection of the mouse on the line defined by the first
                // two points
                cancel_helper(project(free, [start, p1]), free, t, snap),
                // the projection of the first point on the line defined by the
                // last two
                cancel_helper(project(start, [p1, free]), start, t, snap),
            ],

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

#[cfg(test)]
mod tests  {
    use pretty_assertions::assert_eq;

    use super::*;

    const SNAP: ScreenUnit = ScreenUnit::from_float(10.0);

    #[test]
    fn figure_is_cancelled() {
        let t = Default::default();
        let mut grid = FreeGrid::start(Vec2D::new_world(0.0, 0.0), Default::default());

        grid.handle_mouse_moved((1.0, 1.0).into(), t, SNAP);

        // a circle goes around the cursor
        assert_eq!(grid.draw_commands(t, SNAP), vec![
            DrawCommand::ScreenCircle {
                center: Vec2D::new_screen(1.0, 1.0),
                radius: 10.0.into(),
                style: Style::circle_helper(),
            }
        ]);

        // start the grid at the origin
        grid.handle_mouse_moved((0.0, 0.0).into(), t, SNAP);
        grid.handle_button_released((0.0, 0.0).into(), t, SNAP);

        // vove a little bit and observe a circle around the first point
        grid.handle_mouse_moved((1.0, 1.0).into(), t, SNAP);

        assert_eq!(grid.draw_commands(t, SNAP), vec![
            DrawCommand::Path {
                commands: vec![
                    MoveTo((0.0, 0.0).into()),
                    LineTo((1.0, 1.0).into()),
                ],
                style: Default::default(),
            },

            DrawCommand::ScreenCircle {
                center: Vec2D::new_screen(0.0, 0.0),
                radius: SNAP,
                style: Style::red_circle_helper(),
            },
        ]);

        Some(grid).map(|mut grid| {
            // releasing here would cause a cancel
            grid.handle_button_pressed((1.0, 1.0).into(), t, SNAP);
            match grid.handle_button_released((1.0, 1.0).into(), t, SNAP) {
                ShapeFinished::Cancelled => {}
                _ => panic!(),
            }
        }).unwrap();

        // move to the second point and use it to define the grid
        grid.handle_mouse_moved((20.0, 0.0).into(), t, SNAP);
        grid.handle_button_pressed((20.0, 0.0).into(), t, SNAP);
        grid.handle_button_released((20.0, 0.0).into(), t, SNAP);

        // moving around the two original points would threaten to cancel the
        // shape
        grid.handle_mouse_moved((1.0, 1.0).into(), t, SNAP);

        assert_eq!(grid.draw_commands(t, SNAP), vec![
            DrawCommand::Path {
                commands: vec![
                    MoveTo((0.0, 0.0).into()),
                    LineTo((20.0, 0.0).into()),
                    LineTo((1.0, 1.0).into()),
                    LineTo((-19.0, 1.0).into()),
                    LineTo((0.0, 0.0).into()),
                ],
                style: Default::default(),
            },

            // first circle red
            DrawCommand::ScreenCircle {
                center: (0.0, 0.0).into(),
                radius: SNAP,
                style: Style::red_circle_helper(),
            },

            // second circle gray
            DrawCommand::ScreenCircle {
                center: (20.0, 0.0).into(),
                radius: SNAP,
                style: Style::circle_helper(),
            },
        ]);

        // releasing here would cancel
        Some(grid).map(|mut grid| {
            grid.handle_button_pressed((1.0, 1.0).into(), t, SNAP);
            match grid.handle_button_released((1.0, 1.0).into(), t, SNAP) {
                ShapeFinished::Cancelled => {}
                _ => panic!()
            }
        }).unwrap();

        grid.handle_mouse_moved((21.0, 1.0).into(), t, SNAP);

        assert_eq!(grid.draw_commands(t, SNAP), vec![
            DrawCommand::Path {
                commands: vec![
                    MoveTo((0.0, 0.0).into()),
                    LineTo((20.0, 0.0).into()),
                    LineTo((21.0, 1.0).into()),
                    LineTo((1.0, 1.0).into()),
                    LineTo((0.0, 0.0).into()),
                ],
                style: Default::default(),
            },

            // first circle red
            DrawCommand::ScreenCircle {
                center: (0.0, 0.0).into(),
                radius: SNAP,
                style: Style::circle_helper(),
            },

            // second circle gray
            DrawCommand::ScreenCircle {
                center: (20.0, 0.0).into(),
                radius: SNAP,
                style: Style::red_circle_helper(),
            },
        ]);

        // releasing here would cancel
        Some(grid).map(|mut grid| {
            grid.handle_button_pressed((21.0, 1.0).into(), t, SNAP);
            match grid.handle_button_released((21.0, 1.0).into(), t, SNAP) {
                ShapeFinished::Cancelled => {}
                _ => panic!()
            }
        }).unwrap();
    }

    /// Let's suppose you have defined the first two points of a free grid. When
    /// choosing the third one, if it is too close to the line defined by the
    /// first two it can happen that in the next stage (the expansion of the
    /// grid) it will create too many lines and make pizarra hang or crash.
    #[test]
    fn cannot_make_the_grid_too_tight() {
        todo!()
    }
}
