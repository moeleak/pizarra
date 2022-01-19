use crate::shape::{ShapeBuilder, ShapeFinished};
use crate::draw_commands::DrawCommand;
use crate::path_command::PathCommand::*;
use crate::point::{Point, Vec2DScreen, Vec2DWorld};
use crate::transform::Transform;
use crate::style::Style;

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

            State::DefineGrid { start, p1, p2, .. } => {
                self.state = State::DefineGrid { start, p1, p2, free: wpos };
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
                self.state = State::DefineGrid { start, p1, p2: free, free: wpos };

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

            State::DefineGrid { .. } => {
                todo!()
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
