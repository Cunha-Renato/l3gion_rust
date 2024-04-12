use std::mem::size_of;

use vulkanalia::vk;
use crate::{lg_core::lg_types::reference::Rfc, MyError};
use super::{vk_buffer, vk_device::VkDevice, vk_memory_allocator::{VkMemoryManager, VkMemoryRegion, VkMemoryUsageFlags}};

const MAX_SIZE: u64 = 1000;

pub struct VkUniformBuffer {
    pub buffer: vk::Buffer,
    pub region: Rfc<VkMemoryRegion>,
    pub range: u64,
    pub size: u64,
    pub offset: u64,
}
impl VkUniformBuffer {
    pub unsafe fn new<T>(
        device: &VkDevice,
        memory_manager: &mut VkMemoryManager
    ) -> Result<Self, MyError>
    {
        let range = size_of::<T>() as u64;
        let size = range * MAX_SIZE;
        let (buffer, region) = vk_buffer::create_buffer(
            device, 
            memory_manager,
            size, 
            vk::BufferUsageFlags::UNIFORM_BUFFER, 
            VkMemoryUsageFlags::CPU_GPU,
        )?;
        
        Ok(Self {
            buffer,
            region,
            range,
            size,
            offset: 0,
        })
    }
    pub unsafe fn from_buffer(device: &VkDevice, memory_manager: &mut VkMemoryManager, other: &Self) -> Result<Self, MyError> {
        let range = other.range;
        let size = other.size;
        let offset = other.offset;

        let (buffer, region) = vk_buffer::create_buffer(
            device, 
            memory_manager,
            size, 
            vk::BufferUsageFlags::UNIFORM_BUFFER, 
            VkMemoryUsageFlags::CPU_GPU,
        )?;
        
        Ok(Self {
            buffer,
            region,
            range,
            size,
            offset,
        })
    }
}