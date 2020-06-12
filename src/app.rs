use std::mem;

use rstar::AABB;

use crate::draw_commands::DrawCommand;
use crate::storage::ShapeStorage;
use crate::shape::{ShapeType, ShapeId};
use crate::color::Color;
use crate::action::Action;
use crate::point::Point;

#[derive(Copy,Clone,Debug)]
/// Describes the possible states of the undo_queue and a pointer that moves
/// in it
enum UndoState {
    /// the last action is effectively the last action of que queue. This means
    /// such an action exists.
    InSync,

    /// some actions have been undone and thus the pointer of the last action
    /// is at this (valid) location
    At(usize),

    /// Either due to excesive undoing the pointer points past the begining of the
    /// undo_queue or there are no actions.
    Reset,
}

/// In order to add a shape to the storage it is necessary that it is fully constructed.
/// Most shapes need at least two points to be valid so this enum handles the states before
/// all the needed points have been collected.
enum DrawingState {
    /// In the begining there were no points
    Start,

    /// Then god added one point
    OnePoint(Point),

    /// And it was good, so she added nother point, and built a shape
    TwoPoints([Point; 2]),

    /// From there the shape new what to do with its life
    AndSoOn,
}

enum MovingState {
    Start,
    From(Point),
}

/// Indicates wether or not an action should trigger redraw of the canvas
pub enum ShouldRedraw {
    /// Something has changed and we should redraw
    Yes,

    /// Nothing has changed, don't queue a draw event
    No,
}

pub struct App {
    is_drawing: bool,
    is_moving: bool,
    storage: ShapeStorage,
    selected_tool: ShapeType,

    /// This value represents a vector from the top left corner of the window to
    /// the center of the drawing
    offset: Point,
    scale_factor: f64,

    /// The window's dimensions
    dimensions: Point,
    selected_color: Color,

    /// Undo
    undo_state: UndoState,
    undo_queue: Vec<Action>,

    /// Since at least two points are needed to create a shape this enum keeps
    /// track of some intermediate states like Start, OnePoint and TwoPoints
    drawing_state: DrawingState,

    /// Since gtk doesn't offer a cursor offset event we sould compute it by hand,
    /// this enum handles the evolution of such state.
    moving_state: MovingState,

    /// Next available id for a shape
    next_id: ShapeId,
}

impl App {
    pub fn new(dimensions: Point) -> App {
        App {
            is_drawing: false,
            is_moving: false,
            storage: ShapeStorage::new(),
            selected_tool: ShapeType::Line,
            dimensions,

            /// a vector from the top-left corner of the screen to the origin
            /// of coordinates in the storage
            offset: dimensions * 0.5,
            scale_factor: 1.0,

            undo_state: UndoState::Reset,
            undo_queue: Vec::new(),

            moving_state: MovingState::Start,

            selected_color: Color::green(),
            drawing_state: DrawingState::Start,
            next_id: ShapeId::from(1),
        }
    }

    /// Compute the bounds of that is visible in the board according to current
    /// offset and zoom level
    pub fn visible_extent(&self) -> (Point, Point) {
        let inv_offset = self.offset * -1.0;
        (inv_offset, inv_offset + self.dimensions)
    }

    pub fn draw_commands(&self, bbox: AABB<[f64; 2]>) -> impl Iterator<Item=DrawCommand> + '_ {
        self.storage.draw_commands(bbox)
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// p is a point build from an event triggered in the canvas, this function
    /// converts p's coordinates to storage coordinates given the current zoom
    /// and offset.
    fn to_storage_coordinates(&self, p: Point) -> Point {
        p + (self.offset * -1.0)
    }

    /// applyes a transform to the given cairo context that scales a shape from
    /// the storage to screen coordinates
    pub fn to_screen_coordinates(&self, point: Point) -> Point {
        point + self.offset
    }

    fn handle_offset(&mut self, delta: Point) {
        self.offset += delta;
    }

    pub fn handle_scroll(&mut self, delta: Point) {
        self.offset = self.offset + delta * 8.0;
    }

    pub fn resize(&mut self, new_size: Point) {
        let delta = (self.dimensions * -1.0 + new_size) * 0.5;

        self.handle_offset(delta);
        self.dimensions = new_size;
    }

    pub fn set_tool(&mut self, shape_type: ShapeType) {
        self.selected_tool = shape_type;
    }

