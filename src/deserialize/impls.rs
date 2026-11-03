use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

use crate::geom::Angle;

use super::Error;

lazy_static! {
    static ref POINT_REGEX: Regex = Regex::new(
        r"(?x)
        (?P<x>-?\d+(\.\d+)?)
        \s+
        (?P<y>-?\d+(\.\d+)?)
    "
    )
    .unwrap();
    static ref ANGLE_REGEX: Regex = Regex::new(
        r"(?x)
        rotate\(
            (?P<angle>-?\d+(\.\d*)?)
            (\s+(-?\d+(\.\d*)?)\s+(-?\d+(\.\d*)?))?
        \)
    "
    )
    .unwrap();
}

impl FromStr for Angle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Angle, Error> {
        if let Some(angle) = ANGLE_REGEX.captures(s) {
            Ok(Angle::from_degrees(
                angle["angle"]
                    .parse()
                    .map_err(|_| Error::AngleDoesntMatchRegex(s.into()))?,
            ))
        } else {
            Err(Error::AngleDoesntMatchRegex(s.into()))
        }
    }
}
