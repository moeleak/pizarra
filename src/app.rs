use std::mem;

use piston::input::RenderArgs;
use graphics::math::{self, Vec2d, Matrix2d};
use graphics::Graphics as PistonGraphics;
use rstar::AABB;

use crate::draw_commands::DrawCommand;
use crate::storage::ShapeStorage;
use crate::zoom::Zoom;
use crate::shape::{ShapeType, ShapeId};
use crate::color::Color;
use crate::action::Action;
use crate::graphics::Graphics;

#[derive(Copy,Clone,Debug)]
/// Describes the possible states of the undo_queue and a pointer that moves
/// in it
enum UndoState {
    /// the last action is effectively the last action of que queue
    InSync,

    /// some actions have been undone and thus the pointer of the last action
    /// is at this location
    At(usize),

    /// due to excesive undoing the pointer points past the begining of the
    /// undo_queue but the queue is not empty, some actions can be redone
    Reset,
}

/// In order to add a shape to the storage it is necessary that it is fully constructed.
/// Most shapes need at least two points to be valid so this enum handles the states before
/// all the needed points have been collected.
enum DrawingState {
    /// In the begining there were no points
    Start,

    /// Then god added one point
    OnePoint([f64; 2]),

    /// And it was good, so she added nother point, and built a shape
    TwoPoints([[f64; 2]; 2]),

    /// From there the shape new what to do with its life
    AndSoOn,
}

pub struct App<G> {
    is_drawing: bool,
    is_moving: bool,
    storage: ShapeStorage,
    selected_tool: ShapeType,
    gl: G,

    /// This value represents a vector from the top left corner of the window to
    /// the center of the drawing
    offset: Vec2d,

    /// The window's dimensions
    dimensions: Vec2d,
    selected_color: Color,
    zoom: Zoom,

    /// Undo
    undo_state: UndoState,
    undo_queue: Vec<Action>,

    /// Since at least two points are needed to create a shape this enum keeps
    /// track of some intermediate states like Start, OnePoint and TwoPoints
    drawing_state: DrawingState,

    /// Next available id for a shape
    next_id: ShapeId,
}

impl<G: Graphics + PistonGraphics> App<G> {
    pub fn render(&mut self, args: &RenderArgs) {
        let bgcolor = Color::Black.to_a();
        let guidecolor = Color::Gray.to_a();
        let offset_t = self.get_offset_t();
        let mut commands = Vec::new();
        let zoom = self.zoom;
        let selected_color = self.selected_color;
        let selected_tool = self.selected_tool;

        // the bbox of the visible things
        let corner_1 = math::mul_scalar(self.offset, -1.0);
        let corner_2 = math::add(corner_1, self.dimensions);

        self.storage.draw_commands(zoom, AABB::from_corners(corner_1, corner_2), &mut commands);

        self.gl.draw(args.viewport(), |c, g| {
            let t = math::multiply(c.transform, offset_t);

            graphics::clear(bgcolor, g);

            // content
            for cmd in commands {
                match cmd {
                    DrawCommand::Line{
                        color, line, thickness,
                    } => graphics::line(color, thickness, line, t, g),
                    DrawCommand::Rectangle{
                        color, rect,
                    } => graphics::rectangle(color, rect, t, g),
                    DrawCommand::Circle{
                        color, rect,
                    } => graphics::ellipse(color, rect, t, g),
                }
            }

            // UI
            // a cross that marks the center
            graphics::line(guidecolor, 1.0, [-20.0, 0.0, 20.0, 0.0], t, g);
            graphics::line(guidecolor, 1.0, [0.0, 20.0, 0.0, -20.0], t, g);

            // a symbol that represents the tool in use
            match selected_tool {
                ShapeType::Circle => graphics::ellipse(selected_color.to_a(), [25.0, 25.0, 25.0, 25.0], c.transform, g),
                ShapeType::Rectangle => graphics::rectangle(selected_color.to_a(), [25.0, 25.0, 25.0, 25.0], c.transform, g),
                ShapeType::Line => {
                    graphics::line(selected_color.to_a(), 2.0, [25.0, 25.0, 25.0, 50.0], c.transform, g);
                    graphics::line(selected_color.to_a(), 2.0, [25.0, 50.0, 50.0, 25.0], c.transform, g);
                    graphics::line(selected_color.to_a(), 2.0, [50.0, 25.0, 50.0, 50.0], c.transform, g);
                }
            }
        });
    }
}

