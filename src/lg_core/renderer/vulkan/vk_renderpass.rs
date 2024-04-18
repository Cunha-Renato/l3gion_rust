use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};

use crate::StdError;

use super::vk_device::VkDevice;

#[derive(Default)]
struct VkSubPass {
    bind_point: vk::PipelineBindPoint,
    color_attachments: Vec<vk::AttachmentReference>,
    resolve_attachments: Vec<vk::AttachmentReference>,
    depth_attachment: vk::AttachmentReference,
}

#[derive(Default)]
pub struct VkRenderPassBuilder {
    attachments: Vec<vk::AttachmentDescription>,
    subpasses: Vec<VkSubPass>,
    current_subpass: usize,
}
impl VkRenderPassBuilder {
    pub unsafe fn build(self, device: &VkDevice) -> Result<vk::RenderPass, StdError> {
        let device = device.get_device();
        
        let subpasses = self.subpasses.iter().map(|pass|  {
            let mut sub_desc = vk::SubpassDescription::builder();
            
            if !pass.color_attachments.is_empty() {
                sub_desc = sub_desc.color_attachments(&pass.color_attachments);
            }
            sub_desc = sub_desc.depth_stencil_attachment(&pass.depth_attachment);
            if !pass.resolve_attachments.is_empty() {
                sub_desc = sub_desc.resolve_attachments(&pass.resolve_attachments);
            }
            
            sub_desc
        })
        .collect::<Vec<_>>();

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
        let dependencies = &[dependency];

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&self.attachments)
            .subpasses(&subpasses)
            .dependencies(dependencies);
        
        Ok(device.create_render_pass(&info, None)?)
    }
    pub fn begin() -> Self {
        Self::default()
    }
    pub fn add_attachment(mut self, attachment: vk::AttachmentDescription) -> Self {
        self.attachments.push(attachment);

        self
    }
    pub fn add_color_attachment_ref(mut self, reference: vk::AttachmentReference) -> Self {
        self.subpasses[self.current_subpass].color_attachments.push(reference);

        self
    }
    pub fn add_resolve_attachment_ref(mut self, reference: vk::AttachmentReference) -> Self {
        self.subpasses[self.current_subpass].resolve_attachments.push(reference);

        self
    }
    pub fn set_depth_attachment_ref(mut self, reference: vk::AttachmentReference) -> Self {
        self.subpasses[self.current_subpass].depth_attachment = reference;

        self
    }
    pub fn set_bind_point(mut self, bind_point: vk::PipelineBindPoint) -> Self {
        self.subpasses[self.current_subpass].bind_point = bind_point;

        self
    }
    pub fn new_subpass(mut self) -> Self {
        if self.subpasses.is_empty() {
            self.subpasses.push(VkSubPass::default());
        } else {
            self.current_subpass += 1;
        }

        self
    }
}
pub unsafe fn get_depth_format(instance: &Instance, physical_device: &vk::PhysicalDevice) -> Result<vk::Format, StdError>
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
) -> Result<vk::Format, StdError>
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