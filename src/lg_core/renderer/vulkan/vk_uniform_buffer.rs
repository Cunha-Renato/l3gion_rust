use std::mem::size_of;

use vulkanalia::vk;
use crate::MyError;
use super::{buffer, vk_device::VkDevice, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice};

const MAX_SIZE: u64 = 1000;

pub struct VkUniformBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub range: u64,
    pub size: u64,
    pub offset: u64,
}
impl VkUniformBuffer {
    pub unsafe fn new<T>(
        instance: &VkInstance,
        device: &VkDevice,
        physical_device: &VkPhysicalDevice,
    ) -> Result<Self, MyError>
    {
        let range = size_of::<T>() as u64;
        let size = range * MAX_SIZE;
        let (buffer, memory) = buffer::create_buffer(
            instance, 
            device, 
            physical_device, 
            size, 
            vk::BufferUsageFlags::UNIFORM_BUFFER, 
            vk::MemoryPropertyFlags::HOST_COHERENT
                | vk::MemoryPropertyFlags::HOST_VISIBLE
        )?;
        
        Ok(Self {
            buffer,
            memory,
            range,
            size,
            offset: 0,
        })
    }
}