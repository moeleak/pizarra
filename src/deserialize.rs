use std::fmt;
use std::collections::HashMap;

use xml::{
    attribute::OwnedAttribute, reader::{EventReader, XmlEvent, Error as XmlError}
};

use crate::app::Pizarra;
use crate::storage::Storage;
use crate::shape::{ShapeStored, stored::{path::Path, ellipse::Ellipse}};
use crate::color::Color;
use crate::point::{Vec2D, Unit, WorldUnit};
use crate::geom::Angle;
use crate::config::Config;
use crate::style::{Style, Stroke};

mod path;
mod impls;

use path::{parse_path, PathBuilder as SvgPathBuilder};

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
    UnexpectedTag(String),
    UnsupportedShape(String),

    /// An SVG tag that pizarra doesn't use was found
    UnknownTag(String),
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

impl std::error::Error for Error { }

/// A partial result of deserialization
#[derive(Debug)]
pub struct Partial {
    /// The portion that was properly read
    read: Pizarra,

    /// All the errors that were found
    errors: Vec<Error>,
}

/// Result of the deserialization process.
///
/// With this the user can be warned when a file was not created with pizarra.
#[derive(Debug)]
pub enum DeserializeResult {
    /// The file was parsed without problems as the stabilized file format
    Ok(Pizarra),

    /// The file could be understood, but problems where found. Likely not made
    /// with pizarra but with another program, or made with pizarra and edited
    /// with non-supported features.
    Partial(Partial),

    /// Something is very wrong with the file, like the encoding or the
    /// formatting, thus the file cannot yield any usable content.
    Err(Error),
}

impl DeserializeResult {
    pub fn unwrap(self) -> Pizarra {
        match self {
            DeserializeResult::Ok(p) => p,
            _ => panic!("Deserialization was not successful: {self:?}"),
        }
    }

    pub fn unwrap_partial(self) -> Partial {
        match self {
            DeserializeResult::Partial(p) => p,
            _ => panic!("Result of deserialization is not partial: {self:?}"),
        }
    }

    pub fn unwrap_err(self) -> Error {
        match self {
            DeserializeResult::Err(e) => e,
            _ => panic!("Deserialization didn't error: {self:?}"),
        }
    }
}

/// Converst a string taken from an element's style="" attribtue to a hashmap of
/// the applied styles.
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

fn parse_color(color_str: &str) -> Result<Option<Color>, Error> {
    if color_str.to_lowercase() == "none" {
        Ok(None)
    } else {
        Ok(Some(color_str.parse()?))
    }
}

impl Path {
    pub fn from_xml_attributes(style: &str, path: &str, config: Config) -> Result<Path, Error> {
        let attrs = css_attrs_as_hashmap(style);

        let color: Option<Color> = attrs.get("stroke").map(|s| parse_color(s)).transpose()?.flatten();
        let fill: Option<Color> = attrs.get("fill").map(|s| parse_color(s)).transpose()?.flatten();
        let thickness: WorldUnit = attrs.get("stroke-width").map(|s| s.parse()).transpose()?.unwrap_or_else(|| config.thickness.val().into());
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
    fn deserialize(attributes: HashMap<String, String>, config: Config) -> Result<Box<dyn ShapeStored>, Error>;
}

impl Deserialize for Path {
    fn deserialize(attributes: HashMap<String, String>, config: Config) -> Result<Box<dyn ShapeStored>, Error> {
        let styleattr = attributes.get("style").ok_or_else(|| Error::MissingAttribute("style".into()))?;
        let dattr = attributes.get("d").ok_or_else(|| Error::MissingAttribute("d".into()))?;

        Ok(Box::new(Path::from_xml_attributes(styleattr, dattr, config)?))
    }
}

impl Deserialize for Ellipse {
    fn deserialize(attributes: HashMap<String, String>, config: Config) -> Result<Box<dyn ShapeStored>, Error> {
        let cx = attributes.get("cx").ok_or_else(|| Error::MissingAttribute("cx".into()))?;
        let cy = attributes.get("cy").ok_or_else(|| Error::MissingAttribute("cy".into()))?;
        let rx = attributes.get("rx").ok_or_else(|| Error::MissingAttribute("rx".into()))?;
        let ry = attributes.get("ry").ok_or_else(|| Error::MissingAttribute("ry".into()))?;
        let angle= attributes.get("transform").map(|s| s.as_str()).unwrap_or("rotate(0)");
        let styleattr = attributes.get("style").ok_or_else(|| Error::MissingAttribute("style".into()))?;

        let attrs = css_attrs_as_hashmap(styleattr);

        let color: Option<Color> = attrs.get("stroke").map(|s| parse_color(s)).transpose()?.flatten();
        let fill: Option<Color> = attrs.get("fill").map(|s| parse_color(s)).transpose()?.flatten();
        let thickness: WorldUnit = attrs.get("stroke-width").map(|s| s.parse()).transpose()?.unwrap_or_else(|| config.thickness.val().into());
        let alpha = attrs.get("stroke-opacity").map(|s| s.parse()).transpose()?.unwrap_or(1.0);
        let fill_alpha = attrs.get("fill-opacity").map(|s| s.parse()).transpose()?.unwrap_or(1.0);

        let angle: Angle = angle.parse()?;

        let center = Vec2D::new_world(cx.parse()?, cy.parse()?);
        let rx = rx.parse()?;
        let ry = ry.parse()?;

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

fn color_from_style(style: &str) -> Result<Color, Error> {
    let attrs = css_attrs_as_hashmap(style);

    todo!()
}

/// Keeps the state of deserialization
#[derive(Debug)]
enum DeState {
    /// Start, we know nothing about the format yet
    Initial,

    /// So far it looks like a correct v2 file
    StartedV2,

    /// So far it looks like a correct v1 file
    StartedV1,

    /// Just found <g id="storage">
    V1StorageOpened,

    V1Correct {
        storage: Storage,
        background: Color,
    },

    /// This file its likely not made with pizarra or it was edited in
    /// non-supported ways. Pizarra will do its best to understand it but the
    /// user should be warned.
    External(Storage, Vec<Error>),
}

use DeState::*;

/// Gets a single attribute's value from a list of them. Used for cases where
/// only one attribute is needed to spare the hashmap allocation.
fn get_single_attr<'a, 'b, 'c>(attrs: &'a [OwnedAttribute], key: &'b str, namespace: Option<&'c str>) -> Option<&'a str> {
    for attr in attrs {
        if attr.name.prefix.as_ref().map(|s| s.as_str()) == namespace && attr.name.local_name == key {
            return Some(&attr.value)
        }
    }

