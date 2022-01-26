use crate::point::{Vec2D, Unit, WorldUnit, ScreenUnit, Unittless};
use crate::geom::Angle;

fn multiply(t2: &Transform, t1: &Transform) -> Transform {
    Transform {
        xx: t1.xx * t2.xx + t1.yx * t2.xy,
        yx: t1.xx * t2.yx + t1.yx * t2.yy,
        xy: t1.xy * t2.xx + t1.yy * t2.xy,
        yy: t1.xy * t2.yx + t1.yy * t2.yy,
        x0: t1.x0 * t2.xx + t1.y0 * t2.xy + t2.x0,
        y0: t1.x0 * t2.yx + t1.y0 * t2.yy + t2.y0,
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub xx: f64,
    pub yx: f64,
    pub xy: f64,
    pub yy: f64,
    pub x0: f64,
    pub y0: f64,
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            xx: 1.0,
            yx: 0.0,
            xy: 0.0,
            yy: 1.0,
            x0: 0.0,
            y0: 0.0,
        }
    }
}

impl Transform {
    fn new(xx: f64, xy: f64, x0: f64, yx: f64, yy: f64, y0: f64) -> Transform {
        Transform {
            xx, yx, xy, yy, x0, y0,
        }
    }

    pub fn new_scale(factor: f64) -> Transform {
        Transform::new(
            factor,    0.0, 0.0,
            0.0,    factor, 0.0,
        )
    }

    pub fn new_scale_x(factor: f64) -> Transform {
        Transform::new(
            factor, 0.0, 0.0,
            0.0,    1.0, 0.0,
        )
    }

    pub fn new_scale_y(factor: f64) -> Transform {
        Transform::new(
            1.0,    0.0, 0.0,
            0.0, factor, 0.0,
        )
    }

    pub fn new_translate(vector: Vec2D<Unittless>) -> Transform {
        Transform::new(
            1.0, 0.0, vector.x.val(),
            0.0, 1.0, vector.y.val(),
        )
    }

    /// Returns a rotating transform that rotates the world `angle` degrees
    /// counterclockwise
    pub fn new_rotate(angle: Angle) -> Transform {
        let theta = angle.radians();

        Transform::new(
            theta.cos(), -theta.sin(), 0.0,
            theta.sin(),  theta.cos(), 0.0,
        )
    }

    pub fn new_shear_x(factor: f64) -> Transform {
        Transform::new(
            1.0, factor, 0.0,
            0.0,    1.0, 0.0,
        )
    }

    /// Computes a transform to go back to 1.0 zoom with the drawing's origin centered in the viewport.
    pub fn default_for_viewport(viewport_dimensions: Vec2D<ScreenUnit>) -> Transform {
        Transform::new_translate(Vec2D::new_unitless(viewport_dimensions.x.val(), viewport_dimensions.y.val()) * 0.5)
    }

    /// Take the current transformation matrix and scale it by this factor. New
    /// transformation is retured.
    ///
    /// Implemented as multiplying the current matrix to the left by a scaling
    /// matrix
    pub fn scale(&self, factor: f64) -> Transform {
        multiply(&Transform::new_scale(factor), self)
    }

    pub fn scale_x(&self, factor: f64) -> Transform {
        multiply(&Transform::new_scale_x(factor), self)
    }

    pub fn scale_y(&self, factor: f64) -> Transform {
        multiply(&Transform::new_scale_y(factor), self)
    }

    /// Take the current transformation matrix and translate it by this factor.
    /// New transformation is retured.
    ///
    /// Implemented as multiplying the current matrix to the left by a
    /// translation matrix
    pub fn translate(&self, delta: Vec2D<Unittless>) -> Transform {
        multiply(&Transform::new_translate(delta), self)
    }

    /// Take the current transformation matrix and rotate it by this angle.
    /// New transformation is retured.
    ///
    /// Implemented as multiplying the current matrix to the left by a rotation
    /// matrix.
    pub fn rotate(&self, angle: Angle) -> Transform {
        multiply(&Transform::new_rotate(angle), self)
    }

    pub fn shear_x(&self, factor: f64) -> Transform {
        multiply(&Transform::new_shear_x(factor), self)
    }

