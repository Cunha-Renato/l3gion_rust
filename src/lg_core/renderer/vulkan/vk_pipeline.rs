use std::path::Path;

use vulkanalia:: {
    prelude::v1_2::*, 
    vk,
};
use crate::{lg_core::{lg_types::reference::Rfc, renderer::{helper, uniform_buffer_object::{ModelUBOId, ViewProjUBO}, vertex::{Vertex, VkVertex}}}, StdError};

use super::{framebuffer, shader::Shader, vk_descriptor::VkDescriptorData, vk_device::VkDevice, vk_image::VkImage, vk_instance::VkInstance, vk_memory_manager::VkMemoryManager, vk_renderpass::{get_depth_format, VkRenderPassBuilder}, vk_physical_device::VkPhysicalDevice, vk_swapchain::VkSwapchain, vk_uniform_buffer::VkUniformBuffer};

pub struct VkPipelineCreateInfo {
    msaa_samples: vk::SampleCountFlags,
    color_format: vk::Format,
    color_image_usage: vk::ImageUsageFlags,
    attachments_count: u32,
    shaders: Vec<Shader>,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    dynamic_states: Vec<vk::DynamicState>,
    render_pass: vk::RenderPass,
    uniform_buffers: Vec<VkUniformBuffer>,
    should_present: bool,
}

pub struct VkPipeline {
    memory_manager: Rfc<VkMemoryManager>,
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub descriptor_data: Vec<VkDescriptorData>,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub render_pass: vk::RenderPass,
    pub color_image: Rfc<VkImage>,
    pub depth_image: Rfc<VkImage>,
    pub present: bool,
}
impl VkPipeline {
    pub unsafe fn new<T: VkVertex>(
        device: Rfc<VkDevice>,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        swapchain: &VkSwapchain,
        memory_manager: Rfc<VkMemoryManager>,
        mut info: VkPipelineCreateInfo,
    ) -> Result<Self, StdError>
    {
        let dev = device.borrow();
        let v_device = dev.get_device();
        
        let binding = [T::binding_description()];
        let attribute = T::attribute_descritptions();
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding)
            .vertex_attribute_descriptions(&attribute);
        
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        
        let vps = &[info.viewport];
        let scs = &[info.scissor];
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
        
