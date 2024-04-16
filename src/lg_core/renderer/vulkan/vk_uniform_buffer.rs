use std::mem::size_of;

use vulkanalia::vk;
use crate::{lg_core::lg_types::reference::Rfc, MyError};
use super::{vk_buffer::VkBuffer, vk_memory_manager::{VkMemoryManager, VkMemoryUsageFlags}};

const MAX_SIZE: u64 = 1000;

pub struct VkUniformBuffer {
    pub buffer: Rfc<VkBuffer>,
    pub range: u64,
    pub size: u64,
    pub offset: u64,
}
impl VkUniformBuffer {
    pub unsafe fn new<T>(
        memory_manager: &mut VkMemoryManager
    ) -> Result<Self, MyError>
    {
        let range = size_of::<T>() as u64;
        let size = range * MAX_SIZE;
        let buffer = memory_manager.new_buffer(
            size, 
            vk::BufferUsageFlags::UNIFORM_BUFFER, 
            VkMemoryUsageFlags::CPU_GPU,
        )?;
        
        Ok(Self {
            buffer,
            range,
            size,
            offset: 0,
        })
    }
    pub unsafe fn from_buffer(memory_manager: &mut VkMemoryManager, other: &Self) -> Result<Self, MyError> {
        let range = other.range;
        let size = other.size;
        let offset = other.offset;

        let buffer  = memory_manager.new_buffer(
            size, 
            vk::BufferUsageFlags::UNIFORM_BUFFER, 
            VkMemoryUsageFlags::GPU_CPU,
        )?;
        
        Ok(Self {
            buffer,
            range,
            size,
            offset,
        })
    }
}