use std::mem::size_of;

use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};
use crate::{lg_core::uuid::UUID, MyError};
use super::{vertex::Vertex, vulkan::{command_buffer::VkCommandPool, index_buffer::VkIndexBuffer, vertex_buffer::VkVertexBuffer}};

// TODO: Maybe, just maybe could you please make the Vertex and UniformBuffer structs a fucking trait, so I dont have to create a milion structs for differend rendering styles. Thank you
#[derive(Default)]
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
        instance: &Instance,
        device: &Device,
        physical_device: &vk::PhysicalDevice,
        command_pool: &VkCommandPool,
        queue: &vk::Queue,
    ) -> Result<(), MyError> 
    {
        unsafe { 
            self.v_buffer = Some(VkVertexBuffer::new(
            instance, 
            device, 
            physical_device, 
            command_pool, 
            queue, 
            self.vertices(), 
            self.vertex_size(),
        )?);}

        Ok(()) 
    }
    pub fn create_index_buffer(
        &mut self,
        instance: &Instance,
        device: &Device,
        physical_device: &vk::PhysicalDevice,
        command_pool: &VkCommandPool,
        queue: &vk::Queue
    ) -> Result<(), MyError>
    {
        unsafe { 
            self.i_buffer = Some(VkIndexBuffer::new(
            instance, 
            device, 
            physical_device, 
            command_pool, 
            queue, 
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
        match self.v_buffer {
            Some(buffer) => Ok(&buffer),
            None => Err("No vertex buffer in object!".into())
        }
    }
    pub fn index_buffer(&self) -> Result<&VkIndexBuffer, MyError> {
        match self.i_buffer {
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
        self.uuid
    }
}