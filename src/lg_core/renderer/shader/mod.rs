pub(crate) mod utils;

use std::hash::Hash;
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
    pub(crate) fn to_gl_stage(&self) -> gl::types::GLenum {
        match self {
            ShaderStage::VERTEX => gl::VERTEX_SHADER,
            ShaderStage::FRAGMENT => gl::FRAGMENT_SHADER,
            ShaderStage::COMPUTE => gl::COMPUTE_SHADER,
        }
    }
    pub fn from_str(val: &str) -> Result<Self, StdError> {
        Ok(match val {
            "vert" => Self::VERTEX,
            "frag" => Self::FRAGMENT,
            "comp" => Self::COMPUTE,

            _ => return Err(std::format!("{} is an invalid shader stage!", val).into()),
        })
    }
    pub fn from_u32(val: u32) -> Result<Self, StdError> {
        Ok(match val {
            0 => Self::VERTEX,
            1 => Self::FRAGMENT,
            2 => Self::COMPUTE,
            _ => return Err("Failed to convert u32 into ShaderStage!".into())
        })
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

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn stage(&self) -> ShaderStage {
        self.stage
    }

    pub fn src_code(&self) -> &str {
        &self.src_code
    }
}
impl Hash for Shader {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}