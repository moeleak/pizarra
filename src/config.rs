use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::color::Color;
use crate::point::{ScreenUnit, WorldUnit};

#[serde_as]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Thikness in pixels used by default while drawing.
    pub thickness: ScreenUnit,

    /// Padding in pixels added to drawings while exporting.
    pub export_padding: WorldUnit,

    /// If the cursor's distance to a shape is less than this it will be
    /// considered to be touching and thus be erased.
    pub erase_radius: ScreenUnit,

    /// When drawing polygons and possibly other shapes in the future this
    /// radius in screen units is used to decide if the cursor is touching a
    /// previous point of the current shape.
    pub point_snap_radius: ScreenUnit,

    /// When pressing + and - keys on the keyboard the zoom increases and
    /// decreases by this factor (must be > 1 so that + zooms in).
    pub zoom_factor: f64,

    /// When creating a new drawing this will be the background color. Specified
    /// in CSS hexadecimal format, like '#cebada'
    #[serde_as(as = "DisplayFromStr")]
    pub background_color: Color,

    /// When creating a new drawing this will be the starting color of the pen.
    /// Specified in CSS hexadecimal format.
    #[serde_as(as = "DisplayFromStr")]
    pub stroke_color: Color,

    /// multiply the scroll delta given by the scroll event by this factor to
    /// make it smoother or faster
    pub scroll_factor: f64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            thickness: 2.0.into(),
            export_padding: 20.0.into(),
            erase_radius: 10.0.into(),
            point_snap_radius: 10.0.into(),
            zoom_factor: 2.0,
            background_color: Color::black(),
            stroke_color: Color::white(),
            scroll_factor: -5.0,
        }
    }
}
