use vulkanalia::{
    prelude::v1_2::*, vk
};

use crate::{lg_core::lg_types::reference::Rfc, MyError};

use super::{vk_device::VkDevice, vk_memory_manager::VkMemoryRegion};
#[derive(Default)]
pub struct VkImage {
    pub image: vk::Image,
    pub region: Rfc<VkMemoryRegion>,
    pub view: vk::ImageView,
    pub width: u32,
    pub height: u32,
}
impl VkImage {
    pub unsafe fn new(
        image: vk::Image,
        region: Rfc<VkMemoryRegion>,
        view: vk::ImageView,
        width: u32,
        height: u32,
    ) -> Self {
        
        Self {
            image,
            region,
            view,
            width,
            height,
        }
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
        device.destroy_image(self.image, None);
    }
}