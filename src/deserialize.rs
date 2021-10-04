use std::fmt;
use std::collections::HashMap;
use std::str::FromStr;

use xml::{
    attribute::OwnedAttribute, reader::{EventReader, XmlEvent, Error as XmlError}
};
use regex::Regex;
use lazy_static::lazy_static;

use crate::storage::Storage;
use crate::shape::{ShapeStored, path::Path, ellipse::Ellipse};
use crate::color::Color;
use crate::consts::DEFAULT_THICKNESS;
use crate::point::{Point, Vec2DDrawing};
use crate::geom::Radians;

mod path;

use path::{parse_path, PathBuilder as SvgPathBuilder};

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
    static ref CSS_COLORS: HashMap<String, Color> = vec![
        ("aliceblue".into(), Color::from_int_rgb(0xF0, 0xF8, 0xFF)),
        ("antiquewhite".into(), Color::from_int_rgb(0xFA, 0xEB, 0xD7)),
        ("aqua".into(), Color::from_int_rgb(0x00, 0xFF, 0xFF)),
        ("aquamarine".into(), Color::from_int_rgb(0x7F, 0xFF, 0xD4)),
        ("azure".into(), Color::from_int_rgb(0xF0, 0xFF, 0xFF)),
        ("beige".into(), Color::from_int_rgb(0xF5, 0xF5, 0xDC)),
        ("bisque".into(), Color::from_int_rgb(0xFF, 0xE4, 0xC4)),
        ("black".into(), Color::from_int_rgb(0x00, 0x00, 0x00)),
        ("blanchedalmond".into(), Color::from_int_rgb(0xFF, 0xEB, 0xCD)),
        ("blue".into(), Color::from_int_rgb(0x00, 0x00, 0xFF)),
        ("blueviolet".into(), Color::from_int_rgb(0x8A, 0x2B, 0xE2)),
        ("brown".into(), Color::from_int_rgb(0xA5, 0x2A, 0x2A)),
        ("burlywood".into(), Color::from_int_rgb(0xDE, 0xB8, 0x87)),
        ("cadetblue".into(), Color::from_int_rgb(0x5F, 0x9E, 0xA0)),
        ("chartreuse".into(), Color::from_int_rgb(0x7F, 0xFF, 0x00)),
        ("chocolate".into(), Color::from_int_rgb(0xD2, 0x69, 0x1E)),
        ("coral".into(), Color::from_int_rgb(0xFF, 0x7F, 0x50)),
        ("cornflowerblue".into(), Color::from_int_rgb(0x64, 0x95, 0xED)),
        ("cornsilk".into(), Color::from_int_rgb(0xFF, 0xF8, 0xDC)),
        ("crimson".into(), Color::from_int_rgb(0xDC, 0x14, 0x3C)),
        ("cyan".into(), Color::from_int_rgb(0x00, 0xFF, 0xFF)),
        ("darkblue".into(), Color::from_int_rgb(0x00, 0x00, 0x8B)),
        ("darkcyan".into(), Color::from_int_rgb(0x00, 0x8B, 0x8B)),
        ("darkgoldenrod".into(), Color::from_int_rgb(0xB8, 0x86, 0x0B)),
        ("darkgray".into(), Color::from_int_rgb(0xA9, 0xA9, 0xA9)),
        ("darkgreen".into(), Color::from_int_rgb(0x00, 0x64, 0x00)),
        ("darkgrey".into(), Color::from_int_rgb(0xA9, 0xA9, 0xA9)),
        ("darkkhaki".into(), Color::from_int_rgb(0xBD, 0xB7, 0x6B)),
        ("darkmagenta".into(), Color::from_int_rgb(0x8B, 0x00, 0x8B)),
        ("darkolivegreen".into(), Color::from_int_rgb(0x55, 0x6B, 0x2F)),
        ("darkorange".into(), Color::from_int_rgb(0xFF, 0x8C, 0x00)),
        ("darkorchid".into(), Color::from_int_rgb(0x99, 0x32, 0xCC)),
        ("darkred".into(), Color::from_int_rgb(0x8B, 0x00, 0x00)),
        ("darksalmon".into(), Color::from_int_rgb(0xE9, 0x96, 0x7A)),
        ("darkseagreen".into(), Color::from_int_rgb(0x8F, 0xBC, 0x8F)),
        ("darkslateblue".into(), Color::from_int_rgb(0x48, 0x3D, 0x8B)),
        ("darkslategray".into(), Color::from_int_rgb(0x2F, 0x4F, 0x4F)),
        ("darkslategrey".into(), Color::from_int_rgb(0x2F, 0x4F, 0x4F)),
        ("darkturquoise".into(), Color::from_int_rgb(0x00, 0xCE, 0xD1)),
        ("darkviolet".into(), Color::from_int_rgb(0x94, 0x00, 0xD3)),
        ("deeppink".into(), Color::from_int_rgb(0xFF, 0x14, 0x93)),
        ("deepskyblue".into(), Color::from_int_rgb(0x00, 0xBF, 0xFF)),
        ("dimgray".into(), Color::from_int_rgb(0x69, 0x69, 0x69)),
        ("dimgrey".into(), Color::from_int_rgb(0x69, 0x69, 0x69)),
        ("dodgerblue".into(), Color::from_int_rgb(0x1E, 0x90, 0xFF)),
        ("firebrick".into(), Color::from_int_rgb(0xB2, 0x22, 0x22)),
        ("floralwhite".into(), Color::from_int_rgb(0xFF, 0xFA, 0xF0)),
        ("forestgreen".into(), Color::from_int_rgb(0x22, 0x8B, 0x22)),
        ("fuchsia".into(), Color::from_int_rgb(0xFF, 0x00, 0xFF)),
        ("gainsboro".into(), Color::from_int_rgb(0xDC, 0xDC, 0xDC)),
        ("ghostwhite".into(), Color::from_int_rgb(0xF8, 0xF8, 0xFF)),
        ("gold".into(), Color::from_int_rgb(0xFF, 0xD7, 0x00)),
        ("goldenrod".into(), Color::from_int_rgb(0xDA, 0xA5, 0x20)),
        ("gray".into(), Color::from_int_rgb(0x80, 0x80, 0x80)),
        ("green".into(), Color::from_int_rgb(0x00, 0x80, 0x00)),
        ("greenyellow".into(), Color::from_int_rgb(0xAD, 0xFF, 0x2F)),
        ("grey".into(), Color::from_int_rgb(0x80, 0x80, 0x80)),
        ("honeydew".into(), Color::from_int_rgb(0xF0, 0xFF, 0xF0)),
        ("hotpink".into(), Color::from_int_rgb(0xFF, 0x69, 0xB4)),
        ("indianred".into(), Color::from_int_rgb(0xCD, 0x5C, 0x5C)),
        ("indigo".into(), Color::from_int_rgb(0x4B, 0x00, 0x82)),
        ("ivory".into(), Color::from_int_rgb(0xFF, 0xFF, 0xF0)),
        ("khaki".into(), Color::from_int_rgb(0xF0, 0xE6, 0x8C)),
        ("lavender".into(), Color::from_int_rgb(0xE6, 0xE6, 0xFA)),
        ("lavenderblush".into(), Color::from_int_rgb(0xFF, 0xF0, 0xF5)),
        ("lawngreen".into(), Color::from_int_rgb(0x7C, 0xFC, 0x00)),
        ("lemonchiffon".into(), Color::from_int_rgb(0xFF, 0xFA, 0xCD)),
        ("lightblue".into(), Color::from_int_rgb(0xAD, 0xD8, 0xE6)),
        ("lightcoral".into(), Color::from_int_rgb(0xF0, 0x80, 0x80)),
        ("lightcyan".into(), Color::from_int_rgb(0xE0, 0xFF, 0xFF)),
        ("lightgoldenrodyellow".into(), Color::from_int_rgb(0xFA, 0xFA, 0xD2)),
        ("lightgreen".into(), Color::from_int_rgb(0x90, 0xEE, 0x90)),
        ("lightgrey".into(), Color::from_int_rgb(0xD3, 0xD3, 0xD3)),
        ("lightpink".into(), Color::from_int_rgb(0xFF, 0xB6, 0xC1)),
        ("lightsalmon".into(), Color::from_int_rgb(0xFF, 0xA0, 0x7A)),
        ("lightseagreen".into(), Color::from_int_rgb(0x20, 0xB2, 0xAA)),
        ("lightskyblue".into(), Color::from_int_rgb(0x87, 0xCE, 0xFA)),
        ("lightslategray".into(), Color::from_int_rgb(0x77, 0x88, 0x99)),
        ("lightslategrey".into(), Color::from_int_rgb(0x77, 0x88, 0x99)),
        ("lightsteelblue".into(), Color::from_int_rgb(0xB0, 0xC4, 0xDE)),
        ("lightyellow".into(), Color::from_int_rgb(0xFF, 0xFF, 0xE0)),
        ("lime".into(), Color::from_int_rgb(0x00, 0xFF, 0x00)),
        ("limegreen".into(), Color::from_int_rgb(0x32, 0xCD, 0x32)),
        ("linen".into(), Color::from_int_rgb(0xFA, 0xF0, 0xE6)),
        ("magenta".into(), Color::from_int_rgb(0xFF, 0x00, 0xFF)),
        ("maroon".into(), Color::from_int_rgb(0x80, 0x00, 0x00)),
        ("mediumaquamarine".into(), Color::from_int_rgb(0x66, 0xCD, 0xAA)),
        ("mediumblue".into(), Color::from_int_rgb(0x00, 0x00, 0xCD)),
        ("mediumorchid".into(), Color::from_int_rgb(0xBA, 0x55, 0xD3)),
        ("mediumpurple".into(), Color::from_int_rgb(0x93, 0x70, 0xDB)),
        ("mediumseagreen".into(), Color::from_int_rgb(0x3C, 0xB3, 0x71)),
        ("mediumslateblue".into(), Color::from_int_rgb(0x7B, 0x68, 0xEE)),
        ("mediumspringgreen".into(), Color::from_int_rgb(0x00, 0xFA, 0x9A)),
        ("mediumturquoise".into(), Color::from_int_rgb(0x48, 0xD1, 0xCC)),
        ("mediumvioletred".into(), Color::from_int_rgb(0xC7, 0x15, 0x85)),
        ("midnightblue".into(), Color::from_int_rgb(0x19, 0x19, 0x70)),
        ("mintcream".into(), Color::from_int_rgb(0xF5, 0xFF, 0xFA)),
        ("mistyrose".into(), Color::from_int_rgb(0xFF, 0xE4, 0xE1)),
        ("moccasin".into(), Color::from_int_rgb(0xFF, 0xE4, 0xB5)),
        ("navajowhite".into(), Color::from_int_rgb(0xFF, 0xDE, 0xAD)),
        ("navy".into(), Color::from_int_rgb(0x00, 0x00, 0x80)),
        ("oldlace".into(), Color::from_int_rgb(0xFD, 0xF5, 0xE6)),
        ("olive".into(), Color::from_int_rgb(0x80, 0x80, 0x00)),
        ("olivedrab".into(), Color::from_int_rgb(0x6B, 0x8E, 0x23)),
        ("orange".into(), Color::from_int_rgb(0xFF, 0xA5, 0x00)),
        ("orangered".into(), Color::from_int_rgb(0xFF, 0x45, 0x00)),
        ("orchid".into(), Color::from_int_rgb(0xDA, 0x70, 0xD6)),
        ("palegoldenrod".into(), Color::from_int_rgb(0xEE, 0xE8, 0xAA)),
        ("palegreen".into(), Color::from_int_rgb(0x98, 0xFB, 0x98)),
        ("paleturquoise".into(), Color::from_int_rgb(0xAF, 0xEE, 0xEE)),
        ("palevioletred".into(), Color::from_int_rgb(0xDB, 0x70, 0x93)),
        ("papayawhip".into(), Color::from_int_rgb(0xFF, 0xEF, 0xD5)),
        ("peachpuff".into(), Color::from_int_rgb(0xFF, 0xDA, 0xB9)),
        ("peru".into(), Color::from_int_rgb(0xCD, 0x85, 0x3F)),
        ("pink".into(), Color::from_int_rgb(0xFF, 0xC0, 0xCB)),
        ("plum".into(), Color::from_int_rgb(0xDD, 0xA0, 0xDD)),
        ("powderblue".into(), Color::from_int_rgb(0xB0, 0xE0, 0xE6)),
        ("purple".into(), Color::from_int_rgb(0x80, 0x00, 0x80)),
        ("red".into(), Color::from_int_rgb(0xFF, 0x00, 0x00)),
        ("rosybrown".into(), Color::from_int_rgb(0xBC, 0x8F, 0x8F)),
        ("royalblue".into(), Color::from_int_rgb(0x41, 0x69, 0xE1)),
        ("saddlebrown".into(), Color::from_int_rgb(0x8B, 0x45, 0x13)),
        ("salmon".into(), Color::from_int_rgb(0xFA, 0x80, 0x72)),
        ("sandybrown".into(), Color::from_int_rgb(0xF4, 0xA4, 0x60)),
        ("seagreen".into(), Color::from_int_rgb(0x2E, 0x8B, 0x57)),
        ("seashell".into(), Color::from_int_rgb(0xFF, 0xF5, 0xEE)),
        ("sienna".into(), Color::from_int_rgb(0xA0, 0x52, 0x2D)),
        ("silver".into(), Color::from_int_rgb(0xC0, 0xC0, 0xC0)),
        ("skyblue".into(), Color::from_int_rgb(0x87, 0xCE, 0xEB)),
        ("slateblue".into(), Color::from_int_rgb(0x6A, 0x5A, 0xCD)),
        ("slategray".into(), Color::from_int_rgb(0x70, 0x80, 0x90)),
        ("slategrey".into(), Color::from_int_rgb(0x70, 0x80, 0x90)),
        ("snow".into(), Color::from_int_rgb(0xFF, 0xFA, 0xFA)),
        ("springgreen".into(), Color::from_int_rgb(0x00, 0xFF, 0x7F)),
        ("steelblue".into(), Color::from_int_rgb(0x46, 0x82, 0xB4)),
        ("tan".into(), Color::from_int_rgb(0xD2, 0xB4, 0x8C)),
        ("teal".into(), Color::from_int_rgb(0x00, 0x80, 0x80)),
        ("thistle".into(), Color::from_int_rgb(0xD8, 0xBF, 0xD8)),
        ("tomato".into(), Color::from_int_rgb(0xFF, 0x63, 0x47)),
        ("turquoise".into(), Color::from_int_rgb(0x40, 0xE0, 0xD0)),
        ("violet".into(), Color::from_int_rgb(0xEE, 0x82, 0xEE)),
        ("wheat".into(), Color::from_int_rgb(0xF5, 0xDE, 0xB3)),
        ("white".into(), Color::from_int_rgb(0xFF, 0xFF, 0xFF)),
        ("whitesmoke".into(), Color::from_int_rgb(0xF5, 0xF5, 0xF5)),
        ("yellow".into(), Color::from_int_rgb(0xFF, 0xFF, 0x00)),
        ("yellowgreen".into(), Color::from_int_rgb(0x9A, 0xCD, 0x32)),
    ].into_iter().collect();
}

