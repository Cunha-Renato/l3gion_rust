use std::hash::Hash;

use lg_renderer::renderer::lg_uniform::LgUniform;

use crate::lg_core::{lg_types::reference::Rfc, uuid::UUID};
use super::{shader::LgShader, texture::LgTexture};

#[derive(Clone)]
pub struct LgMaterial {
    uuid: UUID,
    name: String, // TODO: Placeholder
    /* shaders: Vec<Rfc<LgShader>>,
    texture: Option<Rfc<LgTexture>>, */
    shaders: Vec<String>, // TODO: Replace with UUID
    texture: Option<String>, // TODO: Replace with UUID
    pub uniforms: Vec<LgUniform>,
}
impl LgMaterial {
    pub fn new(name: &str, shaders: Vec<String>, texture: Option<String>, uniforms: Vec<LgUniform>) -> Self {
        Self {
            uuid: UUID::generate(),
            name: String::from(name),
            shaders,
            texture,
            uniforms,
        }
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    /* pub fn shaders(&self) -> &[Rfc<LgShader>] {
        &self.shaders
    }
    pub fn texture(&self) -> &Option<Rfc<LgTexture>> {
        &self.texture
    } */
    pub fn texture(&self) -> &Option<String> {
        &self.texture
    }
    pub fn shaders(&self) -> &[String] {
        &self.shaders
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
impl Hash for LgMaterial {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}