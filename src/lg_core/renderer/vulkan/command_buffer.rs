use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};
use crate::MyError;
use super::queue_family::QueueFamilyIndices;

#[derive(Default)]
pub struct VkCommandPool {
    pool: vk::CommandPool,
}
impl VkCommandPool {
    pub unsafe fn new(
        instance: &Instance,
        device: &Device,
        indices: &QueueFamilyIndices
    ) -> Result<Self, MyError> {
        let pool = create_command_pool(
            instance, 
            device, 
            indices
        )?;
        
        Ok(Self {
            pool,
        })
    }
    pub unsafe fn begin_single_time_commands(
        &self,
        device: &Device,
    ) -> Result<vk::CommandBuffer, MyError>
    {
        Ok(begin_single_time_commands(
            device, 
            &self.pool
        )?)
    }
    pub unsafe fn end_single_time_commands(
        &self,
        device: &Device,
        queue: &vk::Queue,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), MyError>
    {
        end_single_time_commands(device, queue, &self.pool, command_buffer);
        
        Ok(())
    }
}

unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    indices: &QueueFamilyIndices,
) -> Result<vk::CommandPool, MyError>
{
    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty())
        .queue_family_index(indices.graphics);
    
    Ok(device.create_command_pool(&info, None)?)
}
unsafe fn begin_single_time_commands(
    device: &Device,
    command_pool: &vk::CommandPool
) -> Result<vk::CommandBuffer, MyError>
{
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(*command_pool)
        .command_buffer_count(1);
    
    let command_buffer = device.allocate_command_buffers(&info)?[0];
    
    let info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    
    device.begin_command_buffer(command_buffer, &info)?;
    
    Ok(command_buffer)
}
unsafe fn end_single_time_commands(
    device: &Device,
    queue: &vk::Queue,
    command_pool: &vk::CommandPool,
    command_buffer: vk::CommandBuffer
) -> Result<(), MyError>
{
    device.end_command_buffer(command_buffer)?;
    
    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder()
        .command_buffers(command_buffers);
    
    device.queue_submit(*queue, &[info], vk::Fence::null())?;
    device.queue_wait_idle(*queue)?;
    
    device.free_command_buffers(*command_pool, &[command_buffer]);
    
    Ok(())
}