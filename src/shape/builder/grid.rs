use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::draw_commands::{DrawCommand, circle_helper, red_circle_helper};
use crate::path_command::PathCommand::{self, *};
use crate::point::{Point, Vec2DScreen, Vec2DWorld};
use crate::transform::Transform;
use crate::style::Style;
use crate::matrix::Matrix;

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

            State::DefineGrid { cell, free } => {
                let grid = grid_from_cell_and_point(cell, free);

                todo!()
            }
        }
    }
}

/// Given the two points that define a cell and a third point outside (or
/// inside) it, return a matrix of all the points that make it and its width
fn grid_from_cell_and_point<P: Point>((p1, p2): (P, P), free: P) -> Matrix<P> {
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

    for xi in x_separators {
        for &yi in y_separators.iter() {
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

        assert_eq!(grid, Matrix::from_parts(xs.into_iter().map(|x| {
            ys.iter().map(move |y| Vec2D::new(x, *y))
        }).flatten().collect(), 3));
    }
}
