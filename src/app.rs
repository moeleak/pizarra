use std::mem;
use std::path::PathBuf;

use crate::draw_commands::DrawCommand;
use crate::storage::Storage;
use crate::shape::{ShapeType, ShapeId, ShapeFinished};
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

/// Needed to handle pointer offset during canvas moving
#[derive(Copy, Clone)]
enum MovingState {
    Start,
    From(Point),
}

/// Manages what is happening with the board
#[derive(Copy, Clone)]
enum BoardState {
    Idle,
    UsingTool,
    Moving(MovingState),
    MovingWhileUsingTool(MovingState),
}

/// Indicates wether or not an action should trigger redraw of the canvas
pub enum ShouldRedraw {
    /// Something has changed and we should redraw
    Yes,

    /// Nothing has changed, don't queue a draw event
    No,
}

pub enum SelectedTool {
    Shape(ShapeType),
    Eraser,
}

/// Possible statuses of file save
#[derive(Clone)]
pub enum SaveStatus {
    NewAndEmpty,
    NewAndChanged,
    Saved(PathBuf),
    Unsaved(PathBuf),
}

/// Mouse input management
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Unknown,
}

pub struct App {
    storage: Storage,

    selected_tool: SelectedTool,
    selected_color: Color,

    /// This value represents the coordinate of the drwing space that is currently
    /// at the top-left corner of the screen.
    offset: Point,
    scale_factor: f64,
    dimensions: Point,

    /// Undo
    undo_state: UndoState,
    undo_queue: Vec<Action>,

    /// What is happening with the board right now?
    board_state: BoardState,

    /// Next available id for a shape
    next_id: ShapeId,

    /// File management
    save_status: SaveStatus,
}

impl App {
    pub fn new(dimensions: Point) -> App {
        App {
            storage: Storage::new(),
            selected_tool: SelectedTool::Shape(ShapeType::Line),

            /// a vector from the top-left corner of the screen to the origin
            /// of coordinates in the storage
            offset: dimensions * -0.5,
            scale_factor: 1.0,
            dimensions,

            board_state: BoardState::Idle,

            undo_state: UndoState::Reset,
            undo_queue: Vec::new(),

            selected_color: Color::green(),
            next_id: ShapeId::from(1),

            save_status: SaveStatus::NewAndEmpty,
        }
    }