#[derive(Debug)]
pub enum Error {
    Xml(XmlError),
    PathWithNoDAttr,
    CouldntUnderstandColor(String),
    ParseFloatError(std::num::ParseFloatError),
    PointDoesntMatchRegex,
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

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = CSS_COLORS.get(&s.to_lowercase()) {
            Ok(c.clone())
        } else {
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
                    Err(Error::CouldntUnderstandColor(s.into()))
                }
            }
        }
    }
}

impl FromStr for Vec2DDrawing {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = POINT_REGEX.captures(s) {
            Ok(Vec2DDrawing::new(
                captures["x"].parse::<f64>()?,
                captures["y"].parse::<f64>()?,
            ))
        } else {
            Err(Error::PointDoesntMatchRegex)
        }
    }
}

impl FromStr for Radians {
    type Err = Error;

    fn from_str(source: &str) -> Result<Radians, Self::Err> {
        todo!()
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

impl Path {
    pub fn from_xml_attributes(style: &str, path: &str) -> Result<Path, Error> {
        let attrs = css_attrs_as_hashmap(style);

        let color = if let Some(color_str) = attrs.get("stroke") {
                if let Ok(c) = color_str.parse() {
                    c
                } else {
                    Color::white()
                }
            } else {
                Color::white()
            };

        let thickness = if let Some(thickness_str) = attrs.get("stroke-width") {
                thickness_str.parse()?
            } else {
                DEFAULT_THICKNESS
            };

        let alpha = if let Some(alpha_str) = attrs.get("stroke-opacity") {
                alpha_str.parse()?
            } else {
                1.0
            };

        let mut builder = SvgPathBuilder::new();

        parse_path(path, &mut builder)?;

        Ok(Path::from_parts(builder.into_path(), color.with_alpha(alpha), thickness))
    }
}

impl Ellipse {
    pub fn from_xml_attributes(cx: &str, cy: &str, rx: &str, ry: &str, style: &str) -> Result<Ellipse, Error> {
        let attrs = css_attrs_as_hashmap(style);

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

        let alpha = if let Some(alpha_str) = attrs.get("stroke-opacity") {
                alpha_str.parse()?
            } else {
                1.0
            };

        let angle: Radians = if let Some(angle_str) = attrs.get("transform") {
            angle_str.parse()?
        } else {
            Default::default()
        };

        let center = Vec2DDrawing::new(cx.parse()?, cy.parse()?);
        let rx: f64 = rx.parse()?;
        let ry: f64 = ry.parse()?;

        Ok(Ellipse::from_parts(
            center,
            rx,
            ry,
            angle,
            color.with_alpha(alpha),
            thickness,
        ))
    }
}

trait Deserialize {
    fn deserialize(attributes: Vec<OwnedAttribute>) -> Result<Box<dyn ShapeStored>, Error>;
}

impl Deserialize for Path{
    fn deserialize(attributes: Vec<OwnedAttribute>) -> Result<Box<dyn ShapeStored>, Error> {
        let styleattr = attributes.iter().find(|a| a.name.local_name == "style").ok_or(Error::MissingAttribute("style".into()))?;
        let dattr = attributes.iter().find(|a| a.name.local_name == "d").ok_or(Error::MissingAttribute("d".into()))?;

        Ok(Box::new(Path::from_xml_attributes(&styleattr.value, &dattr.value)?))
    }
}

impl Deserialize for Ellipse {
    fn deserialize(attributes: Vec<OwnedAttribute>) -> Result<Box<dyn ShapeStored>, Error> {
        let cx = attributes.iter().find(|a| a.name.local_name == "cx").ok_or(Error::MissingAttribute("style".into()))?;
        let cy = attributes.iter().find(|a| a.name.local_name == "cy").ok_or(Error::MissingAttribute("style".into()))?;
        let rx = attributes.iter().find(|a| a.name.local_name == "rx").ok_or(Error::MissingAttribute("style".into()))?;
        let ry = attributes.iter().find(|a| a.name.local_name == "ry").ok_or(Error::MissingAttribute("style".into()))?;
        let styleattr = attributes.iter().find(|a| a.name.local_name == "style").ok_or(Error::MissingAttribute("style".into()))?;

        Ok(Box::new(Ellipse::from_xml_attributes(&cx.value, &cy.value, &rx.value, &ry.value, &styleattr.value)?))
    }
}

impl Storage {
    pub fn from_svg(svg: &str) -> Result<Storage, Error> {
        let parser = EventReader::from_str(svg);
        let mut storage = Storage::new();

        for e in parser {
            match e {
                Ok(XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "path" => {
                    storage.add(Path::deserialize(attributes)?);
                }
                Ok(XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "ellipse" => {
                    storage.add(Ellipse::deserialize(attributes)?);
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
    use crate::shape::path::PathCommand;
    use crate::draw_commands::DrawCommand;

    use super::*;

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
    fn test_can_deserialize_circle() {
        let svg_data = include_str!("../res/circle.svg");
        let storage = Storage::from_svg(svg_data).unwrap();

        assert_eq!(storage.shape_count(), 1);
    }

    #[test]
    fn test_can_deserialize_ellipse() {
        let svg_data = include_str!("../res/ellipse.svg");
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
        let line = Path::from_xml_attributes("fill:none;stroke-width:3;stroke-linecap:round;stroke-linejoin:round;stroke:rgb(53.90625%,88.28125%,20.3125%);stroke-opacity:1;stroke-miterlimit:10;", "M 147.570312 40.121094 L 146.9375 40.121094 L 145.296875 40.460938 L 142.519531 41.710938 L 139.613281 43.304688 L 138.097656 44.894531 L 137.339844 46.714844 L 138.097656 47.621094 L 139.992188 47.851562 L 142.898438 47.621094 L 146.179688 47.167969 L 150.097656 47.964844 L 151.738281 49.667969 L 152.496094 51.714844 L 152.875 53.191406 ").unwrap();

        if let DrawCommand::Path {
            color,
            commands,
            thickness,
        } = line.draw_commands() {
            assert_eq!(color, Color::from_rgb(0.5390625, 0.8828125, 0.203125));
            assert_eq!(commands, vec![
                PathCommand::MoveTo(Vec2DDrawing::new(147.570312, 40.121094)),
                PathCommand::LineTo(Vec2DDrawing::new(146.9375, 40.121094)),
                PathCommand::LineTo(Vec2DDrawing::new(145.296875, 40.460938)),
                PathCommand::LineTo(Vec2DDrawing::new(142.519531, 41.710938)),
                PathCommand::LineTo(Vec2DDrawing::new(139.613281, 43.304688)),
                PathCommand::LineTo(Vec2DDrawing::new(138.097656, 44.894531)),
                PathCommand::LineTo(Vec2DDrawing::new(137.339844, 46.714844)),
                PathCommand::LineTo(Vec2DDrawing::new(138.097656, 47.621094)),
                PathCommand::LineTo(Vec2DDrawing::new(139.992188, 47.851562)),
                PathCommand::LineTo(Vec2DDrawing::new(142.898438, 47.621094)),
                PathCommand::LineTo(Vec2DDrawing::new(146.179688, 47.167969)),
                PathCommand::LineTo(Vec2DDrawing::new(150.097656, 47.964844)),
                PathCommand::LineTo(Vec2DDrawing::new(151.738281, 49.667969)),
                PathCommand::LineTo(Vec2DDrawing::new(152.496094, 51.714844)),
                PathCommand::LineTo(Vec2DDrawing::new(152.875, 53.191406)),
            ]);
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

        assert_eq!(p.parse::<Vec2DDrawing>().unwrap(), Vec2DDrawing::new(152.496094, 51.714844));
    }

    #[test]
    fn test_parse_alpha() {
        let line = Path::from_xml_attributes("fill:none;stroke-width:3;stroke-linecap:round;stroke-linejoin:round;stroke:#FF0000;stroke-opacity:0.8;stroke-miterlimit:10;", "M 10 10 L 20 20").unwrap();

        assert_eq!(line.color(), "#FF0000".parse::<Color>().unwrap().with_alpha(0.8));
    }

    #[test]
    fn can_deserialize_rotated_ellipse() {
        assert!(false);
    }
}
