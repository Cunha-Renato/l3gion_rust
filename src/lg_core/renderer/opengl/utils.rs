use sllog::error;

#[macro_export]
macro_rules! gl_vertex {
    ($struct_name:ident, $($fields:tt), *) => {
        impl GlVertex for $struct_name {
            unsafe fn set_attrib_locations(vao: &mut GlVertexArray, program: &mut GlProgram) -> Result<(), StdError> {
                const fn size_of_raw<T>(_: *const T) -> usize {
                    core::mem::size_of::<T>()
                }
                $(
                    let attrib = program.get_attrib_location(stringify!($fields))?;

                    let dummy = core::mem::MaybeUninit::<$struct_name>::uninit();
                    let dummy_ptr = dummy.as_ptr();
                    let member_ptr = core::ptr::addr_of!((*dummy_ptr).$fields);
                    let member_offset = member_ptr as i32 - dummy_ptr as i32;

                    vao.set_attribute::<$struct_name>(
                        attrib,
                        (size_of_raw(member_ptr) / core::mem::size_of::<f32>()) as i32,
                        member_offset,
                    );
                )*
                
                Ok(())
            }
        }
    };
}

pub(crate) fn check_gl_error(stmt: &str, fname: &str, line: u32) {
    let err = unsafe { gl::GetError() };
    if err != gl::NO_ERROR {
        error!("OpenGL error {:08x}, at {}:{} - for {}", err, fname, line, stmt);
        std::process::abort();
    }
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! gl_check {
    ($stmt:expr) => {{
        $stmt;
        crate::lg_core::renderer::opengl::utils::check_gl_error(stringify!($stmt), file!(), line!());
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! gl_check {
    ($stmt:expr) => {{
        $stmt;
    }};
}

pub(crate) extern "system" fn debug_callback(
    source: gl::types::GLenum,
    gltype: gl::types::GLenum,
    id: gl::types::GLuint,
    severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _user_param: *mut std::ffi::c_void,
) {
    let source_str = match source {
        gl::DEBUG_SOURCE_API => "API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
        gl::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
        gl::DEBUG_SOURCE_APPLICATION => "Application",
        _ => "Unknown",
    };

    let severity_str = match severity {
        gl::DEBUG_SEVERITY_HIGH => "High",
        gl::DEBUG_SEVERITY_MEDIUM => "Medium",
        gl::DEBUG_SEVERITY_LOW => "Low",
        gl::DEBUG_SEVERITY_NOTIFICATION => "Notification",
        _ => "Unknown",
    };

    let gltype_str = match gltype {
        gl::DEBUG_TYPE_ERROR => "Error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior",
        gl::DEBUG_TYPE_PORTABILITY => "Portability",
        gl::DEBUG_TYPE_PERFORMANCE => "Performance",
        gl::DEBUG_TYPE_OTHER => "Other",
        gl::DEBUG_TYPE_MARKER => "Marker",
        gl::DEBUG_TYPE_PUSH_GROUP => "Push Group",
        gl::DEBUG_TYPE_POP_GROUP => "Pop Group",
        _ => "Unknown",
    };

    let message_str = unsafe { 
        std::str::from_utf8(std::ffi::CStr::from_ptr(message).to_bytes()).unwrap() 
    };
    error!(
        "OpenGL Debug Message:\n  Source: {}\n  Type: {}\n  ID: {}\n  Severity: {}\n  Message: {}",
        source_str, gltype_str, id, severity_str, message_str
    );
}