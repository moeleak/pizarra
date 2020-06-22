use std::ops::{Mul, Add, Sub, AddAssign, Div, SubAssign};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Point {
        Point {
            x, y,
        }
    }

    pub fn to_a(self) -> [f64; 2] {
        [self.x, self.y]
    }

    pub fn distance(&self, other: Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl Mul<f64> for Point {
    type Output = Point;

    fn mul(self, rhs: f64) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Point) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Div<f64> for Point {
    type Output = Point;

    fn div(self, rhs: f64) -> Point {
        Point {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Point) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl From<[f64; 2]> for Point {
    fn from(other: [f64; 2]) -> Point {
        Point::new(other[0], other[1])
    }
}

#[cfg(test)]
mod tests {
    use super::Point;

    #[test]
    fn distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);

        assert_eq!(p1.distance(p2), 5.0);
    }
}
