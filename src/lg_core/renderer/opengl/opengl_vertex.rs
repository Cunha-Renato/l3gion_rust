use crate::StdError;

use super::{opengl_program::GlProgram, opengl_vertex_array::GlVertexArray};

pub trait GlVertex {
    unsafe fn set_attrib_locations(vao: &mut GlVertexArray, program: &mut GlProgram) -> Result<(), StdError>
    where Self: Sized;
}