use crate::lg_core::lg_types::reference::Rfc;

use super::opengl_buffer::GlBuffer;

#[derive(Debug, Clone)]
pub(crate) struct GlUniformBuffer {
    pub buffer: Rfc<GlBuffer>,
    binding: usize,
}
impl GlUniformBuffer {
    pub(crate) unsafe fn new(buffer: Rfc<GlBuffer>, binding: usize) -> Self {
        Self {
            buffer,
            binding,
        }
    }
    pub(crate) fn binding(&self) -> usize {
        self.binding
    }
    pub(crate) unsafe fn bind(&self) {
        self.buffer.borrow().bind();
    }
    pub(crate) unsafe fn bind_base(&self) {
        self.buffer.borrow().bind_base(self.binding);
    }
    pub(crate) unsafe fn unbind(&self) {
        self.buffer.borrow().unbind();
    }
}