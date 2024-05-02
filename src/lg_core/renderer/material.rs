use crate::lg_core::{lg_types::reference::Rfc, uuid::UUID};

use super::{shader::LgShader, texture::LgTexture, uniform::LgUniform};

#[derive(Clone)]
pub struct LgMaterial {
    uuid: UUID,
    shaders: Vec<Rfc<LgShader>>,
    uniforms: Vec<LgUniform>,
}
impl LgMaterial {
    pub fn new(shaders: Vec<Rfc<LgShader>>, uniforms: Vec<LgUniform>) -> Self {
        Self {
            uuid: UUID::generate(),
            shaders,
            uniforms,
        }
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    pub fn shaders(&self) -> &[Rfc<LgShader>] {
        &self.shaders
    }
    pub fn uniforms(&self) -> &[LgUniform] {
        &self.uniforms
    }
}