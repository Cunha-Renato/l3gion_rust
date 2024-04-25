use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};

use crate::{lg_core::lg_types::reference::Rfc, StdError};

use super::{vk_device::VkDevice, vk_image::VkImage};

pub unsafe fn create_framebuffers(
    device: &VkDevice,
    render_pass: &vk::RenderPass,
    present: bool,
    swapchain_image_views: &Vec<vk::ImageView>,
    images: &[Rfc<VkImage>],
    width: u32,
    height: u32,
) -> Result<Vec<vk::Framebuffer>, StdError>
{
    let framebuffers = swapchain_image_views
        .iter()
        .map(|i| {
            let mut attachments = Vec::new();
            
            for img in images {
                attachments.push(img.borrow().view);
            }
            if present {
                attachments.push(*i);
            }
            
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(*render_pass)
                .attachments(&attachments)
                .width(width)
                .height(height)
                .layers(1);

            device.get_device().create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(framebuffers)
}