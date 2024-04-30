use crate::lg_core::{lg_types::reference::Rfc, uuid::UUID};

use super::{shader::LgShader, texture::LgTexture};

#[derive(Debug, Clone)]
pub struct LgMaterial {
    uuid: UUID,
    shaders: Vec<Rfc<LgShader>>,
    texture: Rfc<LgTexture>,
}
impl LgMaterial {
    pub fn new(shaders: Vec<Rfc<LgShader>>, texture: Rfc<LgTexture>) -> Self {
        Self {
            uuid: UUID::generate(),
            shaders,
            texture,
        }
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    pub fn shaders(&self) -> &[Rfc<LgShader>] {
        &self.shaders
    }
    pub fn texture(&self) -> &Rfc<LgTexture> {
        &self.texture
    }
}