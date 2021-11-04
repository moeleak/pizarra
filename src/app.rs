use std::mem;
use std::path::PathBuf;

use crate::draw_commands::DrawCommand;
use crate::storage::Storage;
use crate::shape::{Shape, ShapeTool, ShapeFinished, ShapeBuilder};
use crate::color::Color;
use crate::action::Action;
use crate::point::{Point, Vec2DWorld, Vec2DScreen};
use crate::transform::Transform;
use crate::deserialize;
use crate::geom::{Angle, bbox_from_points};
use crate::key::Key;
use crate::config::Config;

#[derive(Copy,Clone,Debug, PartialEq, Eq)]
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
#[derive(Debug, Copy, Clone, PartialEq)]
enum BoardState {
    Idle,
    UsingTool,
    Moving(Vec2DScreen),
    MovingWhileUsingTool(Vec2DScreen),
}

/// Indicates wether or not an action should trigger redraw of the canvas
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ShouldRedraw {
    /// The change was so big that it is worth repainting the whole visible
    /// screen
    All,

    /// Only the last shape changed, no need to repaint the whole screen
    Shape,

    /// Nothing has changed, don't queue a draw event
    No,
}

#[derive(Debug)]
pub enum SelectedTool {
    Shape(ShapeTool),
    Eraser,
}

/// Possible statuses of file save
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveStatus {
    NewAndEmpty,
    NewAndChanged,
    Saved(PathBuf),
    Unsaved(PathBuf),
}

/// Mouse input management
#[derive(Debug)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Unknown,
}

/// Main controller of the pizarra application.
///
/// This struct keeps the state of a drawing and exposes the methods necessary
/// to mutate it according to events in the interface.
pub struct Pizarra {
    /// The settings that where loaded at app start
    config: Config,

    /// Finished shapes
    storage: Storage,

    /// What is happening with the board right now?
    board_state: BoardState,
    selected_tool: SelectedTool,
    current_shape: Option<Box<dyn ShapeBuilder>>,

    /// Some paremeters used during drawing
    selected_color: Color,
    bgcolor: Color,
    stroke: f64,
    erase_radius: f64,

    /// The transformation matrix that takes points from the drawing coordinate
    /// system to the screen coordinate system.
    transform: Transform,
    /// What is the  size of the canvas in pixels?
    dimensions: Vec2DScreen,

    /// Stores erased shapes temporarily while being erased
    erased: Vec<Shape>,

    /// Undo
    undo_state: UndoState,
    undo_queue: Vec<Action>,

    /// Is the shift key pressed on the keyboard?
    is_shift_pressed: bool,

    /// File management
    save_status: SaveStatus,
}

// Private methods
impl Pizarra {
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

    fn handle_tool_button_pressed(&mut self, point: Vec2DScreen) -> ShouldRedraw {
        let transformed_point = self.transform.to_world_coordinates(point);

        match self.selected_tool {
            SelectedTool::Shape(shape) => {
                if let Some(shape) = self.current_shape.as_mut() {
                    shape.handle_button_pressed(point, self.transform, self.config.point_snap_radius);
                } else {
                    let new_shape = shape.start(self.selected_color, transformed_point, self.transform.to_world_units(self.stroke));

                    self.current_shape = Some(new_shape);
                }

                ShouldRedraw::Shape
            },
            SelectedTool::Eraser => {
                let deleted_shape = self.storage
                    .query_circle(transformed_point, self.transform.to_world_units(self.erase_radius))
                    .map(|id| self.storage.remove(id))
                    .flatten();

                if let Some(shape) = deleted_shape {
                    self.erased.push(shape);

                    ShouldRedraw::All
                } else {
                    ShouldRedraw::No
                }
            },
        }
    }

