use crate::point::{Point, Vec2D, Vec2DDrawing, Vec2DScreen};

#[derive(Copy, Clone)]
pub struct Transform {
    scale_factor: f64,
    offset: Vec2D,
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            scale_factor: 1.0,
            offset: Vec2D::new(0.0, 0.0),
        }
    }
}

impl Transform {
    pub fn new(offset: Vec2D, scale_factor: f64) -> Transform {
        Transform {
            offset,
            scale_factor,
        }
    }

    pub fn with_rotation(self, _rotation: f64) -> Transform {
        Transform {
            ..self
        }
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    pub fn offset(&self) -> Vec2D {
        self.offset
    }

    pub fn translate_screen(&mut self, delta: Vec2DScreen) {
        self.offset = self.offset - delta.to_vec2d() / self.scale_factor;
    }

    pub fn rotate_screen(&mut self) {
    }

    pub fn zoom_in(&mut self, factor: f64, dimensions: Vec2DScreen) {
        self.offset = self.offset + ((dimensions / self.scale_factor) * 0.25).to_vec2d();
        self.scale_factor *= factor;
    }

    pub fn zoom_out(&mut self, factor: f64, dimensions: Vec2DScreen) {
        self.offset = self.offset - ((dimensions / self.scale_factor) * 0.5).to_vec2d();
        self.scale_factor /= factor;
    }

    pub fn to_screen_coordinates(&self, p: Vec2DDrawing) -> Vec2DScreen {
        ((p.to_vec2d() - self.offset) * self.scale_factor).into()
    }

    pub fn to_storage_coordinates(&self, p: Vec2DScreen) -> Vec2DDrawing {
        (p.to_vec2d() / self.scale_factor + self.offset).into()
    }
}

/// Computes a transform to go back to 1.0 zoom with the drawing's origin centered in the viewport.
pub fn default_viewport_transform(viewport_dimensions: Vec2DScreen) -> Transform {
    Transform::new(viewport_dimensions.to_vec2d() * -0.5, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_viewport_transform_centers_the_origin() {
        let dimensions = Vec2DScreen::new(200.0, 100.0);
        let t = default_viewport_transform(dimensions);

        let center = Vec2DScreen::new(100.0, 50.0);
        let transformed = t.to_storage_coordinates(center);
        assert_eq!(transformed, Vec2DDrawing::new(0.0, 0.0));
    }

    #[test]
    fn screen_center_is_drawing_origin() {
        let dimensions = Vec2D::new(200.0, 100.0);
        let t = Transform::new(dimensions * -0.5, 1.0);

        let center = Vec2DScreen::new(100.0, 50.0);
        let transformed = t.to_storage_coordinates(center);
        assert_eq!(transformed, Vec2DDrawing::new(0.0, 0.0));
    }

    #[test]
    fn drawing_origin_is_screen_center() {
        let dimensions = Vec2D::new(200.0, 100.0);
        let t = Transform::new(dimensions * -0.5, 1.0);

        let origin = Vec2DDrawing::new(0.0, 0.0);
        let transformed = t.to_screen_coordinates(origin);
        assert_eq!(transformed, Vec2DScreen::new(100.0, 50.0));
    }

    #[test]
    fn zoom_in_preserves_center_point() {
        let dimensions = Vec2DScreen::new(200.0, 100.0);
        let mut t = Transform::new(dimensions.to_vec2d(), 1.0);

        let center = dimensions * 0.5;
        let fixed_point = t.to_storage_coordinates(center);

        t.zoom_in(2.0, dimensions);
        let zoomed_in = t.to_storage_coordinates(center);
        assert_eq!(fixed_point, zoomed_in);
    }

    #[test]
    fn zoom_out_preserves_center_point() {
        let dimensions = Vec2DScreen::new(200.0, 100.0);
        let mut t = Transform::new(dimensions.to_vec2d(), 1.0);

        let center = dimensions * 0.5;
        let fixed_point = t.to_storage_coordinates(center);

        t.zoom_out(2.0, dimensions);
        let zoomed_in = t.to_storage_coordinates(center);
        assert_eq!(fixed_point, zoomed_in);
    }

    #[test]
    fn translated_screen_point_preserves_drawing_coordinates() {
        let dimensions = Vec2DScreen::new(200.0, 100.0);
        let mut t = Transform::new(dimensions.to_vec2d(), 2.0);

        let screen_point = Vec2DScreen::new(20.0, 30.0);
        let drawing_point = t.to_storage_coordinates(screen_point);

        t.translate_screen(Vec2DScreen::new(10.0, 5.0));
        let translated_screen_point = Vec2DScreen::new(30.0, 35.0);
        let translated_drawing_point = t.to_storage_coordinates(translated_screen_point);
        assert_eq!(drawing_point, translated_drawing_point);
    }
}
