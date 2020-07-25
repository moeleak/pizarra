use std::mem;
use std::path::PathBuf;

use crate::draw_commands::DrawCommand;
use crate::storage::Storage;
use crate::shape::{Shape, ShapeType, ShapeFinished};
use crate::color::Color;
use crate::action::Action;
use crate::point::Point;
use crate::transform::Transform;
use crate::consts::{DEFAULT_THICKNESS, TOUCH_RADIUS};
use crate::deserialize;

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

/// Manages what is happening with the board
#[derive(Copy, Clone)]
enum BoardState {
    Idle,
    UsingTool,
    Moving(Point),
    MovingWhileUsingTool(Point),
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
    bgcolor: Color,
    stroke: f64,
    erase_radius: f64,

    /// This value represents the coordinate of the drwing space that is currently
    /// at the top-left corner of the screen.
    offset: Point,
    scale_factor: f64,
    dimensions: Point,

    /// Stores erased shapes temporarily while being erased
    erased: Vec<Shape>,

    /// Undo
    undo_state: UndoState,
    undo_queue: Vec<Action>,

    /// What is happening with the board right now?
    board_state: BoardState,

    /// File management
    save_status: SaveStatus,
}

// Private methods
impl App {
    /// p is a point in the coordinate space of the drawing area. This function
    /// converts p's coordinates to storage coordinates given the current zoom
    /// and offset.
    fn to_storage_coordinates(&self, p: Point) -> Point {
        p / self.scale_factor + self.offset
    }

    /// This function is in charge of changing the position of the canvas with respect to the
    /// window.
    fn handle_offset(&mut self, delta: Point) {
        self.offset -= delta / self.scale_factor;
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
        let transformed_point = self.to_storage_coordinates(point);

        match self.selected_tool {
            SelectedTool::Shape(shape) => {
                if let Some(shape) = self.storage.last_mut() {
                    shape.handle_button_pressed(transformed_point);
                } else {
                    let new_shape = shape.start(self.selected_color, transformed_point, self.stroke * 1.0 / self.scale_factor);

                    self.storage.add(new_shape);
                }
            },
            SelectedTool::Eraser => {
                if let Some(shape) = self.storage.query_circle(transformed_point, self.erase_radius / self.scale_factor).map(|id| self.storage.remove(id)).flatten() {
                    self.erased.push(shape);
                }
            },
        }
    }

    fn handle_tool_button_released(&mut self, point: Point) -> Action {
        let transformed_point = self.to_storage_coordinates(point);

        match self.selected_tool {
            SelectedTool::Shape(_shape) => {
                let finished = if let Some(shape) = self.storage.last_mut() {
                    shape.handle_button_released(transformed_point)
                } else {
                    ShapeFinished::No
                };

                if let ShapeFinished::Yes = finished {
                    if let Some(shape_id) = self.storage.flush() {
                        Action::Draw(shape_id)
                    } else {
                        Action::Empty
                    }
                } else {
                    Action::Unfinished
                }
            },
            SelectedTool::Eraser => {
                if let Some(shape) = self.storage.query_circle(transformed_point, self.erase_radius / self.scale_factor).map(|id| self.storage.remove(id)).flatten() {
                    self.erased.push(shape);
                }

                if self.erased.len() > 0 {
                    Action::Erase(self.erased.drain(..).collect())
                } else {
                    Action::Empty
                }
            },
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
            SelectedTool::Eraser => {
                if let Some(shape) = self.storage.query_circle(transformed_point, self.erase_radius / self.scale_factor).map(|id| self.storage.remove(id)).flatten() {
                    self.erased.push(shape);
                    ShouldRedraw::Yes
                } else {
                    ShouldRedraw::No
                }
            },
        }
    }

