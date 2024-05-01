use crate::{lg_core::renderer::{mesh::LgMesh, vertex::Vertex}, StdError};

use super::{opengl_program::GlProgram, opengl_vertex::GlVertex, opengl_vertex_array::GlVertexArray};

pub(crate) trait GlMesh {
    fn set_attrib_locations(&self, vao: &mut GlVertexArray, program: &mut GlProgram) -> Result<(), StdError>;
}

impl GlMesh for LgMesh {
    fn set_attrib_locations(&self, vao: &mut GlVertexArray, program: &mut GlProgram) -> Result<(), StdError> {
        unsafe { Vertex::set_attrib_locations(vao, program) }
    }
}