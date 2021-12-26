use std::fmt;

use xml::writer::{EmitterConfig, XmlEvent, EventWriter};

use crate::point::{Point, Vec2DWorld};
use crate::draw_commands::DrawCommand;
use crate::path_command::PathCommand;
use crate::storage::Storage;
use crate::color::Color;

impl fmt::Display for PathCommand<Vec2DWorld> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathCommand::MoveTo(p) => write!(f, "M {} ", p),
            PathCommand::LineTo(p) => write!(f, "L {} ", p),
            PathCommand::CurveTo(c) => write!(f, "C {}, {}, {} ", c.pt1, c.pt2, c.to),
        }
    }
}

impl DrawCommand {
    fn serialize(self, writer: &mut EventWriter<&mut Vec<u8>>) {
        match self {
            DrawCommand::Path {
                color, commands, thickness, fill,
            } => {
                let d: String = commands.iter().map(|p| p.to_string()).collect();

                writer.write(XmlEvent::start_element("path")
                    .attr("style", &format!(
                        "fill:{fill};stroke-width:{stroke};stroke-linecap:round;stroke-linejoin:round;stroke:{color};stroke-opacity:{alpha};stroke-miterlimit:10;",
                        fill = fill.map(|c| c.css()).unwrap_or_else(|| "none".into()),
                        stroke = thickness,
                        color = color.map(|c| c.css()).unwrap_or_else(|| "none".into()),
                        alpha = color.map(|c| c.float_alpha()).unwrap_or(1.0),
                    ))
                    .attr("d", &d)).unwrap();
                writer.write(XmlEvent::end_element()).unwrap();
            }

            DrawCommand::Circle {
                color, fill, thickness, center, radius,
            } => {
                writer.write(XmlEvent::start_element("ellipse")
                    .attr("cx", &center.x.to_string())
                    .attr("cy", &center.y.to_string())
                    .attr("rx", &radius.to_string())
                    .attr("ry", &radius.to_string())
                    .attr("style", &format!(
                        "fill:{fill};stroke-width:{stroke};stroke:{color};stroke-opacity:{alpha};stroke-miterlimit:10",
                        stroke = thickness,
                        color = color.map(|c| c.css()).unwrap_or_else(|| "none".into()),
                        fill = fill.map(|c| c.css()).unwrap_or_else(|| "none".into()),
                        alpha = color.map(|c| c.float_alpha()).unwrap_or(1.0),
                    ))
                ).unwrap();
                writer.write(XmlEvent::end_element()).unwrap();
            }

            DrawCommand::Ellipse(e) => {
                writer.write(XmlEvent::start_element("ellipse")
                    .attr("cx", &e.center.x.to_string())
                    .attr("cy", &e.center.y.to_string())
                    .attr("rx", &e.semimajor.to_string())
                    .attr("ry", &e.semiminor.to_string())
                    .attr("transform", &format!(
                            "rotate({angle} {x} {y})",
                            angle = e.angle.degrees(),
                            x = e.center.x,
                            y = e.center.y,
                    ))
                    .attr("style", &format!("\
                            fill:{fill};\
                            stroke-width:{stroke};\
                            stroke:{color};\
                            stroke-opacity:{alpha};\
                            stroke-miterlimit:10;",
                        stroke = e.thickness,
                        color = e.color.map(|c| c.css()).unwrap_or_else(|| "none".into()),
                        fill = e.fill.map(|c| c.css()).unwrap_or_else(|| "none".into()),
                        alpha = e.color.map(|c| c.float_alpha()).unwrap_or(1.0),
                    ))
                ).unwrap();
                writer.write(XmlEvent::end_element()).unwrap();
            }

            DrawCommand::ScreenPath { .. } | DrawCommand::ScreenCircle { .. } => {}
        }
    }
}

pub struct Settings {
    pub export_padding: f64,
    pub bgcolor: Color,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            export_padding: 20.0,
            bgcolor: Color::black(),
        }
    }
}

impl Storage {
    pub fn to_svg(&self, settings: Settings) -> String {
        let mut output = Vec::new();
        let mut writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut output);

        let export_padding = settings.export_padding;

        let bbox = self.get_bounds().unwrap_or([Vec2DWorld::new(0.0, 0.0), Vec2DWorld::new(0.0, 0.0)]);
        let svg_dimensions = (bbox[0] - bbox[1]).abs() + Vec2DWorld::new(export_padding*2.0, export_padding*2.0);
        let width = svg_dimensions.x.to_string();
        let height = svg_dimensions.y.to_string();
        let min = bbox[0].min(bbox[1]);
        let min_x = (min.x - export_padding).to_string();
        let min_y = (min.y - export_padding).to_string();
        let view_box = format!("{} {} {} {}", &min_x, &min_y, width, height);
        let background_style = format!("fill:{fill};fill-opacity:1;stroke:none;", fill = settings.bgcolor.css());