    fn revert(&mut self, action: Action) -> Action {
        match action {
            Action::Unfinished => Action::Unfinished,
            Action::Empty => Action::Empty,

            Action::Draw(id) => Action::Delete(self.storage.remove(id).unwrap()),

            Action::Delete(shape) => {
                let id = self.storage.restore(shape);

                Action::Draw(id)
            },

            Action::Erase(shapes) => {
                let ids = shapes.into_iter().map(|shape| self.storage.restore(shape)).collect();

                self.storage.flush();

                Action::Redraw(ids)
            },

            Action::Redraw(ids) => {
                let shapes = ids.into_iter().map(|id| self.storage.remove(id)).filter_map(|maybe_shape| maybe_shape).collect();

                Action::Erase(shapes)
            },
        }
    }
}

// Public interface
impl App {
    /// Create a new instance of Pizarra's controller. Only one is needed and it
    /// can be put inside an Rc<RefCell<_>> to use it inside event callbacks in
    /// UI implementations.
    pub fn new(dimensions: Point) -> App {
        App {
            storage: Storage::new(),
            selected_tool: SelectedTool::Shape(ShapeType::Line),
            bgcolor: Color::black(),
            stroke: DEFAULT_THICKNESS,
            erase_radius: TOUCH_RADIUS,

            /// a vector from the top-left corner of the screen to the origin
            /// of coordinates in the storage
            offset: dimensions * -0.5,
            scale_factor: 1.0,
            dimensions,

            erased: Vec::new(),

            board_state: BoardState::Idle,

            undo_state: UndoState::Reset,
            undo_queue: Vec::new(),

            selected_color: Default::default(),

            save_status: SaveStatus::NewAndEmpty,
        }
    }

    /// Returns a transform object needed for proper rendering. This object can
    /// translate a point from storage coordinates to screen coordinates.
    pub fn get_transform(&self) -> Transform {
        Transform::new(self.offset, self.scale_factor)
    }

    /// Compute the bounds of what is visible in the board according to current
    /// offset and zoom level
    pub fn visible_extent(&self) -> [Point; 2] {
        [
            self.offset,
            self.offset + self.dimensions / self.scale_factor
        ]
    }

    /// Returns the dimensions of the screen
    pub fn get_dimensions(&self) -> Point {
        self.dimensions
    }

    /// Returns current background color
    pub fn bgcolor(&self) -> Color {
        self.bgcolor
    }

    /// Returns an iterator over the draw commands required to render the visible
    /// portion of the drawing
    pub fn draw_commands_for_screen(&self) -> Vec<DrawCommand> {
        self.storage.draw_commands(self.visible_extent())
    }

    /// Returns an interator over the draw commands required to render the wole
    /// drawing
    pub fn draw_commands_for_drawing(&self) -> Vec<DrawCommand> {
        let bbox = if let Some(bbox) = self.get_bounds() {
            bbox
        } else {
            [Point::new(0.0, 0.0), Point::new(0.0, 0.0)]
        };

        self.storage.draw_commands(bbox)
    }

    /// p is a point in the coordinate space of the shape storage, and this function
    /// translates it to drawing area's coordinate system.
    pub fn to_screen_coordinates(&self, p: Point) -> Point {
        (p - self.offset) * self.scale_factor
    }

    /// Notify the application that the screen size has changed
    pub fn resize(&mut self, new_size: Point) {
        let delta = (self.dimensions * -1.0 + new_size) * 0.5;

        self.handle_offset(delta);
        self.dimensions = new_size;
    }

    /// Set a new tool used to handle user input
    pub fn set_tool(&mut self, selected_tool: SelectedTool) {
        self.selected_tool = selected_tool;
    }

    /// Set a new stroke that will be used in next shapes
    pub fn set_stroke(&mut self, stroke: f64) {
        self.stroke = stroke;
    }

    /// Set the background color of the drawing
    pub fn set_bgcolor(&mut self, bgcolor: Color) {
        self.bgcolor = bgcolor;
    }

    /// Set the radius of action for the eraser tool
    pub fn set_erase_radius(&mut self, erase_radius: f64) {
        self.erase_radius = erase_radius;
    }

