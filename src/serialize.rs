use std::fmt;

use xml::writer::{EmitterConfig, XmlEvent, EventWriter};

use crate::point::{Vec2D, WorldUnit};
use crate::draw_commands::DrawCommand;
use crate::path_command::PathCommand;
use crate::storage::Storage;
use crate::color::Color;

impl fmt::Display for PathCommand<WorldUnit> {
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
                commands, style,
            } => {
                let d: String = commands.iter().map(|p| p.to_string()).collect();

                writer.write(XmlEvent::start_element("path")
                    .attr("style", &format!(
                        "fill:{fill};stroke-width:{stroke};stroke-linecap:round;stroke-linejoin:round;stroke:{color};stroke-opacity:{alpha};stroke-miterlimit:10;",
                        fill = style.fill.map(|c| c.css()).unwrap_or_else(|| "none".into()),
                        stroke = style.stroke.map(|s| s.size).unwrap_or_else(|| 0.0.into()),
                        color = style.stroke.map(|s| s.color.css()).unwrap_or_else(|| "none".into()),
                        alpha = style.stroke.map(|s| s.color.float_alpha()).unwrap_or(1.0),
                    ))
                    .attr("d", &d)).unwrap();
                writer.write(XmlEvent::end_element()).unwrap();
            }

            DrawCommand::Ellipse { ellipse: e, style } => {
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
                        stroke = style.stroke.map(|s| s.size).unwrap_or_else(|| 0.0.into()),
                        color = style.stroke.map(|s| s.color.css()).unwrap_or_else(|| "none".into()),
                        alpha = style.stroke.map(|s| s.color.float_alpha()).unwrap_or(1.0),
                        fill = style.fill.map(|c| c.css()).unwrap_or_else(|| "none".into()),
                    ))
                ).unwrap();
                writer.write(XmlEvent::end_element()).unwrap();
            }

            DrawCommand::ScreenPath { .. } | DrawCommand::ScreenCircle { .. } => {}
        }
    }
}

pub struct Settings {
    pub export_padding: WorldUnit,
    pub bgcolor: Color,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            export_padding: 20.0.into(),
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

        let bbox = self.get_bounds().unwrap_or([Vec2D::new_world(0.0, 0.0), Vec2D::new_world(0.0, 0.0)]);
        let svg_dimensions = (bbox[0] - bbox[1]).abs() + Vec2D::new(export_padding * 2.0.into(), export_padding * 2.0.into());
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
    use pretty_assertions::assert_eq;

    use crate::color::Color;
    use crate::point::ScreenUnit;
    use crate::app::{Pizarra, MouseButton, SelectedTool};
    use crate::shape::ShapeTool;
    use crate::shape::stored::{path::Path, ellipse::Ellipse};
    use crate::path_command::PathCommand;
    use crate::geom::Angle;
    use crate::style::{Style, Stroke};
    use crate::config::Config;

    use super::*;

    #[test]
    fn test_serialize() {
        let mut app = Pizarra::new_for_testing();

        app.resize(Vec2D::new_screen(40.0, 40.0));

        app.set_tool(SelectedTool::Shape(ShapeTool::Path));
        app.set_color(Color::red());
        app.set_stroke(3.5.into());

        app.handle_mouse_button_pressed(MouseButton::Left, Vec2D::new_screen(20.0, 20.0));
        app.handle_mouse_button_released(MouseButton::Left, Vec2D::new_screen(21.0, 20.0));

        assert_eq!(app.to_svg(), include_str!("../res/serialize_test.svg"));
    }

    #[test]
    fn serialize_ellipse() {
        let mut app = Pizarra::new_for_testing();

        app.resize(Vec2D::new_screen(40.0, 40.0));

        app.set_tool(SelectedTool::Shape(ShapeTool::ThreePointEllipse));
        app.set_color(Color::red());
        app.set_stroke(3.5.into());

        app.set_config(Config {
            point_snap_radius: 1.0.into(),
            ..Default::default()
        });

        let a: Vec2D<ScreenUnit> = (-5.61146, 3.80508).into();
        let c: Vec2D<ScreenUnit> = (-0.44, -1.28).into();
        let e: Vec2D<ScreenUnit> = (4.3, -0.4).into();

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
            PathCommand::MoveTo(Vec2D::new_world(10.0, 10.0)),
            PathCommand::LineTo(Vec2D::new_world(10.0, 10.0)),
        ], Style {
            stroke: Some(Stroke {
                color: Color::green().with_alpha(127),
                size: 3.0.into(),
            }),
            fill: None,
        })));

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
            PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
            PathCommand::LineTo(Vec2D::new_world(10.0, 10.0)),
        ], Style {
            stroke: None,
            fill: None,
        })));
        storage.add(Box::new(Ellipse::from_parts(
            Vec2D::new_world(0.0, 0.0),
            10.0.into(),
            15.0.into(),
            Angle::from_radians(0.0),
            Style {
                fill: None,
                stroke: None,
            },
        )));

        assert_eq!(storage.to_svg(Default::default()), include_str!("../res/serialize_stroke_fill_none.svg"));
    }

    #[test]
    fn new_format() {
        todo!("store background in a layer, shapes in a different one");
        todo!("background color is stored");
        todo!("pizarra version is written");
    }
}
