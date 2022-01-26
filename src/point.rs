use std::fmt;
use std::ops::{Mul, Add, Sub, Div, Neg, Rem};
use std::f64::consts::FRAC_PI_2;
use std::str::FromStr;
use std::num::ParseFloatError;

use num_traits::{float::Float, sign::Signed, bounds::Bounded};
use serde::{Serialize, Deserialize};

use crate::geom::Angle;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Zero, One, Num, ToPrimitive, NumCast, Float, Serialize, Deserialize)]
pub struct ScreenUnit(f64);
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Zero, One, Num, ToPrimitive, NumCast, Float, Serialize, Deserialize)]
pub struct WorldUnit(f64);
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Zero, One, Num, ToPrimitive, NumCast, Float, Serialize, Deserialize)]
pub struct Unittless(f64);

macro_rules! impls {
    ( $x:ident ) => {
        impl $x {
            pub const fn from_float(f: f64) -> Self {
                Self(f)
            }
        }

        impl From<f64> for $x {
            fn from(a: f64) -> Self {
                Self(a)
            }
        }

        impl Add for $x {
            type Output = Self;

            fn add(self, other: Self) -> Self {
                Self (self.0 + other.0)
            }
        }

        impl Sub for $x {
            type Output = Self;

            fn sub(self, other: Self) -> Self {
                Self (self.0 - other.0)
            }
        }

        impl Neg for $x {
            type Output = Self;

            fn neg(self) -> Self {
                Self (-self.0)
            }
        }

        impl Div<f64> for $x {
            type Output = Self;

            fn div(self, other: f64) -> Self {
                Self (self.0 / other)
            }
        }

        impl Mul for $x {
            type Output = Self;

            fn mul(self, other: Self) -> Self {
                Self (self.0 * other.0)
            }
        }

        impl Div for $x {
            type Output = Self;

            fn div(self, other: Self) -> Self {
                Self (self.0 / other.0)
            }
        }

        impl Rem for $x {
            type Output = Self;

            fn rem(self, other: Self) -> Self {
                Self (self.0 % other.0)
            }
        }

        impl Signed for $x {
            fn abs(&self) -> Self {
                if self.0.is_nan() {
                    Self (f64::NAN)
                } else {
                    Self (self.0.abs())
                }
            }

            fn abs_sub(&self, other: &Self) -> Self {
                if self.0 <= other.0 {
                    Self (0.0)
                } else {
                    Self (self.0 - other.0)
                }
            }

            fn signum(&self) -> Self {
                Self (self.0.signum())
            }

            fn is_positive(&self) -> bool {
                self.0.is_sign_positive()
            }

            fn is_negative(&self) -> bool {
                self.0.is_sign_negative()
            }
        }

        impl Bounded for $x {
            fn min_value() -> Self {
                Self (f64::MAX)
            }

            fn max_value() -> Self {
                Self (f64::MIN)
            }
        }

        impl fmt::Display for $x {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "{}", self.0)
            }
        }

        impl FromStr for $x {
            type Err = ParseFloatError;

            fn from_str(s: &str) -> Result<$x, Self::Err> {
                Ok(Self (s.parse()?))
            }
        }
    };
}

impls!(ScreenUnit);
impls!(WorldUnit);
impls!(Unittless);

pub trait Unit: Copy + Clone + From<f64> + PartialEq + PartialOrd + Add<Output=Self> + Sub<Output=Self> + Mul<Output=Self> + Div<Output=Self> + Div<f64, Output=Self> + Float + fmt::Display {
    fn val(self) -> f64;
}

impl Unit for ScreenUnit {
    fn val(self) -> f64 {
        self.0
    }
}
impl Unit for WorldUnit {
    fn val(self) -> f64 {
        self.0
    }
}
impl Unit for Unittless {
    fn val(self) -> f64 {
        self.0
    }
}

/// A 2D (unitless) vector
#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct Vec2D<T: Unit> {
    pub x: T,
    pub y: T,
}

impl Vec2D<Unittless> {
    pub fn new_unitless(x: f64, y: f64) -> Vec2D<Unittless> {
        Vec2D {
            x: Unittless(x),
            y: Unittless(y),
        }
    }
}

impl Vec2D<WorldUnit> {
    pub fn new_world(x: f64, y: f64) -> Vec2D<WorldUnit> {
        Vec2D {
            x: WorldUnit(x),
            y: WorldUnit(y),
        }
    }
}

impl Vec2D<ScreenUnit> {
    pub fn new_screen(x: f64, y: f64) -> Vec2D<ScreenUnit> {
        Vec2D {
            x: ScreenUnit(x),
            y: ScreenUnit(y),
        }
    }
}

