use std::mem;
use std::path::PathBuf;

use crate::draw_commands::DrawCommand;
use crate::storage::Storage;
use crate::shape::{Shape, ShapeTool, ShapeFinished, ShapeBuilder};
use crate::color::Color;
use crate::action::Action;
use crate::point::{Vec2D, Unit, WorldUnit, ScreenUnit};
use crate::transform::Transform;
use crate::geom::{Angle, bbox_from_points};
use crate::key::Key;
use crate::config::Config;
use crate::ui::Flags;
use crate::style::{Style, Stroke};
use crate::shape::ShapeStored;

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
    Moving(Vec2D<ScreenUnit>),
    MovingWhileUsingTool(Vec2D<ScreenUnit>),
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

#[derive(Debug, Copy, Clone)]
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
#[derive(Debug)]
pub struct Pizarra {
    /// The settings that where loaded at app start
    config: Config,

    /// Finished shapes
    storage: Storage,

    /// What is happening with the board right now?
    board_state: BoardState,
    selected_tool: SelectedTool,
    current_shape: Option<Box<dyn ShapeBuilder>>,
    last_mouse_pos: Vec2D<ScreenUnit>,
    eraser_is_active: bool,

    /// Some paremeters used during drawing
    selected_color: Color,
    bgcolor: Color,
    stroke: ScreenUnit,
    erase_radius: ScreenUnit,

    /// The transformation matrix that takes points from the drawing coordinate
    /// system to the screen coordinate system.
    transform: Transform,
    /// What is the  size of the canvas in pixels?
    dimensions: Vec2D<ScreenUnit>,

    /// Stores erased shapes temporarily while being erased
    erased: Vec<Shape>,

    /// Undo
    undo_state: UndoState,
    undo_queue: Vec<Action>,

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

    fn handle_tool_button_pressed(&mut self, point: Vec2D<ScreenUnit>, tool_override: Option<SelectedTool>) -> ShouldRedraw {
        let transformed_point = self.transform.to_world_coordinates(point);

        match tool_override.unwrap_or(self.selected_tool) {
            SelectedTool::Shape(shape) => {
                if let Some(shape) = self.current_shape.as_mut() {
                    shape.handle_button_pressed(point, self.transform, self.config.point_snap_radius);
                } else {
                    let new_shape = shape.start(transformed_point, Style {
                        stroke: Some(Stroke {
                            size: self.transform.to_world_units(self.stroke),
                            color: self.selected_color,
                        }),
                        fill: None,
                    });

                    self.current_shape = Some(new_shape);
                }

                ShouldRedraw::Shape
            },
            SelectedTool::Eraser => {
                let deleted_shapes = self.storage
                    .remove_circle(transformed_point, self.transform.to_world_units(self.erase_radius));

                self.eraser_is_active = true;
                let prev_len = self.erased.len();
                self.erased.extend(deleted_shapes);

                if self.erased.len() > prev_len {
                    ShouldRedraw::All
                } else {
                    ShouldRedraw::No
                }
            },
        }
    }


    fn handle_tool_button_released(&mut self, point: Vec2D<ScreenUnit>, tool_override: Option<SelectedTool>) -> Action {
        let transformed_point = self.transform.to_world_coordinates(point);

        match tool_override.unwrap_or(self.selected_tool) {
            SelectedTool::Shape(_shape) => {
                let finished = if let Some(shape) = self.current_shape.as_mut() {
                    shape.handle_button_released(point, self.transform, self.config.point_snap_radius)
                } else {
                    ShapeFinished::No
                };

                match finished {
                    ShapeFinished::Yes(shapes) => {
                        let shape_ids = shapes.into_iter().map(|shape| {
                            self.storage.add(shape)
                        }).collect();

                        self.current_shape = None;

                        Action::Draw(shape_ids)
                    },
                    ShapeFinished::Cancelled => {
                        self.current_shape = None;

                        Action::Empty
                    },
                    ShapeFinished::No => Action::Unfinished,
                }
            },
            SelectedTool::Eraser => {
                let deleted_shapes = self.storage
                    .remove_circle(transformed_point, self.transform.to_world_units(self.erase_radius));

                self.eraser_is_active = false;

                self.erased.extend(deleted_shapes);

                if !self.erased.is_empty() {
                    Action::Erase(self.erased.drain(..).collect())
                } else {
                    Action::Empty
                }
            },
        }
    }

