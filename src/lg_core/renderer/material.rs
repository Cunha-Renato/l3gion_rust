use std::hash::Hash;

use crate::{lg_core::uuid::UUID, StdError};

use super::{opengl::{gl_program::GlProgram, gl_shader::GlShader}, shader::Shader, uniform::Uniform};

#[derive(Debug)]
pub struct Material {
    uuid: UUID,
    name: String,
    shaders: Vec<UUID>,
    textures: Vec<UUID>,
    pub uniforms: Vec<Uniform>,
    
    pub(crate) gl_program: Option<GlProgram>,
}
impl Material {
    pub fn new(uuid: UUID, name: &str, shaders: Vec<UUID>, textures: Vec<UUID>, uniforms: Vec<Uniform>) -> Self {
        Self {
            uuid,
            name: String::from(name),
            shaders,
            textures,
            uniforms,
            
            gl_program: None
        }
    }

    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }

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

// Public(crate)
impl Material {
    pub(crate) fn init_opengl(&mut self, shaders: &[&Shader]) -> Result<(), StdError> {
        if self.gl_program.is_some() { return Ok(()); }

        let gl_shaders = shaders.iter()
            .map(|s| GlShader::new(s.src_code(), s.stage().to_gl_stage()).unwrap())
            .collect::<Vec<_>>();

        let mut gl_program = GlProgram::new()?;
        gl_program.set_shaders(gl_shaders)?;

        self.gl_program = Some(gl_program);

        Ok(())
    }
}

impl Hash for Material {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}