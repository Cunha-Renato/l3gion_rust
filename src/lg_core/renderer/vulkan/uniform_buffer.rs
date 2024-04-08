use std::mem::size_of;
use vulkanalia::vk;

use crate::{lg_core::lg_types::reference::Rfc, MyError};

use super::{vk_buffer, vk_device::VkDevice, vk_memory_allocator::{VkMemoryManager, VkMemoryRegion, VkMemoryUsageFlags}};
pub struct UniformBuffer {
    pub buffer: vk::Buffer,
    pub region: Rfc<VkMemoryRegion>,
    pub ubo_size: u64,
}
impl UniformBuffer {
    pub unsafe fn new<T>(
        device: &VkDevice,
        memory_manager: &mut VkMemoryManager,
    ) -> Result<Self, MyError> 
    {
        let ubo_size = size_of::<T>() as u64 * 2000;
        let (buffer, region) = create_uniform_buffers(
            device, 
            memory_manager,
            ubo_size
        )?;
        
        Ok(Self {
            buffer,
            region,
            ubo_size
        })
    }
}
unsafe fn create_uniform_buffers(
    device: &VkDevice,
    memory_manager: &mut VkMemoryManager,
    size: u64,
) -> Result<(vk::Buffer, Rfc<VkMemoryRegion>), MyError>
{
    let (buffer, region) = vk_buffer::create_buffer(
        device, 
        memory_manager,
        size, 
        vk::BufferUsageFlags::UNIFORM_BUFFER, 
        VkMemoryUsageFlags::CPU_GPU,
    )?;


    Ok((buffer, region))
}