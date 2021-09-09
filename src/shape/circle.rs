use crate::color::Color;
use crate::point::Vec2DDrawing;
use crate::shape::{ShapeTrait, ShapeFinished, ShapeType};
use crate::draw_commands::DrawCommand;

pub struct Circle {
    center: Vec2DDrawing,
    radius: f64,
    thickness: f64,
    color: Color,
}

impl Circle {
    pub fn new(color: Color, initial: Vec2DDrawing, thickness: f64) -> Circle {
        Circle {
            center: initial,
            thickness,
            color,
            radius: 0.0,
        }
    }
}

impl ShapeTrait for Circle {
    fn handle_mouse_moved(&mut self, pos: Vec2DDrawing) {
        self.radius = self.center.distance(pos);
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DDrawing) {}

    fn handle_button_released(&mut self, pos: Vec2DDrawing) -> ShapeFinished {
        self.handle_mouse_moved(pos);

        ShapeFinished::Yes
    }

    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Circle {
            center: self.center,
            radius: self.radius,
            thickness: self.thickness,
            color: self.color,
        }
    }

    fn bbox(&self) -> [Vec2DDrawing; 2] {
        [
            Vec2DDrawing::new(self.center.x - self.radius, self.center.y - self.radius),
            Vec2DDrawing::new(self.center.x + self.radius, self.center.y + self.radius),
        ]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Circle
    }

    fn intersects_circle(&self, center: Vec2DDrawing, radius: f64) -> bool {
        let d = self.center.distance(center);

        (d <= self.radius + radius) && (d >= self.radius - radius)
    }

    fn color(&self) -> Color {
        self.color
    }
}
