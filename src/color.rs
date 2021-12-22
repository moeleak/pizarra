use std::fmt;
use std::str::FromStr;
use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;

use crate::deserialize::Error;

lazy_static! {
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
    static ref CSS_COLOR_REGEX: Regex = Regex::new(r"(?x)#
        (?P<r>[\dA-Fa-f]{2})
        (?P<g>[\dA-Fa-f]{2})
        (?P<b>[\dA-Fa-f]{2})
        (?P<a>[\dA-Fa-f]{2})?
    ").unwrap();
    static ref COLOR_REGEX: Regex = Regex::new(r"(?x)rgb\(
        (?P<r>\d+(\.\d+)?)
        %\s*,\s*
        (?P<g>\d+(\.\d+))?
        %\s*,\s*
        (?P<b>\d+(\.\d+))?
    %\s*\)").unwrap();
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn with_alpha(self, alpha: u8) -> Color {
        Color {
            a: alpha,
            ..self
        }
    }

    pub fn with_float_alpha(self, alpha: f64) -> Color {
        Color {
            a: (alpha * 255.0) as u8,
            ..self
        }
    }

    pub fn alpha(self) -> u8 {
        self.a
    }

    pub fn float_r(self) -> f64 {
        self.r as f64 / 255.0
    }

    pub fn float_g(self) -> f64 {
        self.g as f64 / 255.0
    }

    pub fn float_b(self) -> f64 {
        self.b as f64 / 255.0
    }

    pub fn float_alpha(self) -> f64 {
        self.a as f64 / 255.0
    }

    /// Creates a color from its integer rgb codes
    pub fn from_int_rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            r, g, b,
            a: 255,
        }
    }

    /// Creates a color from its integer rgba codes
    pub fn from_int_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r, g, b, a
        }
    }

    /// Creates an opaque color from float rgb.
    ///
    /// Values must be between 0.0 and 1.0
    pub fn from_float_rgb(r: f64, g: f64, b: f64) -> Color {
        Color {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            a: 255,
        }
    }

    /// Creates a color from rgba values
    ///
    /// Values must be between 0.0 and 1.0
    pub fn from_float_rgba(r: f64, g: f64, b: f64, a: f64) -> Color {
        Color {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
            a: (a * 255.0) as u8,
        }
    }

    pub fn green() -> Color {
        Color {
            r: 138,
            g: 226,
            b: 52,
            a: 255,
        }
    }

    pub fn red() -> Color {
        Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn blue() -> Color {
        Color {
            r: 0,
            g: 0,
            b: 255,
            a: 255,
        }
    }

    pub fn white() -> Color {
        Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }

    pub fn black() -> Color {
        Color {
            r: 48,
            g: 54,
            b: 51,
            a: 255,
        }
    }

    pub fn gray() -> Color {
        Color {
            r: 0x94,
            g: 0x94,
            b: 0x94,
            a: 0xff,
        }
    }

    pub fn css(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::white()
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:X}{:X}{:X}{:X}", self.r, self.g, self.b, self.a)
    }
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = CSS_COLORS.get(&s.to_lowercase()) {
            Ok(*c)
        } else if let Some(captures) = COLOR_REGEX.captures(s) {
            Ok(Color::from_float_rgb(
                captures["r"].parse::<f64>()?/ 100.0,
                captures["g"].parse::<f64>()?/100.0,
                captures["b"].parse::<f64>()?/100.0,
            ))
        } else if let Some(captures) = CSS_COLOR_REGEX.captures(s) {
            let c = Color::from_int_rgb(
                u8::from_str_radix(&captures["r"], 16).unwrap(),
                u8::from_str_radix(&captures["g"], 16).unwrap(),
                u8::from_str_radix(&captures["b"], 16).unwrap(),
            );

            Ok(if let Some(m) = captures.name("a") {
                c.with_alpha(u8::from_str_radix(m.as_str(), 16).unwrap())
            } else {
                c
            })
        } else {
            Err(Error::CouldntUnderstandColor(s.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn test_to_css() {
        let c = Color::red();

        assert_eq!(c.css(), "#FF0000");
    }

    #[test]
    fn test_with_alpha() {
        let c = Color::red();

        assert_eq!(c.alpha(), 255);
        assert_eq!(c.with_alpha(204).alpha(), 204);
    }

    #[test]
    fn test_parse_color() {
        let color: Color = "rgb(53.90625%,88.28125%,20.3125%)".parse().unwrap();

        assert_eq!(color, Color::from_float_rgb(0.5390625, 0.8828125, 0.203125));
    }

    #[test]
    fn test_parse_css_color() {
        let color: Color = "#FF2030".parse().unwrap();

        assert_eq!(color, Color::from_float_rgb(1.0, 0.12549019607843137, 0.18823529411764706));

        let color: Color = "#FF2003".parse().unwrap();

        assert_eq!(color, Color::from_float_rgb(1.0, 0.12549019607843137, 0.011764705882352941));
    }

    #[test]
    fn parse_css_color_with_alpha() {
        let color: Color = "#FF203080".parse().unwrap();

        assert_eq!(color, Color::from_float_rgb(1.0, 0.12549019607843137, 0.18823529411764706).with_float_alpha(0.5019607843137255));
        assert_eq!(color, Color::from_int_rgba(0xff, 0x20, 0x30, 0x80));
    }

    #[test]
    fn can_serialize_and_deserialize() {
        let color = Color::from_int_rgba(0xce, 0xba, 0xda, 0x80);

        assert_eq!(color.to_string().parse::<Color>().unwrap(), color);
    }
}
