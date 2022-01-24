use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::draw_commands::{DrawCommand, circle_helper, red_circle_helper};
use crate::path_command::PathCommand::{self, *};
use crate::point::{Point, Vec2DScreen, Vec2DWorld};
use crate::transform::Transform;
use crate::style::Style;
use crate::matrix::Matrix;
use crate::shape::stored::path::Path;
use crate::shape::ShapeStored;

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

            State::DefineGrid { cell: (p1, p2), free } => {
                let cell = (
                    t.to_screen_coordinates(p1),
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
                let cell = (
                    t.to_screen_coordinates(p1),
                    t.to_screen_coordinates(p2),
                );
                let free = t.to_screen_coordinates(free);
                let grid = grid_from_cell_and_point(cell, free);

                let mut commands = Vec::with_capacity(grid.row_count() + grid.col_count());

                for row in grid.rows() {
                    commands.push(DrawCommand::Path {
                        commands: vec![
                            MoveTo(t.to_world_coordinates(row[0])),
                            LineTo(t.to_world_coordinates(*row.last().unwrap())),
                        ],
                        style: self.style,
                    });
                }

                for col in grid.cols() {
                    commands.push(DrawCommand::Path {
                        commands: vec![
                            MoveTo(t.to_world_coordinates(col[0])),
                            LineTo(t.to_world_coordinates(*col.last().unwrap())),
                        ],
                        style: self.style,
                    });
                }

                commands
            }
        }
    }
}

/// Given the two points that define a cell and a third point outside (or
/// inside) it, return a matrix of all the points that make it and its width
pub(crate) fn grid_from_cell_and_point<P: Point>((p1, p2): (P, P), free: P) -> Matrix<P> {
    let diagonal = p2 - p1;
    let freep = free - p1;

    let width = diagonal.x();
    let height = diagonal.y();

    let x_cells = freep.x() / width;
    let y_cells = freep.y() / height;

    // integer amount of cells that will be created
    let num_x_cells = x_cells.abs().ceil() as usize;
    let num_y_cells = y_cells.abs().ceil() as usize;

    // lines that divide the space (the grid separators)
    let x_separators: Vec<_> = (0..num_x_cells+1).map(|n| n as i32 * x_cells.signum() as i32).collect();
    let y_separators: Vec<_> = (0..num_y_cells+1).map(|n| n as i32 * y_cells.signum() as i32).collect();

    let mut grid = Vec::with_capacity((num_x_cells + 1) * (num_y_cells + 1));

    for yi in y_separators {
        for &xi in x_separators.iter() {
            grid.push(p1 + P::new(xi as f64 * width, yi as f64 * height));
        }
    }

    Matrix::from_parts(grid, num_x_cells + 1)
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::point::Vec2D;

    use super::*;

    #[test]
    fn can_make_a_grid() {
        // the points for a 2x2 grid
        let a: Vec2D = (-2.9,1.66).into();
        let b: Vec2D = (-1.44,0.82).into();
        let c: Vec2D = (-1.0, 0.0).into();

        let grid = grid_from_cell_and_point((a, b), c);

        let xs = [-2.9, -1.44, 0.020000000000000018];
        let ys = [1.66, 0.82, -0.020000000000000018];

        assert_eq!(grid, Matrix::from_parts(ys.into_iter().map(|y| {
            xs.iter().map(move |&x| Vec2D::new(x, y))
        }).flatten().collect(), 3));
    }

    #[test]
    fn can_make_a_rectangular_grid() {
        // the points for a 2x2 grid
        let a = Vec2D::new(-3.0, 3.0);
        let b = Vec2D::new(-1.5, 2.0);
        let c = Vec2D::new(0.85, 1.39);

        let grid = grid_from_cell_and_point((a, b), c);

        let xs = [-3.0, -1.5, 0.0, 1.5];
        let ys = [3.0, 2.0, 1.0];

        assert_eq!(grid, Matrix::from_parts(ys.into_iter().map(|y| {
            xs.iter().map(move |x| Vec2D::new(*x, y))
        }).flatten().collect(), 4));
    }

    #[test]
    fn can_render_a_grid() {
        let a = Vec2DScreen::new(-3.0, 3.0);
        let b = Vec2DScreen::new(-1.5, 2.0);
        let p = Vec2DScreen::new(0.85, 1.39);

        let cols = [-3.0, -1.5, 0.0, 1.5];
        let rows = [3.0, 2.0, 1.0];

        let t = Transform::default();
        let snap = 1.0;

        let mut grid = Grid::start(t.to_world_coordinates(a), Default::default());

        grid.handle_mouse_moved(b, t, snap);
        grid.handle_button_released(b, t, snap);
        grid.handle_mouse_moved(p, t, snap);

        let mut expected_rows: Vec<_> = rows.iter().map(|&r| {
            DrawCommand::Path {
                commands: vec![
                    MoveTo(Vec2DWorld::new(cols[0], r)),
                    LineTo(Vec2DWorld::new(cols[3], r)),
                ],
                style: Default::default(),
            }
        }).collect();
        expected_rows.extend(cols.iter().map(|&c| {
            DrawCommand::Path {
                commands: vec![
                    MoveTo(Vec2DWorld::new(c, rows[0])),
                    LineTo(Vec2DWorld::new(c, rows[2])),
                ],
                style: Default::default(),
            }
        }));

        assert_eq!(grid.draw_commands(t, snap), expected_rows);
    }

    #[test]
    #[ignore]
    fn can_draw_a_grid() {
        todo!("the thing is, this is hard to test because of the trait object,
        but it would be nice to have it converted to an enum and then this can
        be properly tested")
    }
}
