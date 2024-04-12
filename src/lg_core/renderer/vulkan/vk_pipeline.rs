use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};
use crate::{lg_core::{lg_types::reference::Rfc, renderer::{helper, uniform_buffer_object::{ModelUBO, ViewProjUBO}}}, MyError};
use super::{framebuffer, shader::Shader, vk_device::VkDevice, vk_image::VkImage, vk_instance::VkInstance, vk_memory_allocator::VkMemoryManager, vk_descriptor::VkDescriptorData, vk_physical_device::VkPhysicalDevice, vk_renderpass::VkRenderPass, vk_swapchain::VkSwapchain, vk_uniform_buffer::VkUniformBuffer};

pub trait VulkanPipeline {}

pub struct DefaultPipeline {
    memory_manager: Rfc<VkMemoryManager>,
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub descriptor_data: Vec<VkDescriptorData>,
    pub color_image: VkImage,
    pub depth_image: VkImage,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub render_pass: VkRenderPass,
}
impl DefaultPipeline {
    pub unsafe fn new(
        device: Rfc<VkDevice>,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        memory_manager: Rfc<VkMemoryManager>,
        vertex_binding_descriptions: &[vk::VertexInputBindingDescription],
        vertex_attribute_descriptions: &[vk::VertexInputAttributeDescription],
        swapchain: &VkSwapchain,
        msaa_samples: vk::SampleCountFlags,
        render_pass: VkRenderPass
    ) -> Result<Self, MyError> 
    {
        let dev = device.borrow();
        let v_device = dev.get_device();

        // Shaders
        let mut vert_shader = Shader::new(
            &device.borrow(), 
            "assets/shaders/compiled/2DShader.spv",
        )?;
        let mut frag_shader = Shader::new(
            &device.borrow(), 
            "assets/shaders/compiled/shader.spv",
        )?;

        // Viewport and Scissor
        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain.extent.width as f32)
            .height(swapchain.extent.width as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();
        
        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain.extent)
            .build();

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(vertex_binding_descriptions)
        .vertex_attribute_descriptions(vertex_attribute_descriptions);
        
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let vps = &[viewport];
        let scs = &[scissor];
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(vps)
            .scissors(scs);

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
            let model = VkUniformBuffer::new::<ModelUBO>(
                        &device.borrow(), 
                        &mut memory_manager.borrow_mut()
                )?;
            let view_proj = VkUniformBuffer::new::<ViewProjUBO>(
                        &device.borrow(), 
                        &mut memory_manager.borrow_mut()
                )?;
            descriptor_data.push(VkDescriptorData::new(
                device.clone(),
                &[&vert_shader, &frag_shader],
                memory_manager.clone(),
                vec![model, view_proj]
            )?);
        }

        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_data[0].layouts[0]);

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

        let color_image = VkImage::new(
            &device.borrow(),
            &mut memory_manager.borrow_mut(),
            swapchain.extent.width, 
            swapchain.extent.height, 
            swapchain.format, 
            vk::ImageAspectFlags::COLOR, 
            msaa_samples, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT, 
            1
        )?;
        let depth_image = VkImage::new(
            &device.borrow(), 
            &mut memory_manager.borrow_mut(),
            swapchain.extent.width, 
            swapchain.extent.height, 
            helper::get_depth_format(instance.get_instance(), physical_device.get_device())?, 
            vk::ImageAspectFlags::DEPTH, 
            msaa_samples, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, 
            1
        )?;

        let framebuffers = framebuffer::create_framebuffers(
            &device.borrow(), 
            &render_pass, 
            &swapchain.views, 
            &color_image, 
            &depth_image, 
            swapchain.extent.width, 
            swapchain.extent.height
        )?;

        vert_shader.destroy_module(v_device);
        frag_shader.destroy_module(v_device);

        Ok(Self {
            memory_manager,
            layout,
            pipeline,
            descriptor_data,
            color_image,
            depth_image,
            framebuffers,
            render_pass,
        })
    }
    pub unsafe fn destroy(&mut self, device: &VkDevice) -> Result<(), MyError> {
        let v_device = device.get_device();
        
        for dd in &mut self.descriptor_data {
            dd.destroy()?;
        }

        self.color_image.destroy(device, &mut self.memory_manager.borrow_mut())?;
        self.depth_image.destroy(device, &mut self.memory_manager.borrow_mut())?;

        self.framebuffers
            .iter()
            .for_each(|f| v_device.destroy_framebuffer(*f, None));
        v_device.destroy_pipeline(self.pipeline, None);
        v_device.destroy_pipeline_layout(self.layout, None);
        v_device.destroy_render_pass(*self.render_pass.get_render_pass(), None);
        
        Ok(())
    }
}
pub struct ObjectPickerPipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub descriptor_data: Vec<VkDescriptorData>,
    pub color_image: VkImage,
    pub depth_image: VkImage,
    pub framebuffers: Vec<vk::Framebuffer>,
}
impl ObjectPickerPipeline {
    pub unsafe fn new (
        device: Rfc<VkDevice>,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        memory_manager: &mut VkMemoryManager,
        vertex_binding_descriptions: &[vk::VertexInputBindingDescription],
        vertex_attribute_descriptions: &[vk::VertexInputAttributeDescription],
        swapchain: &VkSwapchain,
        render_pass: &VkRenderPass
    ) -> Result<Self, MyError>
    {
        todo!()
    }
}

impl VulkanPipeline for DefaultPipeline {}
impl VulkanPipeline for ObjectPickerPipeline {}