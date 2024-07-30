use crate::{gl_check, lg_core::renderer::texture::Texture, };

use super::GlError;

#[derive(Debug, Default, Clone)]
pub(crate) struct GlTexture {
    pub(crate) id: gl::types::GLuint,
}
impl GlTexture {
    pub(crate) fn new() -> Result<Self, GlError> {
        let mut id = 0;
        gl_check!(gl::GenTextures(1, &mut id), "Failed to generate texture!")?;
        
        Ok(Self { id })
    }
    pub(crate) fn bind(&self, location: u32) -> Result<(), GlError> {
        gl_check!(gl::ActiveTexture(gl::TEXTURE0 + location), "Failed to activate texture! (binding)")?;
        gl_check!(gl::BindTexture(gl::TEXTURE_2D, self.id), "Failed to bind texture! (binding)")
    }
    pub(crate) fn unbind(&self) -> Result<(), GlError> {
        gl_check!(gl::BindTexture(gl::TEXTURE_2D, 0), "Failed to unbind texture!")
    }
    pub(crate) fn load(&self, texture: &Texture) -> Result<(), GlError> {
        gl_check!(
            gl::TexImage2D(
                gl::TEXTURE_2D, 
                0, 
                texture.specs().tex_format.to_opengl() as i32,
                texture.width() as i32, 
                texture.height() as i32, 
                0, 
                texture.specs().tex_format.to_opengl(),
                texture.specs().tex_type.to_opengl(), 
                texture.bytes().as_ptr() as *const _,
            ),
            "Failed to load texture!"
        )?;

        gl_check!(gl::GenerateMipmap(gl::TEXTURE_2D), "Failed to generate mip map!")?;
        gl_check!(gl::GenerateTextureMipmap(self.id), "Failed to generate mip map for texture!")
    }
}
impl Drop for GlTexture {
    fn drop(&mut self) {
        if self.id != 0 {
            gl_check!(gl::DeleteTextures(1, [self.id].as_ptr()), "Failed to delete texture!").unwrap();
        }
    }
}