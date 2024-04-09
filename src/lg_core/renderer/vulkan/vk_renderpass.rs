use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};
use crate::MyError;

use super::{vk_device::VkDevice, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice};

#[derive(Default)]
pub struct VkRenderPass {
    render_pass: vk::RenderPass,
}
impl VkRenderPass {
    pub unsafe fn new(
        device: &VkDevice,
        attachments: Vec<vk::AttachmentDescription>,
    ) -> Result<Self, MyError>
    {
        let device = device.get_device();
        
        // Subpasses
        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        
        let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        
        let color_resolve_attachment_ref = vk::AttachmentReference::builder()
            .attachment(2)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachments = &[color_attachment_ref];

        let mut resolve_attachments = &[] as &[vk::AttachmentReferenceBuilder];
        let color_resolve = &[color_resolve_attachment_ref];
        if attachments.len() > 2 {
            resolve_attachments = color_resolve;
        }

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments)
            .depth_stencil_attachment(&depth_stencil_attachment_ref)
            .resolve_attachments(resolve_attachments);
        
        // Dependencies
        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT 
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT 
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE 
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);
        
        // Create
        let subpasses = &[subpass];
        let dependencies = &[dependency];
        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(subpasses)
            .dependencies(dependencies);

        Ok(Self {
            render_pass: device.create_render_pass(&info, None)?,
        })
    }
    pub unsafe fn get_default(
        instance: &VkInstance,
        device: &VkDevice,
        physical_device: &VkPhysicalDevice,
        format: vk::Format,
        msaa_samples: vk::SampleCountFlags,
    ) -> Result<Self, MyError>
    {
        let instance = instance.get_instance();
        let physical_device = physical_device.get_device();

        let color_attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(msaa_samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_stencil_attachment = vk::AttachmentDescription::builder()
            .format(get_depth_format(instance, physical_device)?)
            .samples(msaa_samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_resolve_attachment = vk::AttachmentDescription::builder()
            .format(format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
        
        Self::new(
            device, 
            vec![*color_attachment, *depth_stencil_attachment, *color_resolve_attachment]
        )
    }
    pub unsafe fn get_object_picker(
        instance: &VkInstance,
        device: &VkDevice,
        physical_device: &VkPhysicalDevice,
    ) -> Result<Self, MyError>
    {
        let instance = instance.get_instance();
        let physical_device = physical_device.get_device();

        let color_attachment = vk::AttachmentDescription::builder()
            .format(vk::Format::R32_SFLOAT)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_stencil_attachment = vk::AttachmentDescription::builder()
            .format(get_depth_format(instance, physical_device)?)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        
        Self::new(
            device, 
            vec![*color_attachment, *depth_stencil_attachment]
        )
    }
    pub fn get_render_pass(&self) -> &vk::RenderPass {
        &self.render_pass
    }
}
unsafe fn get_depth_format(instance: &Instance, physical_device: &vk::PhysicalDevice) -> Result<vk::Format, MyError>
{
    let canditates = &[
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];
    
    get_supported_format(
        instance, 
        physical_device, 
        canditates, 
        vk::ImageTiling::OPTIMAL, 
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
    )
}
unsafe fn get_supported_format(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    canditates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags
) -> Result<vk::Format, MyError>
{
    match canditates
        .iter()
        .cloned()
        .find(|f| {
            let properties = instance.get_physical_device_format_properties(
                *physical_device, 
                *f
            );
            
            match tiling {
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }
        })
    {
        Some(result) => Ok(result),
        None => Err("Failed to find supported format!".into())
    }
}