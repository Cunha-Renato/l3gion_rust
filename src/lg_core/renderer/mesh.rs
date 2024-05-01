use crate::lg_core::uuid::UUID;

use super::vertex::Vertex;

#[derive(Debug, Clone)]
pub struct LgMesh {
    uuid: UUID,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}
impl LgMesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) -> Self
    {
        Self {
            uuid: UUID::generate(),
            vertices,
            indices,
        }
    }
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
    pub fn uuid(&self) -> &UUID {
        &self.uuid
    }
}