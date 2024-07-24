use super::GlError;

#[cfg(debug_assertions)]
pub(crate) fn check_gl_error(stmt: &str, fname: &str, line: u32, description: &str) -> Result<(), GlError>{
    let err = unsafe { gl::GetError() };
    if err != gl::NO_ERROR {
        let gl_error = std::format!("OpenGL error {:08x}, at {}:{} - for {}", err, fname, line, stmt);
        Err(GlError::Error(gl_error, description.to_string()))
    } else {
        Ok(())
    }
}
#[cfg(not(debug_assertions))]
pub(crate) fn check_gl_error(stmt: &str, fname: &str, line: u32, description: &str) -> Result<(), GlError>{
    Ok(())
}

#[macro_export]
macro_rules! gl_check {
    ($stmt:expr, $desc:tt) => {{
        unsafe {$stmt;}
        crate::lg_core::renderer::opengl::macros::check_gl_error(stringify!($stmt), file!(), line!(), $desc)
    }};
}