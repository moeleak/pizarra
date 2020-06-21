use std::mem;
use std::path::PathBuf;

use crate::draw_commands::DrawCommand;
use crate::storage::Storage;
use crate::shape::{ShapeType, ShapeId};
use crate::color::Color;
use crate::action::Action;
use crate::point::Point;
use crate::graphics::Transform;

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

/// Possible statuses of file save
#[derive(Clone)]
pub enum SaveStatus {
    NewAndEmpty,
    NewAndChanged,
    Saved(PathBuf),
    Unsaved(PathBuf),
}

pub struct App {
    is_drawing: bool,
    is_moving: bool,
    storage: Storage,
    selected_tool: ShapeType,
    selected_color: Color,

    /// This value represents the coordinate of the drwing space that is currently
    /// at the top-left corner of the screen.
    offset: Point,
    scale_factor: f64,
    dimensions: Point,

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

    /// File management
    save_status: SaveStatus,
}

impl App {
    pub fn new(dimensions: Point) -> App {
        App {
            is_drawing: false,
            is_moving: false,
            storage: Storage::new(),
            selected_tool: ShapeType::Line,

            /// a vector from the top-left corner of the screen to the origin
            /// of coordinates in the storage
            offset: dimensions * -0.5,
            scale_factor: 1.0,
            dimensions,

            undo_state: UndoState::Reset,
            undo_queue: Vec::new(),

            moving_state: MovingState::Start,

            selected_color: Color::green(),
            drawing_state: DrawingState::Start,
            next_id: ShapeId::from(1),

            save_status: SaveStatus::NewAndEmpty,
        }
    }

    pub fn get_transform(&self) -> Transform {
        Transform::new(self.offset, self.scale_factor)
    }

    /// Compute the bounds of what is visible in the board according to current
    /// offset and zoom level
    pub fn visible_extent(&self) -> (Point, Point) {
        (
            self.offset,
            self.offset + self.dimensions / self.scale_factor
        )
    }

    pub fn draw_commands(&self, bbox: [Point; 2]) -> impl Iterator<Item=DrawCommand> + '_ {
        self.storage.draw_commands(bbox)
    }

    /// p is a point in the coordinate space of the drawing area. This function
    /// converts p's coordinates to storage coordinates given the current zoom
    /// and offset.
    fn to_storage_coordinates(&self, p: Point) -> Point {
        p / self.scale_factor + self.offset
    }

    /// p is a point in the coordinate space of the shape storage, and this function
    /// translates it to drawing area's coordinate system.
    pub fn to_screen_coordinates(&self, p: Point) -> Point {
        (p - self.offset) * self.scale_factor
    }

    fn handle_offset(&mut self, delta: Point) {
        self.offset -= delta / self.scale_factor;
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

        self.save_status = match &self.save_status {
            SaveStatus::NewAndEmpty => SaveStatus::NewAndChanged,
            SaveStatus::NewAndChanged => SaveStatus::NewAndChanged,
            SaveStatus::Saved(path) => SaveStatus::Unsaved(path.clone()),
            SaveStatus::Unsaved(path) => SaveStatus::Unsaved(path.clone()),
        };
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
                        self.selected_tool.make(self.selected_color, points, self.next_id, self.scale_factor),
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
        self.offset += (self.dimensions / self.scale_factor) * 0.25;
        self.scale_factor *= 2.0;
    }

    pub fn zoom_out(&mut self) {
        self.offset -= (self.dimensions / self.scale_factor) * 0.5;
        self.scale_factor /= 2.0;
    }

    pub fn go_home(&mut self) {
        self.offset = self.dimensions * -0.5;
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
                if !self.undo_queue.is_empty() {
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

    pub fn shape_count(&self) -> usize {
        self.storage.shape_count()
    }

    pub fn get_bounds(&self) -> Option<[Point; 2]> {
        self.storage.get_bounds()
    }

    pub fn get_save_status(&self) -> &SaveStatus {
        &self.save_status
    }

    pub fn set_saved(&mut self, path: PathBuf) {
        self.save_status = SaveStatus::Saved(path);
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

        assert_eq!(app.shape_count(), 1);
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

        assert_eq!(app.shape_count(), 0);
    }

    #[test]
    fn move_while_in_zoom_is_coherent() {
        let mut app = App::new(Point::new(80.0, 60.0));

        assert_eq!(app.visible_extent(), (Point::new(-40.0, -30.0), Point::new(40.0, 30.0)));

        app.zoom_in();

        assert_eq!(app.visible_extent(), (Point::new(-20.0, -15.0), Point::new(20.0, 15.0)));

        app.start_moving();
        app.handle_cursor(Point::new(0.0, 0.0));
        app.handle_cursor(Point::new(-1.0, 0.0)); // moves one screen pixel to the right
        app.end_moving();

        assert_eq!(app.visible_extent(), (Point::new(-19.5, -15.0), Point::new(20.5, 15.0)));
    }

    #[test]
    fn test_zoom_must_be_idempotent() {
        let mut app = App::new(Point::new(80.0, 60.0));

        assert_eq!(app.visible_extent(), (Point::new(-40.0, -30.0), Point::new(40.0, 30.0)));

        app.zoom_in();

        assert_eq!(app.visible_extent(), (Point::new(-20.0, -15.0), Point::new(20.0, 15.0)));

        app.zoom_in();

        assert_eq!(app.visible_extent(), (Point::new(-10.0, -7.5), Point::new(10.0, 7.5)));
    }
}
