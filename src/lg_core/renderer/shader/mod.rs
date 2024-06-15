pub(crate) mod utils;

use std::hash::Hash;
use lg_renderer::renderer_core::lg_shader::ShaderStage;
use crate::lg_core::uuid::UUID;

/// src_code can be empty if you are using SPIR-V, bytes can be empty if you are using raw glsl.
///
/// Both cannot be empty, be sure to set the one that you are using.
///
/// stage is set to ShaderStage::VERTEX by default.
///
/// Use LgShaderBuilder
#[derive(Debug, Clone)]
pub struct Shader {
    uuid: UUID,
    name: String, // TODO: Placeholder
    bytes: Vec<u8>,
    stage: ShaderStage,
    src_code: String,
}
impl Shader {
    pub fn new(
        uuid: UUID,
        name: String,
        bytes: Vec<u8>,
        stage: ShaderStage,
        src_code: String,
    ) -> Self 
    {
        Self {
            uuid,
            name,
            bytes,
            stage,
            src_code,
        }
    }
}
impl Shader {
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
impl lg_renderer::renderer_core::lg_shader::LgShader for Shader {
    fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn stage(&self) -> ShaderStage {
        self.stage
    }

    fn src_code(&self) -> &str {
        &self.src_code
    }
}
impl Hash for Shader {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}