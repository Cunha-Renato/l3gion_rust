use vulkanalia::{
    prelude::v1_2::*, vk
};
use crate::MyError;
use super::{vk_device::VkDevice, vk_instance::VkInstance, vk_memory_allocator, vk_physical_device::VkPhysicalDevice};

#[derive(Default)]
pub struct VkImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
}
impl VkImage {
    pub unsafe fn new(
        instance: &VkInstance,
        device: &VkDevice,
        physical_device: &VkPhysicalDevice,
        width: u32,
        height: u32,
        format: vk::Format,
        aspect_mask: vk::ImageAspectFlags,
        samples: vk::SampleCountFlags,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        mip_levels: u32,
    ) -> Result<Self, MyError> {
        let info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(mip_levels)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(samples);

        let image = device.get_device().create_image(&info, None)?;
        
        // Memory
        let requirements = device.get_device().get_image_memory_requirements(image);
        
        let memory_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(vk_memory_allocator::get_memory_type_index(
                instance, 
                physical_device, 
                vk::MemoryPropertyFlags::DEVICE_LOCAL, 
                requirements
            )?);
        
        let memory = device.get_device().allocate_memory(&memory_info, None)?;
        
        // View
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(1);
        
        let info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::_2D)
            .format(format)
            .subresource_range(subresource_range);
        
        let view = device.get_device().create_image_view(&info, None)?;

        device.get_device().bind_image_memory(image, memory, 0)?;
        
        Ok(Self {
            image,
            view,
            memory
        })
    }
    pub unsafe fn transition_layout(
        &mut self,
        device: &VkDevice,
        mip_levels: u32,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> Result<(), MyError>
    {
        let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) = match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            _ => return Err("Unsupported image layout transition!".into()),
        };
    
        let command_buffer = device.get_transfer_queue().begin_single_time_commands(device)?;
    
        let subresource = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(1);
    
        let barrier = vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.image)
            .subresource_range(subresource)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask);
    
        device.get_device().cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[barrier],
        );
    
        device.get_transfer_queue().end_single_time_commands(device, command_buffer)?;
    
        Ok(()) 
    }
    pub unsafe fn destroy(&mut self, device: &VkDevice) {
        let device = device.get_device();
        
        device.destroy_image_view(self.view, None);
        device.free_memory(self.memory, None);
        device.destroy_image(self.image, None);
    }
}