    pub fn start_drawing(&mut self) {
        self.is_drawing = true;
    }

    pub fn end_drawing(&mut self) {
        self.is_drawing = false;
        self.drawing_state = DrawingState::Start;

        if let Some(inserted_id) = self.storage.flush() {
            self.conclude_action(Action::Draw(inserted_id));
        }
    }

    pub fn conclude_action(&mut self, action: Action) {
        // this means some undos have been applied. Doing something from here
        // means effectively dropping some actions such that we're again in the
        // end of the undo_queue
        match self.undo_state {
            UndoState::At(point) => {
                self.undo_queue.resize_with(point+1, Default::default);
            },
            UndoState::Reset => {
                self.undo_queue.resize_with(0, Default::default);
            },
            UndoState::InSync => {},
        }

        self.undo_queue.push(action);
        self.undo_state = UndoState::InSync;
    }

    pub fn start_moving(&mut self) {
        self.is_moving = true;
    }

    pub fn end_moving(&mut self) {
        self.is_moving = false;
        self.moving_state = MovingState::Start;
    }

    pub fn handle_cursor(&mut self, pos: Point) -> ShouldRedraw {
        if self.is_drawing && !self.is_moving {
            let transformed_point = self.to_storage_coordinates(pos);

            match self.drawing_state {
                DrawingState::Start => {
                    self.drawing_state = DrawingState::OnePoint(transformed_point);
                    ShouldRedraw::No
                },
                DrawingState::OnePoint(p1) => {
                    self.drawing_state = DrawingState::TwoPoints([p1, transformed_point]);
                    ShouldRedraw::No
                },
                DrawingState::TwoPoints(points) => {
                    self.storage.add(
                        self.selected_tool.make(self.selected_color, points, self.next_id),
                    );
                    self.next_id.incr();
                    self.drawing_state = DrawingState::AndSoOn;
                    ShouldRedraw::Yes
                },
                DrawingState::AndSoOn => {
                    if let Some(shape) = self.storage.last_mut() {
                        shape.handle(transformed_point);
                    }
                    ShouldRedraw::Yes
                },
            }
        } else if self.is_moving {
            match self.moving_state {
                MovingState::Start => {
                    self.moving_state = MovingState::From(pos);

                    ShouldRedraw::No
                },
                MovingState::From(point) => {
                    self.moving_state = MovingState::From(pos);
                    self.handle_offset(pos - point);

                    ShouldRedraw::Yes
                },
            }
        } else {
            ShouldRedraw::No
        }
    }

    pub fn zoom_in(&mut self) {
        self.offset = (self.offset + self.dimensions * -0.25) * 2.0;
        self.scale_factor *= 2.0;
    }

    pub fn zoom_out(&mut self) {
        self.offset = (self.offset + self.dimensions * 0.5) * 0.5;
        self.scale_factor /= 2.0;
    }

    pub fn go_home(&mut self) {
        self.offset = self.dimensions * 0.5;
        self.scale_factor = 1.0;
    }

    pub fn set_color(&mut self, color: Color) {
        self.selected_color = color;
    }

    fn revert(&mut self, action: Action) -> Action {
        match action {
            Action::Skip => Action::Skip,

            Action::Draw(id) => Action::Delete(self.storage.remove(id).unwrap()),

            Action::Delete(shape) => {
                let id = self.storage.add(shape);

                self.storage.flush();

                Action::Draw(id)
            },
        }
    }

    pub fn undo(&mut self) {
        match self.undo_state {
            // this is the first undo of this queue, we'll invert the last state
            // to its oposite and move the pointer backwards
            UndoState::InSync => {
                if let Some(action) = self.undo_queue.pop() {
                    let reverted = self.revert(action);

                    self.undo_queue.push(reverted);

                    if self.undo_queue.len() == 1 {
                        self.undo_state = UndoState::Reset;
                    } else {
                        self.undo_state = UndoState::At(self.undo_queue.len() - 2);
                    }
                }
            },

            UndoState::At(point) => {
                let action_to_undo = mem::replace(&mut self.undo_queue[point], Action::Skip);

                self.undo_queue[point] = self.revert(action_to_undo);

                if point == 0 {
                    self.undo_state = UndoState::Reset;
                } else {
                    self.undo_state = UndoState::At(point - 1);
                }
            },

            // nothing has to be done for we're past the end of the queue
            UndoState::Reset => { },
        }
    }

