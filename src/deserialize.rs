use std::fmt;
use std::collections::HashMap;
use std::str::FromStr;

use xml::{
    attribute::OwnedAttribute, reader::{EventReader, XmlEvent, Error as XmlError}
};
use regex::Regex;
use lazy_static::lazy_static;

use crate::storage::Storage;
use crate::shape::{ShapeStored, stored::{path::Path, ellipse::Ellipse}};
use crate::color::Color;
use crate::point::{Vec2D, WorldUnit};
use crate::geom::Angle;
use crate::config::Config;
use crate::style::{Style, Stroke};

mod path;

use path::{parse_path, PathBuilder as SvgPathBuilder};

lazy_static! {
    static ref POINT_REGEX: Regex = Regex::new(r"(?x)
        (?P<x>-?\d+(\.\d+)?)
        \s+
        (?P<y>-?\d+(\.\d+)?)
    ").unwrap();
    static ref ANGLE_REGEX: Regex = Regex::new(r"(?x)
        rotate\(
            (?P<angle>-?\d+(\.\d*)?)
            (\s+(-?\d+(\.\d*)?)\s+(-?\d+(\.\d*)?))?
        \)
    ").unwrap();
}

#[derive(Debug)]
pub enum Error {
    Xml(XmlError),
    PathWithNoDAttr,
    CouldntUnderstandColor(String),
    ParseFloatError(std::num::ParseFloatError),
    PointDoesntMatchRegex,
    AngleDoesntMatchRegex(String),
    MissingAttribute(String),
    PathParseError(path::ParseError),
}

impl From<path::ParseError> for Error {
    fn from(error: path::ParseError) -> Error {
        Error::PathParseError(error)
    }
}

impl From<XmlError> for Error {
    fn from(error: XmlError) -> Error {
        Error::Xml(error)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(error: std::num::ParseFloatError) -> Self {
        Error::ParseFloatError(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Load error caused by {:?}", self)
    }
}

impl std::error::Error for Error {
}

type Result<T> = std::result::Result<T, Error>;

impl FromStr for Vec2D<WorldUnit> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let Some(captures) = POINT_REGEX.captures(s) {
            Ok(Vec2D::new(
                captures["x"].parse::<f64>()?,
                captures["y"].parse::<f64>()?,
            ))
        } else {
            Err(Error::PointDoesntMatchRegex)
        }
    }
}

impl FromStr for Angle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Angle> {
        if let Some(angle) = ANGLE_REGEX.captures(s) {
            Ok(Angle::from_degrees(angle["angle"].parse()?))
        } else {
            Err(Error::AngleDoesntMatchRegex(s.into()))
        }
    }
}

fn css_attrs_as_hashmap(attrs: &str) -> HashMap<&str, &str> {
    attrs.split(';').filter_map(|s| {
        let pieces: Vec<_> = s.split(':').collect();

        if pieces.len() != 2 {
            return None
        }

        Some((pieces[0].trim(), pieces[1].trim()))
    }).collect()
}

fn xml_attrs_as_hashmap(attrs: Vec<OwnedAttribute>) -> HashMap<String, String> {
    attrs.into_iter().map(|a| {
        (a.name.local_name, a.value)
    }).collect()
}

fn parse_color(color_str: &str) -> Result<Option<Color>> {
    if color_str.to_lowercase() == "none" {
        Ok(None)
    } else {
        Ok(Some(color_str.parse()?))
    }
}

impl Path {
    pub fn from_xml_attributes(style: &str, path: &str, config: Config) -> Result<Path> {
        let attrs = css_attrs_as_hashmap(style);

        let color: Option<Color> = attrs.get("stroke").map(|s| parse_color(s)).transpose()?.flatten();
        let fill: Option<Color> = attrs.get("fill").map(|s| parse_color(s)).transpose()?.flatten();
        let thickness = attrs.get("stroke-width").map(|s| s.parse()).transpose()?.unwrap_or(config.thickness);
        let alpha = attrs.get("stroke-opacity").map(|s| s.parse()).transpose()?.unwrap_or(1.0);
        let fill_alpha = attrs.get("fill-opacity").map(|s| s.parse()).transpose()?.unwrap_or(1.0);

        let mut builder = SvgPathBuilder::new();

        parse_path(path, &mut builder)?;

        Ok(Path::from_parts(
            builder.into_path(),
            Style {
                stroke: color.map(|c| Stroke {
                    color: c.with_float_alpha(alpha),
                    size: thickness,
                }),
                fill: fill.map(|c| c.with_float_alpha(fill_alpha)),
            },
        ))
    }
}

trait Deserialize {
    fn deserialize(attributes: HashMap<String, String>, config: Config) -> Result<Box<dyn ShapeStored>>;
}

impl Deserialize for Path {
    fn deserialize(attributes: HashMap<String, String>, config: Config) -> Result<Box<dyn ShapeStored>> {
        let styleattr = attributes.get("style").ok_or_else(|| Error::MissingAttribute("style".into()))?;
        let dattr = attributes.get("d").ok_or_else(|| Error::MissingAttribute("d".into()))?;

        Ok(Box::new(Path::from_xml_attributes(styleattr, dattr, config)?))
    }
}

impl Deserialize for Ellipse {
    fn deserialize(attributes: HashMap<String, String>, config: Config) -> Result<Box<dyn ShapeStored>> {
        let cx = attributes.get("cx").ok_or_else(|| Error::MissingAttribute("cx".into()))?;
        let cy = attributes.get("cy").ok_or_else(|| Error::MissingAttribute("cy".into()))?;
        let rx = attributes.get("rx").ok_or_else(|| Error::MissingAttribute("rx".into()))?;
        let ry = attributes.get("ry").ok_or_else(|| Error::MissingAttribute("ry".into()))?;
        let angle= attributes.get("transform").map(|s| s.as_str()).unwrap_or("rotate(0)");
        let styleattr = attributes.get("style").ok_or_else(|| Error::MissingAttribute("style".into()))?;

        let attrs = css_attrs_as_hashmap(styleattr);

        let color: Option<Color> = attrs.get("stroke").map(|s| parse_color(s)).transpose()?.flatten();
        let fill: Option<Color> = attrs.get("fill").map(|s| parse_color(s)).transpose()?.flatten();
        let thickness = attrs.get("stroke-width").map(|s| s.parse()).transpose()?.unwrap_or(config.thickness);
        let alpha = attrs.get("stroke-opacity").map(|s| s.parse()).transpose()?.unwrap_or(1.0);
        let fill_alpha = attrs.get("fill-opacity").map(|s| s.parse()).transpose()?.unwrap_or(1.0);

        let angle: Angle = angle.parse()?;

        let center = Vec2D::new_world(cx.parse()?, cy.parse()?);
        let rx: f64 = rx.parse()?;
        let ry: f64 = ry.parse()?;

        Ok(Box::new(Ellipse::from_parts(
            center,
            rx,
            ry,
            angle,
            Style {
                stroke: color.map(|c| Stroke {
                    color: c.with_float_alpha(alpha),
                    size: thickness,
                }),
                fill: fill.map(|c| c.with_float_alpha(fill_alpha)),
            },
        )))
    }
}

impl Storage {
    pub fn from_svg(svg: &str, config: Config) -> Result<Storage> {
        let parser = EventReader::from_str(svg);
        let mut storage = Storage::new();

        for e in parser {
            match e {
                Ok(XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "path" => {
                    storage.add(Path::deserialize(xml_attrs_as_hashmap(attributes), config)?);
                }
                Ok(XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "ellipse" => {
                    storage.add(Ellipse::deserialize(xml_attrs_as_hashmap(attributes), config)?);
                }
                Err(e) => {
                    return Err(e.into());
                }
                _ => {}
            }
        }

        Ok(storage)
    }
}

#[cfg(test)]
mod tests {
    use crate::path_command::PathCommand;
    use crate::draw_commands::DrawCommand;

    use super::*;

    #[test]
    fn test_from_svg() {
        let svg_data = include_str!("../res/simple_file_load.svg");
        let storage = Storage::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(storage.shape_count(), 1);
    }

    #[test]
    fn test_from_custom_svg() {
        let svg_data = include_str!("../res/serialize_test.svg");
        let storage = Storage::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(storage.shape_count(), 1);
    }

    #[test]
    fn test_can_deserialize_circle() {
        let svg_data = include_str!("../res/circle.svg");
        let storage = Storage::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(storage.shape_count(), 1);
    }

    #[test]
    fn test_can_deserialize_ellipse() {
        let svg_data = include_str!("../res/ellipse.svg");
        let storage = Storage::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(storage.shape_count(), 1);
    }

    #[test]
    fn test_point_bug() {
        let svg_data = include_str!("../res/bug_opening.svg");
        let storage = Storage::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(storage.shape_count(), 4);
    }

    #[test]
    fn test_line_from_xml_attributes() {
        let line = Path::from_xml_attributes("fill:none;stroke-width:3;stroke-linecap:round;stroke-linejoin:round;stroke:rgb(53.90625%,88.28125%,20.3125%);stroke-opacity:1;stroke-miterlimit:10;", "M 147.570312 40.121094 L 146.9375 40.121094 L 145.296875 40.460938 L 142.519531 41.710938 L 139.613281 43.304688 L 138.097656 44.894531 L 137.339844 46.714844 L 138.097656 47.621094 L 139.992188 47.851562 L 142.898438 47.621094 L 146.179688 47.167969 L 150.097656 47.964844 L 151.738281 49.667969 L 152.496094 51.714844 L 152.875 53.191406 ", Default::default()).unwrap();

        if let DrawCommand::Path {
            commands, style, ..
        } = line.draw_commands() {
            assert_eq!(style.stroke.unwrap().color, Color::from_float_rgb(0.5390625, 0.8828125, 0.203125));
            assert_eq!(commands, vec![
                PathCommand::MoveTo(Vec2D::new_world(147.570312, 40.121094)),
                PathCommand::LineTo(Vec2D::new_world(146.9375, 40.121094)),
                PathCommand::LineTo(Vec2D::new_world(145.296875, 40.460938)),
                PathCommand::LineTo(Vec2D::new_world(142.519531, 41.710938)),
                PathCommand::LineTo(Vec2D::new_world(139.613281, 43.304688)),
                PathCommand::LineTo(Vec2D::new_world(138.097656, 44.894531)),
                PathCommand::LineTo(Vec2D::new_world(137.339844, 46.714844)),
                PathCommand::LineTo(Vec2D::new_world(138.097656, 47.621094)),
                PathCommand::LineTo(Vec2D::new_world(139.992188, 47.851562)),
                PathCommand::LineTo(Vec2D::new_world(142.898438, 47.621094)),
                PathCommand::LineTo(Vec2D::new_world(146.179688, 47.167969)),
                PathCommand::LineTo(Vec2D::new_world(150.097656, 47.964844)),
                PathCommand::LineTo(Vec2D::new_world(151.738281, 49.667969)),
                PathCommand::LineTo(Vec2D::new_world(152.496094, 51.714844)),
                PathCommand::LineTo(Vec2D::new_world(152.875, 53.191406)),
            ]);
            assert_eq!(style.stroke.unwrap().size, 3.0);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_point() {
        let p = " 152.496094 51.714844 ";

        assert_eq!(p.parse::<Vec2DWorld>().unwrap(), Vec2D::new_world(152.496094, 51.714844));
    }

    #[test]
    fn test_parse_alpha() {
        let line = Path::from_xml_attributes("fill:none;stroke-width:3;stroke-linecap:round;stroke-linejoin:round;stroke:#FF0000;stroke-opacity:0.8;stroke-miterlimit:10;", "M 10 10 L 20 20", Default::default()).unwrap();

        assert_eq!(line.style().stroke.unwrap().color, Color::red().with_float_alpha(0.8));
    }

    #[test]
    fn can_deserialize_rotated_ellipse() {
        let attrs: HashMap<String, String> = vec![
            ("cx".into(), "20.4".into()),
            ("cy".into(), "-30.5".into()),
            ("rx".into(), "5.6".into()),
            ("ry".into(), "3.5".into()),
            ("transform".into(), "rotate(34.5)".into()),
            ("style".into(), "stroke:#cabada;stroke-width:3.5;stroke-opacity:0.8".into()),
        ].into_iter().collect();

        let deserialized = Ellipse::deserialize(attrs, Default::default()).unwrap();

        match deserialized.draw_commands() {
            DrawCommand::Ellipse { ellipse: e, style } => {
                assert_eq!(e.center, Vec2D::new_world(20.4, -30.5));
                assert_eq!(e.semimajor, 5.6);
                assert_eq!(e.semiminor, 3.5);
                assert_eq!(e.angle.degrees(), 34.5);
                assert_eq!(style.stroke.unwrap().color, Color::from_int_rgb(0xca, 0xba, 0xda).with_float_alpha(0.8));
                assert_eq!(style.stroke.unwrap().size, 3.5);
            },
            _ => panic!()
        }
    }

    #[test]
    fn can_deserialize_ellipse_serialization_test() {
        let svg_data = include_str!("../res/serialize_ellipse.svg");

        Storage::from_svg(svg_data, Default::default()).unwrap();
    }

    #[test]
    fn fill_and_stroke_can_be_none_in_path() {
        let svg_data = include_str!("../res/color_fill_none_path.svg");
        let storage = Storage::from_svg(svg_data, Default::default()).unwrap();

        let commands = storage.draw_commands(storage.get_bounds().unwrap());
        let command = &commands[0];

        assert_eq!(command.color(), None);
        assert_eq!(command.fill(), None);
    }

    #[test]
    fn fill_and_stroke_can_be_none_in_ellipse() {
        let svg_data = include_str!("../res/color_fill_none_ellipse.svg");
        let storage = Storage::from_svg(svg_data, Default::default()).unwrap();

        let commands = storage.draw_commands(storage.get_bounds().unwrap());
        let command = &commands[0];

        assert_eq!(command.color(), None);
        assert_eq!(command.fill(), None);
    }
}
