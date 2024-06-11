pub struct LgWindow {
    window: winit::window::Window,
}
// Public
impl LgWindow {
    pub fn size(&self) -> (u32, u32) {
        (self.window.inner_size().width, self.window.inner_size().height)
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