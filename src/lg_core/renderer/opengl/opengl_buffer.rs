#[derive(Debug)]
pub(crate) struct GlBuffer {
    id: gl::types::GLuint,
    target: gl::types::GLuint
}
impl GlBuffer {
    pub(crate) unsafe fn new(target: gl::types::GLuint) -> Self {
        let mut id = 0;
        gl::GenBuffers(1, &mut id);
        
        Self { id, target }
    }
    pub unsafe fn bind(&self) {
        gl::BindBuffer(self.target, self.id);
    }
    pub unsafe fn set_data<D>(&self, data: &[D], usage: gl::types::GLuint) {
        self.bind();
        let (_, data_bytes, _) = data.align_to::<u8>();
        gl::BufferData(
            self.target,
            data_bytes.len() as gl::types::GLsizeiptr,
            data_bytes.as_ptr() as *const _,
            usage
        );
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
        unsafe { gl::DeleteBuffers(1, [self.id].as_ptr()); }
    }
}