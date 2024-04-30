use crate::lg_core::uuid::UUID;

use super::{opengl::opengl_vertex::GlVertex, vertex::LgVertex};

#[derive(Debug, Clone)]
pub struct LgMesh<T: LgVertex + GlVertex> {
    uuid: UUID,
    pub vertices: Vec<T>,
    pub indices: Vec<u16>,
}
impl<T: LgVertex + GlVertex> LgMesh<T> {
    pub fn new(
        vertices: Vec<T>,
        indices: Vec<u16>,
    ) -> Self
    {
        Self {
            uuid: UUID::generate(),
            vertices,
            indices,
        }
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
}