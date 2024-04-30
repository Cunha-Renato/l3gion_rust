pub(crate) mod utils;

use std::io::Read;
use crate::{lg_core::uuid::UUID, StdError};

#[derive(Debug, Clone, Copy)]
pub enum ShaderStage {
    VERTEX,
    FRAGMENT,
    COMPUTE,
}
impl ShaderStage {
    pub fn to_shaderc_stage(&self) -> Result<shaderc::ShaderKind, StdError> {
        match self {
            ShaderStage::VERTEX => Ok(shaderc::ShaderKind::Vertex),
            ShaderStage::FRAGMENT => Ok(shaderc::ShaderKind::Fragment),
            ShaderStage::COMPUTE => Err("Invalid ShaderStage! (Shader)".into()),
        }
    }
    pub fn to_opengl_stage(&self) -> Result<u32, StdError> {
        match self {
            ShaderStage::VERTEX => Ok(gl::VERTEX_SHADER),
            ShaderStage::FRAGMENT => Ok(gl::FRAGMENT_SHADER),
            ShaderStage::COMPUTE => Err("Invalid ShaderStage! (Shader)".into()),
        }
    }
}

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
    pub fn builder() -> LgShaderBuilder {
        LgShaderBuilder::new()
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn stage(&self) -> ShaderStage {
        self.stage
    }
    pub fn src_code(&self) -> &str {
        &self.src_code
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
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
