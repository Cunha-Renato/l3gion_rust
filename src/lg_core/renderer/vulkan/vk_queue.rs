use vulkanalia::{
    prelude::v1_2::*, vk,
};
use crate::StdError;

use super::{vk_command_pool, vk_device::VkDevice};

pub struct VkQueue {
    pub queue: vk::Queue,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
}
impl VkQueue {
    pub unsafe fn new(
        device: &Device,
        queue_indice: u32,
    ) -> Result<Self, StdError>
    {
        let queue = device.get_device_queue(queue_indice, 0);
        let pool = vk_command_pool::create_command_pool(device, queue_indice)?;
        let buffers = Vec::new();
        
        Ok(Self {
            queue,
            command_pool: pool,
            command_buffers: buffers,
        })
    }
    pub unsafe fn allocate_command_buffers(
        &mut self, 
        device: &Device,
        lenght: u32
    ) -> Result<(), StdError> 
    {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(lenght);
    
        self.command_buffers = device.allocate_command_buffers(&allocate_info)?;
        
        Ok(())
    }
    pub unsafe fn begin_single_time_commands(
        &self,
        device: &VkDevice
    ) -> Result<vk::CommandBuffer, StdError>
    {
        let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(self.command_pool)
        .command_buffer_count(1);
    
        let command_buffer = device.get_device().allocate_command_buffers(&info)?[0];
        
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        
        device.get_device().begin_command_buffer(command_buffer, &info)?;
    
        Ok(command_buffer)
    }
    pub unsafe fn end_single_time_commands(
        &self,
        device: &VkDevice,
        command_buffer: vk::CommandBuffer
    ) -> Result<(), StdError>
    {
        let device = device.get_device();
        device.end_command_buffer(command_buffer)?;
        
        let command_buffers = &[command_buffer];
        let info = vk::SubmitInfo::builder()
            .command_buffers(command_buffers);
        
        device.queue_submit(self.queue, &[info], vk::Fence::null())?;
        device.queue_wait_idle(self.queue)?;
        
        device.free_command_buffers(self.command_pool, &[command_buffer]);
        
        Ok(())
    }    
    pub unsafe fn destroy_pool(&self, device: &VkDevice) {
        device.get_device().destroy_command_pool(self.command_pool, None);
    }
    pub unsafe fn free_command_buffers(&self, device: &VkDevice) {
        if self.command_buffers.len() > 0 {
            device.get_device().free_command_buffers(self.command_pool, &self.command_buffers);
        }
    }
}