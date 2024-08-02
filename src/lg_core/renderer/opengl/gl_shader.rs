use std::ffi::CString;
use crate::{gl_check, StdError};

#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct GlShader {
    id: gl::types::GLuint,
}
impl GlShader {
    pub(crate) fn new(src: &str, stage: gl::types::GLenum) -> Result<Self, StdError> {
        let id ;
        gl_check!(id = gl::CreateShader(stage), "Failed to create shader!")?;

        let src_code_c_str = CString::new(src)?;
        gl_check!(
            gl::ShaderSource(
                id, 
                1,
                &src_code_c_str.as_ptr(),
                std::ptr::null()
            ),
            "Failed to set shader source!"
        )?;
        gl_check!(gl::CompileShader(id), "Failed to compile shader!")?;

        Ok(Self { id }) 
    }
    pub(crate) fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub(crate) fn delete(&self) {
        if self.id != 0 {
            gl_check!(gl::DeleteShader(self.id), "Failed to delete shader!").unwrap();
        }
    }
}