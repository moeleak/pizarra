use crate::color::Color;
use crate::point::Point;
use crate::shape::{ShapeTrait, ShapeFinished, ShapeType};
use crate::draw_commands::DrawCommand;

pub struct Circle {
    center: Point,
    radius: f64,
    thickness: f64,
    color: Color,
}

impl Circle {
    pub fn new(color: Color, initial: Point, thickness: f64) -> Circle {
        Circle {
            center: initial,
            thickness,
            color,
            radius: 0.0,
        }
    }
}

impl ShapeTrait for Circle {
    fn handle_mouse_moved(&mut self, pos: Point) {
        self.radius = self.center.distance(pos);
    }

    fn handle_button_pressed(&mut self, _pos: Point) {}

    fn handle_button_released(&mut self, pos: Point) -> ShapeFinished {
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

    fn bbox(&self) -> [[f64; 2]; 2] {
        [
            [self.center.x - self.radius, self.center.y - self.radius],
            [self.center.x + self.radius, self.center.y + self.radius],
        ]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Circle
    }

    fn intersects_circle(&self, center: Point, radius: f64) -> bool {
        let d = self.center.distance(center);

        (d <= self.radius + radius) && (d >= self.radius - radius)
    }

    fn color(&self) -> Color {
        self.color
    }
}
