extern crate lg_renderer;
use lg_renderer::lg_vertex;

use nalgebra_glm as glm;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub tex_coord: glm::Vec2,
}
lg_vertex!(Vertex, position, tex_coord);
pub trait LgVertex {}
impl LgVertex for Vertex {}

pub struct BachVertex {
    pub position: glm::Vec3,
    pub tex_coord: glm::Vec2,
    pub id: glm::UVec4, // u128 UUID
}