        writer.write(XmlEvent::start_element("svg")
            .ns("", "http://www.w3.org/2000/svg")
            .attr("width", &width)
            .attr("height", &height)
            .attr("viewBox", &view_box)
            .attr("version", "1.1")).unwrap();

        writer.write(XmlEvent::start_element("g")
            .attr("id", "storage")).unwrap();

        writer.write(XmlEvent::start_element("rect")
            .attr("x", &min_x)
            .attr("y", &min_y)
            .attr("width", &width)
            .attr("height", &height)
            .attr("style", &background_style)).unwrap();
        writer.write(XmlEvent::end_element()).unwrap();

        if let Some(bbox) = self.get_bounds() {
            for command in self.draw_commands(bbox) {
                command.serialize(&mut writer);
            }
        }

        writer.write(XmlEvent::end_element()).unwrap(); // g
        writer.write(XmlEvent::end_element()).unwrap(); // svg

        output.push(b'\n');

        String::from_utf8(output).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::color::Color;
    use crate::point::Vec2DScreen;
    use crate::app::{Pizarra, MouseButton, SelectedTool};
    use crate::shape::ShapeTool;
    use crate::shape::path::Path;
    use crate::shape::ellipse::Ellipse;
    use crate::path_command::PathCommand;
    use crate::geom::Angle;

    #[test]
    fn test_serialize() {
        let mut app = Pizarra::new_for_testing();

        app.resize(Vec2DScreen::new(40.0, 40.0));

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));
        app.set_color(Color::red());
        app.set_stroke(3.5);

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(20.0, 20.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(21.0, 20.0));

        assert_eq!(app.to_svg(), include_str!("../res/serialize_test.svg"));
    }

    #[test]
    fn serialize_ellipse() {
        let mut app = Pizarra::new_for_testing();

        app.resize(Vec2DScreen::new(40.0, 40.0));

        app.set_tool(SelectedTool::Shape(ShapeTool::ThreePointEllipse));
        app.set_color(Color::red());
        app.set_stroke(3.5);

        let a: Vec2DScreen = (-5.61146,3.80508).into();
        let c: Vec2DScreen = (-0.44,-1.28).into();
        let e: Vec2DScreen = (4.3,-0.4).into();

        // start and first focus of the ellipse
        app.handle_mouse_button_pressed(MouseButton::Left, a);
        app.handle_mouse_button_released(MouseButton::Left, a);

        // second focus of the ellipse
        app.handle_mouse_move(c);
        app.handle_mouse_button_released(MouseButton::Left, c);

        // external point of the ellipse, finish shape
        app.handle_mouse_move(e);
        app.handle_mouse_button_released(MouseButton::Left, e);

        assert_eq!(app.to_svg(), include_str!("../res/serialize_ellipse.svg"));
    }

    #[test]
    fn test_serialize_color_with_alpha() {
        let mut storage = Storage::new();

        storage.add(Box::new(Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(10.0, 10.0)),
            PathCommand::LineTo(Vec2DWorld::new(10.0, 10.0)),
        ], Some(Color::green().with_alpha(127)), None, 3.0)));

        assert_eq!(storage.to_svg(Default::default()), include_str!("../res/serialize_alpha.svg"));
    }

    #[test]
    fn an_empty_drawing_is_serializable() {
        let storage = Storage::new();

        assert_eq!(storage.to_svg(Default::default()), include_str!("../res/serialize_empty.svg"));
    }

    #[test]
    fn fill_and_stroke_color_are_properly_serialized() {
        let mut storage = Storage::new();

        storage.add(Box::new(Path::from_parts(vec![
            PathCommand::MoveTo(Vec2DWorld::new(0.0, 0.0)),
            PathCommand::LineTo(Vec2DWorld::new(10.0, 10.0)),
        ], None, None, 3.0)));
        storage.add(Box::new(Ellipse::from_parts(
            Vec2DWorld::new(0.0, 0.0),
            10.0,
            15.0,
            Angle::from_radians(0.0),
            None,
            None,
            2.0,
        )));

        assert_eq!(storage.to_svg(Default::default()), include_str!("../res/serialize_stroke_fill_none.svg"));
    }
}
