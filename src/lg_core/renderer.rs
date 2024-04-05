pub mod camera;
pub mod model;
pub mod vertex;
pub mod texture;
pub mod vulkan;
pub mod object;
pub mod object_storage;
pub mod helper;
pub mod uniform_buffer_object;

use std::{any::Any,  mem::size_of};
use std::ptr::copy_nonoverlapping as memcpy;

use nalgebra_glm as glm;
use sllog::*;
use vulkanalia::vk::{ExtDebugUtilsExtension, InstanceV1_0, KhrSurfaceExtension};
use winit::window::Window;
use vulkanalia::{
    vk::{
        self, DeviceV1_0, Handle, HasBuilder, KhrSwapchainExtension
    }, 
    window as vk_window, 
    Entry, 
};
use crate::MyError;
use texture::Texture;
use helper::RendererData;
use self::uniform_buffer_object::{ModelUBO, ViewProjUBO};
use self::vulkan::vk_descriptor::BufferCategory;
use self::{camera::Camera, vulkan::vk_texture::VkTexture};
use self::object_storage::ObjectStorage;
use self::vulkan::vk_device::{VkDevice, VkQueueFamily};
use self::vulkan::vk_image::VkImage;
use self::vulkan::vk_instance::VkInstance;
use self::vulkan::vk_physical_device::VkPhysicalDevice;
use self::vulkan::vk_renderpass::VkRenderPass;
use self::{object::Object, uniform_buffer_object::UniformBufferObject, vertex::Vertex, vulkan::{framebuffer, pipeline::VkPipeline, shader::Shader, vk_swapchain::VkSwapchain}};

use super::{lg_types::reference::Rfc, uuid::UUID};

