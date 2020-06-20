use std::f64::consts::PI;

use cairo::{Context, LineCap, LineJoin};

use crate::draw_commands::DrawCommand;
use crate::point::Point;
use crate::shape::corners_to_props;

#[derive(Copy, Clone)]
pub struct Transform {
    scale_factor: f64,
    offset: Point,
}

impl Transform {
    pub fn new(offset: Point, scale_factor: f64) -> Transform {
        Transform {
            offset,
            scale_factor,
        }
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn to_screen_coordinates(&self, p: Point) -> Point {
        (p - self.offset) * self.scale_factor
    }
}

impl DrawCommand {
    pub fn draw(self, ctx: &Context, t: Transform) {
        match self {
            DrawCommand::Line {
                color, line, thickness,
            } => {
                ctx.set_line_width(thickness * t.scale_factor());
                ctx.set_source_rgb(color.r, color.g, color.b);
                ctx.set_line_cap(LineCap::Round);
                ctx.set_line_join(LineJoin::Round);

                let mut iter_points = line.iter();

                if let Some(first_point) = iter_points.next() {
                    let p = t.to_screen_coordinates(*first_point);

                    ctx.move_to(p.x, p.y);

                    for point in iter_points {
                        let p = t.to_screen_coordinates(*point);
                        ctx.line_to(p.x, p.y);
                    }

                    ctx.stroke();
                }
            },
            DrawCommand::Rectangle {
                color, corner_1, corner_2,
            } => {
                let (x, y, width, height) = corners_to_props(
                    t.to_screen_coordinates(corner_1),
                    t.to_screen_coordinates(corner_2)
                );
                ctx.rectangle(x, y, width, height);
                ctx.set_source_rgb(color.r, color.g, color.b);
                ctx.fill();
            },
            DrawCommand::Ellipse {
                color, corner_1, corner_2,
            } => {
                ctx.set_source_rgb(color.r, color.g, color.b);
                ctx.move_to(corner_1.x, corner_2.y);
                ctx.arc(0.5, 0.5, 0.5, 0.0, 2.0*PI);
                ctx.scale(corner_2.x - corner_1.x, corner_2.y - corner_1.y);
                ctx.fill();
            },
        }
    }
}
