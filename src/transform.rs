use crate::point::Point;

#[derive(Copy, Clone)]
pub struct Transform {
    scale_factor: f64,
    offset: Point,
}

impl Transform {
    pub fn new(offset: Point, scale_factor: f64) -> Transform {
        Transform {
            offset,
            scale_factor,
        }
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn to_screen_coordinates(&self, p: Point) -> Point {
        (p - self.offset) * self.scale_factor
    }
}
