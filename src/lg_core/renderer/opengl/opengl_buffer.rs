use std::ffi::c_void;

use crate::gl_check;

#[derive(Debug)]
pub(crate) struct GlBuffer {
    id: gl::types::GLuint,
    target: gl::types::GLuint
}
impl GlBuffer {
    pub(crate) unsafe fn new(target: gl::types::GLuint) -> Self {
        let mut id = 0;
        gl_check!(gl::GenBuffers(1, &mut id));
        
        Self { id, target }
    }
    pub unsafe fn bind(&self) {
        gl_check!(gl::BindBuffer(self.target, self.id));
    }
    pub unsafe fn bind_base(&self, binding_index: usize) {
        gl_check!(gl::BindBufferBase(
            self.target, 
            binding_index as gl::types::GLuint, 
            self.id
        ));
    }
    pub unsafe fn unbind(&self) {
        gl_check!(gl::BindBuffer(self.target, 0));
    }
    pub unsafe fn set_data<D>(&self, data: &[D], usage: gl::types::GLuint) {
        let (_, data_bytes, _) = data.align_to::<u8>();
        gl_check!(gl::BufferData(
            self.target,
            data_bytes.len() as gl::types::GLsizeiptr,
            data_bytes.as_ptr() as *const _,
            usage
        ));
    }
    pub unsafe fn set_data_full(
        &self,
        size: usize,
        data: *const c_void,
        usage: gl::types::GLuint
    ) {
        gl_check!(gl::BufferData(
            self.target, 
            size as gl::types::GLsizeiptr, 
            data, 
            usage
        ));
    }
    pub unsafe fn map(&self, access: gl::types::GLenum) -> *mut std::ffi::c_void {
        let result;
        gl_check!(result = gl::MapBuffer(self.target, access));
        
        result
    }
    pub unsafe fn unmap(&self) {
        gl_check!(gl::UnmapBuffer(self.target));
    }
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
    pub fn target(&self) -> gl::types::GLuint {
        self.target
    }
}
impl Drop for GlBuffer {
    fn drop(&mut self) {
        unsafe { gl_check!(gl::DeleteBuffers(1, [self.id].as_ptr())); }
    }
}