use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};

use crate::MyError;

use super::{vk_device::VkDevice, vk_image::VkImage, vk_renderpass::VkRenderPass};

pub unsafe fn create_framebuffers(
    device: &VkDevice,
    render_pass: &VkRenderPass,
    attachments_count: u32,
    swapchain_image_views: &Vec<vk::ImageView>,
    color_image_data: &VkImage,
    depth_image_data: &VkImage,
    width: u32,
    height: u32,
) -> Result<Vec<vk::Framebuffer>, MyError>
{
    let framebuffers = swapchain_image_views
        .iter()
        .map(|i| {
            let mut attachments = Vec::new();
            
            if attachments_count > 0 {
                attachments.push(color_image_data.view);
            }
            if attachments_count > 1 {
                attachments.push(depth_image_data.view);
            }
            if attachments_count > 2 {
                attachments.push(*i);
            }
            
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(*render_pass.get_render_pass())
                .attachments(&attachments)
                .width(width)
                .height(height)
                .layers(1);

            device.get_device().create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(framebuffers)
}