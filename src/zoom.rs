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

    pub fn can_see(&self, other: Zoom) -> bool {
        other > *self || (*self - other) <= 5
    }

    pub fn relative_to(&self, other: Zoom) -> i32 {
        *self - other
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

impl std::ops::Sub<Zoom> for Zoom {
    type Output = i32;

    fn sub(self, other: Zoom) -> i32 {
        self.0 - other.0
    }
}

impl std::cmp::PartialOrd for Zoom {
    fn partial_cmp(&self, other: &Zoom) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
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
    fn test_sub() {
        let zoom = Zoom::from(23);

        assert_eq!(zoom-4, Zoom::from(19));
    }

    #[test]
    fn test_can_see() {
        let zoom = Zoom::from(0);

        // too small to be seen
        for i in -10_000..=-6 {
            assert!(!zoom.can_see(Zoom::from(i)));
        }

        // visible range
        for i in -5..=10_000 {
            println!("{}", i);
            assert!(zoom.can_see(Zoom::from(i)))
        }
    }

    #[test]
    fn test_relative_to() {
        let zoom = Zoom::from(0);

        assert_eq!(zoom.relative_to(Zoom::from(3)), -3);
        assert_eq!(zoom.relative_to(Zoom::from(-3)), 3);
    }
}
