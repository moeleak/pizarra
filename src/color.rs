use std::fmt;

fn to_float(color: [u8; 4]) -> [f64; 4] {
    [
        color[0] as f64/0xff as f64,
        color[1] as f64/0xff as f64,
        color[2] as f64/0xff as f64,
        color[3] as f64/0xff as f64,
    ]
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color {
    Red,
    Blue,
    Green,
    Yellow,
    Orange,
    Brown,
    White,
    Black,
    Gray,
}

impl Color {
    pub fn code(&self) -> [u8; 4] {
        match self {
            Color::Red => [0xF4, 0x43, 0x36, 0xff],
            Color::Blue => [0x21, 0x96, 0xF3, 0xff],
            Color::Green => [0x4C, 0xAF, 0x50, 0xff],
            Color::Yellow => [0xFF, 0xEB, 0x3B, 0xff],
            Color::Orange => [0xFF, 0x98, 0x00, 0xff],
            Color::Brown => [0x79, 0x55, 0x48, 0xff],
            Color::White => [0xFF, 0xFF, 0xFF, 0xff],
            Color::Black => [0x30, 0x36, 0x33, 0xff],
            Color::Gray => [0x7f, 0x7b, 0x82, 0xff],
        }
    }

    pub fn cycle(&self) -> Color {
        match self {
            Color::Red => Color::Blue,
            Color::Blue => Color::Green,
            Color::Green => Color::Yellow,
            Color::Yellow => Color::Orange,
            Color::Orange => Color::Brown,
            Color::Brown => Color::White,
            Color::White => Color::Gray,
            Color::Gray => Color::Red,

            Color::Black => Color::Green,
        }
    }

    pub fn to_a(&self) -> [f64; 4] {
        to_float(self.code())
    }
}

impl fmt::UpperHex for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = self.code();

        write!(f, "#{:X}{:X}{:X}", code[0], code[1], code[2])
    }
}

impl fmt::LowerHex for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = self.code();

        write!(f, "#{:x}{:x}{:x}", code[0], code[1], code[2])
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let code = self.code();

        write!(f, "#{:X}{:X}{:X}", code[0], code[1], code[2])
    }
}
