use graphics::{Viewport, Context};
use opengl_graphics::GlGraphics;

pub trait Graphics {
    fn draw<F, U>(&mut self, v: Viewport, f: F) -> U
        where F: FnOnce(Context, &mut Self) -> U;
}

impl Graphics for GlGraphics {
    fn draw<F, U>(&mut self, v: Viewport, f: F) -> U
        where F: FnOnce(Context, &mut Self) -> U
    {
        self.draw(v, f)
    }
}
