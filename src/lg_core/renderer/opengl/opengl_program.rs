extern crate alloc;
use alloc::ffi::CString;

use crate::{gl_check, lg_core::lg_types::reference::Rfc, StdError};

use super::{opengl_buffer::GlBuffer, opengl_shader::GlShader};

#[derive(Debug)]
pub struct GlProgram {
    id: gl::types::GLuint,
    shaders: Vec<Rfc<GlShader>>,
}
impl GlProgram {
    unsafe fn new() -> Self {
        let id: u32;
        gl_check!(id = gl::CreateProgram());

        Self {
            id,
            shaders: Vec::new()
        }
    }
    pub(crate) fn id(&self) -> gl::types::GLuint {
        self.id
    }
    pub(crate) unsafe fn builder() -> GlProgramBuilder {
        GlProgramBuilder::new()        
    }
    pub(crate) unsafe fn bind(&self) {
        gl_check!(gl::UseProgram(self.id));
    }
    pub(crate) unsafe fn unbind(&self) {
        gl_check!(gl::UseProgram(0));
    }
    pub(crate) unsafe fn get_attrib_location(&self, attrib: &str) -> Result<gl::types::GLuint, StdError>
    {
        let attrib = CString::new(attrib)?;
        let location: u32;
        gl_check!(location = gl::GetAttribLocation(self.id, attrib.as_ptr()) as gl::types::GLuint);
            
        Ok(location)
    }
}
impl Drop for GlProgram {
    fn drop(&mut self) {
        unsafe { gl_check!(gl::DeleteProgram(self.id)) };
    }
}

pub(crate) struct GlProgramBuilder {
    program: GlProgram
}
impl GlProgramBuilder {
    pub(crate) unsafe fn new() -> Self {
        Self {
            program: GlProgram::new()
        }
    }
    pub(crate) unsafe fn add_shaders(mut self, shaders: Vec<Rfc<GlShader>>) -> Self {
        shaders.iter().for_each(|s| {
            gl_check!(gl::AttachShader(self.program.id, s.borrow().id()));
            self.program.shaders.push(s.clone());
        });
        
        self
    }
    pub(crate) unsafe fn add_shader(mut self, shader: Rfc<GlShader>) -> Self {
        gl_check!(gl::AttachShader(self.program.id, shader.borrow().id()));
        self.program.shaders.push(shader);
        
        self
    }
    pub(crate) unsafe fn build(self) -> Result<GlProgram, StdError> {
        let program = self.program;
        gl_check!(gl::LinkProgram(program.id));

        let mut success = 0;
        gl_check!(gl::GetProgramiv(program.id, gl::LINK_STATUS, &mut success));

        if success == 1 {
            Ok(program)
        } else {
            let mut error_log_size = 0;
            gl_check!(gl::GetProgramiv(program.id, gl::INFO_LOG_LENGTH, &mut error_log_size));
            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl_check!(gl::GetProgramInfoLog(
                program.id,
                error_log_size,
                &mut error_log_size,
                error_log.as_mut_ptr() as *mut _,
            ));

            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log)?;
            Err(log.into())
        }
    }
}