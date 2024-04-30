use crate::lg_core::{lg_types::reference::Rfc, uuid::UUID};

use super::shader::LgShader;

#[derive(Debug, Clone)]
pub struct LgMaterial {
    uuid: UUID,
    shaders: Vec<Rfc<LgShader>>
}
impl LgMaterial {
    pub fn new(shaders: Vec<Rfc<LgShader>>) -> Self {
        Self {
            uuid: UUID::generate(),
            shaders
        }
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    pub fn shaders(&self) -> &[Rfc<LgShader>] {
        &self.shaders
    }
}