    /// Public method that the UI must call whenever a button of the mouse or pen
    /// is pressed.
    pub fn handle_mouse_button_pressed(&mut self, button: MouseButton, point: Point) -> ShouldRedraw {
        let (new_board_state, should_redraw) = match self.board_state {
            current@BoardState::Idle => match button {
                MouseButton::Left => {
                    self.handle_tool_button_pressed(point);

                    (BoardState::UsingTool, ShouldRedraw::Yes)
                },
                MouseButton::Middle => (BoardState::Moving(point), ShouldRedraw::No),
                MouseButton::Right => (current, ShouldRedraw::No),
                MouseButton::Unknown => (current, ShouldRedraw::No),
            },
            current@BoardState::UsingTool => match button {
                MouseButton::Left => {
                    self.handle_tool_button_pressed(point);

                    (current, ShouldRedraw::Yes)
                },
                MouseButton::Middle => (BoardState::MovingWhileUsingTool(point), ShouldRedraw::No),
                MouseButton::Right => (current, ShouldRedraw::No),
                MouseButton::Unknown => (current, ShouldRedraw::No),
            },
            current@BoardState::Moving(_) => (current, ShouldRedraw::No),
            current@BoardState::MovingWhileUsingTool(_) => (current, ShouldRedraw::No),
        };

        self.board_state = new_board_state;

        should_redraw
    }

    /// Public method that the UI must call whenever a button of the mouse or pen
    /// is released.
    pub fn handle_mouse_button_released(&mut self, button: MouseButton, point: Point) -> ShouldRedraw {
        let (new_board_state, should_redraw) = match self.board_state {
            current@BoardState::Idle => (current, ShouldRedraw::No),
            current@BoardState::UsingTool => match button {
                MouseButton::Left => match self.handle_tool_button_released(point) {
                    Action::Unfinished => (current, ShouldRedraw::Yes),
                    Action::Empty => (BoardState::Idle, ShouldRedraw::No),
                    x => {
                        self.conclude_action(x);
                        (BoardState::Idle, ShouldRedraw::Yes)
                    }
                },
                MouseButton::Middle => (current, ShouldRedraw::No),
                MouseButton::Right => (current, ShouldRedraw::No),
                MouseButton::Unknown => (current, ShouldRedraw::No),
            },
            BoardState::Moving(old_point) => match button {
                MouseButton::Left => (BoardState::Moving(old_point), ShouldRedraw::No),
                MouseButton::Middle => {
                    self.handle_offset(point - old_point);

                    (BoardState::Idle, ShouldRedraw::Yes)
                },
                MouseButton::Right => (BoardState::Moving(old_point), ShouldRedraw::No),
                MouseButton::Unknown => (BoardState::Moving(old_point), ShouldRedraw::No),
            },
            // TODO check if https://github.com/rust-lang/rust/issues/65490 is resolved
            BoardState::MovingWhileUsingTool(old_point) => match button {
                MouseButton::Left => match self.handle_tool_button_released(point) {
                        Action::Unfinished => (BoardState::MovingWhileUsingTool(old_point), ShouldRedraw::Yes),
                        Action::Empty => (BoardState::Moving(point), ShouldRedraw::Yes),
                        x => {
                            self.conclude_action(x);

                            (BoardState::Moving(point), ShouldRedraw::Yes)
                        }
                },
                MouseButton::Middle => {
                    self.handle_offset(point - old_point);

                    (BoardState::UsingTool, ShouldRedraw::Yes)
                },
                MouseButton::Right => (BoardState::MovingWhileUsingTool(old_point), ShouldRedraw::No),
                MouseButton::Unknown => (BoardState::MovingWhileUsingTool(old_point), ShouldRedraw::No),
            },
        };

        self.board_state = new_board_state;

        should_redraw
    }

    /// Public methid that the UI must call with updates on the cursor position
    pub fn handle_mouse_move(&mut self, pos: Point) -> ShouldRedraw {
        let (new_board_state, should_redraw) = match self.board_state {
            BoardState::Idle => (BoardState::Idle, ShouldRedraw::No),
            BoardState::UsingTool => (BoardState::UsingTool, self.handle_tool_mouse_move(pos)),
            BoardState::Moving(old_point) => {
                self.handle_offset(pos - old_point);

                (BoardState::Moving(pos), ShouldRedraw::Yes)
            },
            BoardState::MovingWhileUsingTool(old_point) => {
                self.handle_offset(pos - old_point);

                (BoardState::MovingWhileUsingTool(pos), ShouldRedraw::Yes)
            },
        };

        self.board_state = new_board_state;

        should_redraw
    }