    None
}

fn step(state: DeState, event: XmlEvent, config: Config) -> DeState {
    match (state, event) {
        (state@Initial, XmlEvent::StartDocument { .. }) => state,
        (Initial, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "svg" => {
            if let Some(version) = get_single_attr(&attributes, "format", Some("pizarra")) {
                if version.trim() == "2" {
                    // So far we are looking at a correct V2 file
                    StartedV2
                } else {
                    // version field exists but it is not "2". We might be
                    // dealing with a file created using a future version of
                    // pizarra
                    let d = todo!();
                }
            } else {
                // No version field at all (or at least not in the pizarra
                // namespace) so this looks so far like a V1 format
                StartedV1
            }
        }

        (_, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "svg" => {
            todo!("file with two svg tags?")
        }

        (StartedV1, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "g" => {
            if let Some(id) = get_single_attr(&attributes, "id", None) {
                if id == "storage" {
                    V1StorageOpened
                } else {
                    todo!("first group has a different id than storage, likely not a v1 file");
                }
            } else {
                todo!("group without id");
            }
        }

        (V1StorageOpened, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "rect" => {
            // The first rect in this state is the background, lets read it.
            if let Some(style) = get_single_attr(&attributes, "style", None) {
                match color_from_style(style) {
                    Ok(color) => V1Correct {
                        storage: Storage::new(),
                        background: color,
                    },
                    Err(e) => todo!("background color was not understood"),
                }
            } else {
                todo!("No style, this is malformed v1 file or external");
            }
        }

        // Explicitly ignored events
        (state, XmlEvent::Whitespace(_)) => state,

        (state, event) => {
            dbg!(state);
            dbg!(event);
            todo!();
        }
    }
}

impl Pizarra {
    pub fn from_svg(svg: &str, config: Config) -> DeserializeResult {
        let parser = EventReader::from_str(svg);
        let mut state = Initial;

        for e in parser {
            state = match e {
                Ok(event) => step(state, event, config),
                Err(e) => return DeserializeResult::Err(e.into()),
            };
        }

        match state {
            x => {
                dbg!(x);
                todo!()
            }
        }
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
        let app = Pizarra::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(app.storage().shape_count(), 1);
    }

    /// This represents the very first SVG format of pizarra.
    #[test]
    fn read_original_serialization_test() {
        let svg_data = include_str!("../res/deserialize/original.svg");
        let app = Pizarra::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(app.storage().shape_count(), 1);
    }

    #[test]
    fn test_can_deserialize_circle() {
        let svg_data = include_str!("../res/circle.svg");
        let app = Pizarra::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(app.storage().shape_count(), 1);
    }

    #[test]
    fn test_can_deserialize_ellipse() {
        let svg_data = include_str!("../res/ellipse.svg");
        let app = Pizarra::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(app.storage().shape_count(), 1);
    }

    #[test]
    fn test_point_bug() {
        let svg_data = include_str!("../res/bug_opening.svg");
        let app = Pizarra::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(app.storage().shape_count(), 4);
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
            assert_eq!(style.stroke.unwrap().size, 3.0.into());
        } else {
            panic!();
        }
    }

    #[test]
    fn test_parse_point() {
        let p = " 152.496094 51.714844 ";

        assert_eq!(p.parse::<Vec2D<WorldUnit>>().unwrap(), Vec2D::new_world(152.496094, 51.714844));
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
                assert_eq!(e.semimajor, 5.6.into());
                assert_eq!(e.semiminor, 3.5.into());
                assert_eq!(e.angle.degrees(), 34.5);
                assert_eq!(style.stroke.unwrap().color, Color::from_int_rgb(0xca, 0xba, 0xda).with_float_alpha(0.8));
                assert_eq!(style.stroke.unwrap().size, 3.5.into());
            },
            _ => panic!()
        }
    }

    #[test]
    fn can_deserialize_ellipse_serialization_test() {
        let svg_data = include_str!("../res/serialize/ellipse.svg");

        Pizarra::from_svg(svg_data, Default::default()).unwrap();
    }

    #[test]
    fn fill_and_stroke_can_be_none_in_path() {
        let svg_data = include_str!("../res/color_fill_none_path.svg");
        let app = Pizarra::from_svg(svg_data, Default::default()).unwrap();

        let commands = app.storage().draw_commands(app.storage().get_bounds().unwrap());
        let command = &commands[0];

        assert_eq!(command.color(), None);
        assert_eq!(command.fill(), None);
    }

    #[test]
    fn fill_and_stroke_can_be_none_in_ellipse() {
        let svg_data = include_str!("../res/color_fill_none_ellipse.svg");
        let app = Pizarra::from_svg(svg_data, Default::default()).unwrap();

        let commands = app.storage().draw_commands(app.storage().get_bounds().unwrap());
        let command = &commands[0];

        assert_eq!(command.color(), None);
        assert_eq!(command.fill(), None);
    }

    #[test]
    fn new_format_can_be_read() {
        let svg_data = include_str!("../res/deserialize/v2_format.svg");
        let app = Pizarra::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(app.bgcolor(), Color::from_int_rgb(0xba, 0xda, 0xba));
    }

    /// It must be possible to tell which files were made with pizarra, and
    /// which files were made in some other program. This is important to warn
    /// the user about the risk of modifying a file that was made with some
    /// other software because pizarra files are a subset of SVG.

    /// The return value of the from_svg function will have the following shape:
    ///
    /// `Result<Pizarra, Result<(Pizarra, Vec<Error>), Error>>`
    ///
    /// Where the Ok variant is a file that was created with pizarra and was
    /// read without problms, and the Err variant can have two cases:
    ///
    /// * it either can be partially read, but it likely was made with a
    ///   different program (and a collection o errors is returned), or
    /// * it failed miserably at being read. Most likely it is malformed SVG,
    ///   XML or it doesn't have shapes at all.
    #[test]
    fn file_is_identified_as_pizarra_file_or_external() {
        // correct v2 file
        let correct_file = include_str!("../res/deserialize/correct_v2_file.svg");
        Pizarra::from_svg(correct_file, Default::default()).unwrap();

        // correct file but old format
        let correct_old = include_str!("../res/deserialize/correct_old.svg");
        Pizarra::from_svg(correct_old, Default::default()).unwrap();

        // file made in some other software
        let made_external = include_str!("../res/deserialize/made_external.svg");
        let partial = Pizarra::from_svg(made_external, Default::default()).unwrap_partial();
        assert_eq!(partial.read.storage().shape_count(), 1);
        assert_eq!(partial.errors.len(), 2);

        for error in partial.errors {
            assert!(matches!(error, Error::UnknownTag(x) if x == "de"));
        }

        // cannot even understand this
        let errored_file_1 = include_str!("../res/deserialize/errored_file_1.svg");
        Pizarra::from_svg(errored_file_1, Default::default()).unwrap_err();

        todo!("try some correct xml that is not svg");
    }

    #[test]
    fn background_color_can_be_read_from_old_format() {
        let v1_format_file = include_str!("../res/deserialize/v1_format_bgcolor.svg");
        let pizarra = Pizarra::from_svg(v1_format_file, Default::default()).unwrap();

        assert_eq!(pizarra.bgcolor(), Color::from_int_rgb(0xba, 0xda, 0xba));
    }

    #[test]
    fn a_file_saved_from_pizarra_and_not_very_edited_in_inkscape_works() {
        todo!()
    }
}
