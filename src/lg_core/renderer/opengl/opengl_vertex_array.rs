#[derive(Debug)]
pub struct GlVertexArray {
    id: gl::types::GLuint,
}
impl GlVertexArray {
    pub(crate) unsafe fn new() -> Self {
        let mut id = 0;
        gl::GenVertexArrays(1, &mut id);
        
        Self { id }
    }
    pub(crate) unsafe fn bind(&self) {
        gl::BindVertexArray(self.id);
    }
    pub(crate) unsafe fn unbind(&self) {
        gl::BindVertexArray(0);
    }
    pub(crate) unsafe fn set_attribute<V: Sized>(
        &self,
        attrib_pos: gl::types::GLuint,
        components: gl::types::GLint,
        offset: gl::types::GLint,
    ) {
        self.bind();
        gl::VertexAttribPointer(
            attrib_pos, 
            components,
            gl::FLOAT, 
            gl::FALSE, 
            std::mem::size_of::<V>() as gl::types::GLint, 
            offset as *const _,
        );
        gl::EnableVertexAttribArray(attrib_pos);
        self.unbind();
    }
}
impl Drop for GlVertexArray {
    fn drop(&mut self) {
        unsafe { gl::DeleteVertexArrays(1, [self.id].as_ptr()); }
    }
}