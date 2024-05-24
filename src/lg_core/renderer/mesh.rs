use std::hash::Hash;
use crate::lg_core::uuid::UUID;
use super::vertex::Vertex;

#[derive(Debug, Clone)]
pub struct Mesh {
    uuid: UUID,
    name: String, // TODO: Placeholder
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}
impl Mesh {
    pub fn new(
        name: &str,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) -> Self
    {
        Self {
            uuid: UUID::generate(),
            name: String::from(name),
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
    pub fn name(&self) -> &str {
        &self.name
    }
}
impl Hash for Mesh {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}