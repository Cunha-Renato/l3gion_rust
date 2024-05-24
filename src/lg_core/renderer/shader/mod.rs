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
pub struct Shader {
    uuid: UUID,
    name: String, // TODO: Placeholder
    bytes: Vec<u8>,
    stage: ShaderStage,
    src_code: String,
}
impl Shader {
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
    pub fn builder(name: &str) -> LgShaderBuilder {
        LgShaderBuilder::new(name)
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
impl lg_renderer::renderer::lg_shader::LgShader for Shader {
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

pub struct LgShaderBuilder {
    shader: Shader
}
impl LgShaderBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            shader: Shader {
                uuid: UUID::generate(),
                name: String::from(name),
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
    pub fn build(self) -> Shader {
        self.shader
    }
}