    fn handle_tool_mouse_move(&mut self, point: Vec2D<ScreenUnit>, tool_override: Option<SelectedTool>) -> ShouldRedraw {
        let transformed_point = self.transform.to_world_coordinates(point);

        match tool_override.unwrap_or(self.selected_tool) {
            SelectedTool::Shape(_shape_type) => {
                if let Some(shape) = self.current_shape.as_mut() {
                    shape.handle_mouse_moved(point, self.transform, self.config.point_snap_radius);
                    ShouldRedraw::Shape
                } else {
                    ShouldRedraw::No
                }
            },
            SelectedTool::Eraser => {
                let deleted_shapes = self.storage
                    .remove_circle(transformed_point, self.transform.to_world_units(self.erase_radius));

                let prev_len = self.erased.len();

                self.erased.extend(deleted_shapes);

                if self.erased.len() > prev_len {
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

            Action::Draw(ids) => Action::Erase(ids.into_iter().filter_map(|id| {
                self.storage.remove(id)
            }).collect()),

            Action::Erase(shapes) => {
                let ids = shapes.into_iter().map(|shape| self.storage.restore(shape)).collect();

                Action::Draw(ids)
            },
        }
    }

    /// Move the canvas by this offset
    fn translate(&mut self, delta: Vec2D<ScreenUnit>) {
        self.transform = self.transform.r#move(delta);
    }

    fn rotate(&mut self, angle: Angle, fixed: Vec2D<ScreenUnit>) {
        self.transform = self.transform.turn(angle, fixed);
    }

    /// Handles the canvas moving logic. Must be kept private
    fn handle_move(&mut self, old: Vec2D<ScreenUnit>, new: Vec2D<ScreenUnit>, flags: Flags) {
        if flags.shift {
            let center = self.dimensions / 2.0;

            let a = old - center;
            let b = new - center;

            self.rotate(Angle::between(a, b), center);
        } else {
            let delta = new - old;

            self.translate(delta);
        }
    }

    /// Compute the bounds of what is visible in the board according to current
    /// offset, zoom level and rotation. Return value is an array of the
    /// top-left, top-right, bottom-right, and bottom-left corners.
    fn visible_extent(&self) -> [Vec2D<WorldUnit>; 4] {
        [
            self.transform.to_world_coordinates(Vec2D::new_screen(0.0, 0.0)),
            self.transform.to_world_coordinates(Vec2D::new(self.dimensions.x, 0.0.into())),
            self.transform.to_world_coordinates(self.dimensions),
            self.transform.to_world_coordinates(Vec2D::new(0.0.into(), self.dimensions.y)),
        ]
    }

    /// return the screen-axis-aligned bounding box of the visible portion of
    /// the screen.
    fn visible_bbox(&self) -> [Vec2D<WorldUnit>; 2] {
        bbox_from_points(self.visible_extent().iter().copied())
    }
}

// Public interface
impl Pizarra {
    /// Create a new instance of Pizarra's controller. Only one is needed and it
    /// can be put inside an Rc<RefCell<_>> to use it inside event callbacks in
    /// UI implementations.
    pub fn new(dimensions: Vec2D<ScreenUnit>, config: Config) -> Pizarra {
        Pizarra {
            config,

            storage: Storage::new(),
            current_shape: None,

            selected_tool: SelectedTool::Shape(ShapeTool::Path),
            last_mouse_pos: dimensions / 2.0,
            eraser_is_active: false,
            bgcolor: config.background_color,
            stroke: config.thickness,
            selected_color: config.stroke_color,
            erase_radius: config.erase_radius,

            /// a vector from the top-left corner of the screen to the origin
            /// of coordinates in the storage
            transform: Transform::default_for_viewport(dimensions),
            dimensions,

            erased: Vec::new(),

            board_state: BoardState::Idle,

            undo_state: UndoState::Reset,
            undo_queue: Vec::new(),

            save_status: SaveStatus::NewAndEmpty,
        }
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
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
    pub fn get_dimensions(&self) -> Vec2D<ScreenUnit> {
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
            [Vec2D::new_world(0.0, 0.0), Vec2D::new_world(0.0, 0.0)]
        };

        self.storage.draw_commands(bbox)
    }

    pub fn draw_commands_for_tool(&self) -> Vec<DrawCommand> {
        if let SelectedTool::Eraser = self.selected_tool {
            vec![
                DrawCommand::ScreenCircle {
                    center: self.last_mouse_pos,
                    radius: self.erase_radius,
                    style: Style {
                        stroke: Some(Stroke {
                            size: 1.0.into(),
                            color: if self.eraser_is_active {
                                Color::red()
                            } else {
                                Color::gray()
                            },
                        }),
                        fill: if self.eraser_is_active {
                            Some(Color::red().half_transparent())
                        } else {
                            None
                        },
                    },
                }
            ]
        } else {
            vec![]
        }
    }

    /// returns only the draw commands needed to paint the current shape on the
    /// screen.
    pub fn draw_commands_for_current_shape(&self) -> Option<Vec<DrawCommand>> {
        self.current_shape.as_ref().map(|s| s.draw_commands(self.transform, self.config.point_snap_radius))
    }

    /// Returns all the shapes of the storage ordered by their z-index. Used for
    /// serializing
    pub fn shapes_by_index(&self) -> impl Iterator<Item=&dyn ShapeStored> {
        self.storage.shapes_by_index()
    }

    /// Notify the application that the screen size has changed
    pub fn resize(&mut self, new_size: Vec2D<ScreenUnit>) {
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
    pub fn set_stroke(&mut self, stroke: ScreenUnit) {
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
    pub fn set_erase_radius(&mut self, erase_radius: ScreenUnit) {
        log::debug!("set_erase_radius({})", erase_radius);

        self.erase_radius = erase_radius;
    }

    /// Public method that the UI must call whenever a button of the mouse or
    /// pen is pressed.
    pub fn handle_mouse_button_pressed_flags(&mut self, button: MouseButton, point: Vec2D<ScreenUnit>, tool_override: Option<SelectedTool>) -> ShouldRedraw {
        log::debug!("handle_mouse_button_pressed_flags({:?}, {:?}, {:?})", button, point, tool_override);

        let (new_board_state, should_redraw) = match self.board_state {
            current@BoardState::Idle => match button {
                MouseButton::Left => (BoardState::UsingTool, self.handle_tool_button_pressed(point, tool_override)),
                MouseButton::Middle => (BoardState::Moving(point), ShouldRedraw::No),
                MouseButton::Right => (current, ShouldRedraw::No),
                MouseButton::Unknown => (current, ShouldRedraw::No),
            },
            current@BoardState::UsingTool => match button {
                MouseButton::Left => (current, self.handle_tool_button_pressed(point, tool_override)),
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

    pub fn handle_mouse_button_pressed(&mut self, button: MouseButton, point: Vec2D<ScreenUnit>) -> ShouldRedraw {
        self.handle_mouse_button_pressed_flags(button, point, None)
    }

    /// Public method that the UI must call whenever a button of the mouse or pen
    /// is released.
    pub fn handle_mouse_button_released_flags(&mut self, button: MouseButton, point: Vec2D<ScreenUnit>, flags: Flags, tool_override: Option<SelectedTool>) -> ShouldRedraw {
        log::debug!("handle_mouse_button_released_flags({:?}, {:?}, {:?}, {:?})", button, point, flags, tool_override);

        let (new_board_state, should_redraw) = match self.board_state {
            current@BoardState::Idle => (current, ShouldRedraw::No),
            current@BoardState::UsingTool => match button {
                MouseButton::Left => match self.handle_tool_button_released(point, tool_override) {
                    Action::Unfinished => (current, ShouldRedraw::Shape),
                    Action::Empty => (BoardState::Idle, ShouldRedraw::Shape),
                    x => {
                        self.conclude_action(x);
                        (BoardState::Idle, ShouldRedraw::All)
                    }
                },
                MouseButton::Middle => (current, ShouldRedraw::No),
                MouseButton::Right => (current, ShouldRedraw::No),
                MouseButton::Unknown => (current, ShouldRedraw::No),
            },
            current@BoardState::Moving(old_point) => match button {
                MouseButton::Left => (current, ShouldRedraw::No),
                MouseButton::Middle => {
                    self.handle_move(old_point, point, flags);

                    (BoardState::Idle, ShouldRedraw::All)
                },
                MouseButton::Right => (current, ShouldRedraw::No),
                MouseButton::Unknown => (current, ShouldRedraw::No),
            },
            current@BoardState::MovingWhileUsingTool(old_point) => match button {
                MouseButton::Left => match self.handle_tool_button_released(point, tool_override) {
                        Action::Unfinished => (current, ShouldRedraw::All),
                        Action::Empty => (BoardState::Moving(point), ShouldRedraw::All),
                        x => {
                            self.conclude_action(x);

                            (BoardState::Moving(point), ShouldRedraw::All)
                        }
                },
                MouseButton::Middle => {
                    self.handle_move(old_point, point, flags);

                    (BoardState::UsingTool, ShouldRedraw::All)
                },
                MouseButton::Right => (current, ShouldRedraw::No),
                MouseButton::Unknown => (current, ShouldRedraw::No),
            },
        };

        self.board_state = new_board_state;

        should_redraw
    }

    pub fn handle_mouse_button_released(&mut self, button: MouseButton, point: Vec2D<ScreenUnit>) -> ShouldRedraw {
        self.handle_mouse_button_released_flags(button, point, Default::default(), None)
    }

    /// Public methid that the UI must call with updates on the cursor position
    pub fn handle_mouse_move_flags(&mut self, pos: Vec2D<ScreenUnit>, flags: Flags, tool_override: Option<SelectedTool>) -> ShouldRedraw {
        log::debug!("handle_mouse_move_flags({:?}, {:?}, {:?})", pos, flags, tool_override);

        let (new_board_state, should_redraw) = match self.board_state {
            BoardState::Idle => (BoardState::Idle, ShouldRedraw::No),
            BoardState::UsingTool => (BoardState::UsingTool, self.handle_tool_mouse_move(pos, tool_override)),
            BoardState::Moving(old_point) => {
                self.handle_move(old_point, pos, flags);

                (BoardState::Moving(pos), ShouldRedraw::All)
            },
            BoardState::MovingWhileUsingTool(old_point) => {
                self.handle_move(old_point, pos, flags);

                (BoardState::MovingWhileUsingTool(pos), ShouldRedraw::All)
            },
        };

        self.board_state = new_board_state;
        self.last_mouse_pos = pos;

        should_redraw
    }

    pub fn handle_mouse_move(&mut self, pos: Vec2D<ScreenUnit>) -> ShouldRedraw {
        self.handle_mouse_move_flags(pos, Default::default(), None)
    }

    pub fn handle_key_pressed(&mut self, key: Key) {
        log::debug!("handle_key_pressed({:?})", key);
    }

    pub fn handle_key_released(&mut self, key: Key) -> ShouldRedraw {
        log::debug!("handle_key_released({:?})", key);

        match key {
            Key::Escape => {
                self.current_shape = None;
                self.board_state = BoardState::Idle;

                ShouldRedraw::All
            }
            _ => {
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

    pub fn scroll(&mut self, delta: Vec2D<ScreenUnit>, flags: Flags) {
        log::debug!("scroll({})", delta);
        if flags.shift {
            self.rotate(Angle::from_degrees((delta.y * self.config.scroll_factor.into()).val()), self.dimensions / 2.0);
        } else {
            self.translate(delta * self.config.scroll_factor);
        }
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
    pub fn get_bounds(&self) -> Option<[Vec2D<WorldUnit>; 2]> {
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
}

/// Crate-private interface
impl Pizarra {
    pub(crate) fn set_storage(&mut self, storage: Storage) {
        self.storage = storage;
    }
}

// For testing purposes only
#[cfg(test)]
impl Pizarra {
    /// Only for tests. Creates a pizarra instance with a 80x60 window size and
    /// default config
    pub fn new_for_testing() -> Pizarra {
        Pizarra::new(Vec2D::new_screen(80.0, 60.0), Default::default())
    }

    pub fn add_sample_shape(&mut self, shape: Box<dyn ShapeStored>) {
        self.storage.add(shape);
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        draw_commands::DrawCommand, app::SelectedTool,
        path_command::{PathCommand::*, CubicBezierCurve},
    };
    use crate::style::{Style, Stroke};
    use crate::shape::stored::path::Path;

    use super::*;

    #[test]
    fn draw_a_line() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 0.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(2.0, 0.0));

        assert_eq!(app.storage.shape_count(), 1);

        let commands = app.draw_commands_for_screen();

        assert_eq!(commands[0], DrawCommand::Path {
            style: Default::default(),
            commands: vec![
                MoveTo(Vec2D::new_world( -40.0, -30.0 )),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2D::new_world( -39.666666666666664, -30.0 ),
                    pt2: Vec2D::new_world( -39.333333333333336, -30.0 ),
                    to: Vec2D::new_world( -39.0, -30.0 )
                }),
            ],
        });
    }

    #[test]
    fn draw_a_line_in_zoom() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        app.zoom_in();

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 0.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(2.0, 0.0));

        assert_eq!(app.storage.shape_count(), 1);

        let commands = app.draw_commands_for_screen();

        assert_eq!(commands[0], DrawCommand::Path {
            style: Style {
                stroke: Some(Stroke {
                    color: Default::default(),
                    size: (Config::default().thickness / 2.0).val().into(),
                }),
                fill: None,
            },
            commands: vec![
                MoveTo(Vec2D::new_world( -20.0, -15.0 )),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2D::new_world( -19.833333333333332, -15.0 ),
                    pt2: Vec2D::new_world( -19.666666666666668, -15.0 ),
                    to: Vec2D::new_world( -19.5, -15.0 )
                }),
            ],
        });
    }

    #[test]
    fn erase() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(10.0, 10.0));
        app.handle_mouse_move(Vec2D::new_screen(15.0, 10.0));
        app.handle_mouse_move(Vec2D::new_screen(15.0, 15.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(10.0, 15.0));

        assert_eq!(app.storage.shape_count(), 1);

        app.set_tool(SelectedTool::Eraser);

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(10.0, 10.0));
        app.handle_mouse_move(Vec2D::new_screen(15.0, 10.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(15.0, 10.0));

        assert_eq!(app.storage.shape_count(), 0);
    }

    #[test]
    fn erase_with_zoom() {
        let mut app = Pizarra::new_for_testing();

        app.resize(Vec2D::new_screen(4.0, 4.0));

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));
        app.set_stroke(0.0.into());

        // draw the shape
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(2.0, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(2.0, 1.5));
        app.handle_mouse_move(Vec2D::new_screen(2.0, 2.0));
        app.handle_mouse_move(Vec2D::new_screen(2.0, 2.5));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(2.0, 3.0));

        assert_eq!(app.storage.shape_count(), 1);

        app.zoom_in();

        app.set_tool(SelectedTool::Eraser);
        app.set_erase_radius(0.5.into());

        // attempt erase and fail
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(2.6, 0.0));
        app.handle_mouse_move(Vec2D::new_screen(2.6, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(2.6, 3.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(2.6, 4.0));

        assert_eq!(app.storage.shape_count(), 1);

        // attempt erase and succeed
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(2.24, 0.0));
        app.handle_mouse_move(Vec2D::new_screen(2.4, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(2.4, 3.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(2.24, 4.0));

        assert_eq!(app.storage.shape_count(), 0);
    }

    #[test]
    fn zoom_out_is_plugged() {
        let mut app = Pizarra::new_for_testing();

        app.zoom_out();

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-80.0, -60.0), Vec2D::new_world(80.0, 60.0)]);
    }

    #[test]
    fn do_do_do_undo_redo_redo_undo() {
        let mut app = Pizarra::new_for_testing();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));

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
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(1.0, 1.0));

        app.undo();

        app.redo();

        app.undo();

        assert_eq!(app.storage.shape_count(), 0);
    }

    #[test]
    fn do_undo_do_do_undo() {
        let mut app = Pizarra::new_for_testing();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(1.0, 1.0));

        app.undo();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(1.0, 1.0));

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(1.0, 1.0));

        app.undo();

        assert_eq!(app.storage.shape_count(), 1);
    }

    #[test]
    fn do_undo_do_undo_undo() {
        let mut app = Pizarra::new_for_testing();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(1.0, 1.0));

        app.undo();

        // Do
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 1.0));
        app.handle_mouse_move(Vec2D::new_screen(1.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(1.0, 1.0));

        app.undo();
        app.undo();

        assert_eq!(app.storage.shape_count(), 0);
    }

    #[test]
    fn r#move() {
        let mut app = Pizarra::new_for_testing();

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-40.0, -30.0), Vec2D::new_world(40.0, 30.0)]);

        app.handle_mouse_button_pressed(MouseButton::Middle, Vec2D::new_screen(0.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Middle, Vec2D::new_screen(-1.0, 0.0));

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-39.0, -30.0), Vec2D::new_world(41.0, 30.0)]);
    }

    #[test]
    fn move_while_in_zoom_is_coherent() {
        let mut app = Pizarra::new_for_testing();

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-40.0, -30.0), Vec2D::new_world(40.0, 30.0)]);

        app.zoom_in();

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-20.0, -15.0), Vec2D::new_world(20.0, 15.0)]);

        app.handle_mouse_button_pressed(MouseButton::Middle, Vec2D::new_screen(0.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Middle, Vec2D::new_screen(-1.0, 0.0));

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-19.5, -15.0), Vec2D::new_world(20.5, 15.0)]);
    }

    #[test]
    fn move_while_drawing() {
        let mut app = Pizarra::new_for_testing();

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-40.0, -30.0), Vec2D::new_world(40.0, 30.0)]);

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(10.0, 10.0));
        app.handle_mouse_move(Vec2D::new_screen(20.0, 10.0));
        app.handle_mouse_button_pressed(MouseButton::Middle, Vec2D::new_screen(20.0, 10.0));
        app.handle_mouse_move(Vec2D::new_screen(30.0, 10.0));
        app.handle_mouse_button_released(MouseButton::Middle, Vec2D::new_screen(30.0, 10.0));
        app.handle_mouse_move(Vec2D::new_screen(40.0, 10.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(40.0, 10.0));

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-50.0, -30.0), Vec2D::new_world(30.0, 30.0)]);
        assert_eq!(app.storage.shape_count(), 1);

        let commands = app.draw_commands_for_screen();

        assert_eq!(commands.len(), 1);

        assert_eq!(commands[0], DrawCommand::Path {
            style: Default::default(),
            commands: vec![
                MoveTo(Vec2D::new_world( -30.0, -20.0 )),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2D::new_world( -26.666666666666668, -20.0 ),
                    pt2: Vec2D::new_world( -23.333333333333332, -20.0 ),
                    to: Vec2D::new_world( -20.0, -20.0 )
                }),
                CurveTo(CubicBezierCurve {
                    pt1: Vec2D::new_world( -16.666666666666668, -20.0 ),
                    pt2: Vec2D::new_world( -13.333333333333332, -20.0 ),
                    to: Vec2D::new_world( -10.0, -20.0 )
                }),
            ],
        });
    }

    #[test]
    fn zoom_must_be_idempotent() {
        let mut app = Pizarra::new_for_testing();

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-40.0, -30.0), Vec2D::new_world(40.0, 30.0)]);

        app.zoom_in();

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-20.0, -15.0), Vec2D::new_world(20.0, 15.0)]);

        app.zoom_in();

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-10.0, -7.5), Vec2D::new_world(10.0, 7.5)]);
    }

    #[test]
    fn cache_invalidation_on_ctrl_z() {
        let mut app = Pizarra::new_for_testing();

        app.config.point_snap_radius = 1.0.into();

        app.resize(Vec2D::new_screen(10.0, 10.0));

        app.set_tool(SelectedTool::Shape(ShapeTool::Rectangle));

        // draw a shape
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(5.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(6.0, 6.0));

        assert_eq!(app.draw_commands_for_screen().len(), 1);

        // move the screen
        app.handle_mouse_button_pressed(MouseButton::Middle, Vec2D::new_screen(10.0, 5.0));
        app.handle_mouse_move(Vec2D::new_screen(5.0, 5.0));
        app.handle_mouse_move(Vec2D::new_screen(0.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Middle, Vec2D::new_screen(0.0, 5.0));

        assert_eq!(app.draw_commands_for_screen().len(), 0);

        // draw another shape
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(5.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(6.0, 6.0));

        assert_eq!(app.draw_commands_for_screen().len(), 1);

        // and draw a third shape
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(6.0, 6.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(7.0, 7.0));

        assert_eq!(app.draw_commands_for_screen().len(), 2);

        // undo that last one
        app.undo();

        assert_eq!(app.draw_commands_for_screen().len(), 1);

        // move back to where the first shape was
        app.handle_mouse_button_pressed(MouseButton::Middle, Vec2D::new_screen(0.0, 5.0));
        app.handle_mouse_move(Vec2D::new_screen(5.0, 5.0));
        app.handle_mouse_move(Vec2D::new_screen(10.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Middle, Vec2D::new_screen(10.0, 5.0));

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

        app.config.point_snap_radius = 1.0.into();

        app.set_tool(SelectedTool::Shape(ShapeTool::Rectangle));

        // draw a shape
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(5.0, 5.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(6.0, 6.0));

        app.set_tool(SelectedTool::Eraser);

        // erase nothing
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(90.0, 90.0));
        app.handle_mouse_move(Vec2D::new_screen(91.0, 91.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(92.0, 92.0));

        // move to where the shape is
        app.handle_mouse_move(Vec2D::new_screen(5.0, 5.0));

        assert_eq!(app.storage.shape_count(), 1);
    }

    #[test]
    fn app_state_after_finished_shape() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));

        let p1 = Vec2D::new_screen(1.0, 1.0);
        let p2 = Vec2D::new_screen(20.0, 20.0);

        app.handle_mouse_button_pressed(MouseButton::Left, p1);
        app.handle_mouse_move(p2);
        app.handle_mouse_button_released(MouseButton::Left, p2);

        assert_eq!(app.board_state, BoardState::Idle);
        assert_eq!(app.undo_state, UndoState::InSync);
        assert_eq!(app.save_status, SaveStatus::NewAndChanged);
        assert_eq!(app.storage.shape_count(), 1);

        if app.current_shape.is_some() {
            panic!()
        }
    }

    #[test]
    fn resize_screen_keeps_center() {
        let mut app = Pizarra::new_for_testing();

        app.resize(Vec2D::new_screen(100.0, 100.0));

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-50.0, -50.0), Vec2D::new_world(50.0, 50.0)]);

        app.resize(Vec2D::new_screen(200.0, 300.0));

        assert_eq!(app.visible_bbox(), [Vec2D::new_world(-100.0, -150.0), Vec2D::new_world(100.0, 150.0)]);
    }

    #[test]
    fn esc_clears_current_shape() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Polygon));
        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 0.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(0.0, 0.0));

        assert_eq!(app.board_state, BoardState::UsingTool);
        assert_eq!(app.undo_state, UndoState::Reset);

        app.handle_key_pressed(Key::Escape);
        assert_eq!(app.handle_key_released(Key::Escape), ShouldRedraw::All);

        assert_eq!(app.board_state, BoardState::Idle);
        assert!(app.current_shape.is_none());
        assert_eq!(app.undo_state, UndoState::Reset);
        assert_eq!(app.save_status, SaveStatus::NewAndEmpty);
    }

    #[test]
    fn tool_override_can_be_undone() {
        let mut app = Pizarra::new_for_testing();

        use super::MouseButton::Left;
        use super::SelectedTool::Eraser;

        app.resize(Vec2D::new_screen( 800.0, 600.0 ));

        // draw the shape
        app.handle_mouse_move_flags(Vec2D::new_screen( 403.578857421875, 281.02178955078125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_button_pressed(Left, Vec2D::new_screen( 403.578857421875, 281.02178955078125 ));
        app.handle_mouse_move_flags(Vec2D::new_screen( 400.51129150390625, 279.93914794921875 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 399.669189453125, 278.4234619140625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 399.6090087890625, 278.0986328125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 399.6090087890625, 277.88214111328125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 399.6090087890625, 277.88214111328125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 399.6090087890625, 277.88214111328125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 399.6090087890625, 277.88214111328125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 399.54888916015625, 278.04449462890625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 399.18798828125, 278.85650634765625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 397.74444580078125, 281.23834228515625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 395.5189208984375, 287.5177001953125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 395.5791015625, 289.3040771484375 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 397.08282470703125, 291.0904541015625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 398.16546630859375, 292.2813720703125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 400.03009033203125, 294.554931640625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 402.25555419921875, 297.4781494140625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 405.92462158203125, 301.700439453125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 410.6162109375, 304.1905517578125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 419.9993896484375, 303.05377197265625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 428.54046630859375, 298.29010009765625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 434.13427734375, 292.93096923828125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 435.938720703125, 285.29827880859375 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 431.06671142578125, 277.2325439453125 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 424.69097900390625, 275.716796875 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 416.8115234375, 277.17840576171875 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 411.097412109375, 279.7767333984375 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 408.33056640625, 281.5631103515625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 406.64642333984375, 283.72845458984375 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 406.10504150390625, 284.91937255859375 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_button_released_flags(Left, Vec2D::new_screen( 406.10504150390625, 284.91937255859375 ), Flags { shift: false, ctrl: false, alt: false }, None);

        // there must be a line
        assert_eq!(app.storage.shape_count(), 1);

        app.handle_mouse_move_flags(Vec2D::new_screen( 405.3231201171875, 287.9508056640625 ), Flags { shift: false, ctrl: false, alt: false }, None);
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.6610107421875, 291.14459228515625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));

        // use the hardware eraser
        app.handle_mouse_button_pressed(Left, Vec2D::new_screen( 420.6610107421875, 291.14459228515625 ));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.6610107421875, 291.14459228515625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.6610107421875, 291.14459228515625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.6610107421875, 291.14459228515625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.54071044921875, 290.982177734375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.54071044921875, 290.982177734375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.54071044921875, 290.982177734375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.54071044921875, 290.982177734375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.54071044921875, 290.982177734375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.4805908203125, 291.14459228515625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.600830078125, 291.415283203125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.6610107421875, 291.57763671875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.78131103515625, 291.74005126953125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.901611328125, 291.9024658203125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.901611328125, 291.9024658203125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.901611328125, 291.9024658203125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 420.901611328125, 291.9024658203125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 421.1422119140625, 291.84832763671875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 421.3828125, 291.74005126953125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 421.68353271484375, 291.5235595703125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 422.04443359375, 291.19873046875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 422.525634765625, 290.7115478515625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 423.00677490234375, 290.0078125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 423.6082763671875, 289.1417236328125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 424.20977783203125, 288.1131591796875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 424.87139892578125, 286.97637939453125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 425.59320068359375, 285.839599609375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 426.31494140625, 284.70281982421875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 427.096923828125, 283.62017822265625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 427.87884521484375, 282.645751953125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 428.6005859375, 281.7796630859375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 429.26226806640625, 281.02178955078125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 429.92388916015625, 280.53460693359375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 430.4652099609375, 280.1015625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 430.88623046875, 279.83087158203125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 431.30731201171875, 279.6143798828125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 431.6080322265625, 279.45196533203125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 431.8486328125, 279.34368896484375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 432.14935302734375, 279.1812744140625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 432.38995361328125, 279.07305908203125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 432.7508544921875, 278.8023681640625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 433.171875, 278.47760009765625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 433.653076171875, 277.93621826171875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 434.19439697265625, 277.17840576171875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 434.8560791015625, 276.0416259765625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 435.6981201171875, 274.36346435546875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 436.72064208984375, 272.19818115234375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 437.7431640625, 269.8704833984375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 438.58526611328125, 267.975830078125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 439.24688720703125, 266.5142822265625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 439.547607421875, 265.7022705078125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 439.90850830078125, 265.1068115234375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 440.2694091796875, 264.5113525390625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 440.510009765625, 264.024169921875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 440.7506103515625, 263.6993408203125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 440.9310302734375, 263.5369873046875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 440.9310302734375, 263.5369873046875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 441.2318115234375, 263.48284912109375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 441.4122314453125, 263.48284912109375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 441.4122314453125, 263.48284912109375 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 441.59271240234375, 263.59112548828125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 441.59271240234375, 263.59112548828125 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 441.65283203125, 263.75347900390625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 441.65283203125, 263.75347900390625 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 441.65283203125, 263.9158935546875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 441.65283203125, 263.9158935546875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_move_flags(Vec2D::new_screen( 441.65283203125, 263.9158935546875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));
        app.handle_mouse_button_released_flags(Left, Vec2D::new_screen( 441.65283203125, 263.9158935546875 ), Flags { shift: false, ctrl: false, alt: false }, Some(Eraser));

        // there must be no line
        assert_eq!(app.storage.shape_count(), 0);

        app.undo();

        // line is visible because erasing was undone
        assert_eq!(app.storage.shape_count(), 1);

        app.undo();

        // no line because line itself was undone
        assert_eq!(app.storage.shape_count(), 0);
    }

    #[test]
    fn all_shapes_that_are_touched_are_erased_at_once() {
        let mut app = Pizarra::new_for_testing();

        app.add_sample_shape(Box::new(Path::from_parts(vec![
            MoveTo(Vec2D::new_world(0.0, 0.0)), LineTo(Vec2D::new_world(20.0, 0.0)),
        ], Default::default())));
        app.add_sample_shape(Box::new(Path::from_parts(vec![
            MoveTo(Vec2D::new_world(0.0, 0.0)), LineTo(Vec2D::new_world(20.0, 0.0)),
        ], Default::default())));

        assert_eq!(app.storage.shape_count(), 2);

        app.set_tool(SelectedTool::Eraser);

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(0.0, 0.0));
        app.handle_mouse_move(Vec2D::new_screen(40.0, 30.0));

        assert_eq!(app.storage.shape_count(), 0);
    }

    /// it turned out that cancelled shapes seem to say on the screen after the
    /// mouse release and are deleted in the next mouse move. It is most
    /// noticeable with mouse and touchpad
    #[test]
    fn on_cancelled_shape_remaining_is_deleted_on_mousebutton_release() {
        let mut app = Pizarra::new_for_testing();

        app.set_tool(SelectedTool::Shape(ShapeTool::Rectangle));

        app.handle_mouse_button_pressed(MouseButton::Left, (0.0, 0.0).into());
        app.handle_mouse_move((1.0, 1.0).into());

        match app.handle_mouse_button_released(MouseButton::Left, (1.0, 1.0).into()) {
            ShouldRedraw::Shape => {}
            _ => panic!()
        }
    }
}
