use std::fmt;

use gdk::RGBA;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub fn green() -> Color {
        Color {
            r: 138.0/256.0,
            g: 226.0/256.0,
            b: 52.0/256.0,
            a: 1.0,
        }
    }

    pub fn red() -> Color {
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }

    pub fn blue() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 0.0,
        }
    }

    pub fn black() -> Color {
        Color {
            r: 48.0/256.0,
            g: 54.0/256.0,
            b: 51.0/256.0,
            a: 1.0,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:X}{:X}{:X}{:X}",
            (self.r*256.0) as u32,
            (self.g*256.0) as u32,
            (self.b*256.0) as u32,
            (self.a*256.0) as u32,
        )
    }
}

impl From<RGBA> for Color {
    fn from(rgba: RGBA) -> Color {
        Color {
            r: rgba.red,
            g: rgba.green,
            b: rgba.blue,
            a: rgba.alpha,
        }
    }
}
