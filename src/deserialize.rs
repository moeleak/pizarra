use std::fmt;
use std::collections::HashMap;
use std::str::FromStr;

use xml::reader::{EventReader, XmlEvent, Error as XmlError};
use regex::Regex;
use lazy_static::lazy_static;

use crate::storage::Storage;
use crate::shape::Line;
use crate::color::Color;
use crate::consts::DEFAULT_THICKNESS;
use crate::point::Point;

lazy_static! {
    static ref CSS_COLOR_REGEX: Regex = Regex::new(r"(?x)#
        (?P<r>[\dA-Fa-f]{2})
        (?P<g>[\dA-Fa-f]{2})
        (?P<b>[\dA-Fa-f]{2})
    ").unwrap();
    static ref COLOR_REGEX: Regex = Regex::new(r"(?x)rgb\(
        (?P<r>\d+(\.\d+)?)
        %\s*,\s*
        (?P<g>\d+(\.\d+))?
        %\s*,\s*
        (?P<b>\d+(\.\d+))?
    %\s*\)").unwrap();
    static ref POINT_REGEX: Regex = Regex::new(r"(?x)
        (?P<x>-?\d+(\.\d+)?)
        \s+
        (?P<y>-?\d+(\.\d+)?)
    ").unwrap();
}

#[derive(Debug)]
pub enum Error {
    Xml(XmlError),
    PathWithNoDAttr,
    ColorDoesntMatchRegex,
    ParseFloatError(std::num::ParseFloatError),
    PointDoesntMatchRegex,
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

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = COLOR_REGEX.captures(s) {
            Ok(Color::from_rgb(
                captures["r"].parse::<f64>()?/ 100.0,
                captures["g"].parse::<f64>()?/100.0,
                captures["b"].parse::<f64>()?/100.0,
            ))
        } else {
            if let Some(captures) = CSS_COLOR_REGEX.captures(s) {
                Ok(Color::from_rgb(
                    u32::from_str_radix(&captures["r"], 16).unwrap() as f64 / 255.0,
                    u32::from_str_radix(&captures["g"], 16).unwrap() as f64 / 255.0,
                    u32::from_str_radix(&captures["b"], 16).unwrap() as f64 / 255.0,
                ))
            } else {
                Err(Error::ColorDoesntMatchRegex)
            }
        }
    }
}

impl FromStr for Point {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = POINT_REGEX.captures(s) {
            Ok(Point::new(
                captures["x"].parse::<f64>()?,
                captures["y"].parse::<f64>()?,
            ))
        } else {
            Err(Error::PointDoesntMatchRegex)
        }
    }
}

impl Line {
    pub fn from_xml_attributes(style: &str, path: &str) -> Result<Line, Error> {
        let attrs: HashMap<_, _> = style.split(';').filter_map(|s| {
            let pieces: Vec<_> = s.split(':').collect();

            if pieces.len() != 2 {
                return None
            }

            Some((pieces[0].trim(), pieces[1].trim()))
        }).collect();

        let color = if let Some(color_str) = attrs.get("stroke") {
                color_str.parse()?
            } else {
                Color::white()
            };

        let thickness = if let Some(thickness_str) = attrs.get("stroke-width") {
                thickness_str.parse()?
            } else {
                DEFAULT_THICKNESS
            };

        let points = path
            .split('L')
            .map(|point_str| point_str.parse())
            .collect::<Result<Vec<Point>, _>>()?;

        Ok(Line::with_params(color, points, thickness))
    }
}

impl Storage {
    pub fn from_svg(svg: &str) -> Result<Storage, Error> {
        let parser = EventReader::from_str(svg);
        let mut storage = Storage::new();

        for e in parser {
            match e {
                Ok(XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "path" => {
                    if let Some(styleattr) = attributes.iter().find(|a| a.name.local_name == "style") {
                        if let Some(dattr) = attributes.iter().find(|a| a.name.local_name == "d") {
                            storage.add(Box::new(Line::from_xml_attributes(&styleattr.value, &dattr.value)?));
                        }
                    } else {
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
                _ => {}
            }
        }

        storage.flush();

        Ok(storage)
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::storage::Storage;
    use crate::shape::{Line, DrawCommand, ShapeTrait};
    use crate::color::Color;
    use crate::point::Point;

    #[test]
    fn test_from_svg() {
        let svg_data = include_str!("../res/simple_file_load.svg");
        let storage = Storage::from_svg(svg_data).unwrap();

        assert_eq!(storage.shape_count(), 1);
    }

    #[test]
    fn test_from_custom_svg() {
        let svg_data = include_str!("../res/serialize_test.svg");
        let storage = Storage::from_svg(svg_data).unwrap();

        assert_eq!(storage.shape_count(), 1);
    }

    #[test]
    fn test_point_bug() {
        let svg_data = include_str!("../res/bug_opening.svg");
        let storage = Storage::from_svg(svg_data).unwrap();

        assert_eq!(storage.shape_count(), 4);
    }

    #[test]
    fn test_line_from_xml_attributes() {
        let line = Line::from_xml_attributes("fill:none;stroke-width:3;stroke-linecap:round;stroke-linejoin:round;stroke:rgb(53.90625%,88.28125%,20.3125%);stroke-opacity:1;stroke-miterlimit:10;", "M 147.570312 40.121094 L 146.9375 40.121094 L 145.296875 40.460938 L 142.519531 41.710938 L 139.613281 43.304688 L 138.097656 44.894531 L 137.339844 46.714844 L 138.097656 47.621094 L 139.992188 47.851562 L 142.898438 47.621094 L 146.179688 47.167969 L 150.097656 47.964844 L 151.738281 49.667969 L 152.496094 51.714844 L 152.875 53.191406 ").unwrap();

        if let DrawCommand::Line {
            color,
            line,
            thickness,
        } = line.draw_commands() {
            assert_eq!(color, Color::from_rgb(0.5390625, 0.8828125, 0.203125));
            assert_eq!(line, Rc::new(vec![Point::new(147.570312, 40.121094), Point::new(146.9375, 40.121094), Point::new(145.296875, 40.460938), Point::new(142.519531, 41.710938), Point::new(139.613281, 43.304688), Point::new(138.097656, 44.894531), Point::new(137.339844, 46.714844), Point::new(138.097656, 47.621094), Point::new(139.992188, 47.851562), Point::new(142.898438, 47.621094), Point::new(146.179688, 47.167969), Point::new(150.097656, 47.964844), Point::new(151.738281, 49.667969), Point::new(152.496094, 51.714844), Point::new(152.875, 53.191406)]));
            assert_eq!(thickness, 3.0);
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_color() {
        let color: Color = "rgb(53.90625%,88.28125%,20.3125%)".parse().unwrap();

        assert_eq!(color, Color::from_rgb(0.5390625, 0.8828125, 0.203125));
    }

    #[test]
    fn test_parse_css_color() {
        let color: Color = "#FF2030".parse().unwrap();

        assert_eq!(color, Color::from_rgb(1.0, 0.12549019607843137, 0.18823529411764706));

        let color: Color = "#FF2003".parse().unwrap();

        assert_eq!(color, Color::from_rgb(1.0, 0.12549019607843137, 0.011764705882352941));
    }

    #[test]
    fn test_parse_point() {
        let p = " 152.496094 51.714844 ";

        assert_eq!(p.parse::<Point>().unwrap(), Point::new(152.496094, 51.714844));
    }
}
