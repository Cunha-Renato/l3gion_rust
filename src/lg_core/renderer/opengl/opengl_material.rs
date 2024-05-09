use crate::lg_core::{lg_types::reference::Rfc, uuid::UUID};

use super::{opengl_program::GlProgram, opengl_shader::GlShader, opengl_texture::GlTexture, opengl_uniform_buffer::GlUniformBuffer};

#[derive(Default)]
pub(crate) struct GlMaterial {
    pub(crate) program: Rfc<GlProgram>,
    pub(crate) texture: Option<Rfc<GlTexture>>,
    pub(crate) shaders: Vec<Rfc<GlShader>>,
    pub(crate) ubos: Vec<GlUniformBuffer>
    
}
impl GlMaterial {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}