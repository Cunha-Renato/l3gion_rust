use std::ffi::CString;

use crate::{
    gl_check, lg_core::{
        lg_types::reference::Rfc, 
        renderer::shader::LgShader
    }, StdError
};

#[derive(Debug)]
pub(crate) struct GlShader {
    id: gl::types::GLuint,
    pub shader: Rfc<LgShader>,
}
impl GlShader {
    pub(crate) unsafe fn new(shader: Rfc<LgShader>) -> Result<Self, StdError> {
        let id ;
        gl_check!(id = gl::CreateShader(shader.borrow().stage().to_opengl_stage()?));

        let src_code_c_str = CString::new(shader.borrow().src_code())?;
        gl_check!(gl::ShaderSource(
            id, 
            1,
            &src_code_c_str.as_ptr(),
            std::ptr::null()
        ));
        gl_check!(gl::CompileShader(id));

        let mut success = 0;
        gl_check!(gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success));

        if success == 1 {
            Ok(Self {
                id,
                shader
            })
        } else {
            let mut error_log_size = 0;
            gl_check!(gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut error_log_size));
            let mut error_log: Vec<u8> = Vec::with_capacity(error_log_size as usize);
            gl_check!(gl::GetShaderInfoLog(
                id,
                error_log_size,
                &mut error_log_size,
                error_log.as_mut_ptr() as *mut _,
            ));

            error_log.set_len(error_log_size as usize);
            let log = String::from_utf8(error_log)?;
            Err(log.into())
        }
    }
    pub(crate) fn id(&self) -> gl::types::GLuint {
        self.id
    }
}
impl PartialEq for GlShader {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Drop for GlShader {
    fn drop(&mut self) {
        unsafe {
            gl_check!(gl::DeleteShader(self.id));
        }
    }
}