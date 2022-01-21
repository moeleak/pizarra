use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::draw_commands::DrawCommand;
use crate::path_command::PathCommand::*;
use crate::point::{Point, Vec2DScreen, Vec2DWorld, Vec2D};
use crate::transform::Transform;
use crate::style::Style;
use crate::geom::rombus_to_rectangle;

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

        /// A transform matrix that turns the parallelogram defined by start, p1
        /// and p2 into a square of size 1 in the first quadrant
        t: Transform,
        free: Vec2DWorld,
    },

    PreDefineScale {
        start: Vec2DWorld,
        p1: Vec2DWorld,
    },

    DefineScale {
        start: Vec2DWorld,
        p1: Vec2DWorld,
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

            State::PreDefineScale { .. } => { }

            State::DefineScale { start, p1, .. } => {
                self.state = State::DefineScale { start, p1, free: wpos };
            }
        }
    }

    fn handle_button_pressed(&mut self, pos: Vec2DScreen, t: Transform, snap: f64) {
        let wpos = t.to_world_coordinates(pos);

        match self.state {
            State::Initial(_) => {}

            State::OneDimension { .. } => {
            }

            State::TwoDimension { start, p1, .. } => {
                if wpos.distance(p1) < snap {
                    self.state = State::PreDefineScale { start, p1 };
                }
            }

            State::DefineGrid { .. } => { }

            State::PreDefineScale { .. } => { }

            State::DefineScale { .. } => { }
        }
    }

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

            State::DefineGrid { .. } => {
                ShapeFinished::Yes(todo!())
            }

            State::PreDefineScale { start, p1 } => {
                self.state = State::DefineScale  { start, p1, free: wpos };

                ShapeFinished::No
            }

            State::DefineScale { .. } => {
                ShapeFinished::Yes(todo!())
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

            State::DefineGrid { start, p1, p2, t, free } => {
                let base = t.apply(p1.to_vec2d()).x();
                let height = t.apply(p2.to_vec2d()).y();
                let freep = t.apply(free.to_vec2d());

                let x_cells = freep.x() / base;
                let y_cells = freep.y() / height;

                let x_max = x_cells.abs().ceil() * x_cells.signum() * base;
                let y_max = y_cells.abs().ceil() * y_cells.signum() * height;

                // the boundaries of the grid in the transformed space
                let b1 = Vec2D::new(x_max, 0.0);
                let b2 = Vec2D::new(x_max, y_max);
                let b3 = Vec2D::new(0.0, y_max);

                let inverse = t.invert();

                let world_b1 = inverse.apply(b1).into();
                let world_b2 = inverse.apply(b2).into();
                let world_b3 = inverse.apply(b3).into();

                vec![
                    DrawCommand::Path {
                        commands: vec![
                            MoveTo(start),
                            LineTo(world_b1),
                            LineTo(dbg!(world_b2)),
                            LineTo(world_b3),
                            LineTo(start),
                        ],
                        style: Style::path_helper(),
                    },

                    DrawCommand::Path {
                        commands: vec![
                            MoveTo(start),
                            LineTo(p1),
                            LineTo(p2),
                            LineTo(start + (p2 - p1)),
                            LineTo(start),
                        ],
                        style: self.style,
                    },
                ]
            }

            State::PreDefineScale { .. } => {
                todo!()
            }

            State::DefineScale { .. } => {
                todo!()
            }
        }
    }
}
