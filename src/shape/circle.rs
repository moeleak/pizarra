use crate::color::Color;
use crate::point::{Point, Vec2DWorld, Vec2DScreen};
use crate::shape::{ShapeBuilder, ShapeStored, ShapeFinished, ShapeType};
use crate::draw_commands::{DrawCommand, path_helper, circle_helper};
use crate::transform::Transform;

#[derive(Debug)]
pub struct CircleBuilder {
    center: Vec2DWorld,
    point: Vec2DWorld,
    thickness: f64,
    color: Color,
}

impl CircleBuilder {
    pub fn start(color: Color, initial: Vec2DWorld, thickness: f64) -> CircleBuilder {
        CircleBuilder {
            center: initial,
            thickness,
            color,
            point: initial,
        }
    }
}

impl ShapeBuilder for CircleBuilder {
    fn handle_mouse_moved(&mut self, pos: Vec2DScreen, t: Transform, _snap: f64) {
        self.point = t.to_world_coordinates(pos);
    }

    fn handle_button_pressed(&mut self, _pos: Vec2DScreen, _t: Transform, _snap: f64) {}

    fn handle_button_released(&mut self, pos: Vec2DScreen, t: Transform, snap: f64) -> ShapeFinished {
        self.handle_mouse_moved(pos, t, snap);

        ShapeFinished::Yes(Box::new(
            Circle::from_parts(
                self.center,
                self.point.distance(self.center),
                self.thickness,
                self.color,
            )
        ))
    }

    fn draw_commands(&self, t: Transform, snap: f64) -> Vec<DrawCommand> {
        vec![
            DrawCommand::Circle {
                center: self.center,
                radius: self.center.distance(self.point),
                thickness: self.thickness,
                color: Some(self.color),
                fill: None,
            },

            path_helper(vec![
                t.to_screen_coordinates(self.center),
                t.to_screen_coordinates(self.point),
            ]),

            circle_helper(t.to_screen_coordinates(self.center), snap),
        ]
    }
}

#[derive(Debug)]
pub struct Circle {
    center: Vec2DWorld,
    radius: f64,
    thickness: f64,
    color: Color,
}

impl Circle {
    pub fn from_parts(center: Vec2DWorld, radius: f64, thickness: f64, color: Color) -> Circle {
        Circle {
            center, radius, thickness, color,
        }
    }
}

impl ShapeStored for Circle {
    fn bbox(&self) -> [Vec2DWorld; 2] {
        [
            Vec2DWorld::new(self.center.x - self.radius, self.center.y - self.radius),
            Vec2DWorld::new(self.center.x + self.radius, self.center.y + self.radius),
        ]
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Circle
    }

    fn intersects_circle(&self, center: Vec2DWorld, radius: f64) -> bool {
        let d = self.center.distance(center);

        (d <= self.radius + radius) && (d >= self.radius - radius)
    }

    fn color(&self) -> Option<Color> {
        Some(self.color)
    }

    fn fill(&self) -> Option<Color> {
        None
    }

    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Circle {
            center: self.center,
            radius: self.radius,
            thickness: self.thickness,
            color: Some(self.color),
            fill: None,
        }
    }
}
