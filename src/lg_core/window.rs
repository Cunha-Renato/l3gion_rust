pub struct LgWindow {
    width: u32,
    height: u32,
}
impl LgWindow {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
    pub const fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn set_size(&mut self, size: (u32, u32)) {
        self.width = size.0;
        self.height = size.1;
    }
}