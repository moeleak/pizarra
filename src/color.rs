use std::fmt;

#[derive(Copy, Clone, PartialEq)]
pub struct Color([u8; 4]);

impl Color {

    pub fn red() -> Color {
        Color([0xF4, 0x43, 0x36, 0xff])
    }

    pub fn blue() -> Color {
        Color([0x21, 0x96, 0xF3, 0xff])
    }

    pub fn green() -> Color {
        Color([0x4C, 0xAF, 0x50, 0xff])
    }

    pub fn yellow() -> Color {
        Color([0xFF, 0xEB, 0x3B, 0xff])
    }

    pub fn orange() -> Color {
        Color([0xFF, 0x98, 0x00, 0xff])
    }

    pub fn brown() -> Color {
        Color([0x79, 0x55, 0x48, 0xff])
    }

    pub fn white() -> Color {
        Color([0xFF, 0xFF, 0xFF, 0xff])
    }

    pub fn black() -> Color {
        Color([0x30, 0x36, 0x33, 0xff])
    }

    pub fn gray() -> Color {
        Color([0x7f, 0x7b, 0x82, 0xff])
    }

    pub fn to_a(&self) -> [f32; 4] {
        [
            self.0[0] as f32/0xff as f32,
            self.0[1] as f32/0xff as f32,
            self.0[2] as f32/0xff as f32,
            self.0[3] as f32/0xff as f32,
        ]
    }
}

impl fmt::UpperHex for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:X}{:X}{:X}", self.0[0], self.0[1], self.0[2])
    }
}

impl fmt::LowerHex for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:x}{:x}{:x}", self.0[0], self.0[1], self.0[2])
    }
}