pub struct Renderer {
    window: Rfc<Window>,
    entry: Entry,
    instance: VkInstance,
    data: RendererData,
    device: VkDevice,
    test_pipeline: VkPipeline, // One Pipeline for each kind of rendering (ie. Batch, Circle, Rect, Normal)
    objects: ObjectStorage<Vertex>,
    frame_active_objects: Vec<UUID>,
    frame: usize,
    pub resized: bool,
    camera: Rfc<Camera>,
}
impl Renderer {
    pub unsafe fn init(window: Window) -> Result<(Self, Rfc<Window>), MyError> {
        let mut data = RendererData::default();
        let entry = helper::create_entry()?; 
        let instance = VkInstance::new(&entry, &window)?;

        data.msaa_samples = vk::SampleCountFlags::_8;
        data.surface = vk_window::create_surface(instance.get_instance(), &window, &window)?;
        data.physical_device = VkPhysicalDevice::new(&instance, &data.surface)?;

        let mut device = VkDevice::new(
            &instance, 
            &data.physical_device, 
            &data.surface
        )?;

        data.swapchain = VkSwapchain::new(
            &window, 
            &instance, 
            &data.surface, 
            &data.physical_device, 
            &device
        )?;

        device.allocate_command_buffers(VkQueueFamily::GRAPHICS, data.swapchain.images.len() as u32)?;
        device.allocate_command_buffers(VkQueueFamily::PRESENT, data.swapchain.images.len() as u32)?;
        device.allocate_command_buffers(VkQueueFamily::TRANSFER, data.swapchain.images.len() as u32)?;

        data.render_pass = VkRenderPass::get_default(
            &instance, 
            &device, 
            &data.physical_device, 
            data.swapchain.format, 
            data.msaa_samples
        )?;
        
        // Shaders
        let vert_shader = Shader::new(
            &device, 
            vk::ShaderStageFlags::VERTEX, 
            include_bytes!("../../assets/shaders/compiled/2DShader.spv")
        )?;
        let frag_shader = Shader::new(
            &device, 
            vk::ShaderStageFlags::FRAGMENT, 
            include_bytes!("../../assets/shaders/compiled/shader.spv")
        )?;
        
        // Viewport and Scissor
        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(data.swapchain.extent.width as f32)
            .height(data.swapchain.extent.width as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();
        
        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(data.swapchain.extent)
            .build();

        let test_pipeline = VkPipeline::new(
            &device, 
            &instance,
            &data.physical_device,
            vert_shader, 
            frag_shader, 
            &[Vertex::binding_description()],
            &Vertex::attribute_descritptions(), 
            vec![viewport], 
            vec![scissor], 
            data.msaa_samples, 
            &data.render_pass
        )?;

        data.color_image = VkImage::new(
            &instance, 
            &device, 
            &data.physical_device, 
            data.swapchain.extent.width, 
            data.swapchain.extent.height, 
            data.swapchain.format, 
            vk::ImageAspectFlags::COLOR, 
            data.msaa_samples, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT, 
            1
        )?;
        data.depth_image = VkImage::new(
            &instance, 
            &device, 
            &data.physical_device, 
            data.swapchain.extent.width, 
            data.swapchain.extent.height, 
            helper::get_depth_format(instance.get_instance(), data.physical_device.get_device())?, 
            vk::ImageAspectFlags::DEPTH, 
            data.msaa_samples, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, 
            1
        )?;
        data.framebuffers = framebuffer::create_framebuffers(
            &device, 
            &data.render_pass, 
            &data.swapchain.views, 
            &data.color_image, 
            &data.depth_image, 
            data.swapchain.extent.width, 
            data.swapchain.extent.height
        )?;

        let window = Rfc::new(window);

        helper::create_sync_objects(device.get_device(), &mut data)?;

        Ok((Self {
            window: window.clone(),
            entry,
            instance,
            data,
            device,
            test_pipeline,
            objects: ObjectStorage::init(),
            frame_active_objects: Vec::new(),
            frame: 0,
            resized: false,
            camera: Rfc::default()
        },
        window.clone()))
    }
    
    pub fn vsync(&mut self, option: bool) {
        todo!()
    }
    pub fn msaa(&mut self, value: u32) {
        todo!()
    }
    pub fn set_camera(&mut self, camera: Rfc<Camera>) {
        self.camera = camera;        
    }
    
    // TODO: Create a gigantic uniform buffer, use pushconstants to address them in the shader
    pub unsafe fn draw(&mut self, object: Rfc<Object<Vertex>>) -> Result<(), MyError> {
        optick::event!();

        self.objects.insert(
            object.clone(), 
            &self.device, 
            &self.instance, 
            &self.data.physical_device
        );
        self.frame_active_objects.push(object.borrow().uuid());
        
        Ok(())
    }
    pub unsafe fn render(
        &mut self,
    ) -> Result<(), MyError>
    {
        optick::event!();
        // Wait for gpu to finish the rendering
        self.device.get_device().wait_for_fences(
            &[self.data.sync_objects[self.frame].render_fence],
            true, 
            u64::MAX
        )?;
        self.device.get_device().reset_fences(&[self.data.sync_objects[self.frame].render_fence])?;
        
        let result = self.device
            .get_device()
            .acquire_next_image_khr(
                self.data.swapchain.swapchain, 
                u64::MAX, 
                self.data.sync_objects[self.frame].present_semaphore,
                vk::Fence::null()
            );
        
        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                self.recreate_swapchain()?;
                return Err("Out of date KHR".into());
            },
            Err(e) => return Err(e.into()),
        };
        
        self.objects.destroy_inactive_objects(&self.device);
        self.update_camera_buffer()?;
        self.update_object_uniforms()?;
        self.prepare_cmd_buffer(image_index)?;