impl<T: Unit> Vec2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn to_a(self) -> [f64; 2] {
        [self.x.val(), self.y.val()]
    }

    pub fn to_vec2d(self) -> Vec2D<Unittless> {
        Vec2D {
            x: Unittless(self.x.val()),
            y: Unittless(self.y.val()),
        }
    }

    pub fn magnitude(&self) -> T {
        self.distance(Self::new(0.0.into(), 0.0.into()))
    }

    pub fn distance(&self, other: Self) -> T {
        ((self.x.val() - other.x.val()).powi(2) + (self.y.val() - other.y.val()).powi(2)).sqrt().into()
    }

    pub fn abs(self) -> Self {
        Self::new(
            self.x.val().abs().into(),
            self.y.val().abs().into(),
        )
    }

    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.val().min(other.x.val()).into(),
            y: self.y.val().min(other.y.val()).into(),
        }
    }

    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.val().max(other.x.val()).into(),
            y: self.y.val().max(other.y.val()).into(),
        }
    }

    pub fn dot(self, other: Self) -> T {
        self.x * other.x + self.y * other.y
    }

    /// Returns the area of the parallelogram defined by the two vectors.
    ///
    /// The area is signed
    pub fn cross_area(self, other: Self) -> T {
        (self.x.val() * other.y.val() - self.y.val() * other.x.val()).into()
    }

    pub fn is_nan(&self) -> bool {
        self.x.val().is_nan() || self.y.val().is_nan()
    }

    pub fn unit(self) -> Self {
        self / self.magnitude()
    }

    pub fn angle(self) -> Angle {
        if self.x.val() == 0.0 && self.y.val() == 0.0 {
            Angle::from_radians(0.0)
        } else if self.x.val() == 0.0 {
            Angle::from_radians(FRAC_PI_2) * self.y.val().signum() * -1.0
        } else {
            Angle::from_radians((self.y.val() / self.x.val()).atan())
        }
    }
}

impl<T: Unit> Mul<f64> for Vec2D<T> {
    type Output = Vec2D<T>;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec2D {
            x: (self.x.val() * rhs).into(),
            y: (self.y.val() * rhs).into(),
        }
    }
}

impl<T: Unit> Mul<T> for Vec2D<T> {
    type Output = Vec2D<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Vec2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T: Unit> Add for Vec2D<T> {
    type Output = Vec2D<T>;

    fn add(self, rhs: Vec2D<T>) -> Vec2D<T> {
        Vec2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T: Unit> Sub for Vec2D<T> {
    type Output = Vec2D<T>;

    fn sub(self, rhs: Vec2D<T>) -> Vec2D<T> {
        Vec2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Unit> Div<f64> for Vec2D<T> {
    type Output = Vec2D<T>;

    fn div(self, rhs: f64) -> Vec2D<T> {
        Vec2D {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: Unit> Div<T> for Vec2D<T> {
    type Output = Vec2D<T>;

    fn div(self, rhs: T) -> Vec2D<T> {
        Vec2D {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T: Unit> Neg for Vec2D<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: (-self.x.val()).into(),
            y: (-self.y.val()).into(),
        }
    }
}

impl<T: Unit> From<(f64, f64)> for Vec2D<T> {
    fn from(other: (f64, f64)) -> Vec2D<T> {
        Vec2D::new(other.0.into(), other.1.into())
    }
}

impl From<Vec2D<Unittless>> for Vec2D<WorldUnit> {
    fn from(other: Vec2D<Unittless>) -> Vec2D<WorldUnit> {
        Vec2D::new_world(other.x.val(), other.y.val())
    }
}

impl<T: Unit> fmt::Display for Vec2D<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance() {
        let p1 = Vec2D::new_unitless(0.0, 0.0);
        let p2 = Vec2D::new_unitless(3.0, 4.0);

        assert_eq!(p1.distance(p2).val(), 5.0);
    }

    #[test]
    fn angle_between_two_identical_vectors_is_zero() {
        let p = Vec2D::new_unitless(200.4200439453125, -80.388671875);

        assert_eq!((p - p).angle(), Angle::from_radians(0.0));
    }

    #[test]
    fn angle_of_a_vertical_vector() {
        let p1 = Vec2D::new_unitless(0.0, -80.388671875);
        let p2 = Vec2D::new_unitless(0.0, 80.388671875);

        assert_eq!(p1.angle(), Angle::from_degrees(90.0));
        assert_eq!(p2.angle(), Angle::from_degrees(-90.0));
    }
}
