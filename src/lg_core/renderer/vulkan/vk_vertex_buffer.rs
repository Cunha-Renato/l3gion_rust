use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia::vk;

use crate::{lg_core::lg_types::reference::Rfc, MyError};

use super::{vk_buffer::{self, VkBuffer}, vk_device::VkDevice, vk_memory_manager::{VkMemoryManager, VkMemoryUsageFlags}};

#[derive(Clone)]
pub struct VkVertexBuffer {
    pub buffer: Rfc<VkBuffer>,
}
impl VkVertexBuffer {
    pub unsafe fn new<T>(
        device: &VkDevice,
        memory_manager: &mut VkMemoryManager,
        vertices: &[T],
        size: u64,
    ) -> Result<Self, MyError> 
    {
        let staging_buffer = memory_manager.new_buffer(
            size, 
            vk::BufferUsageFlags::TRANSFER_SRC, 
            VkMemoryUsageFlags::CPU_GPU,
        )?;

        // Copy (staging)
        let memory = memory_manager.map_buffer(staging_buffer.clone(), 0, size, vk::MemoryMapFlags::empty())?;
        memcpy(vertices.as_ptr(), memory.cast(), vertices.len());
        memory_manager.unmap_buffer(staging_buffer.clone())?;

        // Create (vertex)
        let vertex_buffer = memory_manager.new_buffer(
            size, 
            vk::BufferUsageFlags::TRANSFER_DST 
                | vk::BufferUsageFlags::VERTEX_BUFFER, 
            VkMemoryUsageFlags::GPU
        )?;

        // Copy (vertex)
        vk_buffer::copy_buffer(
            device, 
            staging_buffer.borrow().buffer, 
            vertex_buffer.borrow().buffer, 
            size
        )?;

        // Cleanup

        memory_manager.destroy_buffer(staging_buffer)?;

        Ok(Self {
            buffer: vertex_buffer,
        })
    }
}