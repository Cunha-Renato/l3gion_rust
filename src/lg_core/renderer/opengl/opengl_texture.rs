use crate::{gl_check, lg_core::{lg_types::reference::Rfc, renderer::texture::LgTexture}};

#[derive(Debug)]
pub(crate) struct GlTexture {
    id: gl::types::GLuint,
    texture: Rfc<LgTexture>,
}
impl GlTexture {
    pub(crate) unsafe fn new(texture: Rfc<LgTexture>) -> Self {
        let mut id = 0;
        gl_check!(gl::GenTextures(1, &mut id));
        
        Self { id, texture }
    }
    pub(crate) unsafe fn bind(&self) {
        gl_check!(gl::BindTexture(gl::TEXTURE_2D, self.id));
    }
    pub(crate) unsafe fn unbind(&self) {
        gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0));
    }
    pub(crate) unsafe fn load(&self) {
        let borrow = self.texture.borrow();
        gl_check!(gl::TexImage2D(
            gl::TEXTURE_2D, 
            0, 
            gl::RGBA as i32,
            borrow.width() as i32, 
            borrow.height() as i32, 
            0, 
            gl::RGBA, 
            gl::UNSIGNED_BYTE, 
            borrow.bytes().as_ptr() as *const _,
        ));

        gl_check!(gl::GenerateMipmap(gl::TEXTURE_2D));
    }
}
impl Drop for GlTexture {
    fn drop(&mut self) {
        unsafe { gl_check!(gl::DeleteTextures(1, [self.id].as_ptr())); }
    }
}