    fn handle_tool_button_released(&mut self, point: Vec2DScreen) -> Action {
        let transformed_point = self.transform.to_world_coordinates(point);

        match self.selected_tool {
            SelectedTool::Shape(_shape) => {
                let finished = if let Some(shape) = self.current_shape.as_mut() {
                    shape.handle_button_released(point, self.transform, self.config.point_snap_radius)
                } else {
                    ShapeFinished::No
                };

                if let ShapeFinished::Yes(shape) = finished {
                    let shape_id = self.storage.add(shape);

                    self.current_shape = None;

                    Action::Draw(shape_id)
                } else {
                    Action::Unfinished
                }
            },
            SelectedTool::Eraser => {
                let deleted_shape = self.storage
                    .query_circle(transformed_point, self.transform.to_world_units(self.erase_radius))
                    .map(|id| self.storage.remove(id))
                    .flatten();

                if let Some(shape) = deleted_shape {
                    self.erased.push(shape);
                }

                if !self.erased.is_empty() {
                    Action::Erase(self.erased.drain(..).collect())
                } else {
                    Action::Empty
                }
            },
        }
    }

    fn handle_tool_mouse_move(&mut self, point: Vec2DScreen) -> ShouldRedraw {
        let transformed_point = self.transform.to_world_coordinates(point);

        match self.selected_tool {
            SelectedTool::Shape(_shape_type) => {
                if let Some(shape) = self.current_shape.as_mut() {
                    shape.handle_mouse_moved(point, self.transform, self.config.point_snap_radius);
                    ShouldRedraw::Shape
                } else {
                    ShouldRedraw::No
                }
            },
            SelectedTool::Eraser => {
                let deleted_shape = self.storage
                    .query_circle(transformed_point, self.transform.to_world_units(self.erase_radius))
                    .map(|id| self.storage.remove(id))
                    .flatten();

                if let Some(shape) = deleted_shape {
                    self.erased.push(shape);
                    ShouldRedraw::All
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

                Action::Redraw(ids)
            },

            Action::Redraw(ids) => {
                let shapes = ids.into_iter().map(|id| self.storage.remove(id)).flatten().collect();

                Action::Erase(shapes)
            },
        }
    }

    /// Move the canvas by this offset
    fn translate(&mut self, delta: Vec2DScreen) {
        self.transform = self.transform.r#move(delta);
    }

    fn rotate(&mut self, angle: Angle, fixed: Vec2DScreen) {
        self.transform = self.transform.turn(angle, fixed);
    }

    /// Handles the canvas moving logic. Must be kept private
    fn handle_move(&mut self, old_point: Vec2DScreen, new_point: Vec2DScreen) {
        if self.is_shift_pressed {
            let center = self.dimensions / 2.0;

            let a = old_point - center;
            let b = new_point - center;

            self.rotate(Angle::between(a, b), center);
        } else {
            let delta = new_point - old_point;

            self.translate(delta);
        }
    }

    /// Compute the bounds of what is visible in the board according to current
    /// offset, zoom level and rotation. Return value is an array of the
    /// top-left, top-right, bottom-right, and bottom-left corners.
    fn visible_extent(&self) -> [Vec2DWorld; 4] {
        [
            self.transform.to_world_coordinates(Vec2DScreen::new(0.0, 0.0)),
            self.transform.to_world_coordinates(Vec2DScreen::new(self.dimensions.x, 0.0)),
            self.transform.to_world_coordinates(self.dimensions),
            self.transform.to_world_coordinates(Vec2DScreen::new(0.0, self.dimensions.y)),
        ]
    }

    /// return the screen-axis-aligned bounding box of the visible portion of
    /// the screen.
    fn visible_bbox(&self) -> [Vec2DWorld; 2] {
        bbox_from_points(self.visible_extent().iter().copied())
    }
}

// Public interface
impl Pizarra {
    /// Create a new instance of Pizarra's controller. Only one is needed and it
    /// can be put inside an Rc<RefCell<_>> to use it inside event callbacks in
    /// UI implementations.
    pub fn new(dimensions: Vec2DScreen, config: Config) -> Pizarra {
        Pizarra {
            config,

            storage: Storage::new(),
            current_shape: None,

            selected_tool: SelectedTool::Shape(ShapeTool::Path),
            bgcolor: Color::black(),
            stroke: config.thickness,
            selected_color: config.stroke_color,
            erase_radius: config.erase_radius,

            /// a vector from the top-left corner of the screen to the origin
            /// of coordinates in the storage
            transform: Transform::default_for_viewport(dimensions),
            dimensions,

            erased: Vec::new(),

            board_state: BoardState::Idle,
            is_shift_pressed: false,

            undo_state: UndoState::Reset,
            undo_queue: Vec::new(),

            save_status: SaveStatus::NewAndEmpty,
        }
    }

