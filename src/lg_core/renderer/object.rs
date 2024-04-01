use std::mem::size_of;

use crate::{lg_core::uuid::UUID, MyError};
use super::{vertex::Vertex, vulkan::{index_buffer::VkIndexBuffer, vertex_buffer::VkVertexBuffer, vk_device::VkDevice, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice}};

// TODO: Maybe, just maybe could you please make the Vertex and UniformBuffer structs a fucking trait, so I dont have to create a milion structs for differend rendering styles. Thank you
#[derive(Default, Clone)]
pub struct Object<V> {
    uuid: UUID,
    vertices: Vec<V>,
    indices: Vec<u32>,
    v_buffer: Option<VkVertexBuffer>,
    i_buffer: Option<VkIndexBuffer>,
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
            v_buffer: None,
            i_buffer: None,
        }
    }
    pub fn create_vertex_buffer(
        &mut self,
        instance: &VkInstance,
        device: &VkDevice,
        physical_device: &VkPhysicalDevice,
    ) -> Result<(), MyError> 
    {
        unsafe { 
            self.v_buffer = Some(VkVertexBuffer::new(
            instance, 
            device, 
            physical_device, 
            self.vertices(), 
            self.vertex_size(),
        )?);}

        Ok(()) 
    }
    pub fn create_index_buffer(
        &mut self,
        instance: &VkInstance,
        device: &VkDevice,
        physical_device: &VkPhysicalDevice,
    ) -> Result<(), MyError>
    {
        unsafe { 
            self.i_buffer = Some(VkIndexBuffer::new(
            instance, 
            device, 
            physical_device, 
            self.indices(), 
            self.index_size(),
        )?);}

        Ok(())
    }

    pub fn vertices(&self) -> &[V] {
        &self.vertices
    }
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
    pub fn vertex_buffer(&self) -> Result<&VkVertexBuffer, MyError> {
        match &self.v_buffer {
            Some(buffer) => Ok(&buffer),
            None => Err("No vertex buffer in object!".into())
        }
    }
    pub fn index_buffer(&self) -> Result<&VkIndexBuffer, MyError> {
        match &self.i_buffer {
            Some(buffer) => Ok(&buffer),
            None => Err("No vertex buffer in object!".into())
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
}