        let wait_semaphores = &[self.data.sync_objects[self.frame].present_semaphore];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.device.get_graphics_queue().command_buffers[image_index]];
        let signal_semaphores = &[self.data.sync_objects[self.frame].render_semaphore];
        
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);
        
        self.device.get_device().queue_submit(
            self.device.get_graphics_queue().queue, 
            &[submit_info], 
            self.data.sync_objects[self.frame].render_fence
        )?;
        
        let swapchains = &[self.data.swapchain.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);
        
        let result = self.device.get_device().queue_present_khr(self.device.get_present_queue().queue, &present_info);

        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
        
        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain()?;
        }
        else if let Err(e) = result {
            return Err(e.into());
        }
        
        self.frame = (self.frame + 1) % helper::MAX_FRAMES_IN_FLIGHT;
        self.frame_active_objects.clear();

        Ok(())
    }
    unsafe fn update_camera_buffer(&mut self) -> Result<(), MyError> {
        let view = self.camera.borrow().get_view_matrix();
        let proj = self.camera.borrow().get_projection_matrix();
        let ubo = ViewProjUBO { 
            view,
            proj
        };
        
        // Copy

        let memory = self.device.get_device().map_memory(
            self.test_pipeline.descriptor_data[self.frame].buffers[BufferCategory::VIEW_PROJ as usize].memory,
            0,
            size_of::<ViewProjUBO>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;
        memcpy(&ubo, memory.cast(), 1);

        self.device.get_device().unmap_memory(self.test_pipeline.descriptor_data[self.frame].buffers[BufferCategory::VIEW_PROJ as usize].memory);
        
        self.test_pipeline.descriptor_data[self.frame].update_vp(&self.device);

        Ok(())
    }
    unsafe fn update_object_uniforms(&mut self) -> Result<(), MyError>
    {
        let mut offset = 0;
        for (_, fa_object) in self.frame_active_objects.iter().enumerate() {
            let object = self.objects
                .get_objects()
                .get(fa_object)
                .unwrap()
                .object
                .borrow();
            
            let texture = object.vk_texture.as_ref().unwrap();
            let model = glm::Mat4::identity();
            let ubo = ModelUBO { data: model };

            // Copy

            let memory = self.device.get_device().map_memory(
                self.test_pipeline.descriptor_data[self.frame].buffers[BufferCategory::MODEL as usize].memory,
                offset,
                size_of::<ModelUBO>() as u64,
                vk::MemoryMapFlags::empty(),
            )?;
            memcpy(&ubo, memory.cast(), 1);

            self.device.get_device().unmap_memory(self.test_pipeline.descriptor_data[self.frame].buffers[BufferCategory::MODEL as usize].memory);

            self.test_pipeline.descriptor_data[self.frame].update_model(
                &self.device, 
            );
            self.test_pipeline.descriptor_data[self.frame].update_image(
                &self.device, 
                texture
            );

            offset += size_of::<UniformBufferObject>() as u64;
        }

        Ok(())
    }
    pub unsafe fn destroy(&mut self) -> Result<(), MyError> {
        self.device.get_device().device_wait_idle().unwrap();
        
        self.destroy_swapchain();
    
        // Objects
        self.objects.destroy(&self.device);
        
        for sync_obj in &self.data.sync_objects {
            self.device.get_device().destroy_fence(sync_obj.render_fence, None);
            self.device.get_device().destroy_semaphore(sync_obj.present_semaphore, None);
            self.device.get_device().destroy_semaphore(sync_obj.render_semaphore, None);
        }
    
        if helper::VALIDATION_ENABLED {
            self.instance.get_instance().destroy_debug_utils_messenger_ext(self.instance.messenger.unwrap(), None);
        }
        
        self.device.destroy_command_pools();
        self.device.get_device().destroy_device(None);
        self.instance.get_instance().destroy_surface_khr(self.data.surface, None);
        self.instance.get_instance().destroy_instance(None);
    
        Ok(())
    }
    
    unsafe fn prepare_cmd_buffer(&mut self, index: usize) -> Result<(), MyError>
    {
        let info = vk::CommandBufferBeginInfo::builder(); 
        let command_buffer = &self.device.get_graphics_queue().command_buffers[index];
        
        self.device.get_device().reset_command_buffer(*command_buffer, vk::CommandBufferResetFlags::empty());

        // Prepare to submit commands
        self.device.get_device().begin_command_buffer(*command_buffer, &info)?;
        
        // Begin render pass
        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.data.swapchain.extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }; 
        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
        };
        let clear_values = &[color_clear_value, depth_clear_value];

        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(*self.data.render_pass.get_render_pass())
            .framebuffer(self.data.framebuffers[index])
            .render_area(render_area)
            .clear_values(clear_values);

        self.device.get_device().cmd_begin_render_pass(*command_buffer, &begin_info, vk::SubpassContents::INLINE);
        self.device.get_device().cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, self.test_pipeline.pipeline);

        let mut ubo_offset = 0;
        for (_, fa_object) in self.frame_active_objects.iter().enumerate() {
            let object = self.objects
                .get_objects()
                .get(fa_object)
                .unwrap()
                .object
                .borrow();
            
            self.device.get_device().cmd_bind_vertex_buffers(
                *command_buffer, 
                0, 
                &[object.vertex_buffer.as_ref().unwrap().buffer], 
                &[0]
            );
            self.device.get_device().cmd_bind_index_buffer(
                *command_buffer, 
                object.index_buffer.as_ref().unwrap().buffer, 
                0, 
                vk::IndexType::UINT32
            );
            self.device.get_device().cmd_bind_descriptor_sets(
                *command_buffer, 
                vk::PipelineBindPoint::GRAPHICS, 
                self.test_pipeline.layout, 
                0, 
                self.test_pipeline.descriptor_data[self.frame].get_sets().as_slice(),
                &[]
            );
            ubo_offset += size_of::<UniformBufferObject>() as u32;
            
            // Draw call
            self.device.get_device().cmd_draw_indexed(
                *command_buffer, 
                object.object.borrow().indices().len() as u32,
                1, 
                0, 
                0, 
                0
            );
        }

        // End renderpass
        self.device.get_device().cmd_end_render_pass(*command_buffer);
        
        // End command submit
        self.device.get_device().end_command_buffer(*command_buffer)?;
        
        Ok(())
    }
    unsafe fn destroy_swapchain(&mut self) {
        self.device.free_command_buffers();
        self.test_pipeline.descriptor_data.iter_mut().for_each(|dd| dd.destroy(&self.device));
        
        self.data.color_image.destroy(&self.device);
        self.data.depth_image.destroy(&self.device);
        
        self.data.framebuffers.iter().for_each(|f| self.device.get_device().destroy_framebuffer(*f, None));
        
        self.device.get_device().destroy_pipeline(self.test_pipeline.pipeline, None);
        self.device.get_device().destroy_pipeline_layout(self.test_pipeline.layout, None);
    
        self.device.get_device().destroy_render_pass(*self.data.render_pass.get_render_pass(), None);
        
        self.data.swapchain.views.iter().for_each(|v| self.device.get_device().destroy_image_view(*v, None));
        
        self.device.get_device().destroy_swapchain_khr(self.data.swapchain.swapchain, None);
    }
    unsafe fn recreate_swapchain(&mut self) -> Result<(), MyError> {
        self.device.get_device().device_wait_idle()?;
        
        self.destroy_swapchain();
        self.data.swapchain = VkSwapchain::new(
            &self.window.borrow(), 
            &self.instance, 
            &self.data.surface, 
            &self.data.physical_device, 
            &self.device
        )?;
        self.data.render_pass = VkRenderPass::get_default(
            &self.instance, 
            &self.device, 
            &self.data.physical_device, 
            self.data.swapchain.format, 
            self.data.msaa_samples
        )?;

        // Shaders
        let vert_shader = Shader::new(
            &self.device, 
            vk::ShaderStageFlags::VERTEX, 
            include_bytes!("../../assets/shaders/compiled/2DShader.spv")
        )?;
        let frag_shader = Shader::new(
            &self.device, 
            vk::ShaderStageFlags::FRAGMENT, 
            include_bytes!("../../assets/shaders/compiled/shader.spv")
        )?;
        // Viewport and Scissor
        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(self.data.swapchain.extent.width as f32)
            .height(self.data.swapchain.extent.width as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();
        
        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(self.data.swapchain.extent)
            .build();

        self.test_pipeline = VkPipeline::new(
            &self.device, 
            &self.instance,
            &self.data.physical_device,
            vert_shader, 
            frag_shader, 
            &[Vertex::binding_description()],
            &Vertex::attribute_descritptions(), 
            vec![viewport], 
            vec![scissor], 
            self.data.msaa_samples, 
            &self.data.render_pass
        )?;

        self.data.color_image = VkImage::new(
            &self.instance, 
            &self.device, 
            &self.data.physical_device, 
            self.data.swapchain.extent.width, 
            self.data.swapchain.extent.height, 
            self.data.swapchain.format, 
            vk::ImageAspectFlags::COLOR, 
            self.data.msaa_samples, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT, 
            1
        )?;
        self.data.depth_image = VkImage::new(
            &self.instance, 
            &self.device, 
            &self.data.physical_device, 
            self.data.swapchain.extent.width, 
            self.data.swapchain.extent.height, 
            helper::get_depth_format(self.instance.get_instance(), self.data.physical_device.get_device())?, 
            vk::ImageAspectFlags::DEPTH, 
            self.data.msaa_samples, 
            vk::ImageTiling::OPTIMAL, 
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, 
            1
        )?;
        
        // Framebuffer
        self.data.framebuffers = framebuffer::create_framebuffers(
            &self.device, 
            &self.data.render_pass, 
            &self.data.swapchain.views, 
            &self.data.color_image, 
            &self.data.depth_image, 
            self.data.swapchain.extent.width, 
            self.data.swapchain.extent.height
        )?;

        self.device.allocate_command_buffers(VkQueueFamily::GRAPHICS, self.data.swapchain.images.len() as u32)?;
        self.device.allocate_command_buffers(VkQueueFamily::PRESENT, self.data.swapchain.images.len() as u32)?;
        self.device.allocate_command_buffers(VkQueueFamily::TRANSFER, self.data.swapchain.images.len() as u32)?;

        Ok(())
    }
}
pub struct DrawInfo {
    position: glm::Vec3,
    scale: glm::Vec3,
    rotation: f32,
    color: glm::Vec4,
    texture: Option<Texture>,
    tiling: Option<i32>,
}
pub struct CircleInfo {
    draw_info: DrawInfo,
    radius: f32,
    thickness: Option<f32>,
}

struct VerticesInfo {
    vertices: [Vertex; 4],
    indices: [u16; 5],
}
pub struct Renderer2D {
    vertices_info: VerticesInfo
}
impl Renderer2D {
    fn init() -> Self {
        let vertices = [
            Vertex::new(glm::vec3(-0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 0.0)),
            Vertex::new(glm::vec3(0.5, -0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 0.0)),
            Vertex::new(glm::vec3(0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(0.0, 1.0)),
            Vertex::new(glm::vec3(-0.5, 0.5, 0.0), glm::vec3(1.0, 1.0, 1.0), glm::vec2(1.0, 1.0)),
        ];
        let indices = [0, 1, 2, 3, 4];
        
        Self {
            vertices_info: VerticesInfo {
                vertices,
                indices
            }
        }
    }
    pub fn shutdown(&mut self) {
        todo!()
    }

    pub fn begin_batch(&mut self) {
        todo!()
    }
    pub fn end_batch(&mut self) {
        
    }
    
    pub fn submit_quad(&mut self, info: &DrawInfo) {
        todo!()
    }
    pub fn submit_circle(&mut self, info: &CircleInfo) {
        todo!()
    }
}