    #[cfg(test)]
    pub fn new_for_testing() -> Pizarra {
        Pizarra::new(Vec2DScreen::new(80.0, 60.0), Default::default())
    }

    /// Returns a transform object needed for proper rendering. This object can
    /// translate a point from storage coordinates to screen coordinates.
    pub fn get_transform(&self) -> Transform {
        self.transform
    }

    /// Returns the current config
    pub fn config(&self) -> Config {
        self.config
    }

    /// Returns the dimensions of the screen
    pub fn get_dimensions(&self) -> Vec2DScreen {
        self.dimensions
    }

    /// Returns current background color
    pub fn bgcolor(&self) -> Color {
        self.bgcolor
    }

    pub fn selected_color(&self) -> Color {
        self.selected_color
    }

    /// Returns an iterator over the draw commands required to render the visible
    /// portion of the drawing
    pub fn draw_commands_for_screen(&self) -> Vec<DrawCommand> {
        self.storage.draw_commands(self.visible_bbox())
    }

    /// Returns an interator over the draw commands required to render the wole
    /// drawing. Used for exporting to png
    pub fn draw_commands_for_drawing(&self) -> Vec<DrawCommand> {
        let bbox = if let Some(bbox) = self.get_bounds() {
            bbox
        } else {
            [Vec2DWorld::new(0.0, 0.0), Vec2DWorld::new(0.0, 0.0)]
        };

        self.storage.draw_commands(bbox)
    }

    /// returns only the draw commands needed to paint the current shape on the
    /// screen.
    pub fn draw_commands_for_current_shape(&self) -> Option<Vec<DrawCommand>> {
        self.current_shape.as_ref().map(|s| s.draw_commands())
    }

    /// Notify the application that the screen size has changed
    pub fn resize(&mut self, new_size: Vec2DScreen) {
        log::debug!("resize({:?})", new_size);

        let delta = (self.dimensions * -1.0 + new_size) * 0.5;

        self.translate(delta);
        self.dimensions = new_size;
    }

    /// Set a new tool used to handle user input
    pub fn set_tool(&mut self, selected_tool: SelectedTool) {
        log::debug!("set_tool({:?})", selected_tool);

        self.selected_tool = selected_tool;
    }

    /// Set a new stroke that will be used in next shapes
    pub fn set_stroke(&mut self, stroke: f64) {
        log::debug!("set_stroke({})", stroke);

        self.stroke = stroke;
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        log::debug!("set_alpha({})", alpha);

        self.selected_color = self.selected_color.with_alpha(alpha);
    }

    /// Set the background color of the drawing
    pub fn set_bgcolor(&mut self, bgcolor: Color) {
        log::debug!("set_bgcolor({})", bgcolor);

        self.bgcolor = bgcolor;
    }

    /// Set the radius of action for the eraser tool
    pub fn set_erase_radius(&mut self, erase_radius: f64) {
        log::debug!("set_erase_radius({})", erase_radius);

        self.erase_radius = erase_radius;
    }

