use sllog::warn;

use crate::gl_check;
use super::{gl_buffer::GlBuffer, GlError};

#[derive(Debug, Default)]
pub struct GlVertexArray {
    id: gl::types::GLuint,
    vertex_buffer: GlBuffer,
    index_buffer: GlBuffer,
}
impl GlVertexArray {
    pub(crate) fn new() -> Result<Self, GlError> {
        let mut id = 0;
        gl_check!(gl::GenVertexArrays(1, &mut id), "Failed to generate vertex array!")?;
        
        Ok(Self { 
            id, 
            vertex_buffer: GlBuffer::new(gl::ARRAY_BUFFER)?,
            index_buffer: GlBuffer::new(gl::ELEMENT_ARRAY_BUFFER)?,
        })
    }
    pub(crate) fn vertex_buffer(&self) -> &GlBuffer {
        &self.vertex_buffer
    }
    pub(crate) fn index_buffer(&self) -> &GlBuffer {
        &self.index_buffer
    }
    pub(crate) fn bind(&self) -> Result<(), GlError> {
        gl_check!(gl::BindVertexArray(self.id), "Failed o bind vertex array!")
    }
    pub(crate) fn unbind(&self) -> Result<(), GlError> {
        gl_check!(gl::BindVertexArray(0), "Failed to unbind vertex array!")
    }
    pub(crate) fn unbind_buffers(&self) -> Result<(), GlError> {
        self.vertex_buffer.unbind()?;
        self.index_buffer.unbind()
    }
    pub(crate) fn set_attribute(
        &self,
        attrib_pos: gl::types::GLuint,
        components: gl::types::GLint,
        stride: usize,
        offset: gl::types::GLint,
    ) -> Result<(), GlError> 
    {
        gl_check!(
            gl::VertexAttribPointer(
                attrib_pos, 
                components,
                gl::FLOAT, 
                gl::FALSE, 
                stride as gl::types::GLint, 
                offset as *const _,
            ),
            "Failed to call glVertexAttribPointer!"
        )?;
        gl_check!(gl::EnableVertexAttribArray(attrib_pos), "Failed to enable vertex attrib array!")
    }
}
impl Drop for GlVertexArray {
    fn drop(&mut self) {
        if self.id != 0 {
            self.unbind().unwrap();
            gl_check!(gl::DeleteVertexArrays(1, [self.id].as_ptr()), "Failed to delete vertex array!").unwrap();
        }
    }
}