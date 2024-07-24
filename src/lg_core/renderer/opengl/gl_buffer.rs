use std::{ffi::c_void, hash::Hash};

use crate::gl_check;

use super::GlError;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct GlBuffer {
    id: gl::types::GLuint,
    target: gl::types::GLuint
}
impl GlBuffer {
    pub(crate) fn new(target: gl::types::GLuint) -> Result<Self, GlError> {
        let mut id = 0;
        gl_check!(gl::GenBuffers(1, &mut id), "Failed to create buffer!")?;
        
        Ok(Self { id, target })
    }
    pub fn bind(&self) -> Result<(), GlError> {
        gl_check!(gl::BindBuffer(self.target, self.id), "Failed to bind buffer!")
    }
    pub fn bind_base(&self, binding_index: usize) -> Result<(), GlError> {
        gl_check!(
            gl::BindBufferBase(
                self.target, 
                binding_index as gl::types::GLuint, 
                self.id
            ), 
            "Failed to bind base!"
        )
    }
    pub fn unbind(&self) -> Result<(), GlError> {
        gl_check!(gl::BindBuffer(self.target, 0), "Failed to unbind buffer!")
    }
    pub fn set_data<D>(&self, data: &[D], usage: gl::types::GLuint) -> Result<(), GlError> {
        let (_, data_bytes, _) = unsafe { data.align_to::<u8>() };
        gl_check!(
            gl::BufferData(
                self.target,
                data_bytes.len() as gl::types::GLsizeiptr,
                data_bytes.as_ptr() as *const _,
                usage
            ),
            "Failed to set data!"
        )
    }
    pub fn set_data_full(
        &self,
        size: usize,
        data: *const c_void,
        usage: gl::types::GLuint
    ) -> Result<(), GlError> 
    {
        gl_check!(
            gl::BufferData(
                self.target, 
                size as gl::types::GLsizeiptr, 
                data, 
                usage
            ),
            "Failed to set data full!"
        )
    }
    pub fn map(&self, access: gl::types::GLenum) -> Result<*mut std::ffi::c_void, GlError> {
        let result;
        gl_check!(result = gl::MapBuffer(self.target, access), "Failed to map buffer!")?;
        
        Ok(result)
    }
    pub fn unmap(&self) -> Result<(), GlError> {
        gl_check!(gl::UnmapBuffer(self.target), "Failed to unmap buffer!")
    }
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
    pub fn target(&self) -> gl::types::GLuint {
        self.target
    }
}
impl Hash for GlBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl Drop for GlBuffer {
    fn drop(&mut self) {
        if self.id != 0 {
            self.unbind().unwrap();
            gl_check!(gl::DeleteBuffers(1, [self.id].as_ptr()), "Failed to delete buffer!").unwrap();
        }
    }
}