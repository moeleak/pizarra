use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub fn with_alpha(self, alpha: f64) -> Color {
        Color {
            a: alpha,
            ..self
        }
    }

    pub fn alpha(self) -> f64 {
        self.a
    }

    /// Creates a color from its integer rgb codes
    pub fn from_int_rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: 0.0,
        }
    }

    /// Creates a color from its integer rgba codes
    pub fn from_int_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: a as f64 / 255.0,
        }
    }

    /// Creates an opaque color from rgb
    pub fn from_rgb(r: f64, g: f64, b: f64) -> Color {
        Color {
            r, g, b, a: 0.0,
        }
    }

    /// Creates a color from rgba values
    pub fn from_rgba(r: f64, g: f64, b: f64, a: f64) -> Color {
        Color {
            r, g, b, a,
        }
    }

    pub fn green() -> Color {
        Color {
            r: 138.0/255.0,
            g: 226.0/255.0,
            b: 52.0/255.0,
            a: 1.0,
        }
    }

    pub fn red() -> Color {
        Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    pub fn blue() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }
    }

    pub fn white() -> Color {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    pub fn black() -> Color {
        Color {
            r: 48.0/255.0,
            g: 54.0/255.0,
            b: 51.0/255.0,
            a: 1.0,
        }
    }

    pub fn css(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", (self.r*255.0) as u32, (self.g*255.0) as u32, (self.b*255.0) as u32)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::white()
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:X}{:X}{:X}{:X}",
            (self.r*255.0) as u32,
            (self.g*255.0) as u32,
            (self.b*255.0) as u32,
            (self.a*255.0) as u32,
        )
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
}
