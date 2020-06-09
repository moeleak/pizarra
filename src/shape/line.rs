use crate::draw_commands::WireDrawCommand;
use super::{Shape, ShapeTrait, ShapeId};
use crate::color::Color;
use crate::ShapeType;
use crate::serialize::ShapeTraitSerialize;
use crate::zoom::Zoom;
use crate::point::Point;

pub struct Line {
    points: Vec<Point>,
    thickness: f64,
}

impl Line {
    pub fn new(color: Color, initial: [Point; 2], id: ShapeId) -> Shape {
        let mut init_vec = Vec::with_capacity(1000);

        init_vec.push(initial[0]);
        init_vec.push(initial[1]);

        Shape {
            id,
            color,
            shape_impl: Box::new(Line {
                points: init_vec,
                thickness: 4.0,
            }),
        }
    }
}

impl ShapeTraitSerialize for Line {
    fn serialize(&self, id: ShapeId, color: Color, zoom: Zoom) -> String {
        let path_vec: Vec<String> = self.points.iter().map(|p| {
            format!("{},{}", p.x, p.y)
        }).collect();
        let path = path_vec.join(" ");

        format!("<path
           style=\"fill:none;stroke:{color};stroke-width:{width}px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1\"
           d=\"M {path}\"
           id=\"{id}\" />", id=id, color=color, width=2.0, path=path)
    }
}

impl ShapeTrait for Line {
    fn handle(&mut self, val: Point) {
        self.points.push(val);
    }

    fn draw_commands(&self) -> WireDrawCommand {
        WireDrawCommand::Line {
            line: self.points.iter().map(|p| p.clone()).collect(),
            thickness: self.thickness,
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
        ShapeType::Line
    }
}

#[cfg(test)]
mod tests {
    use super::Line;
    use crate::shape::{ShapeTrait, ShapeId};
    use crate::color::Color;
    use crate::point::Point;

    #[test]
    fn test_bbox() {
        let mut line = Line {
            points: Vec::new(),
        };

        line.handle(Point::new(1.0, 0.0));
        line.handle(Point::new(0.0, 1.0));

        assert_eq!(line.bbox(), [[0.0, 0.0], [1.0, 1.0]]);
    }

    #[test]
    fn test_bbox_twisted_line() {
        let mut line = Line::new(Color::Green, [Point::new(-12.0, -1.0), Point::new(-5.0, 0.0)], ShapeId::from(1));

        line.handle(Point::new(-2.0, 7.0));
        line.handle(Point::new(2.0, -8.0));

        assert_eq!(line.shape_impl.bbox(), [[-12.0, -8.0], [2.0, 7.0]]);
    }
}
