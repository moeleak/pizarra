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
}
