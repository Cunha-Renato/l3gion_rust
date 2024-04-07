use std::mem::size_of;

use nalgebra_glm as glm;
use crate::lg_core::{lg_types::reference::Rfc, uuid::UUID};
use super::{texture::Texture, vertex::Vertex};

#[derive(Default, Clone)]
pub struct Transformation {
    pub position: glm::Vec3,
    pub scale: glm::Vec3,
    pub angle: f32,
    pub rotation_axis: glm::Vec3,
}

#[derive(Default, Clone)]
pub struct Object<V> {
    uuid: UUID,
    pub transform: Transformation,
    pub texture: Rfc<Texture>,
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
}
impl<V: Clone> Object<V> {
    pub fn new(
        texture: Rfc<Texture>,
        transform: Transformation,
        vertices: Vec<V>,
        indices: Vec<u32>,
    ) -> Self 
    {
        Self {
            uuid: UUID::generate(),
            transform,
            texture,
            vertices,
            indices,
        }
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
    pub fn replicate(&self) -> Self {
        Self {
            uuid: UUID::generate(),
            texture: self.texture.clone(),
            transform: self.transform.clone(),
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
        }
    }
}