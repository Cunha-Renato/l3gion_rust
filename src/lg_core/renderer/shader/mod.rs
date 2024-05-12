pub(crate) mod utils;

use std::{hash::Hash, io::Read};
use lg_renderer::renderer::lg_shader::ShaderStage;
use crate::{lg_core::uuid::UUID, StdError};

/// src_code can be empty if you are using SPIR-V, bytes can be empty if you are using raw glsl.
///
/// Both cannot be empty, be sure to set the one that you are using.
///
/// stage is set to ShaderStage::VERTEX by default.
///
/// Use LgShaderBuilder
#[derive(Debug, Clone)]
pub struct LgShader {
    uuid: UUID,
    bytes: Vec<u8>,
    stage: ShaderStage,
    src_code: String,
}
impl LgShader {
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    pub fn builder() -> LgShaderBuilder {
        LgShaderBuilder::new()
    }
}
impl lg_renderer::renderer::lg_shader::Shader for LgShader {
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
impl Hash for LgShader {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

pub struct LgShaderBuilder {
    shader: LgShader
}
impl LgShaderBuilder {
    pub fn new() -> Self {
        Self {
            shader: LgShader {
                uuid: UUID::generate(),
                bytes: Vec::new(),
                stage: ShaderStage::VERTEX,
                src_code: String::default()
            }
        }
    }
    pub fn stage(mut self, stage: ShaderStage) -> Self {
        self.shader.stage = stage;
        
        self
    }
    pub fn from_spirv(mut self, path: &std::path::Path) -> Result<Self, StdError> {
        std::fs::File::open(path)?.read_to_end(&mut self.shader.bytes)?;
        
        Ok(self)
    }
    pub fn src_code(mut self, path: &std::path::Path) -> Result<Self, StdError> {
        self.shader.src_code = crate::utils::tools::file_to_string(path.to_str().unwrap())?;

        Ok(self)
    }
    pub fn build(self) -> LgShader {
        self.shader
    }
}