    /// Public method that the UI must call whenever a button of the mouse or
    /// pen is pressed.
    pub fn handle_mouse_button_pressed(&mut self, button: MouseButton, point: Vec2DScreen) -> ShouldRedraw {
        log::debug!("handle_mouse_button_pressed({:?}, {:?})", button, point);

        let (new_board_state, should_redraw) = match self.board_state {
            current@BoardState::Idle => match button {
                MouseButton::Left => (BoardState::UsingTool, self.handle_tool_button_pressed(point)),
                MouseButton::Middle => (BoardState::Moving(point), ShouldRedraw::No),
                MouseButton::Right => (current, ShouldRedraw::No),
                MouseButton::Unknown => (current, ShouldRedraw::No),
            },
            current@BoardState::UsingTool => match button {
                MouseButton::Left => (current, self.handle_tool_button_pressed(point)),
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
    pub fn handle_mouse_button_released(&mut self, button: MouseButton, point: Vec2DScreen) -> ShouldRedraw {
        log::debug!("handle_mouse_button_released({:?}, {:?})", button, point);

        let (new_board_state, should_redraw) = match self.board_state {
            current@BoardState::Idle => (current, ShouldRedraw::No),
            current@BoardState::UsingTool => match button {
                MouseButton::Left => match self.handle_tool_button_released(point) {
                    Action::Unfinished => (current, ShouldRedraw::Shape),
                    Action::Empty => (BoardState::Idle, ShouldRedraw::No),
                    x => {
                        self.conclude_action(x);
                        (BoardState::Idle, ShouldRedraw::All)
                    }
                },
                MouseButton::Middle => (current, ShouldRedraw::No),
                MouseButton::Right => (current, ShouldRedraw::No),
                MouseButton::Unknown => (current, ShouldRedraw::No),
            },
            BoardState::Moving(old_point) => match button {
                MouseButton::Left => (BoardState::Moving(old_point), ShouldRedraw::No),
                MouseButton::Middle => {
                    self.handle_move(old_point, point);

                    (BoardState::Idle, ShouldRedraw::All)
                },
                MouseButton::Right => (BoardState::Moving(old_point), ShouldRedraw::No),
                MouseButton::Unknown => (BoardState::Moving(old_point), ShouldRedraw::No),
            },
            // TODO check if https://github.com/rust-lang/rust/issues/65490 is resolved
            BoardState::MovingWhileUsingTool(old_point) => match button {
                MouseButton::Left => match self.handle_tool_button_released(point) {
                        Action::Unfinished => (BoardState::MovingWhileUsingTool(old_point), ShouldRedraw::All),
                        Action::Empty => (BoardState::Moving(point), ShouldRedraw::All),
                        x => {
                            self.conclude_action(x);

                            (BoardState::Moving(point), ShouldRedraw::All)
                        }
                },
                MouseButton::Middle => {
                    self.handle_move(old_point, point);

                    (BoardState::UsingTool, ShouldRedraw::All)
                },
                MouseButton::Right => (BoardState::MovingWhileUsingTool(old_point), ShouldRedraw::No),
                MouseButton::Unknown => (BoardState::MovingWhileUsingTool(old_point), ShouldRedraw::No),
            },
        };

        self.board_state = new_board_state;

        should_redraw
    }

    /// Public methid that the UI must call with updates on the cursor position
    pub fn handle_mouse_move(&mut self, pos: Vec2DScreen) -> ShouldRedraw {
        log::debug!("handle_mouse_move({:?})", pos);

        let (new_board_state, should_redraw) = match self.board_state {
            BoardState::Idle => (BoardState::Idle, ShouldRedraw::No),
            BoardState::UsingTool => (BoardState::UsingTool, self.handle_tool_mouse_move(pos)),
            BoardState::Moving(old_point) => {
                self.handle_move(old_point, pos);

                (BoardState::Moving(pos), ShouldRedraw::All)
            },
            BoardState::MovingWhileUsingTool(old_point) => {
                self.handle_move(old_point, pos);

                (BoardState::MovingWhileUsingTool(pos), ShouldRedraw::All)
            },
        };

        self.board_state = new_board_state;

        should_redraw
    }

    pub fn handle_key_pressed(&mut self, key: Key) {
        log::debug!("handle_key_pressed({:?})", key);

        match key {
            Key::Shift => {
               self.is_shift_pressed = true;
            }
            Key::Escape => {
            }
            Key::Unknown => {}
        }
    }

    pub fn handle_key_released(&mut self, key: Key) -> ShouldRedraw {
        log::debug!("handle_key_released({:?})", key);

        match key {
            Key::Shift => {
                self.is_shift_pressed = false;

                ShouldRedraw::No
            }
            Key::Escape => {
                if self.current_shape.is_some() {
                    self.current_shape = None;
                    self.board_state = BoardState::Idle;
                }

                ShouldRedraw::All
            }
            Key::Unknown => {
                ShouldRedraw::No
            }
        }
    }

    /// Zooms in the current view, so objects are bigger and the visible portion
    /// of the drawing gets reduced
    pub fn zoom_in(&mut self) {
        log::debug!("zoom_in()");
        self.transform = self.transform.zoom(2.0, self.dimensions / 2.0);
    }

    /// Zooms out the current view, so objects are smaller and the visible portion
    /// of the drawing increases
    pub fn zoom_out(&mut self) {
        log::debug!("zoom_out()");
        self.transform = self.transform.zoom(0.5, self.dimensions / 2.0);
    }

    pub fn scroll_up(&mut self) {
        log::debug!("scroll_up()");
        if self.is_shift_pressed {
            self.rotate(Angle::from_degrees(10.0), self.dimensions / 2.0);
        } else {
            self.translate(Vec2DScreen::new(0.0, 10.0));
        }
    }

    pub fn scroll_down(&mut self) {
        log::debug!("scroll_down()");
        if self.is_shift_pressed {
            self.rotate(Angle::from_degrees(-10.0), self.dimensions / 2.0);
        } else {
            self.translate(Vec2DScreen::new(0.0, -10.0));
        }
    }

    pub fn scroll_right(&mut self) {
        log::debug!("scroll_right()");
        self.translate(Vec2DScreen::new(-10.0, 0.0));
    }

    pub fn scroll_left(&mut self) {
        log::debug!("scroll_left()");
        self.translate(Vec2DScreen::new(10.0, 0.0));
    }

    /// Resets the center and zoom level of the view to its original place
    pub fn go_home(&mut self) {
        log::debug!("go_home()");
        self.transform = Transform::default_for_viewport(self.dimensions);
    }

    /// Sets the color for the new shapes
    pub fn set_color(&mut self, color: Color) {
        log::debug!("set_color({})", color);
        self.selected_color = color;
    }

    /// Undoes the last action if any
    pub fn undo(&mut self) {
        log::debug!("undo()");

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
        log::debug!("redo()");

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

    /// Returns the bounding box that contains all the shapes in the drawing
    pub fn get_bounds(&self) -> Option<[Vec2DWorld; 2]> {
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
        log::debug!("reset()");
        // TODO reconsider this, I suspect its implementation tends to be broken
        self.storage = Storage::new();
        self.selected_tool = SelectedTool::Shape(ShapeTool::Path);
        self.transform = Transform::default_for_viewport(self.dimensions);
        self.undo_state = UndoState::Reset;
        self.undo_queue = Vec::new();
        self.save_status = SaveStatus::NewAndEmpty;
    }

    pub fn open(&mut self, svg: &str) -> Result<(), deserialize::Error> {
        self.reset();
        self.storage = Storage::from_svg(svg, self.config)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        draw_commands::DrawCommand, app::SelectedTool,
        shape::path::{PathCommand, CubicBezierCurve},
    };
    use PathCommand::*;

    #[test]
    fn draw_a_line() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 0.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(2.0, 0.0));

        assert_eq!(app.storage.shape_count(), 1);

        let commands = app.draw_commands_for_screen();

        assert_eq!(commands[0], DrawCommand::Path {
            color: Default::default(),
            commands: vec![
                MoveTo(Vec2DWorld { x: -40.0, y: -30.0 }),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2DWorld { x: -39.666666666666664, y: -30.0 },
                    pt2: Vec2DWorld { x: -39.333333333333336, y: -30.0 },
                    to: Vec2DWorld { x: -39.0, y: -30.0 }
                }),
            ],
            thickness: Config::default().thickness,
        });
    }

