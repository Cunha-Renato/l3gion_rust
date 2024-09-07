use crate::glm;

#[derive(Debug)]
pub struct LgWindow {
    pub(crate) window: winit::window::Window,
}
// Public
impl LgWindow {
    pub fn size(&self) -> glm::Vec2 {
        glm::vec2(self.window.inner_size().width as f32, self.window.inner_size().height as f32)
    }
}
// Public(crate)
impl LgWindow {
    pub(crate) fn new(window: winit::window::Window) -> Self {
        Self { window }
    }

    pub(crate) fn request_redraw(&self) {
        self.window.request_redraw();
    }
}