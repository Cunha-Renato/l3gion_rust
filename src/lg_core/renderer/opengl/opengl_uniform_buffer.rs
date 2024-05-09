use crate::lg_core::{lg_types::reference::Rfc, renderer::uniform::LgUniform};

use super::opengl_buffer::GlBuffer;

#[derive(Clone)]
pub(crate) struct GlUniformBuffer {
    pub buffer: Rfc<GlBuffer>,
    pub specs: LgUniform,
}
impl GlUniformBuffer {
    pub(crate) unsafe fn new(buffer: Rfc<GlBuffer>, specs: LgUniform) -> Self {
        Self {
            buffer,
            specs: specs,
        }
    }
    pub(crate) unsafe fn bind(&self) {
        self.buffer.borrow().bind();
    }
    pub(crate) unsafe fn bind_base(&self) {
        self.buffer.borrow().bind_base(self.specs.binding());
    }
    pub(crate) unsafe fn set_data<D>(&self, data: &[D], usage: gl::types::GLuint) {
        self.buffer.borrow().set_data(data, usage)
    }
    pub(crate) unsafe fn set_data_full(&self, usage: gl::types::GLuint) {
        self.buffer.borrow().set_data_full(
            self.specs.data.size(), 
            self.specs.data(), 
            usage
        );
    }
    pub(crate) unsafe fn unbind(&self) {
        self.buffer.borrow().unbind();
    }
}