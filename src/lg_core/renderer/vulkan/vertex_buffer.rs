use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};

use crate::{lg_core::renderer::vertex::Vertex, MyError};

use super::{buffer, command_buffer::VkCommandPool};

pub struct VkVertexBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}
impl VkVertexBuffer {
    pub unsafe fn new(
        instance: &Instance,
        device: &Device,
        physical_device: &vk::PhysicalDevice,
        command_pool: &VkCommandPool,
        queue: &vk::Queue,
        vertices: &[Vertex],
        size: u64,
    ) -> Result<Self, MyError> 
    {
        let (staging_buffer, staging_buffer_memory) = buffer::create_buffer(
            instance, 
            device, 
            physical_device, 
            size, 
            vk::BufferUsageFlags::TRANSFER_SRC, 
            vk::MemoryPropertyFlags::HOST_COHERENT
                | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        // Copy (staging)

        let memory = device.map_memory(
            staging_buffer_memory, 
            0, 
            size, 
            vk::MemoryMapFlags::empty()
        )?;

        memcpy(vertices.as_ptr(), memory.cast(), vertices.len());

        device.unmap_memory(staging_buffer_memory);

        // Create (vertex)

        let (vertex_buffer, vertex_buffer_memory) = buffer::create_buffer(
            instance, 
            device, 
            physical_device, 
            size, 
            vk::BufferUsageFlags::TRANSFER_DST 
                | vk::BufferUsageFlags::VERTEX_BUFFER, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // Copy (vertex)

        buffer::copy_buffer(
            device, 
            command_pool, 
            queue, 
            staging_buffer, 
            vertex_buffer, 
            size
        )?;

        // Cleanup

        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        Ok(Self {
            buffer: vertex_buffer,
            memory: vertex_buffer_memory,
        })
    }
}