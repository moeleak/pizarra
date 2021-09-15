use std::fmt;
use std::ops::{Mul, Add, Sub, Div};

use crate::geom::Radians;

/// A 2D (unitless) vector
#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct Vec2D {
    pub x: f64,
    pub y: f64,
}

/// A point in the storage as X Y coordinates.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Vec2DDrawing {
    pub x: f64,
    pub y: f64,
}

/// A point as read from the screen in X Y coordinates
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Vec2DScreen {
    pub x: f64,
    pub y: f64,
}

pub trait Point: Sized + Copy + PartialEq +Add<Output=Self> + Sub<Output=Self> + Mul<f64, Output=Self> + Div<f64, Output=Self> {
    fn new(x: f64, y: f64) -> Self;
    fn x(&self) -> f64;
    fn y(&self) -> f64;

    fn to_a(self) -> [f64; 2] {
        [self.x(), self.y()]
    }

    fn to_vec2d(self) -> Vec2D {
        Vec2D {
            x: self.x(),
            y: self.y(),
        }
    }

    fn magnitude(&self) -> f64 {
        self.distance(Self::new(0.0, 0.0))
    }

    fn distance(&self, other: Self) -> f64 {
        ((self.x() - other.x()).powi(2) + (self.y() - other.y()).powi(2)).sqrt()
    }

    fn abs(self) -> Self {
        Self::new(
            self.x().abs(),
            self.y().abs(),
        )
    }

    fn min(self, other: Self) -> Vec2DDrawing {
        Vec2DDrawing {
            x: self.x().min(other.x()),
            y: self.y().min(other.y()),
        }
    }

    fn max(self, other: Self) -> Vec2DDrawing {
        Vec2DDrawing {
            x: self.x().max(other.x()),
            y: self.y().max(other.y()),
        }
    }

    fn dot(self, other: Self) -> f64 {
        self.x() * other.x() + self.y() * other.y()
    }

    fn is_nan(&self) -> bool {
        self.x().is_nan() || self.y().is_nan()
    }

    fn unit(self) -> Self {
        self / self.magnitude()
    }

    fn angle(self) -> Radians {
        (self.y() / self.x()).atan().into()
    }
}

impl Point for Vec2D {
    fn new(x: f64, y: f64) -> Self {
        Vec2D {
            x, y,
        }
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}

impl Point for Vec2DDrawing {
    fn new(x: f64, y: f64) -> Self {
        Vec2DDrawing {
            x, y,
        }
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}

impl Point for Vec2DScreen {
    fn new(x: f64, y: f64) -> Vec2DScreen {
        Vec2DScreen {
            x, y,
        }
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}

impl Mul<f64> for Vec2D {
    type Output = Vec2D;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Add for Vec2D {
    type Output = Vec2D;

    fn add(self, rhs: Vec2D) -> Vec2D {
        Vec2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2D {
    type Output = Vec2D;

    fn sub(self, rhs: Vec2D) -> Vec2D {
        Vec2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Div<f64> for Vec2D {
    type Output = Vec2D;

    fn div(self, rhs: f64) -> Vec2D {
        Vec2D {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Mul<f64> for Vec2DDrawing {
    type Output = Vec2DDrawing;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec2DDrawing {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Add for Vec2DDrawing {
    type Output = Vec2DDrawing;

    fn add(self, rhs: Vec2DDrawing) -> Vec2DDrawing {
        Vec2DDrawing {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2DDrawing {
    type Output = Vec2DDrawing;

    fn sub(self, rhs: Vec2DDrawing) -> Vec2DDrawing {
        Vec2DDrawing {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Div<f64> for Vec2DDrawing {
    type Output = Vec2DDrawing;

    fn div(self, rhs: f64) -> Vec2DDrawing {
        Vec2DDrawing {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Mul<f64> for Vec2DScreen {
    type Output = Vec2DScreen;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec2DScreen {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Add for Vec2DScreen {
    type Output = Vec2DScreen;

    fn add(self, rhs: Vec2DScreen) -> Vec2DScreen {
        Vec2DScreen {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2DScreen {
    type Output = Vec2DScreen;

    fn sub(self, rhs: Vec2DScreen) -> Vec2DScreen {
        Vec2DScreen {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Div<f64> for Vec2DScreen {
    type Output = Vec2DScreen;

    fn div(self, rhs: f64) -> Vec2DScreen {
        Vec2DScreen {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl From<[f64; 2]> for Vec2DDrawing {
    fn from(other: [f64; 2]) -> Vec2DDrawing {
        Vec2DDrawing::new(other[0], other[1])
    }
}

impl From<(f64, f64)> for Vec2DDrawing {
    fn from(other: (f64, f64)) -> Vec2DDrawing {
        Vec2DDrawing::new(other.0, other.1)
    }
}

impl From<Vec2D> for Vec2DDrawing {
    fn from(point: Vec2D) -> Vec2DDrawing {
        Vec2DDrawing {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<Vec2D> for Vec2DScreen {
    fn from(point: Vec2D) -> Vec2DScreen {
        Vec2DScreen {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<(f64, f64)> for Vec2DScreen {
    fn from(other: (f64, f64)) -> Vec2DScreen {
        Vec2DScreen::new(other.0, other.1)
    }
}

impl From<Vec2DScreen> for Vec2D {
    fn from(point: Vec2DScreen) -> Vec2D {
        Vec2D::new(point.x, point.y)
    }
}

impl fmt::Display for Vec2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.x, self.y)
    }
}

impl fmt::Display for Vec2DDrawing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.x, self.y)
    }
}

impl fmt::Display for Vec2DScreen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance() {
        let p1 = Vec2DDrawing::new(0.0, 0.0);
        let p2 = Vec2DDrawing::new(3.0, 4.0);

        assert_eq!(p1.distance(p2), 5.0);
    }
}
