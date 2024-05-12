use std::hash::Hash;

use lg_renderer::renderer::lg_uniform::LgUniform;

use crate::lg_core::{lg_types::reference::Rfc, uuid::UUID};
use super::{shader::LgShader, texture::LgTexture};

#[derive(Clone)]
pub struct LgMaterial {
    uuid: UUID,
    shaders: Vec<Rfc<LgShader>>,
    texture: Option<Rfc<LgTexture>>,
    pub uniforms: Vec<LgUniform>,
}
impl LgMaterial {
    pub fn new(shaders: Vec<Rfc<LgShader>>, texture: Option<Rfc<LgTexture>>, uniforms: Vec<LgUniform>) -> Self {
        Self {
            uuid: UUID::generate(),
            shaders,
            texture,
            uniforms,
        }
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    pub fn shaders(&self) -> &[Rfc<LgShader>] {
        &self.shaders
    }
    pub fn texture(&self) -> &Option<Rfc<LgTexture>> {
        &self.texture
    }
}
impl Hash for LgMaterial {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}