use std::fmt;

use xml::writer::{EmitterConfig, XmlEvent};

use crate::point::{Vec2D, WorldUnit};
use crate::path_command::PathCommand;
use crate::color::Color;
use crate::app::Pizarra;

impl fmt::Display for PathCommand<WorldUnit> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathCommand::MoveTo(p) => write!(f, "M {} ", p),
            PathCommand::LineTo(p) => write!(f, "L {} ", p),
            PathCommand::CurveTo(c) => write!(f, "C {}, {}, {} ", c.pt1, c.pt2, c.to),
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

impl Pizarra {
    pub fn to_svg(&self) -> String {
        let mut output = Vec::new();
        let mut writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut output);

        let export_padding = self.config().export_padding;

        let bbox = self.get_bounds().unwrap_or([Vec2D::new_world(0.0, 0.0), Vec2D::new_world(0.0, 0.0)]);
        let svg_dimensions = (bbox[0] - bbox[1]).abs() + Vec2D::new(export_padding * 2.0.into(), export_padding * 2.0.into());
        let width = svg_dimensions.x.to_string();
        let height = svg_dimensions.y.to_string();
        let min = bbox[0].min(bbox[1]);
        let min_x = (min.x - export_padding).to_string();
        let min_y = (min.y - export_padding).to_string();
        let view_box = format!("{} {} {} {}", &min_x, &min_y, width, height);
        let background_style = format!("fill:{fill};fill-opacity:1;stroke:none;", fill = self.bgcolor().css());

        writer.write(XmlEvent::start_element("svg")
            .ns("", "http://www.w3.org/2000/svg")

            // Aditional namespaces for custom elements
            .ns("pizarra", "http://pizarra.categulario.tk/pizarra_svg.html")
            .ns("inkscape", "http://www.inkscape.org/namespaces/inkscape")
            .ns("sodipodi", "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd")

            .attr("pizarra:format", "2")
            .attr("width", &width)
            .attr("height", &height)
            .attr("viewBox", &view_box)
            .attr("version", "1.1")).unwrap();

        // the "background" group
        writer.write(
            XmlEvent::start_element("g")
                .attr("id", "background")
                .attr("inkscape:groupmode", "layer")
                .attr("inkscape:label", "Background")
                .attr("sodipodi:insensitive", "true")
        ).unwrap();

        // the background rectangle
        writer.write(XmlEvent::start_element("rect")
            .attr("x", &min_x)
            .attr("y", &min_y)
            .attr("width", &width)
            .attr("height", &height)
            .attr("style", &background_style)).unwrap();
        writer.write(XmlEvent::end_element()).unwrap();

        // end the background group
        writer.write(XmlEvent::end_element()).unwrap();

        // the "shapes" group
        writer.write(
            XmlEvent::start_element("g")
                .attr("id", "shapes")
                .attr("inkscape:groupmode", "layer")
                .attr("inkscape:label", "Shapes")
        ).unwrap();

        for shape in self.shapes_by_index() {
            shape.serialize(&mut writer);
        }

        writer.write(XmlEvent::end_element()).unwrap(); // g
        writer.write(XmlEvent::end_element()).unwrap(); // svg

        output.push(b'\n');

        String::from_utf8(output).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_str_eq;

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
        let mut app = Pizarra::new_for_testing();

        app.add_sample_shape(Box::new(Path::from_parts(vec![
            PathCommand::MoveTo(Vec2D::new_world(10.0, 10.0)),
            PathCommand::LineTo(Vec2D::new_world(10.0, 10.0)),
        ], Style {
            stroke: Some(Stroke {
                color: Color::green().with_alpha(127),
                size: 3.0.into(),
            }),
            fill: None,
        })));

        assert_eq!(app.to_svg(), include_str!("../res/serialize_alpha.svg"));
    }

    #[test]
    fn an_empty_drawing_is_serializable() {
        let app = Pizarra::new_for_testing();

        assert_eq!(app.to_svg(), include_str!("../res/serialize_empty.svg"));
    }

    #[test]
    fn fill_and_stroke_color_are_properly_serialized() {
        let mut app = Pizarra::new_for_testing();

        app.add_sample_shape(Box::new(Path::from_parts(vec![
            PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
            PathCommand::LineTo(Vec2D::new_world(10.0, 10.0)),
        ], Style {
            stroke: None,
            fill: None,
        })));
        app.add_sample_shape(Box::new(Ellipse::from_parts(
            Vec2D::new_world(0.0, 0.0),
            10.0.into(),
            15.0.into(),
            Angle::from_radians(0.0),
            Style {
                fill: None,
                stroke: None,
            },
        )));

        assert_eq!(app.to_svg(), include_str!("../res/serialize_stroke_fill_none.svg"));
    }

    #[test]
    fn serialize_to_new_format() {
        let mut app = Pizarra::new_for_testing();

        app.add_sample_shape(Box::new(Path::from_parts(vec![
            PathCommand::MoveTo(Vec2D::new_world(0.0, 0.0)),
            PathCommand::LineTo(Vec2D::new_world(10.0, 10.0)),
        ], Default::default())));
        app.add_sample_shape(Box::new(Ellipse::from_parts(
            Vec2D::new_world(0.0, 0.0),
            10.0.into(),
            15.0.into(),
            Angle::from_radians(0.0),
            Default::default(),
        )));

        assert_str_eq!(app.to_svg(), include_str!("../res/serialize/v2_format.svg"));
    }
}
