use xml::writer::{EmitterConfig, XmlEvent, EventWriter};

use crate::consts::EXPORT_PADDING;
use crate::point::Point;
use crate::draw_commands::DrawCommand;
use crate::app::App;

impl DrawCommand {
    fn serialize(self, writer: &mut EventWriter<&mut Vec<u8>>) {
        match self {
            DrawCommand::Line {
                color, line, thickness,
            } => {
                let mut points_iter = line.iter();
                let first = points_iter.next().unwrap();
                let mut d = format!("M {} {} ", first.x, first.y);

                let rest: String = points_iter.map(|p| format!("L {} {} ", p.x, p.y)).collect();
                d.push_str(&rest);

                writer.write(XmlEvent::start_element("path")
                    .attr("style", &format!(
                        "fill:none;stroke-width:{stroke};stroke-linecap:round;stroke-linejoin:round;stroke:{color};stroke-opacity:1;stroke-miterlimit:10;",
                        stroke = thickness,
                        color = color.css(),
                    ))
                    .attr("d", &d)).unwrap();
                writer.write(XmlEvent::end_element()).unwrap();
            },
            _ => {},
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

            let svg_dimensions = (bbox[0] - bbox[1]).abs() + Point::new(EXPORT_PADDING*2.0, EXPORT_PADDING*2.0);
            let width = svg_dimensions.x.to_string();
            let height = svg_dimensions.y.to_string();
            let min = bbox[0].min(bbox[1]);
            let view_box = format!("{} {} {} {}", min.x - EXPORT_PADDING, min.y - EXPORT_PADDING, width, height);
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
                .attr("x", "0")
                .attr("y", "0")
                .attr("width", &width)
                .attr("height", &height)
                .attr("style", &background_style)).unwrap();
            writer.write(XmlEvent::end_element()).unwrap();

            for command in self.draw_commands_for_drawing() {
                command.serialize(&mut writer);
            }

            writer.write(XmlEvent::end_element()).unwrap(); // g
            writer.write(XmlEvent::end_element()).unwrap(); // svg

            Some(String::from_utf8(output).unwrap())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::color::Color;
    use crate::point::Point;
    use crate::app::{App, MouseButton, SelectedTool};
    use crate::shape::ShapeType;

    #[test]
    fn serialize() {
        let mut app = App::new(Point::new(40.0, 40.0));

        app.set_tool(SelectedTool::Shape(ShapeType::Line));
        app.set_color(Color::red());
        app.set_stroke(3.5);

        app.handle_mouse_button_pressed(MouseButton::Left, Point::new(20.0, 20.0));
        app.handle_mouse_button_released(MouseButton::Left, Point::new(21.0, 20.0));

        assert_eq!(app.to_svg().unwrap(), include_str!("../res/serialize_test.svg"));
    }
}
