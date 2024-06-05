use std::hash::Hash;

use crate::lg_core::uuid::UUID;
use super::uniform::Uniform;

#[derive(Debug, Clone)]
pub struct Material {
    uuid: UUID,
    name: String,
    shaders: Vec<UUID>,
    textures: Vec<UUID>,
    pub uniforms: Vec<Uniform>,
}
impl Material {
    pub fn new(uuid: UUID, name: &str, shaders: Vec<UUID>, textures: Vec<UUID>, uniforms: Vec<Uniform>) -> Self {
        Self {
            uuid,
            name: String::from(name),
            shaders,
            textures,
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
    pub fn texture(&self) -> &[UUID] {
        &self.textures
    }
    pub fn shaders(&self) -> &[UUID] {
        &self.shaders
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
impl Hash for Material {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}