impl<G> App<G> {
    pub fn new(gl: G, dimensions: Vec2d) -> App<G> {
        App {
            is_drawing: false,
            is_moving: false,
            storage: ShapeStorage::new(),
            selected_tool: ShapeType::Line,
            gl,
            dimensions,
            offset: math::mul_scalar(dimensions, 0.5),

            undo_state: UndoState::InSync,
            undo_queue: Vec::new(),

            selected_color: Color::Green,
            zoom: Zoom::from(0),
            drawing_state: DrawingState::Start,
            next_id: ShapeId::from(1),
        }
    }

    pub fn get_dimentions(&self) -> Vec2d {
        self.dimensions
    }

    fn get_inv_offset(&mut self) -> Matrix2d {
        math::translate(math::mul_scalar(self.offset, -1.0))
    }

    fn get_offset_t(&mut self) -> Matrix2d {
        math::translate(self.offset)
    }

    pub fn delta_offset(&mut self, delta: Vec2d) {
        self.offset = math::add(self.offset, delta);
    }

    pub fn resize(&mut self, new_size: Vec2d) {
        let delta = math::mul_scalar(math::add(
            math::mul_scalar(self.dimensions, -1.0),
            new_size
        ), 0.5);

        self.delta_offset(delta);
        self.dimensions = new_size;
    }

    pub fn set_tool(&mut self, shape_type: ShapeType) {
        self.selected_tool = shape_type;
    }

    pub fn start_drawing(&mut self) {
        self.is_drawing = true;
    }

    pub fn finish_drawing(&mut self) {
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
        if let UndoState::At(point) = self.undo_state {
            self.undo_state = UndoState::InSync;
            self.undo_queue.resize_with(point+1, Default::default);
        }

        self.undo_queue.push(action);
        self.undo_state = UndoState::InSync;
    }

    pub fn start_moving(&mut self) {
        self.is_moving = true;
    }

    pub fn end_moving(&mut self) {
        self.is_moving = false;
    }

    pub fn handle_cursor(&mut self, pos: Vec2d) {
        if self.is_drawing && !self.is_moving {
            let inv_offset = self.get_inv_offset();
            let translated_pos = math::transform_pos(inv_offset, pos);

            match self.drawing_state {
                DrawingState::Start => {
                    self.drawing_state = DrawingState::OnePoint(translated_pos);
                },
                DrawingState::OnePoint(p1) => {
                    self.drawing_state = DrawingState::TwoPoints([p1, translated_pos]);
                },
                DrawingState::TwoPoints(points) => {
                    self.storage.add(
                        self.selected_tool.make(self.selected_color, points, self.next_id),
                        self.zoom
                    );
                    self.next_id.incr();
                    self.drawing_state = DrawingState::AndSoOn;
                },
                DrawingState::AndSoOn => {
                    if let Some(shape) = self.storage.last_mut() {
                        shape.handle(translated_pos);
                    }
                },
            }
        }
    }

    pub fn handle_cursor_relative(&mut self, delta: Vec2d) {
        if self.is_moving {
            self.delta_offset(delta);
        }
    }

    pub fn zoom_in(&mut self) {
        self.offset = math::mul_scalar(math::add(
            self.offset,
            math::mul_scalar(self.dimensions, -0.25),
        ), 2.0);

        self.zoom.decrease();
    }

