use std::mem::size_of;
use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};

use crate::MyError;

use super::{buffer, swapchain::VkSwapchain};
pub struct UniformBuffer {
    pub buffers: Vec<vk::Buffer>,
    memories: Vec<vk::DeviceMemory>,
    pub ubo_size: u64,
}
impl UniformBuffer {
    pub unsafe fn new<T>(
        instance: &Instance,
        device: &Device,
        physical_device: &vk::PhysicalDevice,
        swapchain: &VkSwapchain,
    ) -> Result<Self, MyError> 
    {
        let ubo_size = size_of::<T>() as u64;
        let (buffers, memories) = create_uniform_buffers(
            instance, 
            device, 
            physical_device, 
            swapchain, 
            ubo_size
        )?;
        
        Ok(Self {
            buffers,
            memories,
            ubo_size
        })
    }
}
unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    swapchain: &VkSwapchain,
    size: u64,
) -> Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>), MyError>
{
    let mut buffers = Vec::new();
    let mut memories = Vec::new();
    for _ in 0..swapchain.image_data.images.len() {
        let (uniform_buffer, uniform_buffer_memory) = buffer::create_buffer(
            instance, 
            device, 
            physical_device, 
            size, 
            vk::BufferUsageFlags::UNIFORM_BUFFER, 
            vk::MemoryPropertyFlags::HOST_COHERENT
                | vk::MemoryPropertyFlags::HOST_VISIBLE
        )?;

        buffers.push(uniform_buffer);
        memories.push(uniform_buffer_memory);
    }

    Ok((buffers, memories))
}