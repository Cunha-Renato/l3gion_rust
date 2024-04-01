use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};
use crate::MyError;
use super::{buffer, vk_device::VkDevice, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice};

#[derive(Clone)]
pub struct VkIndexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}
impl VkIndexBuffer {
    pub unsafe fn new(
        instance: &VkInstance,
        device: &VkDevice,
        physical_device: &VkPhysicalDevice,
        indices: &[u32],
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

        let memory = device.get_device().map_memory(
            staging_buffer_memory, 
            0, 
            size, 
            vk::MemoryMapFlags::empty()
        )?;

        memcpy(indices.as_ptr(), memory.cast(), indices.len());

        device.get_device().unmap_memory(staging_buffer_memory);

        // Create (vertex)

        let (index_buffer, index_buffer_memory) = buffer::create_buffer(
            instance, 
            device, 
            physical_device, 
            size, 
            vk::BufferUsageFlags::TRANSFER_DST 
                | vk::BufferUsageFlags::INDEX_BUFFER, 
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // Copy (vertex)

        buffer::copy_buffer(
            device, 
            staging_buffer, 
            index_buffer, 
            size
        )?;

        // Cleanup

        device.get_device().destroy_buffer(staging_buffer, None);
        device.get_device().free_memory(staging_buffer_memory, None);

        Ok(Self {
            buffer: index_buffer,
            memory: index_buffer_memory,
        })
    }
}