    pub fn zoom_out(&mut self) {
        self.offset = math::mul_scalar(math::add(
            self.offset,
            math::mul_scalar(self.dimensions, 0.5),
        ), 0.5);

        self.zoom.increase();
    }

    pub fn go_home(&mut self) {
        self.offset = math::mul_scalar(self.dimensions, 0.5);
        self.zoom.reset();
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

            Action::Delete((shape, zoom)) => {
                let id = self.storage.add(shape, zoom);

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
                let action_to_redo = mem::replace(&mut self.undo_queue[0], Action::Skip);

                self.undo_queue[0] = self.revert(action_to_redo);

                if self.undo_queue.len() == 1 {
                    self.undo_state = UndoState::InSync;
                } else {
                    self.undo_state = UndoState::At(0);
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
    use crate::graphics::Graphics;
    use graphics::Viewport;

    struct FakeGraphics {
    }

    impl Graphics for FakeGraphics {
        fn draw<F, U>(&mut self, _v: Viewport, _f: F) -> U {
            unimplemented!()
        }
    }

    #[test]
    fn test_do_do_do_undo_redo_redo_undo() {
        let mut app = App::new(FakeGraphics {}, [800.0, 600.0]);

        // Do
        app.start_drawing();
        app.handle_cursor([0.0, 1.0]);
        app.handle_cursor([1.0, 0.0]);
        app.handle_cursor([0.0, 1.0]);
        app.finish_drawing();

        // Do
        app.start_drawing();
        app.handle_cursor([0.0, 1.0]);
        app.handle_cursor([1.0, 0.0]);
        app.handle_cursor([0.0, 1.0]);
        app.finish_drawing();

        // Do
        app.start_drawing();
        app.handle_cursor([0.0, 1.0]);
        app.handle_cursor([1.0, 0.0]);
        app.handle_cursor([0.0, 1.0]);
        app.finish_drawing();

        app.undo();

        app.redo();
        app.redo();

        app.undo();
    }

    #[test]
    fn test_do_undo_redo_undo() {
        let mut app = App::new(FakeGraphics {}, [800.0, 600.0]);

        // Do
        app.start_drawing();
        app.handle_cursor([0.0, 1.0]);
        app.handle_cursor([1.0, 0.0]);
        app.handle_cursor([1.0, 1.0]);
        app.finish_drawing();

        app.undo();

        app.redo();

        app.undo();
    }

    #[test]
    fn test_do_undo_do_do_undo() {
        let mut app = App::new(FakeGraphics {}, [800.0, 600.0]);

        // Do
        app.start_drawing();
        app.handle_cursor([0.0, 1.0]);
        app.handle_cursor([1.0, 0.0]);
        app.handle_cursor([1.0, 1.0]);
        app.finish_drawing();

        app.undo();

        // Do
        app.start_drawing();
        app.handle_cursor([0.0, 1.0]);
        app.handle_cursor([1.0, 0.0]);
        app.handle_cursor([1.0, 1.0]);
        app.finish_drawing();

        // Do
        app.start_drawing();
        app.handle_cursor([0.0, 1.0]);
        app.handle_cursor([1.0, 0.0]);
        app.handle_cursor([1.0, 1.0]);
        app.finish_drawing();

        app.undo();

        assert_eq!(app.storage().shape_count(), 1);
    }

    #[test]
    fn test_do_undo_do_undo_undo() {
        let mut app = App::new(FakeGraphics {}, [800.0, 600.0]);

        // Do
        app.start_drawing();
        app.handle_cursor([0.0, 1.0]);
        app.handle_cursor([1.0, 0.0]);
        app.handle_cursor([1.0, 1.0]);
        app.finish_drawing();

        app.undo();

        // Do
        app.start_drawing();
        app.handle_cursor([0.0, 1.0]);
        app.handle_cursor([1.0, 0.0]);
        app.handle_cursor([1.0, 1.0]);
        app.finish_drawing();

        app.undo();
        app.undo();

        assert_eq!(app.storage().shape_count(), 0);
    }
}