    /// Objects on the screen look `factor` times bigger. Specified fixed point
    /// is, well, fixed in the sense that before and after the zoom this point
    /// maps to the same point of the world.
    pub fn zoom(&self, factor: f64, fixed: Vec2D<ScreenUnit>) -> Transform {
        let t = fixed * -1.0;
        let t1 = fixed;

        self.translate(t.to_vec2d()).scale(factor).translate(t1.to_vec2d())
    }

    /// Objects on the screen rotate `angle`. Rotation happens around the
    /// specified fixed point
    pub fn turn(&self, angle: Angle, fixed: Vec2D<ScreenUnit>) -> Transform {
        let t = fixed * -1.0;
        let t1 = fixed;

        self.translate(t.to_vec2d()).rotate(angle).translate(t1.to_vec2d())
    }

    /// Objects on the screen move by this delta in screen units
    pub fn r#move(&self, delta: Vec2D<ScreenUnit>) -> Transform {
        self.translate(delta.to_vec2d())
    }

    #[inline]
    fn transform_distance(&self, dx: f64, dy: f64) -> Vec2D<Unittless> {
        Vec2D::new_unitless(dx * self.xx + dy * self.xy, dx * self.yx + dy * self.yy)
    }

    #[inline]
    fn transform_point(&self, px: f64, py: f64) -> (f64, f64) {
        let Vec2D {x, y} = self.transform_distance(px, py);
        (x.val() + self.x0, y.val() + self.y0)
    }

    #[inline]
    fn determinant(&self) -> f64 {
        self.xx * self.yy - self.xy * self.yx
    }

    pub fn invert(&self) -> Self {
        let det = self.determinant();

        let inv_det = 1.0 / det;

        Transform {
            xx: inv_det * self.yy,
            yx: inv_det * (-self.yx),
            xy: inv_det * (-self.xy),
            yy: inv_det * self.xx,
            x0: inv_det * (self.xy * self.y0 - self.yy * self.x0),
            y0: inv_det * (self.yx * self.x0 - self.xx * self.y0),
        }
    }

    pub fn apply(&self, p: Vec2D<Unittless>) -> Vec2D<Unittless> {
        self.transform_point(p.x.val(), p.y.val()).into()
    }

    pub fn to_screen_coordinates(&self, p: Vec2D<WorldUnit>) -> Vec2D<ScreenUnit> {
        // directly transform
        self.transform_point(p.x.val(), p.y.val()).into()
    }

    pub fn to_world_coordinates(&self, p: Vec2D<ScreenUnit>) -> Vec2D<WorldUnit> {
        // invert matrix, then transform
        self.invert().transform_point(p.x.val(), p.y.val()).into()
    }

    pub fn to_world_units(&self, length: ScreenUnit) -> WorldUnit {
        let vec: Vec2D<WorldUnit> = self.invert().transform_distance(length.val(), 0.0).into();

        vec.magnitude()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_viewport_transform_centers_the_origin() {
        let dimensions = Vec2D::new_screen(200.0, 100.0);
        let t = Transform::default_for_viewport(dimensions);

        let center = dimensions / 2.0;
        let transformed = t.to_world_coordinates(center);
        assert_eq!(transformed, Vec2D::new_world(0.0, 0.0));
    }

    #[test]
    fn screen_center_is_drawing_origin() {
        let dimensions = Vec2D::new_screen(200.0, 100.0);
        let t = Transform::default_for_viewport(dimensions);

        let center = Vec2D::new_screen(100.0, 50.0);
        let transformed = t.to_world_coordinates(center);
        assert_eq!(transformed, Vec2D::new_world(0.0, 0.0));
    }

    #[test]
    fn drawing_origin_is_screen_center() {
        let dimensions = (200.0, 100.0).into();
        let t = Transform::default_for_viewport(dimensions);

        let origin = Vec2D::new_world(0.0, 0.0);
        let transformed = t.to_screen_coordinates(origin);
        assert_eq!(transformed, Vec2D::new_screen(100.0, 50.0));
    }

    #[test]
    /// Read this test to understand how the factor affects the coordinates
    /// while zooming
    fn zoom_in_scales_as_expected() {
        let dimensions = (200.0, 100.0).into();
        let a = Vec2D::new_world(0.0, 0.0);
        let b = Vec2D::new_world(3.0, 4.0);
        let t = Transform::default_for_viewport(dimensions);

        let t1 = t.zoom(2.0, dimensions);

        let a1 = t1.to_screen_coordinates(a);
        let b1 = t1.to_screen_coordinates(b);

        assert_eq!(a1.distance(b1), 10.0.into());
    }

    #[test]
    fn zoom_in_preserves_center_point() {
        let dimensions = Vec2D::new_screen(200.0, 100.0);
        let center = dimensions * 0.5;

        let t = Transform::default_for_viewport(dimensions);

        let fixed_point = t.to_world_coordinates(center);

        let t1 = t.zoom(2.0, center);
        let zoomed_in = t1.to_world_coordinates(center);
        assert_eq!(fixed_point, zoomed_in);
    }

    #[test]
    fn zoom_out_preserves_center_point() {
        let dimensions = Vec2D::new_screen(200.0, 100.0);
        let t = Transform::default_for_viewport(dimensions);

        let center = dimensions * 0.5;
        let fixed_point = t.to_world_coordinates(center);

        let t = t.zoom(0.5, center);
        let zoomed_in = t.to_world_coordinates(center);
        assert_eq!(fixed_point, zoomed_in);
    }

    #[test]
    fn translated_screen_point_preserves_drawing_coordinates() {
        let dimensions = Vec2D::new_screen(200.0, 100.0);
        let t = Transform::default_for_viewport(dimensions);

        let screen_point = Vec2D::new_screen(20.0, 30.0);
        let drawing_point = t.to_world_coordinates(screen_point);

        let t = t.r#move(Vec2D::new_screen(10.0, 5.0));
        let translated_screen_point = Vec2D::new_screen(30.0, 35.0);
        let translated_drawing_point = t.to_world_coordinates(translated_screen_point);
        assert_eq!(drawing_point, translated_drawing_point);
    }

    #[test]
    fn units_in_screen_coordinates_can_be_converted_to_world() {
        let dimensions = Vec2D::new_screen(200.0, 100.0);
        let mut t = Transform::default_for_viewport(dimensions);

        assert_eq!(t.to_world_units(1.0.into()), 1.0.into());

        t = t.zoom(2.0, dimensions);

        assert_eq!(t.to_world_units(1.0.into()), 0.5.into());

        t = t.zoom(0.5, dimensions);

        assert_eq!(t.to_world_units(1.0.into()), 1.0.into());

        t = t.zoom(0.5, dimensions);

        assert_eq!(t.to_world_units(1.0.into()), 2.0.into());
    }

    #[test]
    fn zoom_fixes_world_origin() {
        let dimensions = Vec2D::new_screen(200.0, 100.0);
        let center = dimensions / 2.0;
        let mut t = Transform::default_for_viewport(dimensions);

        assert_eq!(t.to_screen_coordinates(Vec2D::new_world(0.0, 0.0)), Vec2D::new_screen(100.0, 50.0));

        t = t.zoom(2.0, center);

        assert_eq!(t.to_screen_coordinates(Vec2D::new_world(0.0, 0.0)), Vec2D::new_screen(100.0, 50.0));

        t = t.zoom(2.0, center);

        assert_eq!(t.to_screen_coordinates(Vec2D::new_world(0.0, 0.0)), Vec2D::new_screen(100.0, 50.0));
    }

    #[test]
    fn zoom_fixes_viewport_center_even_when_translated() {
        let dimensions = Vec2D::new_screen(200.0, 100.0);
        let screen_center = dimensions / 2.0;
        let mut t = Transform::default_for_viewport(dimensions);

        // screen center is world origin
        assert_eq!(t.to_world_coordinates(screen_center), Vec2D::new_world(0.0, 0.0));

        // zoom in
        t = t.zoom(2.0, screen_center);

        // screen center is still world origin
        assert_eq!(t.to_world_coordinates(screen_center), Vec2D::new_world(0.0, 0.0));

        // move the screen by this vector
        t = t.r#move(Vec2D::new_screen(5.0, 0.0));

        // this is the new point in the world that maps to the screen center
        let fixed = t.to_world_coordinates(screen_center);
        // it is not the origin anymore
        assert_ne!(fixed, Vec2D::new_world(0.0, 0.0));

        // zoom out
        t = t.zoom(0.5, screen_center);

        // the screen center must map to the fixed point
        assert_eq!(t.to_world_coordinates(screen_center), fixed);
    }
}
