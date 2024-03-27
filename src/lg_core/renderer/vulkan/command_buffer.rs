use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};
use crate::MyError;
use super::{queue_family::QueueFamilyIndices, swapchain::VkSwapchain};

#[derive(Default)]
pub struct VkCommandPool {
    pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
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
            buffers: Vec::new(),
        })
    }
    pub unsafe fn create_buffers(
        &mut self,
        device: &Device,
        count: u32,
    ) -> Result<(), MyError>
    {
        self.buffers = create_command_buffers(device, &self.pool, count)?;
        Ok(())
    }
    pub unsafe fn reset_command_buffer(
        &mut self,
        device: &Device,
        index: usize,
    ) -> Result<(), MyError>
    {
        // Freeing the old one
        let previous = self.buffers[index];
        device.free_command_buffers(self.pool, &[previous]);

        // Allocating a new one
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        
        let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];
        self.buffers[index] = command_buffer;
        
        Ok(())
    }
    pub unsafe fn get_render_pass_begin_info(
        &self,
        swapchain: &VkSwapchain,
        render_pass: &vk::RenderPass,
        framebuffer: &vk::Framebuffer,
    ) -> vk::RenderPassBeginInfo
    {
        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(swapchain.extent);
        
        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.1, 0.0, 0.0, 1.0]
            }
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            }
        };
        
        let clear_values = &[color_clear_value, depth_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(*render_pass)
            .framebuffer(*framebuffer)
            .render_area(render_area)
            .clear_values(clear_values)
            .build();
        
        info
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
    
    pub unsafe fn free_buffers(&mut self, device: &Device) {
        device.free_command_buffers(self.pool, &self.buffers);
    }
    pub unsafe fn destroy(&mut self, device: &Device) {
        device.destroy_command_pool(self.pool, None);
    }
}

unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    indices: &QueueFamilyIndices,
) -> Result<vk::CommandPool, MyError>
{
    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
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
unsafe fn create_command_buffers(
    device: &Device,
    pool: &vk::CommandPool,
    count: u32,
) -> Result<Vec<vk::CommandBuffer>, MyError>
{
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(*pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(count);
    
    Ok(device.allocate_command_buffers(&allocate_info)?)
}