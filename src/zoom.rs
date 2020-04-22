use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Zoom(i32);

impl Zoom {
    pub fn to_i32(&self) -> i32 {
        self.0
    }

    pub fn increase(&mut self) {
        if self.0 < i32::max_value() {
            self.0 += 1;
        }
    }

    pub fn decrease(&mut self) {
        if self.0 > i32::min_value() {
            self.0 -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }

    pub fn visible_layers(&self) -> Vec<(Zoom, i32)> {
        vec![
            (*self-5, -5),
            (*self-4, -4),
            (*self-3, -3),
            (*self-2, -2),
            (*self-1, -1),
            (*self, 0),
            (*self+1, 1),
            (*self+2, 2),
            (*self+3, 3),
            (*self+4, 4),
            (*self+5, 5),
            (*self+6, 6),
        ]
    }
}

impl From<i32> for Zoom {
    fn from(zoom: i32) -> Zoom {
        Zoom(zoom)
    }
}

impl std::ops::Add<i32> for Zoom {
    type Output = Zoom;

    fn add(self, other: i32) -> Zoom {
        Zoom::from(self.0 + other)
    }
}

impl std::ops::Sub<i32> for Zoom {
    type Output = Zoom;

    fn sub(self, other: i32) -> Zoom {
        Zoom::from(self.0 - other)
    }
}

impl fmt::Display for Zoom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Zoom;

    #[test]
    fn test_visible_layers() {
        let zoom = Zoom::from(0);

        assert_eq!(zoom.visible_layers(), [
                   (Zoom::from(-2), -2),
                   (Zoom::from(-1), -1),
                   (Zoom::from(0), 0),
                   (Zoom::from(1), 1),
                   (Zoom::from(2), 2),
                   (Zoom::from(3), 3),
                   (Zoom::from(4), 4),
        ]);
    }

    #[test]
    fn test_sub() {
        let zoom = Zoom::from(23);

        assert_eq!(zoom-4, Zoom::from(19));
    }
}
