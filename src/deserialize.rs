use std::collections::HashMap;

use xml::{
    attribute::OwnedAttribute, reader::{EventReader, XmlEvent, Error as XmlError}
};
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Xml(#[from] XmlError),

    #[error("A path was found without the '{0}' attribute")]
    PathMissingAttribute(String),

    #[error("An ellipse was found without the '{0}' attribtue")]
    EllipseMissingAttribute(String),

    #[error("The given color '{0}' was not understood")]
    CouldntUnderstandColor(String),

    #[error("Found an unsupported value for attribute {attr}: {value}")]
    UnsupportedAttributeValue { attr: &'static str, value: String },

    #[error(transparent)]
    PathParseError(#[from] path::ParseError),

    #[error("A point part of a path doesn't have a known shape: {0}")]
    PointDoesntMatchRegex(String),

    #[error("the given angle was not understood: {0}")]
    AngleDoesntMatchRegex(String),

    #[error("An SVG tag that pizarra doesn't use was found: {0}")]
    UnsupportedTag(String),

    #[error("File is not svg, instead its first element is <{0}>")]
    NotSvg(String),
}

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

fn parse_length(attr: &'static str, stroke: &str) -> Result<WorldUnit, Error> {
    stroke
        .trim_end_matches("px")
        .parse()
        .map_err(|_| Error::UnsupportedAttributeValue {
            attr,
            value: stroke.to_owned()
        })
}

fn parse_stroke_opacity(opacity: &str) -> Result<f64, Error> {
    opacity
        .parse()
        .map_err(|_| Error::UnsupportedAttributeValue {
            attr: "stroke-opacity".into(),
            value: opacity.to_owned(),
        })
}

fn parse_fill_opacity(opacity: &str) -> Result<f64, Error> {
    opacity
        .parse()
        .map_err(|_| Error::UnsupportedAttributeValue {
            attr: "fill-opacity".into(),
            value: opacity.to_owned(),
        })
}

impl Path {
    pub fn from_xml_attributes(style: &str, path: &str, config: Config) -> Result<Path, Error> {
        let attrs = css_attrs_as_hashmap(style);

        let color: Option<Color> = attrs.get("stroke").map(|s| parse_color(s)).transpose()?.flatten();
        let fill: Option<Color> = attrs.get("fill").map(|s| parse_color(s)).transpose()?.flatten();
        let thickness = attrs.get("stroke-width").map(|s| parse_length("stroke-width", s)).transpose()?.unwrap_or_else(|| config.thickness.val().into());
        let alpha = attrs.get("stroke-opacity").map(|s| parse_stroke_opacity(s)).transpose()?.unwrap_or(1.0);
        let fill_alpha = attrs.get("fill-opacity").map(|s| parse_fill_opacity(s)).transpose()?.unwrap_or(1.0);

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
        let styleattr = attributes.get("style").ok_or_else(|| Error::PathMissingAttribute("style".into()))?;
        let dattr = attributes.get("d").ok_or_else(|| Error::PathMissingAttribute("d".into()))?;

        Ok(Box::new(Path::from_xml_attributes(styleattr, dattr, config)?))
    }
}

impl Deserialize for Ellipse {
    fn deserialize(attributes: HashMap<String, String>, config: Config) -> Result<Box<dyn ShapeStored>, Error> {
        let cx = parse_length("cx", attributes.get("cx").ok_or_else(|| Error::EllipseMissingAttribute("cx".into()))?)?;
        let cy = parse_length("cy", attributes.get("cy").ok_or_else(|| Error::EllipseMissingAttribute("cy".into()))?)?;
        let rx = parse_length("rx", attributes.get("rx").ok_or_else(|| Error::EllipseMissingAttribute("rx".into()))?)?;
        let ry = parse_length("ry", attributes.get("ry").ok_or_else(|| Error::EllipseMissingAttribute("ry".into()))?)?;
        let angle= attributes.get("transform").map(|s| s.as_str()).unwrap_or("rotate(0)");
        let styleattr = attributes.get("style").ok_or_else(|| Error::EllipseMissingAttribute("style".into()))?;

        let attrs = css_attrs_as_hashmap(styleattr);

        let color: Option<Color> = attrs.get("stroke").map(|s| parse_color(s)).transpose()?.flatten();
        let fill: Option<Color> = attrs.get("fill").map(|s| parse_color(s)).transpose()?.flatten();
        let thickness = attrs.get("stroke-width").map(|s| parse_length("stroke-width", s)).transpose()?.unwrap_or_else(|| config.thickness.val().into());
        let alpha = attrs.get("stroke-opacity").map(|s| parse_stroke_opacity(s)).transpose()?.unwrap_or(1.0);
        let fill_alpha = attrs.get("fill-opacity").map(|s| parse_fill_opacity(s)).transpose()?.unwrap_or(1.0);

        let angle: Angle = angle.parse()?;

        let center = Vec2D::new(cx, cy);

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

/// Keeps the state of deserialization
#[derive(Debug)]
enum DeState {
    /// Start, we know nothing about the format yet
    Initial,

    /// So far it looks like a correct v2 file
    V2Started,

    /// <g id="background"> just seen
    V2BackgroundOpened,

    /// The rectangle with the fill color just appeared
    V2BackgroundRead {
        background: Color,
    },

    V2StorageStarted {
        storage: Storage,
        background: Color,
    },

    /// So far it looks like a correct v1 file
    V1Started,

    /// Just found <g id="storage">
    V1StorageOpened,

    V1Correct {
        storage: Storage,
        background: Color,
    },

    Finished {
        storage: Storage,
        background: Color,
    },

    /// This file its likely not made with pizarra or it was edited in
    /// non-supported ways. Pizarra will do its best to understand it but the
    /// user should be warned.
    External {
        background: Color,
        storage: Storage,
        errors: Vec<Error>,
    },

    /// Not an SVG file. This is a final state and contains the first tag that
    /// was found
    NotSvg(String),
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

macro_rules! parse_shape_v1 {
    ($shape_type:ty, $attributes:ident, $config:ident, $storage:ident, $background:ident) => {
        {
            let attributes = xml_attrs_as_hashmap($attributes);
            let path = <$shape_type>::deserialize(attributes, $config);

            match path {
                Ok(path) => {
                    $storage.add(path);
                    V1Correct { $storage, $background }
                }
                Err(e) => todo!("error deserializing path"),
            }
        }
    };
}

macro_rules! parse_shape_v2 {
    ($shape_type:ty, $attributes:ident, $config:ident, $storage:ident, $background:ident) => {
        {
            let attributes = xml_attrs_as_hashmap($attributes);
            let path = <$shape_type>::deserialize(attributes, $config);

            match path {
                Ok(path) => {
                    $storage.add(path);
                    V2StorageStarted { $storage, $background }
                }
                Err(e) => todo!("error deserializing path"),
            }
        }
    };
}

macro_rules! parse_shape_external {
    ($shape_type:ty, $attributes:ident, $config:ident, $storage:ident, $background:ident, $errors:ident) => {
        {
            let attributes = xml_attrs_as_hashmap($attributes);
            let path = <$shape_type>::deserialize(attributes, $config);

            match path {
                Ok(path) => {
                    $storage.add(path);
                    External { $storage, $background, $errors }
                }
                Err(e) => {dbg!(e); todo!("error deserializing path")},
            }
        }
    };
}

fn step(state: DeState, event: XmlEvent, config: Config) -> DeState {
    match (state, event) {
        (state@Initial, XmlEvent::StartDocument { .. }) => state,
        (Initial, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "svg" => {
            if let Some(version) = get_single_attr(&attributes, "format", Some("pizarra")) {
                if version.trim() == "2" {
                    // So far we are looking at a correct V2 file
                    V2Started
                } else {
                    // version field exists but it is not "2". We might be
                    // dealing with a file created using a future version of
                    // pizarra
                    let d = todo!();
                }
            } else {
                // No version field at all (or at least not in the pizarra
                // namespace) so this looks so far like a V1 format
                V1Started
            }
        }

        // First element is not <svg>, discard this file faster for not even
        // trying to be svg
        (Initial, XmlEvent::StartElement { name, .. }) => NotSvg(name.local_name),
        (state@NotSvg(_), _) => state,

        (V2Started, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "g" => {
            if let Some(id) = get_single_attr(&attributes, "id", None) {
                if id == "background" {
                    V2BackgroundOpened
                } else {
                    todo!("first group has id but it is not \"background\". File likely modified")
                }
            } else {
                todo!("first group doesnt have an id, file likely modified")
            }
        }

        (V2BackgroundOpened, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "rect" => {
            // The first and only shape in the background group is a rectangle
            // with the fill set to the background color.
            if let Some(style) = get_single_attr(&attributes, "style", None) {
                let attrs = css_attrs_as_hashmap(style);
                let color = attrs.get("fill").map(|c| parse_color(c)).transpose().map(|o| o.flatten());

                match color {
                    Ok(Some(color)) => V2BackgroundRead {
                        background: color,
                    },
                    Ok(None) => V2BackgroundRead {
                        background: config.background_color,
                    },
                    Err(e) => todo!("background color was not understood"),
                }
            } else {
                todo!("No style, this is malformed v1 file or external");
            }
        }

        (V2BackgroundRead { background }, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "g" => {
            if let Some(id) = get_single_attr(&attributes, "id", None) {
                if id == "shapes" {
                    V2StorageStarted {
                        storage: Storage::new(),
                        background,
                    }
                } else {
                    todo!("second group has a different id than shapes");
                }
            } else {
                todo!("another group with unknown id");
            }
        }

        (V2StorageStarted { mut storage, background }, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "path" => {
            parse_shape_v2!(Path, attributes, config, storage, background)
        }

        (V2StorageStarted { mut storage, background }, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "ellipse" => {
            parse_shape_v2!(Ellipse, attributes, config, storage, background)
        }

        (V2StorageStarted { storage, background }, XmlEvent::EndDocument) => Finished { storage, background },

        (V1Started, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "g" => {
            if let Some(id) = get_single_attr(&attributes, "id", None) {
                if id == "storage" {
                    return V1StorageOpened
                }
            }

            External {
                background: config.background_color,
                storage: Storage::new(),
                errors: Vec::new(),
            }
        }

        (V1StorageOpened, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "rect" => {
            // The first rect in this state is the background, lets read it.
            if let Some(style) = get_single_attr(&attributes, "style", None) {
                let attrs = css_attrs_as_hashmap(style);
                let color = attrs.get("fill").map(|c| parse_color(c)).transpose().map(|o| o.flatten());

                match color {
                    Ok(Some(color)) => V1Correct {
                        storage: Storage::new(),
                        background: color,
                    },
                    Ok(None) => V1Correct {
                        storage: Storage::new(),
                        background: config.background_color,
                    },
                    Err(e) => todo!("background color was not understood"),
                }
            } else {
                todo!("No style, this is malformed v1 file or external");
            }
        }

        (V1Correct { mut storage, background }, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "path" => {
            parse_shape_v1!(Path, attributes, config, storage, background)
        }

        (V1Correct { mut storage, background }, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "ellipse" => {
            parse_shape_v1!(Ellipse, attributes, config, storage, background)
        }

        (V1Correct { storage, background }, XmlEvent::EndDocument) => Finished { storage, background },

        // Explicitly ignored events
        (state, XmlEvent::Whitespace(_)) => state,
        (state, XmlEvent::EndElement { .. }) => state,

        // Once an error has ocurred while reading a file it is tagged as
        // external and further deserializing happens here.
        (External { mut storage, errors, background }, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "path" => {
            parse_shape_external!(Path, attributes, config, storage, background, errors)
        }
        (External { mut storage, errors, background }, XmlEvent::StartElement { name, attributes, .. }) if name.local_name == "ellipse" => {
            parse_shape_external!(Ellipse, attributes, config, storage, background, errors)
        }
        // Ignore groups if reading external file
        (state@External { .. }, XmlEvent::StartElement { name, .. }) if name.local_name == "g" => state,
        (state@External { .. }, XmlEvent::EndDocument) => state,
        // Catch all unknown tags
        (External { storage, mut errors, background }, XmlEvent::StartElement { name, .. }) => {
            errors.push(Error::UnsupportedTag(name.local_name));

            External { storage, errors, background }
        }
        // Ignore characters in external state
        (state@External { .. }, XmlEvent::Characters(_)) => state,

        (_state, XmlEvent::StartElement { name,  .. }) => External {
            background: config.background_color,
            storage: Storage::new(),
            errors: vec![Error::UnsupportedTag(name.local_name)],
        },

        // All other states
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
            Finished { storage, background } => {
                let mut pizarra = Pizarra::new(Vec2D::new_screen(0.0, 0.0), config);

                pizarra.set_bgcolor(background);
                pizarra.set_storage(storage);

                DeserializeResult::Ok(pizarra)
            }
            External { background, storage, errors } => {
                let mut pizarra = Pizarra::new(Vec2D::new_screen(0.0, 0.0), config);

                pizarra.set_bgcolor(background);
                pizarra.set_storage(storage);

                DeserializeResult::Partial(Partial {
                    read: pizarra,
                    errors,
                })
            }
            NotSvg(tag) => DeserializeResult::Err(Error::NotSvg(tag)),
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
        let partial = Pizarra::from_svg(svg_data, Default::default()).unwrap_partial();

        assert_eq!(partial.read.storage().shape_count(), 1);

        assert_eq!(partial.errors.len(), 1);
        assert!(matches!(&partial.errors[0], Error::UnsupportedTag(x) if x == "rect"));
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
    fn v2_format_can_be_read() {
        let svg_data = include_str!("../res/deserialize/v2_format.svg");
        let app = Pizarra::from_svg(svg_data, Default::default()).unwrap();

        assert_eq!(app.bgcolor(), Color::from_int_rgb(0xba, 0xde, 0xba));

        assert_eq!(app.storage().shape_count(), 78);
    }

    #[test]
    fn file_made_in_another_software() {
        let made_external = include_str!("../res/deserialize/made_external.svg");
        let partial = Pizarra::from_svg(made_external, Default::default()).unwrap_partial();

        assert_eq!(partial.read.storage().shape_count(), 1);
        assert_eq!(partial.errors.len(), 3);

        assert!(matches!(&partial.errors[0], Error::UnsupportedTag(x) if x == "namedview"));
        assert!(matches!(&partial.errors[1], Error::UnsupportedTag(x) if x == "defs"));
        assert!(matches!(dbg!(&partial.errors[2]), Error::UnsupportedTag(x) if x == "rect"));
    }

    #[test]
    fn file_with_errors() {
        let errored_file_1 = include_str!("../res/deserialize/errored_file_1.svg");
        let err = Pizarra::from_svg(errored_file_1, Default::default()).unwrap_err();

        assert!(matches!(dbg!(err), Error::Xml(_)));
    }

    #[test]
    fn non_svg_file() {
        let file = include_str!("../res/deserialize/not.svg");
        let err = Pizarra::from_svg(file, Default::default()).unwrap_err();

        assert!(matches!(err, Error::NotSvg(x) if x == "foo"));
    }

    #[test]
    fn v1_format_can_be_read() {
        let v1_format_file = include_str!("../res/deserialize/v1_format_bgcolor.svg");
        let pizarra = Pizarra::from_svg(v1_format_file, Default::default()).unwrap();

        assert_eq!(pizarra.bgcolor(), Color::from_int_rgb(0xca, 0xe3, 0xff));

        assert_eq!(pizarra.storage().shape_count(), 78);
    }

    #[test]
    fn v1_edited_is_recovered() {
        let file = include_str!("../res/deserialize/v1_edited.svg");
        let partial = Pizarra::from_svg(file, Default::default()).unwrap_partial();

        assert_eq!(partial.read.storage().shape_count(), 77);
        assert_eq!(partial.errors.len(), 5);

        assert!(matches!(dbg!(&partial.errors[0]), Error::UnsupportedTag(x) if x == "defs"));
        assert!(matches!(dbg!(&partial.errors[1]), Error::UnsupportedTag(x) if x == "namedview"));
        assert!(matches!(dbg!(&partial.errors[2]), Error::UnsupportedTag(x) if x == "rect"));
        assert!(matches!(dbg!(&partial.errors[3]), Error::UnsupportedTag(x) if x == "circle"));
        assert!(matches!(dbg!(&partial.errors[4]), Error::UnsupportedTag(x) if x == "circle"));
    }

    #[test]
    fn v2_edited_is_recovered() {
        let file = include_str!("../res/deserialize/v2_edited.svg");
        let partial = Pizarra::from_svg(file, Default::default()).unwrap_partial();

        assert_eq!(partial.read.storage().shape_count(), 79);
        assert_eq!(partial.errors.len(), 3);

        assert!(matches!(dbg!(&partial.errors[0]), Error::UnsupportedTag(x) if x == "defs"));
        assert!(matches!(dbg!(&partial.errors[1]), Error::UnsupportedTag(x) if x == "namedview"));
        assert!(matches!(dbg!(&partial.errors[2]), Error::UnsupportedTag(x) if x == "rect"));
    }
}
