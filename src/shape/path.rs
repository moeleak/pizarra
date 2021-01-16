use super::{ShapeTrait, ShapeFinished};
use crate::draw_commands::DrawCommand;
use crate::color::Color;
use crate::ShapeType;
use crate::point::Point;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PathCommand {
    MoveTo(Point),
}

pub struct Path {
    commands: Vec<PathCommand>,
    thickness: f64,
    color: Color,
}

impl Path {
    pub fn new(color: Color, initial: Point, thickness: f64) -> Path {
        let mut init_vec = Vec::with_capacity(1000);

        init_vec.push(PathCommand::MoveTo(initial));

        Path {
            commands: init_vec,
            thickness,
            color,
        }
    }
}

impl ShapeTrait for Path {
    fn handle_mouse_moved(&mut self, pos: Point) {
        self.commands.push(pos);
    }

    fn handle_button_pressed(&mut self, _pos: Point) {
    }

    fn handle_button_released(&mut self, pos: Point) -> ShapeFinished {
        self.commands.push(pos);
        ShapeFinished::Yes
    }

    fn draw_commands(&self) -> DrawCommand {
        DrawCommand::Path {
            Path: Rc::new(self.points.iter().map(|p| p.clone()).collect()),
            thickness: self.thickness,
            color: self.color,
        }
    }

    fn bbox(&self) -> [[f64; 2]; 2] {
        let bbox = self.points.iter().fold([
            [std::f64::INFINITY, std::f64::INFINITY], // bottom-left corner
            [std::f64::NEG_INFINITY, std::f64::NEG_INFINITY], // top right corner
        ], |acc, point| {
            [
                [
                    if point.x < acc[0][0] { point.x } else { acc[0][0] },
                    if point.y < acc[0][1] { point.y } else { acc[0][1] },
                ],
                [
                    if point.x > acc[1][0] { point.x } else { acc[1][0] },
                    if point.y > acc[1][1] { point.y } else { acc[1][1] },
                ],
            ]
        });

        assert_ne!(bbox[0][0], std::f64::INFINITY);
        assert_ne!(bbox[0][1], std::f64::INFINITY);
        assert_ne!(bbox[1][0], std::f64::NEG_INFINITY);
        assert_ne!(bbox[1][1], std::f64::NEG_INFINITY);

        bbox
    }

    fn shape_type(&self) -> ShapeType {
        ShapeType::Path
    }

    fn intersects_circle(&self, center: Point, radius: f64) -> bool {
        for p in self.commands.iter() {
            if p.distance(center) <= radius {
                return true;
            }
        }

        false
    }

    fn color(&self) -> Color {
        self.color
    }
}
