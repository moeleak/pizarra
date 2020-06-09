use std::mem;
use std::f64::consts::PI;

use rstar::AABB;
use gtk::prelude::*;
use gtk::{DrawingArea, Allocation};
use gdk::{EventMotion, EventButton, EventType};
use cairo::{Context as CairoContext, LineCap, LineJoin};

use crate::draw_commands::ColoredDrawCommand;
use crate::storage::ShapeStorage;
use crate::shape::{ShapeType, ShapeId};
use crate::color::Color;
use crate::action::Action;
use crate::point::Point;
use crate::shape::corners_to_props;

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

            undo_state: UndoState::Reset,
            undo_queue: Vec::new(),

            moving_state: MovingState::Start,

            selected_color: Color::Green,
            drawing_state: DrawingState::Start,
            next_id: ShapeId::from(1),
        }
    }

    /// Compute the bounds of that is visible in the board according to current
    /// offset and zoom level
    fn visible_extent(&self) -> (Point, Point) {
        let inv_offset = self.offset * -1.0;
        (inv_offset, inv_offset + self.dimensions)
    }

    /// p is a point build from an event triggered in the canvas, this function
    /// converts p's coordinates to storage coordinates given the current zoom
    /// and offset.
    fn to_storage_coordinates(&self, p: Point) -> Point {
        p + (self.offset * -1.0)
    }

    /// applyes a transform to the given cairo context that scales a shape from
    /// the storage to screen coordinates
    fn to_screen_coordinates(&self, point: Point) -> Point {
        point + self.offset
    }

    pub fn render(&mut self, dw: &DrawingArea, ctx: &CairoContext) -> Inhibit {
        let bgcolor = Color::Black.to_a();

        // the bbox of the visible things
        let (corner_1, corner_2) = self.visible_extent();

        let commands = self.storage.draw_commands(AABB::from_corners(corner_1.to_a(), corner_2.to_a()));

        ctx.set_source_rgb(bgcolor[0], bgcolor[1], bgcolor[2]);
        ctx.paint();

        // content
        for cmd in commands {
            match cmd {
                ColoredDrawCommand::Line {
                    color, line, thickness,
                } => {
                    ctx.set_line_width(thickness);
                    let c = color.to_a();
                    ctx.set_source_rgb(c[0], c[1], c[2]);
                    ctx.set_line_cap(LineCap::Round);
                    ctx.set_line_join(LineJoin::Round);

                    let mut iter_points = line.iter();

                    if let Some(first_point) = iter_points.next() {
                        let p = self.to_screen_coordinates(*first_point);

                        ctx.move_to(p.x, p.y);

                        for point in iter_points {
                            let p = self.to_screen_coordinates(*point);
                            ctx.line_to(p.x, p.y);
                        }

                        ctx.stroke();

                    }
                },
                ColoredDrawCommand::Rectangle {
                    color, corner_1, corner_2,
                } => {
                    let (x, y, width, height) = corners_to_props(corner_1, corner_2);
                    ctx.rectangle(x, y, width, height);
                    let rgb = color.to_a();
                    ctx.set_source_rgb(rgb[0], rgb[1], rgb[2]);
                    ctx.fill();
                },
                ColoredDrawCommand::Ellipse {
                    color, corner_1, corner_2,
                } => {
                    let c = color.to_a();
                    ctx.set_source_rgb(c[0], c[1], c[2]);
                    ctx.move_to(corner_1.x, corner_2.y);
                    ctx.arc(0.5, 0.5, 0.5, 0.0, 2.0*PI);
                    ctx.scale(corner_2.x - corner_1.x, corner_2.y - corner_1.y);
                    ctx.fill();
                },
            }
        }

        // UI
        self.render_ui(dw, ctx);

        Inhibit(false)
    }

    fn render_ui(&self, _dw: &DrawingArea, _ctx: &CairoContext) {
        // a cross that marks the center
        // TODO draw a cross that marks 0, 0

        // TODO a symbol that represents the tool in use
        match self.selected_tool {
            ShapeType::Ellipse => {},
            ShapeType::Rectangle => {},
            ShapeType::Line => {},
        }
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
        unimplemented!()
    }

    pub fn zoom_out(&mut self) {
        unimplemented!()
    }

    pub fn go_home(&mut self) {
        self.offset = self.dimensions * 0.5;
    }

    pub fn set_color(&mut self, color: Color) {
        self.selected_color = color;
    }

    pub fn cycle_color(&mut self) {
        self.selected_color = self.selected_color.cycle();
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

    pub fn on_button_press(&mut self, _dw: &DrawingArea, event: &EventButton) -> Inhibit {
        if let EventType::ButtonPress = event.get_event_type() {
            match event.get_button() {
                1 => self.start_drawing(),
                2 => self.start_moving(),
                _ => {},
            }
        }

        Inhibit(false)
    }

    pub fn on_button_release(&mut self, _dw: &DrawingArea, event: &EventButton) -> Inhibit {
        if let EventType::ButtonRelease = event.get_event_type() {
            match event.get_button() {
                1 => self.end_drawing(),
                2 => self.end_moving(),
                _ => {},
            }
        }

        Inhibit(false)
    }

    pub fn on_pointer_move(&mut self, dw: &DrawingArea, event: &EventMotion) -> Inhibit {
        let (x, y) = event.get_position();

        if let ShouldRedraw::Yes = self.handle_cursor(Point::new(x, y)) {
            dw.queue_draw();
        }

        Inhibit(false)
    }

    pub fn on_size_allocate(&mut self, _dw: &DrawingArea, allocation: Allocation) {
        self.resize(Point::new(allocation.width as f64, allocation.height as f64));
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
}
