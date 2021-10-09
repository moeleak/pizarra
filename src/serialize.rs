use xml::writer::{EmitterConfig, XmlEvent, EventWriter};

use crate::consts::EXPORT_PADDING;
use crate::point::{Point, Vec2DDrawing};
use crate::draw_commands::DrawCommand;
use crate::app::App;

impl DrawCommand {
    fn serialize(self, writer: &mut EventWriter<&mut Vec<u8>>) {
        match self {
            DrawCommand::Path {
                color, commands, thickness,
            } => {
                let d: String = commands.iter().map(|p| p.to_string()).collect();

                writer.write(XmlEvent::start_element("path")
                    .attr("style", &format!(
                        "fill:none;stroke-width:{stroke};stroke-linecap:round;stroke-linejoin:round;stroke:{color};stroke-opacity:{alpha};stroke-miterlimit:10;",
                        stroke = thickness,
                        color = color.css(),
                        alpha = color.alpha(),
                    ))
                    .attr("d", &d)).unwrap();
                writer.write(XmlEvent::end_element()).unwrap();
            },
            DrawCommand::Circle {
                color, thickness, center, radius,
            } => {
                writer.write(XmlEvent::start_element("ellipse")
                    .attr("cx", &center.x.to_string())
                    .attr("cy", &center.y.to_string())
                    .attr("rx", &radius.to_string())
                    .attr("ry", &radius.to_string())
                    .attr("style", &format!(
                        "fill:none;stroke-width:{stroke};stroke:{color};stroke-opacity:{alpha};stroke-miterlimit:10",
                        stroke = thickness,
                        color = color.css(),
                        alpha = color.alpha(),
                    ))
                ).unwrap();
                writer.write(XmlEvent::end_element()).unwrap();
            },
            DrawCommand::Ellipse {
                thickness, color, center, semimajor, semiminor, angle,
            } => {
                writer.write(XmlEvent::start_element("ellipse")
                    .attr("cx", &center.x.to_string())
                    .attr("cy", &center.y.to_string())
                    .attr("rx", &semimajor.to_string())
                    .attr("ry", &semiminor.to_string())
                    .attr("transform", &format!(
                            "rotate({angle} {x} {y})",
                            angle = angle.degrees(),
                            x = center.x,
                            y = center.y,
                    ))
                    .attr("style", &format!("\
                            fill:none;\
                            stroke-width:{stroke};\
                            stroke:{color};\
                            stroke-opacity:{alpha};\
                            stroke-miterlimit:10;",
                        stroke = thickness,
                        color = color.css(),
                        alpha = color.alpha(),
                    ))
                ).unwrap();
                writer.write(XmlEvent::end_element()).unwrap();
            },
        }
    }
}

impl App {
    pub fn to_svg(&self) -> Option<String> {
        if let Some(bbox) = self.get_bounds() {
            let mut output = Vec::new();
            let mut writer = EmitterConfig::new()
                .perform_indent(true)
                .create_writer(&mut output);

            let svg_dimensions = (bbox[0] - bbox[1]).abs() + Vec2DDrawing::new(EXPORT_PADDING*2.0, EXPORT_PADDING*2.0);
            let width = svg_dimensions.x.to_string();
            let height = svg_dimensions.y.to_string();
            let min = bbox[0].min(bbox[1]);
            let min_x = (min.x - EXPORT_PADDING).to_string();
            let min_y = (min.y - EXPORT_PADDING).to_string();
            let view_box = format!("{} {} {} {}", &min_x, &min_y, width, height);
            let background_style = format!("fill:{fill};fill-opacity:1;stroke:none;", fill = self.bgcolor().css());

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

            for command in self.draw_commands_for_drawing() {
                command.serialize(&mut writer);
            }

            writer.write(XmlEvent::end_element()).unwrap(); // g
            writer.write(XmlEvent::end_element()).unwrap(); // svg

            output.push(b'\n');

            Some(String::from_utf8(output).unwrap())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::color::Color;
    use crate::point::Vec2DScreen;
    use crate::app::{App, MouseButton, SelectedTool};
    use crate::shape::ShapeType;

    #[test]
    fn test_serialize() {
        let mut app = App::new(Vec2DScreen::new(40.0, 40.0));

        app.set_tool(SelectedTool::Shape(ShapeType::Path));
        app.set_color(Color::red());
        app.set_stroke(3.5);

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(20.0, 20.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(21.0, 20.0));

        assert_eq!(app.to_svg().unwrap(), include_str!("../res/serialize_test.svg"));
    }

    #[test]
    fn serialize_ellipse() {
        let mut app = App::new(Vec2DScreen::new(40.0, 40.0));

        app.set_tool(SelectedTool::Shape(ShapeType::ThreePointEllipse));
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

        assert_eq!(app.to_svg().unwrap(), include_str!("../res/serialize_ellipse.svg"));
    }

    #[test]
    fn test_serialize_color_with_alpha() {
        let mut app = App::new(Vec2DScreen::new(40.0, 40.0));

        app.set_tool(SelectedTool::Shape(ShapeType::Polygon));
        app.set_color(Color::green());
        app.set_alpha(0.5);

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2DScreen::new(10.0, 10.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(30.0, 30.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2DScreen::new(30.0, 30.0));

        assert_eq!(app.to_svg().unwrap(), include_str!("../res/serialize_alpha.svg"));
    }

    #[test]
    fn can_serialize_rotated_ellipse() {
        assert!(false);
    }
}
