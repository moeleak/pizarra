use std::ops::{Add, Sub, Mul, Neg};

use crate::point::{Vec2D, Unit};

use super::dot;

/// Stores the angle in radians internally
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Angle(f64);

impl Angle {
    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }

    pub fn degrees(self) -> f64 {
        self.0.to_degrees()
    }

    pub fn radians(self) -> f64 {
        self.0
    }

    pub fn from_radians(radians: f64) -> Angle {
        Angle(radians)
    }

    pub fn from_degrees(degrees: f64) -> Angle {
        Angle(degrees.to_radians())
    }

    pub fn between<T: Unit>(a: Vec2D<T>, b: Vec2D<T>) -> Angle {
        if a == b {
            return Angle::from_radians(0.0);
        }

        let angle = (dot(a, b)/(a.magnitude() * b.magnitude())).acos();

        Self::from_radians(if a.x.val() * b.y.val() - a.y.val() * b.x.val() < 0.0 {
            -angle
        } else {
            angle
        })
    }
}

impl Mul<f64> for Angle {
    type Output = Self;

    fn mul(self, s: f64) -> Angle {
        Angle::from_radians(self.radians() *  s)
    }
}

impl Add<Angle> for Angle {
    type Output = Self;

    fn add(self, other: Self) -> Angle {
        Angle::from_radians(self.0 + other.0)
    }
}

impl Sub<Angle> for Angle {
    type Output = Self;

    fn sub(self, other: Self) -> Angle {
        Angle::from_radians(self.0 - other.0)
    }
}

impl Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self {
        Angle(-self.0)
    }
}