    fn next_id(&mut self) -> ShapeId {
        let id = self.next_id;
        self.next_id = self.next_id.next();
        id
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

    /// This function is in charge of changing the position of the canvas with respect to the
    /// window.
    fn handle_offset(&mut self, delta: Point) {
        self.offset -= delta / self.scale_factor;
    }

    pub fn resize(&mut self, new_size: Point) {
        let delta = (self.dimensions * -1.0 + new_size) * 0.5;

        self.handle_offset(delta);
        self.dimensions = new_size;
    }

    pub fn set_tool(&mut self, selected_tool: SelectedTool) {
        self.selected_tool = selected_tool;
    }

    fn conclude_action(&mut self, action: Action) {
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

    fn handle_tool_button_pressed(&mut self, point: Point) {
        match self.selected_tool {
            SelectedTool::Shape(shape) => {
                if let Some(shape) = self.storage.last_mut() {
                    shape.handle_button_pressed(point);
                }
            },
            SelectedTool::Eraser => unimplemented!(),
        }
    }

    fn handle_tool_button_released(&mut self, point: Point) -> Option<ShapeId> {
        match self.selected_tool {
            SelectedTool::Shape(shape) => {
                let finished = if let Some(shape) = self.storage.last_mut() {
                    shape.handle_button_released(point)
                } else {
                    ShapeFinished::No
                };

                if let ShapeFinished::Yes = finished {
                    self.storage.flush()
                } else {
                    None
                }
            },
            SelectedTool::Eraser => unimplemented!(),
        }
    }

    fn handle_tool_mouse_move(&mut self, point: Point) -> ShouldRedraw {
        let transformed_point = self.to_storage_coordinates(point);

        match self.selected_tool {
            SelectedTool::Shape(_shape_type) => {
                if let Some(shape) = self.storage.last_mut() {
                    shape.handle_mouse_moved(transformed_point);
                    ShouldRedraw::Yes
                } else {
                    ShouldRedraw::No
                }
            },
            SelectedTool::Eraser => unimplemented!(),
        }
    }

    /// Public method that the UI must call whenever a button of the mouse or pen
    /// is pressed.
    pub fn handle_mouse_button_pressed(&mut self, button: MouseButton, point: Point) -> ShouldRedraw {
        let (new_board_state, should_redraw) = match self.board_state {
            BoardState::Idle => match button {
                MouseButton::Left => {
                    self.handle_tool_button_pressed(point);

                    (BoardState::UsingTool, ShouldRedraw::Yes)
                },
                MouseButton::Middle => unimplemented!(),
                MouseButton::Right => unimplemented!(),
                MouseButton::Unknown => (BoardState::Idle, ShouldRedraw::No),
            },
            BoardState::UsingTool => unimplemented!(),
            BoardState::Moving(_) => unimplemented!(),
            BoardState::MovingWhileUsingTool(_) => unimplemented!(),
        };

        self.board_state = new_board_state;

        should_redraw
    }

    /// Public method that the UI must call whenever a button of the mouse or pen
    /// is released.
    pub fn handle_mouse_button_released(&mut self, button: MouseButton, point: Point) -> ShouldRedraw {
        let (new_board_state, should_redraw) = match self.board_state {
            BoardState::Idle => (BoardState::Idle, ShouldRedraw::No),
            BoardState::UsingTool => match button {
                MouseButton::Left => {
                    if let Some(shape_id) = self.handle_tool_button_released(point) {
                        self.conclude_action(Action::Draw(shape_id));

                        (BoardState::Idle, ShouldRedraw::Yes)
                    } else {
                        (BoardState::UsingTool, ShouldRedraw::Yes)
                    }
                },
                MouseButton::Middle => unimplemented!(),
                MouseButton::Right => unimplemented!(),
                MouseButton::Unknown => (BoardState::UsingTool, ShouldRedraw::No),
            },
            BoardState::Moving(_) => unimplemented!(),
            BoardState::MovingWhileUsingTool(_) => unimplemented!(),
        };

        self.board_state = new_board_state;

        should_redraw
    }

    /// Public methid that the UI must call with updates on the cursor position
    pub fn handle_mouse_move(&mut self, pos: Point) -> ShouldRedraw {
        match self.board_state {
            BoardState::Idle => ShouldRedraw::No,
            BoardState::UsingTool => self.handle_tool_mouse_move(pos),
            BoardState::Moving(moving_state) => {
                match moving_state {
                    MovingState::Start => ShouldRedraw::No,
                    MovingState::From(point) => {
                        self.handle_offset(pos - point);

                        ShouldRedraw::Yes
                    },
                }
            },
            BoardState::MovingWhileUsingTool(moving_state) => {
                match moving_state {
                    MovingState::Start => ShouldRedraw::No,
                    MovingState::From(point) => {
                        self.handle_offset(pos - point);

                        ShouldRedraw::Yes
                    },
                }
            },
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

    pub fn reset(&mut self) {
        self.storage = Storage::new();
        self.selected_tool = SelectedTool::Shape(ShapeType::Line);
        self.offset = self.dimensions * -0.5;
        self.scale_factor = 1.0;
        self.undo_state = UndoState::Reset;
        self.undo_queue = Vec::new();
        self.selected_color = Color::green();
        self.next_id = ShapeId::from(1);
        self.save_status = SaveStatus::NewAndEmpty;
    }
}

#[cfg(test)]
mod tests {
    use super::{App, MouseButton};
    use crate::{draw_commands::DrawCommand, point::Point, color::Color};

    #[test]
    fn test_draw_a_line() {
        let mut app = App::new(Point::new(800.0, 600.0));

        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 0.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(2.0, 0.0));

        assert_eq!(app.shape_count(), 1);

        let commands: Vec<_> = app.draw_commands([Point::new(-1.0, -1.0), Point::new(3.0, 1.0)]).collect();

        assert_eq!(commands[0], DrawCommand::Line {
            color: Color::green(),
            line: vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0), Point::new(2.0, 0.0)],
            thickness: 4.0,
        });
    }

    #[test]
    fn test_move_screen() {
        assert!(false);
    }

    #[test]
    fn test_move_while_drawing() {
        assert!(false);
    }

    #[test]
    fn test_erase() {
        assert!(false);
    }

    #[test]
    fn test_do_do_do_undo_redo_redo_undo() {
        let mut app = App::new(Point::new(800.0, 600.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 1.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(0.0, 1.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 1.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(0.0, 1.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 1.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(0.0, 1.0));

        app.undo();

        app.redo();
        app.redo();

        app.undo();

        assert_eq!(app.shape_count(), 3);
    }

    #[test]
    fn test_do_undo_redo_undo() {
        let mut app = App::new(Point::new(800.0, 600.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 1.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(1.0, 1.0));

        app.undo();

        app.redo();

        app.undo();

        assert_eq!(app.shape_count(), 0);
    }

    #[test]
    fn test_do_undo_do_do_undo() {
        let mut app = App::new(Point::new(800.0, 600.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 1.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(1.0, 1.0));

        app.undo();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 1.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(1.0, 1.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 1.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(1.0, 1.0));

        app.undo();

        assert_eq!(app.shape_count(), 1);
    }

    #[test]
    fn test_do_undo_do_undo_undo() {
        let mut app = App::new(Point::new(800.0, 600.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 1.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(1.0, 1.0));

        app.undo();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 1.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(1.0, 1.0));

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

        app.handle_mouse_button_pressed(MouseButton::Middle, Point::new(0.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Middle, Point::new(-1.0, 0.0));

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
