use serde::{Serialize, Deserialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::color::Color;

#[serde_as]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Thikness in pixels used by default while drawing.
    #[serde(default = "default_thickness")]
    pub thickness: f64,

    /// Padding in pixels added to drawings while exporting.
    #[serde(default = "default_export_padding")]
    pub export_padding: f64,

    /// If the cursor's distance to a shape is less than this it will be
    /// considered to be touching and thus be erased.
    #[serde(default = "default_erase_radius")]
    pub erase_radius: f64,

    /// When drawing polygons and possibly other shapes in the future this
    /// radius in screen units is used to decide if the cursor is touching a
    /// previous point of the current shape.
    #[serde(default = "default_point_snap_radius")]
    pub point_snap_radius: f64,

    /// When pressing + and - keys on the keyboard the zoom increases and
    /// decreases by this factor (must be > 1 so that + zooms in).
    #[serde(default = "default_zoom_factor")]
    pub zoom_factor: f64,

    /// When creating a new drawing this will be the background color. Specified
    /// in CSS hexadecimal format, like '#cebada'
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_background_color")]
    pub background_color: Color,

    /// When creating a new drawing this will be the starting color of the pen.
    /// Specified in CSS hexadecimal format.
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_stroke_color")]
    pub stroke_color: Color,

    /// multiply the scroll delta given by the scroll event by this factor to
    /// make it smoother or faster
    #[serde(default = "default_scroll_factor")]
    pub scroll_factor: f64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            thickness: 3.0,
            export_padding: 20.0,
            erase_radius: 10.0,
            point_snap_radius: 10.0,
            zoom_factor: 2.0,
            background_color: Color::black(),
            stroke_color: Color::white(),
            scroll_factor: -5.0,
        }
    }
}

fn default_thickness() -> f64 {
    Config::default().thickness
}

fn default_export_padding() -> f64 {
    Config::default().export_padding
}

fn default_erase_radius() -> f64 {
    Config::default().erase_radius
}

fn default_point_snap_radius() -> f64 {
    Config::default().point_snap_radius
}

fn default_zoom_factor() -> f64 {
    Config::default().zoom_factor
}

fn default_background_color() -> Color {
    Config::default().background_color
}

fn default_stroke_color() -> Color {
    Config::default().stroke_color
}

fn default_scroll_factor() -> f64 {
    Config::default().scroll_factor
}
