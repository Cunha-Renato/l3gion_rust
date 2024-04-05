use std::mem::size_of;

use crate::lg_core::{lg_types::reference::Rfc, uuid::UUID};
use super::{texture::Texture, vertex::Vertex};

#[derive(Default, Clone)]
pub struct Object<V> {
    uuid: UUID,
    texture: Rfc<Texture>,
    vertices: Vec<V>,
    indices: Vec<u32>,
}
impl<V> Object<V> {
    pub fn new(
        texture: Rfc<Texture>,
        vertices: Vec<V>,
        indices: Vec<u32>,
    ) -> Self 
    {
        Self {
            uuid: UUID::generate(),
            texture,
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
    pub fn texture(&self) -> Rfc<Texture> {
        self.texture.clone()
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