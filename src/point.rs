use std::fmt;
use std::ops::{Mul, Add, Sub, Div};

use crate::geom::Angle;

/// A 2D (unitless) vector
#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct Vec2D {
    pub x: f64,
    pub y: f64,
}

/// A point in the storage as X Y coordinates.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Vec2DWorld {
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

    fn min(self, other: Self) -> Vec2DWorld {
        Vec2DWorld {
            x: self.x().min(other.x()),
            y: self.y().min(other.y()),
        }
    }

    fn max(self, other: Self) -> Vec2DWorld {
        Vec2DWorld {
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

    fn angle(self) -> Angle {
        if self.x() == 0.0 {
            Angle::from_radians(0.0)
        } else {
            Angle::from_radians((self.y() / self.x()).atan())
        }
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

impl Point for Vec2DWorld {
    fn new(x: f64, y: f64) -> Self {
        Vec2DWorld {
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

impl Mul<f64> for Vec2DWorld {
    type Output = Vec2DWorld;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec2DWorld {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Add for Vec2DWorld {
    type Output = Vec2DWorld;

    fn add(self, rhs: Vec2DWorld) -> Vec2DWorld {
        Vec2DWorld {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2DWorld {
    type Output = Vec2DWorld;

    fn sub(self, rhs: Vec2DWorld) -> Vec2DWorld {
        Vec2DWorld {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Div<f64> for Vec2DWorld {
    type Output = Vec2DWorld;

    fn div(self, rhs: f64) -> Vec2DWorld {
        Vec2DWorld {
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

impl From<[f64; 2]> for Vec2DWorld {
    fn from(other: [f64; 2]) -> Vec2DWorld {
        Vec2DWorld::new(other[0], other[1])
    }
}

impl From<(f64, f64)> for Vec2DWorld {
    fn from(other: (f64, f64)) -> Vec2DWorld {
        Vec2DWorld::new(other.0, other.1)
    }
}

impl From<Vec2D> for Vec2DWorld {
    fn from(point: Vec2D) -> Vec2DWorld {
        Vec2DWorld {
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

impl fmt::Display for Vec2DWorld {
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
        let p1 = Vec2DWorld::new(0.0, 0.0);
        let p2 = Vec2DWorld::new(3.0, 4.0);

        assert_eq!(p1.distance(p2), 5.0);
    }

    #[test]
    fn angle_between_two_identical_vectors_is_zero() {
        let p = Vec2DWorld { x: 200.4200439453125, y: -80.388671875 };

        assert_eq!((p - p).angle(), Angle::from_radians(0.0));
    }
}
