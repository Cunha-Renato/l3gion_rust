use crate::gl_check;

use super::opengl_buffer::GlBuffer;

#[derive(Debug)]
pub struct GlVertexArray {
    id: gl::types::GLuint,
    vertex_buffer: GlBuffer,
    index_buffer: GlBuffer,
}
impl GlVertexArray {
    pub(crate) unsafe fn new() -> Self {
        let mut id = 0;
        gl_check!(gl::GenVertexArrays(1, &mut id));
        
        Self { id, vertex_buffer: GlBuffer::new(gl::ARRAY_BUFFER), index_buffer: GlBuffer::new(gl::ELEMENT_ARRAY_BUFFER) }
    }
    pub(crate) fn vertex_buffer(&self) -> &GlBuffer {
        &self.vertex_buffer
    }
    pub(crate) fn index_buffer(&self) -> &GlBuffer {
        &self.index_buffer
    }
    pub(crate) unsafe fn bind(&self) {
        gl_check!(gl::BindVertexArray(self.id));
    }
    pub(crate) unsafe fn unbind(&self) {
        gl_check!(gl::BindVertexArray(0));
    }
    pub(crate) unsafe fn bind_buffers(&self) {
        self.vertex_buffer.bind();
        self.index_buffer.bind();
    }
    pub(crate) unsafe fn unbind_buffers(&self) {
        self.vertex_buffer.unbind();
        self.index_buffer.unbind();
    }
    pub(crate) unsafe fn set_attribute<V: Sized>(
        &self,
        attrib_pos: gl::types::GLuint,
        components: gl::types::GLint,
        offset: gl::types::GLint,
    ) {
        gl_check!(gl::VertexAttribPointer(
            attrib_pos, 
            components,
            gl::FLOAT, 
            gl::FALSE, 
            std::mem::size_of::<V>() as gl::types::GLint, 
            offset as *const _,
        ));
        gl_check!(gl::EnableVertexAttribArray(attrib_pos));
    }
}
impl Drop for GlVertexArray {
    fn drop(&mut self) {
        unsafe { 
            self.unbind_buffers();
            self.unbind();
            gl_check!(gl::DeleteVertexArrays(1, [self.id].as_ptr()));
        }
    }
}