    pub fn redo(&mut self) {
        match self.undo_state {
            // nothing has to be done for nothing has been undone
            UndoState::InSync => {},

            UndoState::At(point) => {
                let action_to_redo = mem::replace(&mut self.undo_queue[point + 1], Action::Skip);

                self.undo_queue[point + 1] = self.revert(action_to_redo);

                if point == self.undo_queue.len() - 2 {
                    self.undo_state = UndoState::InSync;
                } else {
                    self.undo_state = UndoState::At(point + 1);
                }
            },

            UndoState::Reset => {
                // there's something to redo
                if let false = self.undo_queue.is_empty() {
                    let action_to_redo = mem::replace(&mut self.undo_queue[0], Action::Skip);

                    self.undo_queue[0] = self.revert(action_to_redo);

                    if self.undo_queue.len() == 1 {
                        self.undo_state = UndoState::InSync;
                    } else {
                        self.undo_state = UndoState::At(0);
                    }
                }
            },
        }
    }

    pub fn storage(&self) -> &ShapeStorage {
        &self.storage
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use crate::point::Point;

    #[test]
    fn test_do_do_do_undo_redo_redo_undo() {
        let mut app = App::new(Point::new(800.0, 600.0));

        // Do
        app.start_drawing();
        app.handle_cursor(Point::new(0.0, 1.0));
        app.handle_cursor(Point::new(1.0, 0.0));
        app.handle_cursor(Point::new(0.0, 1.0));
        app.end_drawing();

        // Do
        app.start_drawing();
        app.handle_cursor(Point::new(0.0, 1.0));
        app.handle_cursor(Point::new(1.0, 0.0));
        app.handle_cursor(Point::new(0.0, 1.0));
        app.end_drawing();

        // Do
        app.start_drawing();
        app.handle_cursor(Point::new(0.0, 1.0));
        app.handle_cursor(Point::new(1.0, 0.0));
        app.handle_cursor(Point::new(0.0, 1.0));
        app.end_drawing();

        app.undo();

        app.redo();
        app.redo();

        app.undo();
    }

    #[test]
    fn test_do_undo_redo_undo() {
        let mut app = App::new(Point::new(800.0, 600.0));

        // Do
        app.start_drawing();
        app.handle_cursor(Point::new(0.0, 1.0));
        app.handle_cursor(Point::new(1.0, 0.0));
        app.handle_cursor(Point::new(1.0, 1.0));
        app.end_drawing();

        app.undo();

        app.redo();

        app.undo();
    }

    #[test]
    fn test_do_undo_do_do_undo() {
        let mut app = App::new(Point::new(800.0, 600.0));

        // Do
        app.start_drawing();
        app.handle_cursor(Point::new(0.0, 1.0));
        app.handle_cursor(Point::new(1.0, 0.0));
        app.handle_cursor(Point::new(1.0, 1.0));
        app.end_drawing();

        app.undo();

        // Do
        app.start_drawing();
        app.handle_cursor(Point::new(0.0, 1.0));
        app.handle_cursor(Point::new(1.0, 0.0));
        app.handle_cursor(Point::new(1.0, 1.0));
        app.end_drawing();

        // Do
        app.start_drawing();
        app.handle_cursor(Point::new(0.0, 1.0));
        app.handle_cursor(Point::new(1.0, 0.0));
        app.handle_cursor(Point::new(1.0, 1.0));
        app.end_drawing();

        app.undo();

        assert_eq!(app.storage().shape_count(), 1);
    }

    #[test]
    fn test_do_undo_do_undo_undo() {
        let mut app = App::new(Point::new(800.0, 600.0));

        // Do
        app.start_drawing();
        app.handle_cursor(Point::new(0.0, 1.0));
        app.handle_cursor(Point::new(1.0, 0.0));
        app.handle_cursor(Point::new(1.0, 1.0));
        app.end_drawing();

        app.undo();

        // Do
        app.start_drawing();
        app.handle_cursor(Point::new(0.0, 1.0));
        app.handle_cursor(Point::new(1.0, 0.0));
        app.handle_cursor(Point::new(1.0, 1.0));
        app.end_drawing();

        app.undo();
        app.undo();

        assert_eq!(app.storage().shape_count(), 0);
    }

    #[test]
    fn test_undo_still_broken() {
        assert!(false, "yep, still broken");
    }
}