        let multisample_state = if info.msaa_samples != vk::SampleCountFlags::_1 {
            vk::PipelineMultisampleStateCreateInfo::builder()
                .sample_shading_enable(true)
                .min_sample_shading(0.2)
                .rasterization_samples(info.msaa_samples)
        } else {
            vk::PipelineMultisampleStateCreateInfo::builder()
                .sample_shading_enable(false)
                .rasterization_samples(info.msaa_samples)
        };
        
        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);
        let attachments = &[color_blend_attachment];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);
        
        let mut descriptor_data = Vec::new();
        for _ in 0..helper::MAX_FRAMES_IN_FLIGHT {
            let mut uniform_buffers = Vec::new();
            for buffer in &info.uniform_buffers {
                uniform_buffers.push(VkUniformBuffer::from_buffer(&mut memory_manager.borrow_mut(), buffer)?);
            }

            descriptor_data.push(VkDescriptorData::new(
                device.clone(),
                &info.shaders,
                memory_manager.clone(),
                uniform_buffers,
            )?);
        }

        // Cleaning the unused buffers
        for b in &info.uniform_buffers {
            memory_manager.borrow_mut().destroy_buffer(b.buffer.clone())?;
        }

        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_data[0].layouts[0]);

        let layout = v_device.create_pipeline_layout(&layout_info, None)?;
        
        let stages: Vec<vk::PipelineShaderStageCreateInfo> = info.shaders
            .iter()
            .map(|s| s.info) 
            .collect();
            
        let color_image = memory_manager.borrow_mut().new_image(
            swapchain.extent.width, 
            swapchain.extent.height, 
            info.color_format, 
            vk::ImageAspectFlags::COLOR, 
            info.msaa_samples, 
            vk::ImageTiling::OPTIMAL, 
            info.color_image_usage,
            1
        )?;

        let depth_image = memory_manager.borrow_mut().new_image(
            swapchain.extent.width, 
            swapchain.extent.height, 
            helper::get_depth_format(instance.get_instance(), physical_device.get_device())?, 
            vk::ImageAspectFlags::DEPTH, 
            info.msaa_samples, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, 
            1
        )?;
        
        let framebuffers = framebuffer::create_framebuffers(
            &device.borrow(),
            &info.render_pass,
            info.attachments_count, 
            &swapchain.views, 
            &color_image.borrow(), 
            &depth_image.borrow(), 
            swapchain.extent.width, 
            swapchain.extent.height
        )?;
        
        let mut pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .layout(layout)
            .render_pass(info.render_pass)
            .subpass(0);

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
                .dynamic_states(&info.dynamic_states);
        if !info.dynamic_states.is_empty() {
            pipeline_info = pipeline_info.dynamic_state(&dynamic_state);
        }

        let pipeline = v_device.create_graphics_pipelines(
            vk::PipelineCache::null(), 
            &[pipeline_info], 
            None
        )?.0[0];

        info.shaders
            .iter_mut()
            .for_each(|s| s.destroy_module(v_device));


        Ok(Self {
            memory_manager,
            layout,
            pipeline,
            descriptor_data,
            color_image,
            depth_image,
            framebuffers,
            render_pass: info.render_pass,
            present: info.should_present,
        })
    }
    pub unsafe fn get_2d(
        device: Rfc<VkDevice>,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        swapchain: &VkSwapchain,
        memory_manager: Rfc<VkMemoryManager>,
        msaa_samples: vk::SampleCountFlags,
    ) -> Result<Self, StdError>
    {
        let model = VkUniformBuffer::new::<ModelUBOId>(
            &mut memory_manager.borrow_mut()
        )?;
        let view_proj = VkUniformBuffer::new::<ViewProjUBO>(
            &mut memory_manager.borrow_mut()
        )?;

        let info = VkPipelineCreateInfo {
            msaa_samples,
            color_format: swapchain.format,
            color_image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            attachments_count: 3,
            shaders: vec![
                Shader::new(&device.borrow().get_device(), Path::new("resources/shaders/compiled/2DShader_v.spv"))?,
                Shader::new(&device.borrow().get_device(), Path::new("resources/shaders/compiled/2DShader_f.spv"))?,
            ],
            viewport: vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(swapchain.extent.width as f32)
                .height(swapchain.extent.width as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .build(),
            scissor: vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(swapchain.extent)
                .build(),
            dynamic_states: vec![],
            render_pass: VkRenderPassBuilder::begin()
                .add_attachment(vk::AttachmentDescription::builder()
                    .format(swapchain.format)
                    .samples(msaa_samples)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .build()
                )
                .add_attachment(vk::AttachmentDescription::builder()
                    .format(get_depth_format(instance.get_instance(), physical_device.get_device())?)
                    .samples(msaa_samples)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .build()
                )
                .add_attachment(vk::AttachmentDescription::builder()
                    .format(swapchain.format)
                    .samples(vk::SampleCountFlags::_1)
                    .load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                    .build()
                )
                .new_subpass()
                .set_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .add_color_attachment_ref(vk::AttachmentReference::builder()
                    .attachment(0)
                    .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .build()
                )
                .set_depth_attachment_ref(vk::AttachmentReference::builder()
                    .attachment(1)
                    .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .build()
                )
                .add_resolve_attachment_ref(vk::AttachmentReference::builder()
                    .attachment(2)
                    .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .build()
                )
                .build(&device.borrow())?,
            uniform_buffers: vec![model, view_proj],
            should_present: true,
        };
        
        Self::new::<Vertex>(
            device, 
            instance, 
            physical_device, 
            swapchain, 
            memory_manager, 
            info
        )
    }
    pub unsafe fn obj_picker(
        device: Rfc<VkDevice>,
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        swapchain: &VkSwapchain,
        memory_manager: Rfc<VkMemoryManager>,
    ) -> Result<Self, StdError>
    {
        let model = VkUniformBuffer::new::<ModelUBOId>(
            &mut memory_manager.borrow_mut()
        )?;
        let view_proj = VkUniformBuffer::new::<ViewProjUBO>(
            &mut memory_manager.borrow_mut()
        )?;
        
        let info = VkPipelineCreateInfo {
            msaa_samples: vk::SampleCountFlags::_1,
            color_format: vk::Format::R8G8B8A8_SRGB,
            color_image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_SRC,
            attachments_count: 2,
            shaders: vec![
                Shader::new(&device.borrow().get_device(), Path::new("resources/shaders/compiled/obj_picker_v.spv"))?,
                Shader::new(&device.borrow().get_device(), Path::new("resources/shaders/compiled/obj_picker_f.spv"))?,
            ],
            viewport: vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(swapchain.extent.width as f32)
                .height(swapchain.extent.width as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .build(),
            scissor: vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(swapchain.extent)
                .build(),
            dynamic_states: vec![],
            render_pass: VkRenderPassBuilder::begin()
                .add_attachment(vk::AttachmentDescription::builder()
                    .format(vk::Format::R8G8B8A8_SRGB)
                    .samples(vk::SampleCountFlags::_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .build()
                )
                .add_attachment(vk::AttachmentDescription::builder()
                    .format(get_depth_format(instance.get_instance(), physical_device.get_device())?)
                    .samples(vk::SampleCountFlags::_1)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .build()
                )
                .new_subpass()
                .set_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .add_color_attachment_ref(vk::AttachmentReference::builder()
                    .attachment(0)
                    .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                    .build()
                )
                .set_depth_attachment_ref(vk::AttachmentReference::builder()
                    .attachment(1)
                    .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .build()
                )
                .build(&device.borrow())?,
            uniform_buffers: vec![model, view_proj],
            should_present: false,
        };
        
        Self::new::<Vertex>(
            device, 
            instance, 
            physical_device, 
            swapchain, 
            memory_manager, 
            info
        )
    }
    pub unsafe fn destroy(&mut self, device: &VkDevice) -> Result<(), StdError>{
        let v_device = device.get_device();
        
        for dd in &mut self.descriptor_data {
            dd.destroy()?;
        }

        self.memory_manager.borrow_mut().destroy_image(self.color_image.clone())?;
        self.memory_manager.borrow_mut().destroy_image(self.depth_image.clone())?;

        self.framebuffers
            .iter()
            .for_each(|f| v_device.destroy_framebuffer(*f, None));
        v_device.destroy_pipeline(self.pipeline, None);
        v_device.destroy_pipeline_layout(self.layout, None);
        v_device.destroy_render_pass(self.render_pass, None);
        
        Ok(())
    }
}