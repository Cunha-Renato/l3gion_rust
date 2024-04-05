use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};
use crate::{lg_core::renderer::helper, MyError};
use super::{shader::Shader, vk_descriptor::VkPipelineDescriptorData, vk_device::VkDevice, vk_instance::VkInstance, vk_physical_device::VkPhysicalDevice, vk_renderpass::VkRenderPass};
pub struct VkPipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub descriptor_data: Vec<VkPipelineDescriptorData>,
}
impl VkPipeline {
    pub unsafe fn new(
        device: &VkDevice,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        mut vert_shader: Shader,
        mut frag_shader: Shader,
        vertex_binding_descriptions: &[vk::VertexInputBindingDescription],
        vertex_attribute_descriptions: &[vk::VertexInputAttributeDescription],
        viewports: Vec<vk::Viewport>,
        scissors: Vec<vk::Rect2D>,
        msaa_samples: vk::SampleCountFlags,
        render_pass: &VkRenderPass
    ) -> Result<Self, MyError> 
    {
        let v_device = device.get_device();

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(vertex_binding_descriptions)
        .vertex_attribute_descriptions(vertex_attribute_descriptions);
        
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(viewports.as_slice())
            .scissors(scissors.as_slice());

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);
        
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(true)
            .min_sample_shading(0.2)
            .rasterization_samples(msaa_samples);
        
        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false);

        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);
        
        let attachments = &[attachment];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);
        
        let mut descriptor_data = Vec::new();
        for _ in 0..helper::MAX_FRAMES_IN_FLIGHT {
            descriptor_data.push(VkPipelineDescriptorData::new(
                device, 
                instance, 
                physical_device
            )?);
        }

        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_data[0].layouts);

        let layout = v_device.create_pipeline_layout(&layout_info, None)?;

        let stages = &[
            vert_shader.info, 
            frag_shader.info
        ];
        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .layout(layout)
            .render_pass(*render_pass.get_render_pass())
            .subpass(0);

        let pipeline = v_device.create_graphics_pipelines(
            vk::PipelineCache::null(), 
            &[info], 
            None
        )?.0[0];

        vert_shader.destroy_module(v_device);
        frag_shader.destroy_module(v_device);

        Ok(Self {
            layout,
            pipeline,
            descriptor_data,
        })
    }
}