use std::fmt;
use std::collections::HashMap;

use xml::reader::{EventReader, XmlEvent, Error as XmlError};

use crate::storage::Storage;
use crate::shape::line::Line;
use crate::color::Color;

#[derive(Debug)]
pub enum Error {
    Xml(XmlError),
    PathWithNoDAttr,
}

impl From<XmlError> for Error {
    fn from(error: XmlError) -> Error {
        Error::Xml(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Load error caused by {:?}", self)
    }
}

impl std::error::Error for Error {
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

        let color = Color::red();
        let points = vec![];
        let thickness = 5.0;

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

        Ok(storage)
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::Storage;
    use crate::shape::{Line, DrawCommand, ShapeTrait};
    use crate::color::Color;

    #[test]
    fn test_from_svg() {
        let svg_data = include_str!("../res/simple_file_load.svg");
        let storage = Storage::from_svg(svg_data).unwrap();

        assert_eq!(storage.shape_count(), 1);
    }

    #[test]
    fn test_from_xml_attributes() {
        let line = Line::from_xml_attributes("fill:none;stroke-width:3;stroke-linecap:round;stroke-linejoin:round;stroke:rgb(53.90625%,88.28125%,20.3125%);stroke-opacity:1;stroke-miterlimit:10;", "M 147.570312 40.121094 L 146.9375 40.121094 L 145.296875 40.460938 L 142.519531 41.710938 L 139.613281 43.304688 L 138.097656 44.894531 L 137.339844 46.714844 L 138.097656 47.621094 L 139.992188 47.851562 L 142.898438 47.621094 L 146.179688 47.167969 L 150.097656 47.964844 L 151.738281 49.667969 L 152.496094 51.714844 L 152.875 53.191406 ").unwrap();

        if let DrawCommand::Line {
            color,
            line,
            thickness,
        } = line.draw_commands() {
            assert_eq!(color, Color::from_rgb(0.5390625, 0.8828125, 0.203125));
            assert_eq!(line, vec![]);
            assert_eq!(thickness, 3.0);
        } else {
            panic!();
        }
    }
}