    #[test]
    fn draw_a_line_in_zoom() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        app.zoom_in();

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 0.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(2.0, 0.0));

        assert_eq!(app.storage.shape_count(), 1);

        let commands = app.draw_commands_for_screen();

        assert_eq!(commands[0], DrawCommand::Path {
            color: Default::default(),
            commands: vec![
                MoveTo(Vec2DWorld { x: -20.0, y: -15.0 }),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2DWorld { x: -19.833333333333332, y: -15.0 },
                    pt2: Vec2DWorld { x: -19.666666666666668, y: -15.0 },
                    to: Vec2DWorld { x: -19.5, y: -15.0 }
                }),
            ],
            thickness: Config::default().thickness / 2.0,
        });
    }

    #[test]
    fn erase() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(10.0, 10.0));
        app.handle_mouse_move(Vec2DScreen::new(15.0, 10.0));
        app.handle_mouse_move(Vec2DScreen::new(15.0, 15.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(10.0, 15.0));

        assert_eq!(app.storage.shape_count(), 1);

        app.set_tool(SelectedTool::Eraser);

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(10.0, 10.0));
        app.handle_mouse_move(Vec2DScreen::new(15.0, 10.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(15.0, 10.0));

        assert_eq!(app.storage.shape_count(), 0);
    }

    #[test]
    fn erase_with_zoom() {
        let mut app = Pizarra::new_for_testing();

        app.resize(Vec2DScreen::new(4.0, 4.0));

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(2.0, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(2.0, 1.5));
        app.handle_mouse_move(Vec2DScreen::new(2.0, 2.0));
        app.handle_mouse_move(Vec2DScreen::new(2.0, 2.5));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(2.0, 3.0));

        assert_eq!(app.storage.shape_count(), 1);

        app.zoom_in();

        app.set_tool(SelectedTool::Eraser);
        app.set_erase_radius(0.5);

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(2.6, 0.0));
        app.handle_mouse_move(Vec2DScreen::new(2.6, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(2.6, 3.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(2.6, 4.0));

        assert_eq!(app.storage.shape_count(), 1);

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(2.24, 0.0));
        app.handle_mouse_move(Vec2DScreen::new(2.4, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(2.4, 3.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(2.24, 4.0));

        assert_eq!(app.storage.shape_count(), 0);
    }

    #[test]
    fn zoom_out_is_plugged() {
        let mut app = Pizarra::new_for_testing();

        app.zoom_out();

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-80.0, -60.0), Vec2DWorld::new(80.0, 60.0)]);
    }

    #[test]
    fn do_do_do_undo_redo_redo_undo() {
        let mut app = Pizarra::new_for_testing();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));

        app.undo();

        app.redo();
        app.redo();

        app.undo();

        assert_eq!(app.storage.shape_count(), 2);
    }

    #[test]
    fn do_undo_redo_undo() {
        let mut app = Pizarra::new_for_testing();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(1.0, 1.0));

        app.undo();

        app.redo();

        app.undo();

        assert_eq!(app.storage.shape_count(), 0);
    }

    #[test]
    fn do_undo_do_do_undo() {
        let mut app = Pizarra::new_for_testing();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(1.0, 1.0));

        app.undo();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(1.0, 1.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(1.0, 1.0));

        app.undo();

        assert_eq!(app.storage.shape_count(), 1);
    }

    #[test]
    fn do_undo_do_undo_undo() {
        let mut app = Pizarra::new_for_testing();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(1.0, 1.0));

        app.undo();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 1.0));
        app.handle_mouse_move(Vec2DScreen::new(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(1.0, 1.0));

        app.undo();
        app.undo();

        assert_eq!(app.storage.shape_count(), 0);
    }

    #[test]
    fn r#move() {
        let mut app = Pizarra::new_for_testing();

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-40.0, -30.0), Vec2DWorld::new(40.0, 30.0)]);

        app.handle_mouse_button_pressed(MouseButton::Middle, Vec2DScreen::new(0.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Middle, Vec2DScreen::new(-1.0, 0.0));

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-39.0, -30.0), Vec2DWorld::new(41.0, 30.0)]);
    }

    #[test]
    fn move_while_in_zoom_is_coherent() {
        let mut app = Pizarra::new_for_testing();

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-40.0, -30.0), Vec2DWorld::new(40.0, 30.0)]);

        app.zoom_in();

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-20.0, -15.0), Vec2DWorld::new(20.0, 15.0)]);

        app.handle_mouse_button_pressed(MouseButton::Middle, Vec2DScreen::new(0.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Middle, Vec2DScreen::new(-1.0, 0.0));

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-19.5, -15.0), Vec2DWorld::new(20.5, 15.0)]);
    }

    #[test]
    fn move_while_drawing() {
        let mut app = Pizarra::new_for_testing();

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-40.0, -30.0), Vec2DWorld::new(40.0, 30.0)]);

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(10.0, 10.0));
        app.handle_mouse_move(Vec2DScreen::new(20.0, 10.0));
        app.handle_mouse_button_pressed(MouseButton::Middle, Vec2DScreen::new(20.0, 10.0));
        app.handle_mouse_move(Vec2DScreen::new(30.0, 10.0));
        app.handle_mouse_button_released(MouseButton::Middle, Vec2DScreen::new(30.0, 10.0));
        app.handle_mouse_move(Vec2DScreen::new(40.0, 10.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(40.0, 10.0));

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-50.0, -30.0), Vec2DWorld::new(30.0, 30.0)]);
        assert_eq!(app.storage.shape_count(), 1);

        let commands = app.draw_commands_for_screen();

        assert_eq!(commands.len(), 1);

        assert_eq!(commands[0], DrawCommand::Path {
            color: Default::default(),
            thickness: Config::default().thickness,
            commands: vec![
                MoveTo(Vec2DWorld { x: -30.0, y: -20.0 }),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2DWorld { x: -26.666666666666668, y: -20.0 },
                    pt2: Vec2DWorld { x: -23.333333333333332, y: -20.0 },
                    to: Vec2DWorld { x: -20.0, y: -20.0 }
                }),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2DWorld { x: -16.666666666666668, y: -20.0 },
                    pt2: Vec2DWorld { x: -13.333333333333332, y: -20.0 },
                    to: Vec2DWorld { x: -10.0, y: -20.0 }
                }),
            ],
        });
    }

    #[test]
    fn zoom_must_be_idempotent() {
        let mut app = Pizarra::new_for_testing();

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-40.0, -30.0), Vec2DWorld::new(40.0, 30.0)]);

        app.zoom_in();

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-20.0, -15.0), Vec2DWorld::new(20.0, 15.0)]);

        app.zoom_in();

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-10.0, -7.5), Vec2DWorld::new(10.0, 7.5)]);
    }

    #[test]
    fn cache_invalidation_on_ctrl_z() {
        let mut app = Pizarra::new_for_testing();

        app.resize(Vec2DScreen::new(10.0, 10.0));

        app.set_tool(SelectedTool::Shape(ShapeTool::Rectangle));

        // draw a shape
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(5.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(6.0, 6.0));

        assert_eq!(app.draw_commands_for_screen().len(), 1);

        // move the screen
        app.handle_mouse_button_pressed(MouseButton::Middle, Vec2DScreen::new(10.0, 5.0));
        app.handle_mouse_move(Vec2DScreen::new(5.0, 5.0));
        app.handle_mouse_move(Vec2DScreen::new(0.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Middle, Vec2DScreen::new(0.0, 5.0));

        assert_eq!(app.draw_commands_for_screen().len(), 0);

        // draw another shape
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(5.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(6.0, 6.0));

        assert_eq!(app.draw_commands_for_screen().len(), 1);

        // and draw a third shape
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(6.0, 6.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(7.0, 7.0));

        assert_eq!(app.draw_commands_for_screen().len(), 2);

        // undo that last one
        app.undo();

        assert_eq!(app.draw_commands_for_screen().len(), 1);

        // move back to where the first shape was
        app.handle_mouse_button_pressed(MouseButton::Middle, Vec2DScreen::new(0.0, 5.0));
        app.handle_mouse_move(Vec2DScreen::new(5.0, 5.0));
        app.handle_mouse_move(Vec2DScreen::new(10.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Middle, Vec2DScreen::new(10.0, 5.0));

        assert_eq!(app.draw_commands_for_screen().len(), 1);
    }

    #[test]
    fn not_erasing_a_shape_doesnt_leave_the_eraser_active() {
        // the scenario is: you draw a few shapes, then change to the eraser, click and drag
        // on the screen but without touching a shape and then release the left button.
        //
        // what happens is the eraser remains active and when you move and touch a shape
        // it gets erased
        //
        // it's expeceted for the eraser not to erase the shape
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Rectangle));

        // draw a shape
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(5.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(6.0, 6.0));

        app.set_tool(SelectedTool::Eraser);

        // erase nothing
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(90.0, 90.0));
        app.handle_mouse_move(Vec2DScreen::new(91.0, 91.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(92.0, 92.0));

        // move to where the shape is
        app.handle_mouse_move(Vec2DScreen::new(5.0, 5.0));

        assert_eq!(app.storage.shape_count(), 1);
    }

    #[test]
    fn app_state_after_finished_shape() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        let p1 = Vec2DScreen::new(1.0, 1.0);
        let p2 = Vec2DScreen::new(20.0, 20.0);

        app.handle_mouse_button_pressed(MouseButton::Left, p1);
        app.handle_mouse_move(p2);
        app.handle_mouse_button_released(MouseButton::Left, p2);

        assert_eq!(app.board_state, BoardState::Idle);
        assert_eq!(app.undo_state, UndoState::InSync);
        assert_eq!(app.save_status, SaveStatus::NewAndChanged);
        assert_eq!(app.storage.shape_count(), 1);

        if let Some(_) = app.current_shape.as_ref() {
            panic!()
        }
    }

    #[test]
    fn resize_screen_keeps_center() {
        let mut app = Pizarra::new_for_testing();

        app.resize(Vec2DScreen::new(100.0, 100.0));

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-50.0, -50.0), Vec2DWorld::new(50.0, 50.0)]);

        app.resize(Vec2DScreen::new(200.0, 300.0));

        assert_eq!(app.visible_bbox(), [Vec2DWorld::new(-100.0, -150.0), Vec2DWorld::new(100.0, 150.0)]);
    }

    #[test]
    fn esc_clears_current_shape() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Polygon));
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(0.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(0.0, 0.0));

        assert_eq!(app.board_state, BoardState::UsingTool);
        assert_eq!(app.undo_state, UndoState::Reset);

        app.handle_key_pressed(Key::Escape);
        assert_eq!(app.handle_key_released(Key::Escape), ShouldRedraw::All);

        assert_eq!(app.board_state, BoardState::Idle);
        assert!(app.current_shape.is_none());
        assert_eq!(app.undo_state, UndoState::Reset);
        assert_eq!(app.save_status, SaveStatus::NewAndEmpty);
    }
}
