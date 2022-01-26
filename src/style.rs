use crate::color::Color;
use crate::point::Unit;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Stroke<T: Unit> {
    pub size: T,
    pub color: Color,
}

impl<T: Unit> Default for Stroke<T> {
    fn default() -> Stroke<T> {
        Stroke {
            size: 2.0.into(),
            color: Color::white(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Style<T: Unit> {
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<T>>,
}

impl<T: Unit> Default for Style<T> {
    fn default() -> Style<T> {
        Style {
            fill: None,
            stroke: Some(Default::default()),
        }
    }
}

impl<T: Unit> Style<T> {
    #[cfg(test)]
    pub fn red_line() -> Style<T> {
        Style {
            stroke: Some(Stroke {
                color: Color::red(),
                size: 2.0.into(),
            }),
            fill: None,
        }
    }

    pub fn circle_helper() -> Style<T> {
        Style {
            stroke: None,
            fill: Some(Color::gray().half_transparent()),
        }
    }

    pub fn red_circle_helper() -> Style<T> {
        Style {
            stroke: None,
            fill: Some(Color::red().half_transparent()),
        }
    }

    pub fn path_helper() -> Style<T> {
        Style {
            fill: None,
            stroke: Some(Stroke {
                color: Color::gray(),
                size: 1.0.into(),
            }),
        }
    }
}
