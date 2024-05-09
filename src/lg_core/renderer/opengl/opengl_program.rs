extern crate alloc;

use std::ffi::CString;

use crate::{gl_check, lg_core::lg_types::reference::Rfc, StdError};
use super::opengl_shader::GlShader;

#[derive(Debug, Default)]
pub struct GlProgram {
    id: gl::types::GLuint,
    pub shaders: Vec<Rfc<GlShader>>,
}
impl GlProgram {
    pub(crate)unsafe fn new() -> Self {
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
    pub(crate) fn contains(&self, shaders: &[Rfc<GlShader>]) -> bool {
        shaders.iter().all(|s| self.shaders.contains(s))
    }
    pub(crate) unsafe fn set_shaders(&mut self, shaders: Vec<Rfc<GlShader>>) {
        shaders
            .iter()
            .for_each(|s| {
                gl_check!(gl::AttachShader(self.id, s.borrow().id()));
            });
        
        self.shaders = shaders;
    }
    pub(crate) unsafe fn add_shader(&mut self, shader: Rfc<GlShader>) {
        gl_check!(gl::AttachShader(self.id, shader.borrow().id()));
        
        self.shaders.push(shader);
    }
    pub(crate) unsafe fn use_prog(&self) {
        gl_check!(gl::UseProgram(self.id));
    }
    pub(crate) unsafe fn unuse(&self) {
        gl_check!(gl::UseProgram(0));
    }
    pub(crate) unsafe fn get_attrib_location(&self, attrib: &str) -> Result<gl::types::GLuint, StdError>
    {
        let attrib = CString::new(attrib)?;
        let location: u32;
        gl_check!(location = gl::GetAttribLocation(self.id, attrib.as_ptr()) as gl::types::GLuint);
            
        Ok(location)
    }
    pub(crate) unsafe fn link(&self) -> Result<(), StdError>{
        gl_check!(gl::LinkProgram(self.id));
        
        let mut success = 0;
        gl_check!(gl::GetProgramiv(self.id, gl::LINK_STATUS, &mut success));

        if success != 1 {
            let mut error_log_size = 0;
            gl_check!(gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut error_log_size));
            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl_check!(gl::GetProgramInfoLog(
                self.id,
                error_log_size,
                &mut error_log_size,
                error_log.as_mut_ptr() as *mut _,
            ));

            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log)?;
            Err(log.into())
        } else {
            Ok(())
        }
    }
}
impl Drop for GlProgram {
    fn drop(&mut self) {
        unsafe { gl_check!(gl::DeleteProgram(self.id)) };
    }
}