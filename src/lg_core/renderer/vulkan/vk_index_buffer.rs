use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};
use crate::{lg_core::lg_types::reference::Rfc, MyError};
use super::{vk_buffer, vk_device::VkDevice, vk_memory_allocator::{VkMemoryManager, VkMemoryRegion, VkMemoryUsageFlags}};

#[derive(Clone)]
pub struct VkIndexBuffer {
    pub buffer: vk::Buffer,
    pub region: Rfc<VkMemoryRegion>,
}
impl VkIndexBuffer {
    pub unsafe fn new(
        device: &VkDevice,
        memory_manager: &mut VkMemoryManager,
        indices: &[u32],
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
        memcpy(indices.as_ptr(), memory.cast(), indices.len());
        memory_manager.unmap_buffer(staging_buffer_region.clone())?;

        // Create (vertex)

        let (index_buffer, index_buffer_region) = vk_buffer::create_buffer(
            device, 
            memory_manager,
            size, 
            vk::BufferUsageFlags::TRANSFER_DST 
                | vk::BufferUsageFlags::INDEX_BUFFER, 
            VkMemoryUsageFlags::GPU,
        )?;

        // Copy (vertex)

        vk_buffer::copy_buffer(
            device, 
            staging_buffer, 
            index_buffer, 
            size
        )?;

        // Cleanup

        device.get_device().destroy_buffer(staging_buffer, None);
        memory_manager.free_buffer_region(staging_buffer_region)?;

        Ok(Self {
            buffer: index_buffer,
            region: index_buffer_region,
        })
    }
}