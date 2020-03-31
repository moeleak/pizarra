use opengl_graphics::GlGraphics;
use piston::input::RenderArgs;
use graphics::math::{self, Vec2d, Matrix2d};

use crate::draw_commands::DrawCommand;
use crate::storage::{ShapeStorage, Zoom};
use crate::ShapeType;
use crate::color::Color;
use crate::serialize::Serialize;

#[derive(Copy,Clone)]
enum UndoStatus {
    InSync,
    At(usize),
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

pub struct App {
    is_drawing: bool,
    is_moving: bool,
    storage: ShapeStorage,
    selected_tool: ShapeType,
    gl: GlGraphics,
    offset: Vec2d,
    dimentions: Vec2d,
    undo_status: UndoStatus,
    current_color: Color,
    zoom: Zoom,
    drawing_state: DrawingState,
}

impl App {
    pub fn new(gl: GlGraphics, dimentions: Vec2d) -> App {
        App {
            is_drawing: false,
            is_moving: false,
            storage: ShapeStorage::new(),
            selected_tool: ShapeType::Line,
            gl,
            dimentions,
            offset: math::mul_scalar(dimentions, 0.5),
            undo_status: UndoStatus::InSync,
            current_color: Color::green(),
            zoom: Zoom::from(0),
            drawing_state: DrawingState::Start,
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        let bgcolor = Color::black().to_a();
        let guidecolor = Color::gray().to_a();
        let offset_t = self.get_offset_t();
        let mut commands = Vec::new();
        let zoom = self.zoom;

        for item in self.storage.draw(zoom, [0.0, 0.0, 1.1, 1.1]) {
            commands.push(item);
        }

        self.gl.draw(args.viewport(), |c, g| {
            let t = math::multiply(
                math::multiply(c.transform, offset_t),
                math::scale(2.0_f64.powi(zoom.to_i32()), 2.0_f64.powi(zoom.to_i32()))
            );

            graphics::clear(bgcolor, g);

            // content
            for cmd in commands {
                match cmd {
                    DrawCommand::Line{
                        color, line, relative_layer,
                    } => graphics::line(color, 2.0 * 2.0_f64.powi(relative_layer), line, t, g),
                    DrawCommand::Rectangle{
                        color, rect, relative_layer,
                    } => graphics::rectangle(color, rect, t, g),
                    DrawCommand::Circle{
                        color, rect, relative_layer,
                    } => graphics::ellipse(color, rect, t, g),
                }
            }

            // UI
            graphics::line(guidecolor, 1.0, [-20.0, 0.0, 20.0, 0.0], t, g);
            graphics::line(guidecolor, 1.0, [0.0, 20.0, 0.0, -20.0], t, g);
        });
    }

    pub fn get_dimentions(&self) -> Vec2d {
        self.dimentions
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
            math::mul_scalar(self.dimentions, -1.0),
            new_size
        ), 0.5);

        self.delta_offset(delta);
        self.dimentions = new_size;
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
        self.storage.flush();
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

            match self.drawing_state {
                DrawingState::Start => {
                    self.drawing_state = DrawingState::OnePoint(pos);
                },
                DrawingState::OnePoint(p1) => {
                    self.drawing_state = DrawingState::TwoPoints([p1, pos]);
                },
                DrawingState::TwoPoints(points) => {
                    self.storage.add(
                        self.selected_tool.make(self.current_color, points),
                        self.zoom
                    );
                    self.drawing_state = DrawingState::AndSoOn;
                },
                DrawingState::AndSoOn => {
                    if let Some(shape) = self.storage.last_mut() {
                        shape.handle(math::transform_pos(inv_offset, pos));
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
        self.zoom.increase();
    }

    pub fn zoom_out(&mut self) {
        self.zoom.decrease();
    }

    pub fn go_home(&mut self) {
        self.offset = math::mul_scalar(self.dimentions, 0.5);
        self.zoom.reset();
    }

    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn serialize(&self) -> String {
        self.storage.serialize()
    }

    pub fn undo(&mut self) {
        self.storage.pop();
    }
}
