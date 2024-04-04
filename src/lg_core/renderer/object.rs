use std::mem::size_of;

use crate::lg_core::uuid::UUID;
use super::vertex::Vertex;

#[derive(Default, Clone)]
pub struct Object<V> {
    uuid: UUID,
    vertices: Vec<V>,
    indices: Vec<u32>,
}
impl<V> Object<V> {
    pub fn new(
        vertices: Vec<V>,
        indices: Vec<u32>,
    ) -> Self 
    {
        Self {
            uuid: UUID::generate(),
            vertices,
            indices,
        }
    }
    pub fn vertices(&self) -> &[V] {
        &self.vertices
    }
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
    pub fn vertex_size(&self) -> u64 {
        (size_of::<Vertex>() * self.vertices.len()) as u64
    }
    pub fn index_size(&self) -> u64 {
        (size_of::<u32>() * self.indices.len()) as u64
    }
    pub fn uuid(&self) -> UUID {
        self.uuid.clone()
    }
}