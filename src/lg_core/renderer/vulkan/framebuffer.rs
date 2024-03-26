use vulkanalia:: {
    prelude::v1_0::*, 
    vk,
};

use crate::MyError;

use super::image::ImageData;

pub unsafe fn create_framebuffers(
    device: &Device,
    render_pass: &vk::RenderPass,
    swapchain_image_data: &ImageData,
    color_image_data: &ImageData,
    depth_image_data: &ImageData,
    width: u32,
    height: u32,
) -> Result<Vec<vk::Framebuffer>, MyError>
{
    let framebuffers = swapchain_image_data.views
        .iter()
        .map(|i| {
            let attachments = &[
                color_image_data.views[0],
                depth_image_data.views[0],
                *i,
            ];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(*render_pass)
                .attachments(attachments)
                .width(width)
                .height(height)
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(framebuffers)
}