use crate::point::{Point, Vec2D, Vec2DDrawing, Vec2DScreen};

#[derive(Copy, Clone, Default)]
pub struct Transform {
    scale_factor: f64,
    offset: Vec2D,
    rotation: f64,
}

impl Transform {
    pub fn new(offset: Vec2D, scale_factor: f64) -> Transform {
        Transform {
            offset,
            scale_factor,
            rotation: 0.0,
        }
    }

    pub fn with_rotation(self, rotation: f64) -> Transform {
        Transform {
            rotation,
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

    pub fn go_home(&mut self, center: Vec2DScreen) {
        self.offset = (center * -0.5).into();
        self.scale_factor = 1.0;
    }

    pub fn to_screen_coordinates(&self, p: Vec2DDrawing) -> Vec2DScreen {
        ((p.to_vec2d() - self.offset) * self.scale_factor).into()
    }

    pub fn to_storage_coordinates(&self, p: Vec2DScreen) -> Vec2DDrawing {
        (p.to_vec2d() / self.scale_factor + self.offset).into()
    }
}
