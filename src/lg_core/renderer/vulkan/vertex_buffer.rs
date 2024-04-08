use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};

use crate::{lg_core::lg_types::reference::Rfc, MyError};

use super::{vk_buffer, vk_device::VkDevice, vk_memory_allocator::{VkMemoryManager, VkMemoryRegion, VkMemoryUsageFlags}};

#[derive(Clone)]
pub struct VkVertexBuffer {
    pub buffer: vk::Buffer,
    pub region: Rfc<VkMemoryRegion>,
}
impl VkVertexBuffer {
    pub unsafe fn new<T>(
        device: &VkDevice,
        memory_manager: &mut VkMemoryManager,
        vertices: &[T],
        size: u64,
    ) -> Result<Self, MyError> 
    {
        let (staging_buffer, staging_buffer_region) = vk_buffer::create_buffer(
            device, 
            memory_manager,
            size, 
            vk::BufferUsageFlags::TRANSFER_SRC, 
            VkMemoryUsageFlags::CPU_GPU,
        )?;

        // Copy (staging)
        let memory = memory_manager.map_buffer(staging_buffer_region.clone(), 0, size, vk::MemoryMapFlags::empty())?;
        memcpy(vertices.as_ptr(), memory.cast(), vertices.len());
        memory_manager.unmap_buffer(staging_buffer_region.clone())?;

        // Create (vertex)
        let (vertex_buffer, vertex_buffer_region) = vk_buffer::create_buffer(
            device, 
            memory_manager,
            size, 
            vk::BufferUsageFlags::TRANSFER_DST 
                | vk::BufferUsageFlags::VERTEX_BUFFER, 
            VkMemoryUsageFlags::GPU
        )?;

        // Copy (vertex)
        vk_buffer::copy_buffer(
            device, 
            staging_buffer, 
            vertex_buffer, 
            size
        )?;

        // Cleanup

        device.get_device().destroy_buffer(staging_buffer, None);
        memory_manager.free_buffer_region(staging_buffer_region)?;

        Ok(Self {
            buffer: vertex_buffer,
            region: vertex_buffer_region,
        })
    }
}