    /// Zooms in the current view, so objects are bigger and the visible portion
    /// of the drawing gets reduced
    pub fn zoom_in(&mut self) {
        self.offset += (self.dimensions / self.scale_factor) * 0.25;
        self.scale_factor *= 2.0;
    }

    /// Zooms out the current view, so objects are smaller and the visible portion
    /// of the drawing increases
    pub fn zoom_out(&mut self) {
        self.offset -= (self.dimensions / self.scale_factor) * 0.5;
        self.scale_factor /= 2.0;
    }

    /// Resets the center and zoom level of the view to its original place
    pub fn go_home(&mut self) {
        self.offset = self.dimensions * -0.5;
        self.scale_factor = 1.0;
    }

    /// Sets the color for the new shapes
    pub fn set_color(&mut self, color: Color) {
        self.selected_color = color;
    }

    /// Undoes the last action if any
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
                let action_to_undo = mem::replace(&mut self.undo_queue[point], Action::Empty);

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

    /// Redoes the last undone action, if any
    pub fn redo(&mut self) {
        match self.undo_state {
            // nothing has to be done for nothing has been undone
            UndoState::InSync => {},

            UndoState::At(point) => {
                let action_to_redo = mem::replace(&mut self.undo_queue[point + 1], Action::Empty);

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
                    let action_to_redo = mem::replace(&mut self.undo_queue[0], Action::Empty);

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

    /// Return the drawing's total number of shapes
    pub fn shape_count(&self) -> usize {
        self.storage.shape_count()
    }

    /// Returns the bounding box that contains all the shapes in the drawing
    pub fn get_bounds(&self) -> Option<[Point; 2]> {
        self.storage.get_bounds()
    }

    /// Returns a reference to this drawing's save status
    pub fn get_save_status(&self) -> &SaveStatus {
        &self.save_status
    }

    /// Notify the application that the drawing has been saved at `path`
    pub fn set_saved(&mut self, path: PathBuf) {
        self.save_status = SaveStatus::Saved(path);
    }

    /// Resets the state of the drawing, undo, shapes etc so you can start
    /// drawing again.
    pub fn reset(&mut self) {
        self.storage = Storage::new();
        self.selected_tool = SelectedTool::Shape(ShapeType::Line);
        self.offset = self.dimensions * -0.5;
        self.scale_factor = 1.0;
        self.undo_state = UndoState::Reset;
        self.undo_queue = Vec::new();
        self.save_status = SaveStatus::NewAndEmpty;
    }

    pub fn open(&mut self, svg: &str) -> Result<(), deserialize::Error> {
        self.reset();
        self.storage = Storage::from_svg(svg)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{App, MouseButton};
    use crate::{
        draw_commands::DrawCommand, point::Point, color::Color, app::SelectedTool,
        shape::ShapeType, consts::DEFAULT_THICKNESS,
    };

    #[test]
    fn test_draw_a_line() {
        let mut app = App::new(Point::new(80.0, 60.0));

        app.set_tool(SelectedTool::Shape(ShapeType::Line));

        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 0.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(2.0, 0.0));

        assert_eq!(app.shape_count(), 1);

        let commands = app.draw_commands_for_screen();

        assert_eq!(commands[0], DrawCommand::Line {
            color: Default::default(),
            line: Rc::new(vec![Point::new(-40.0, -30.0), Point::new(-39.0, -30.0), Point::new(-38.0, -30.0)]),
            thickness: DEFAULT_THICKNESS,
        });
    }

    #[test]
    fn test_draw_a_line_in_zoom() {
        let mut app = App::new(Point::new(80.0, 60.0));

        app.set_tool(SelectedTool::Shape(ShapeType::Line));

        app.zoom_in();

        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(0.0, 0.0));
        app.handle_mouse_move(Point::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(2.0, 0.0));

        assert_eq!(app.shape_count(), 1);

        let commands = app.draw_commands_for_screen();

        assert_eq!(commands[0], DrawCommand::Line {
            color: Default::default(),
            line: Rc::new(vec![Point::new(-20.0, -15.0), Point::new(-19.5, -15.0), Point::new(-19.0, -15.0)]),
            thickness: DEFAULT_THICKNESS/2.0,
        });
    }

    #[test]
    #[ignore]
    fn test_draw_a_rectangle() {
        assert!(false);
    }

    #[test]
    #[ignore]
    fn test_draw_an_ellipse() {
        assert!(false);
    }

    #[test]
    fn test_erase() {
        let mut app = App::new(Point::new(80.0, 60.0));

        app.set_tool(SelectedTool::Shape(ShapeType::Line));

        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(10.0, 10.0));
        app.handle_mouse_move(Point::new(15.0, 10.0));
        app.handle_mouse_move(Point::new(15.0, 15.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(10.0, 15.0));

        assert_eq!(app.shape_count(), 1);

        app.set_tool(SelectedTool::Eraser);

        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(10.0, 10.0));
        app.handle_mouse_move(Point::new(15.0, 10.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(15.0, 10.0));

        assert_eq!(app.shape_count(), 0);
    }

    #[test]
    fn test_erase_with_zoom() {
        let mut app = App::new(Point::new(4.0, 4.0));

        app.set_tool(SelectedTool::Shape(ShapeType::Line));

        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(2.0, 1.0));
        app.handle_mouse_move(Point::new(2.0, 1.5));
        app.handle_mouse_move(Point::new(2.0, 2.0));
        app.handle_mouse_move(Point::new(2.0, 2.5));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(2.0, 3.0));

        assert_eq!(app.shape_count(), 1);

        app.zoom_in();

        app.set_tool(SelectedTool::Eraser);
        app.set_erase_radius(0.5);

        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(2.6, 0.0));
        app.handle_mouse_move(Point::new(2.6, 1.0));
        app.handle_mouse_move(Point::new(2.6, 3.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(2.6, 4.0));

        assert_eq!(app.shape_count(), 1);

        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(2.24, 0.0));
        app.handle_mouse_move(Point::new(2.4, 1.0));
        app.handle_mouse_move(Point::new(2.4, 3.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(2.24, 4.0));

        assert_eq!(app.shape_count(), 0);
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

        assert_eq!(app.shape_count(), 2);
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
    fn test_move() {
        let mut app = App::new(Point::new(80.0, 60.0));

        assert_eq!(app.visible_extent(), [Point::new(-40.0, -30.0), Point::new(40.0, 30.0)]);

        app.handle_mouse_button_pressed(MouseButton::Middle, Point::new(0.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Middle, Point::new(-1.0, 0.0));

        assert_eq!(app.visible_extent(), [Point::new(-39.0, -30.0), Point::new(41.0, 30.0)]);
    }

    #[test]
    fn move_while_in_zoom_is_coherent() {
        let mut app = App::new(Point::new(80.0, 60.0));

        assert_eq!(app.visible_extent(), [Point::new(-40.0, -30.0), Point::new(40.0, 30.0)]);

        app.zoom_in();

        assert_eq!(app.visible_extent(), [Point::new(-20.0, -15.0), Point::new(20.0, 15.0)]);

        app.handle_mouse_button_pressed(MouseButton::Middle, Point::new(0.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Middle, Point::new(-1.0, 0.0));

        assert_eq!(app.visible_extent(), [Point::new(-19.5, -15.0), Point::new(20.5, 15.0)]);
    }

    #[test]
    fn test_move_while_drawing() {
        let mut app = App::new(Point::new(80.0, 60.0));

        assert_eq!(app.visible_extent(), [Point::new(-40.0, -30.0), Point::new(40.0, 30.0)]);

        app.set_tool(SelectedTool::Shape(ShapeType::Line));

        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(10.0, 10.0));
        app.handle_mouse_move(Point::new(20.0, 10.0));
        app.handle_mouse_button_pressed(MouseButton::Middle, Point::new(20.0, 10.0));
        app.handle_mouse_move(Point::new(30.0, 10.0));
        app.handle_mouse_button_released(MouseButton::Middle, Point::new(30.0, 10.0));
        app.handle_mouse_move(Point::new(40.0, 10.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(40.0, 10.0));

        assert_eq!(app.visible_extent(), [Point::new(-50.0, -30.0), Point::new(30.0, 30.0)]);
        assert_eq!(app.shape_count(), 1);

        let commands = app.draw_commands_for_screen();

        assert_eq!(commands.len(), 1);

        assert_eq!(commands[0], DrawCommand::Line {
            color: Default::default(),
            thickness: DEFAULT_THICKNESS,
            line: Rc::new(vec![
                Point::new(-30.0, -20.0),
                Point::new(-20.0, -20.0),
                Point::new(-10.0, -20.0),
                Point::new(-10.0, -20.0),
            ]),
        });
    }

    #[test]
    fn test_zoom_must_be_idempotent() {
        let mut app = App::new(Point::new(80.0, 60.0));

        assert_eq!(app.visible_extent(), [Point::new(-40.0, -30.0), Point::new(40.0, 30.0)]);

        app.zoom_in();

        assert_eq!(app.visible_extent(), [Point::new(-20.0, -15.0), Point::new(20.0, 15.0)]);

        app.zoom_in();

        assert_eq!(app.visible_extent(), [Point::new(-10.0, -7.5), Point::new(10.0, 7.5)]);
    }

    #[test]
    fn test_cache_invalidation_on_ctrl_z() {
        let mut app = App::new(Point::new(10.0, 10.0));

        app.set_tool(SelectedTool::Shape(ShapeType::Rectangle));

        // draw a shape
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(5.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(6.0, 6.0));

        assert_eq!(app.draw_commands_for_screen().len(), 1);

        // move the screen
        app.handle_mouse_button_pressed(MouseButton::Middle, Point::new(10.0, 5.0));
        app.handle_mouse_move(Point::new(5.0, 5.0));
        app.handle_mouse_move(Point::new(0.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Middle, Point::new(0.0, 5.0));

        assert_eq!(app.draw_commands_for_screen().len(), 0);

        // draw another shape
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(5.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(6.0, 6.0));

        assert_eq!(app.draw_commands_for_screen().len(), 1);

        // and draw a third shape
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(6.0, 6.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(7.0, 7.0));

        assert_eq!(app.draw_commands_for_screen().len(), 2);

        // undo that last one
        app.undo();

        assert_eq!(app.draw_commands_for_screen().len(), 1);

        // move back to where the first shape was
        app.handle_mouse_button_pressed(MouseButton::Middle, Point::new(0.0, 5.0));
        app.handle_mouse_move(Point::new(5.0, 5.0));
        app.handle_mouse_move(Point::new(10.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Middle, Point::new(10.0, 5.0));

        assert_eq!(app.draw_commands_for_screen().len(), 1);
    }

    #[test]
    fn test_not_erasing_a_shape_doesnt_leave_the_eraser_active() {
        // the scenario is: you draw a few shapes, then change to the eraser, click and drag
        // on the screen but without touching a shape and then release the left button.
        //
        // what happens is the eraser remains active and when you move and touch a shape
        // it gets erased
        //
        // it's expeceted for the eraser not to erase the shape
        let mut app = App::new(Point::new(100.0, 100.0));

        app.set_tool(SelectedTool::Shape(ShapeType::Rectangle));

        // draw a shape
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(5.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(6.0, 6.0));

        app.set_tool(SelectedTool::Eraser);

        // erase nothing
        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(90.0, 90.0));
        app.handle_mouse_move(Point::new(91.0, 91.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(92.0, 92.0));

        // move to where the shape is
        app.handle_mouse_move(Point::new(5.0, 5.0));

        assert_eq!(app.shape_count(), 1);
    }
}
