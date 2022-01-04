use crate::color::Color;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Stroke {
    pub size: f64,
    pub color: Color,
}

impl Default for Stroke {
    fn default() -> Stroke {
        Stroke {
            size: 2.0,
            color: Color::white(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Style {
    pub fill: Option<Color>,
    pub stroke: Option<Stroke>,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            fill: None,
            stroke: Some(Default::default()),
        }
    }
}

impl Style {
    pub fn red_line() -> Style {
        Style {
            stroke: Some(Stroke {
                color: Color::red(),
                size: 2.0,
            }),
            fill: None,
        }
    }

    pub fn circle_helper() -> Style {
        Style {
            stroke: None,
            fill: Some(Color::gray().half_transparent()),
        }
    }

    pub fn path_helper() -> Style {
        Style {
            fill: None,
            stroke: Some(Stroke {
                color: Color::gray(),
                size: 1.0,
            }),
        }
    }
}
