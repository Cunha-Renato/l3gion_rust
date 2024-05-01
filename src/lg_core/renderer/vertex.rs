use nalgebra_glm as glm;
use crate::{
    gl_vertex, 
    StdError
};
use crate::lg_core::renderer::opengl::opengl_vertex::GlVertex;
use super::opengl::{
    opengl_program::GlProgram, 
    opengl_vertex_array::GlVertexArray
};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: glm::Vec2,
    pub tex_coord: glm::Vec2,
}
gl_vertex!(Vertex, position, tex_coord);
pub trait LgVertex {}
impl LgVertex for Vertex {}

pub struct BachVertex {
    pub position: glm::Vec2,
    pub tex_coord: glm::Vec2,
    pub id: glm::UVec4, // u128 UUID
}